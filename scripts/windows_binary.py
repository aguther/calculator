"""Verify portable VC-runtime linkage and embedded Windows application icons."""

from pathlib import Path
import struct
import sys

ICON_SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)


class PortableExecutable:
    """Read PE imports and resource data without platform-specific dependencies."""

    def __init__(self, file):
        self.binary = Path(file).read_bytes()
        if self.binary[:2] != b"MZ":
            raise ValueError("Missing DOS header")
        pe = self.unpack("<I", 0x3C)[0]
        if self.binary[pe : pe + 4] != b"PE\0\0":
            raise ValueError("Missing PE signature")
        count = self.unpack("<H", pe + 6)[0]
        optional_size = self.unpack("<H", pe + 20)[0]
        optional = pe + 24
        magic = self.unpack("<H", optional)[0]
        if magic not in (0x20B, 0x10B):
            raise ValueError("Unsupported PE optional header")
        self.directories = optional + (112 if magic == 0x20B else 96)
        self.ranges = []
        for index in range(count):
            size, base, raw_size, raw = self.unpack(
                "<IIII", optional + optional_size + index * 40 + 8
            )
            self.ranges.append((base, max(size, raw_size), raw))

    def unpack(self, format, offset):
        return struct.unpack_from(format, self.binary, offset)

    def offset(self, rva):
        for base, size, raw in self.ranges:
            if base <= rva < base + size:
                return raw + rva - base
        raise ValueError("Invalid PE RVA")

    def dependencies(self):
        imports_rva = self.unpack("<I", self.directories + 8)[0]
        if not imports_rva:
            return []
        cursor = self.offset(imports_rva)
        names = []
        while any(self.unpack("<IIIII", cursor)):
            start = self.offset(self.unpack("<I", cursor + 12)[0])
            names.append(
                self.binary[start : self.binary.index(0, start)].decode("ascii")
            )
            cursor += 20
        return names

    def resources(self):
        """Return numeric (type, name, language) resource paths and their bytes."""
        rva, size = self.unpack("<II", self.directories + 16)
        if not rva or not size:
            return {}
        base = self.offset(rva)
        result = {}

        def walk(relative, path):
            if len(path) > 3 or relative + 16 > size:
                raise ValueError("Invalid PE resource directory")
            named, numeric = self.unpack("<HH", base + relative + 12)
            for index in range(named + numeric):
                entry = relative + 16 + index * 8
                if entry + 8 > size:
                    raise ValueError("Invalid PE resource entry")
                name, target = self.unpack("<II", base + entry)
                if name & 0x80000000:
                    continue
                child = path + (name,)
                if target & 0x80000000:
                    walk(target & 0x7FFFFFFF, child)
                else:
                    if target + 16 > size:
                        raise ValueError("Invalid PE resource data entry")
                    data_rva, length = self.unpack("<II", base + target)
                    start = self.offset(data_rva)
                    data = self.binary[start : start + length]
                    if len(data) != length:
                        raise ValueError("Truncated PE resource")
                    result[child] = data

        walk(0, ())
        return result


def dependencies(file):
    """Return imported DLL names, retaining the packaging script API."""
    return PortableExecutable(file).dependencies()


def verify_icon(file):
    """Require a complete icon group whose frames match the checked-in artwork."""
    resources = PortableExecutable(file).resources()
    source = (
        Path(__file__).resolve().parents[1] / "assets/windows/calculator.ico"
    ).read_bytes()
    expected = {}
    for index in range(struct.unpack_from("<H", source, 4)[0]):
        width, height, _, _, _, bits, length, offset = struct.unpack_from(
            "<BBBBHHII", source, 6 + index * 16
        )
        expected[(width or 256, height or 256, bits)] = source[offset : offset + length]
    groups = [(path, data) for path, data in resources.items() if path[0] == 14]
    for path, group in groups:
        reserved, kind, count = struct.unpack_from("<HHH", group)
        if reserved or kind != 1 or count != len(ICON_SIZES):
            continue
        found = set()
        for index in range(count):
            width, height, _, _, planes, bits, length, identifier = struct.unpack_from(
                "<BBBBHHIH", group, 6 + index * 14
            )
            key = (width or 256, height or 256, bits)
            frame = resources.get((3, identifier, path[2]))
            if (
                planes != 1
                or frame is None
                or len(frame) != length
                or frame != expected.get(key)
            ):
                break
            found.add(key[0])
        else:
            if found == set(ICON_SIZES):
                return sorted(found)
    raise ValueError(
        "Missing or incorrect embedded RT_GROUP_ICON/RT_ICON application artwork"
    )


def verify_standalone(file):
    names = dependencies(file)
    forbidden = [
        name
        for name in names
        if name.upper().startswith(("VCRUNTIME", "MSVCP", "CONCRT"))
    ]
    if forbidden:
        raise SystemExit(
            "Non-portable VC runtime dependencies: " + ", ".join(forbidden)
        )
    verify_icon(file)
    return names


if __name__ == "__main__":
    print("Windows system DLL imports:", ", ".join(verify_standalone(sys.argv[1])))
    print("Embedded application icon sizes:", ", ".join(map(str, ICON_SIZES)))
