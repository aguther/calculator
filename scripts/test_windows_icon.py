"""Check ICO payloads and exercise PE icon resource validation."""

from pathlib import Path
import struct
import tempfile
import unittest

from windows_binary import ICON_SIZES, verify_icon, verify_standalone

ROOT = Path(__file__).resolve().parents[1]


def icon_frames():
    data = (ROOT / "assets/windows/calculator.ico").read_bytes()
    header = struct.unpack_from("<HHH", data)
    if header != (0, 1, len(ICON_SIZES)):
        raise ValueError("Incorrect ICO header")
    frames = []
    for index in range(header[2]):
        entry = struct.unpack_from("<BBBBHHII", data, 6 + 16 * index)
        frames.append((entry, data[entry[7] : entry[7] + entry[6]]))
    return frames


def executable_fixture():
    """Build a minimal PE resource tree, including actual ICO image payloads."""
    frames = icon_frames()
    group = struct.pack("<HHH", 0, 1, len(frames))
    for index, (entry, frame) in enumerate(frames, 1):
        group += struct.pack("<BBBBHHIH", *entry[:6], len(frame), index)
    resources = [(3, index, frame) for index, (_, frame) in enumerate(frames, 1)]
    resources.append((14, 1, group))
    tree = bytearray()

    def reserve(length):
        offset = len(tree)
        tree.extend(bytes(length))
        return offset

    def directory(items):
        offset = reserve(16 + 8 * len(items))
        struct.pack_into("<H", tree, offset + 14, len(items))
        for index, (identifier, children) in enumerate(items):
            if isinstance(children, bytes):
                child = reserve(16)
                payload = reserve(len(children))
                tree[payload : payload + len(children)] = children
                struct.pack_into(
                    "<IIII", tree, child, 0x1000 + payload, len(children), 0, 0
                )
            else:
                child = directory(children) | 0x80000000
            struct.pack_into("<II", tree, offset + 16 + index * 8, identifier, child)
        return offset

    directory(
        [
            (
                kind,
                [
                    (name, [(1033, payload)])
                    for typ, name, payload in resources
                    if typ == kind
                ],
            )
            for kind in (3, 14)
        ]
    )
    binary = bytearray(0x200)
    binary[:2] = b"MZ"
    struct.pack_into("<I", binary, 0x3C, 0x80)
    binary[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<H", binary, 0x86, 1)
    struct.pack_into("<H", binary, 0x94, 240)
    struct.pack_into("<H", binary, 0x98, 0x20B)
    struct.pack_into("<II", binary, 0x98 + 112 + 16, 0x1000, len(tree))
    struct.pack_into(
        "<IIII", binary, 0x98 + 240 + 8, len(tree), 0x1000, len(tree), 0x200
    )
    return binary + tree


class WindowsIconTests(unittest.TestCase):
    def test_ico_dimensions_and_alpha(self):
        sizes = []
        for entry, frame in icon_frames():
            self.assertEqual(frame[:8], b"\x89PNG\r\n\x1a\n")
            self.assertEqual(frame[12:16], b"IHDR")
            width, height, depth, color = struct.unpack_from(">IIBB", frame, 16)
            self.assertEqual((width, height), (entry[0] or 256, entry[1] or 256))
            self.assertEqual((depth, color), (8, 6))  # Eight-bit RGBA preserves alpha.
            sizes.append(width)
        self.assertEqual(tuple(sizes), ICON_SIZES)

    def test_pe_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "calculator.exe"
            file.write_bytes(executable_fixture())
            self.assertEqual(verify_icon(file), list(ICON_SIZES))
            self.assertEqual(verify_standalone(file), [])
            corrupted = file.read_bytes().replace(b"PNG", b"BAD", 1)
            file.write_bytes(corrupted)
            with self.assertRaisesRegex(ValueError, "RT_GROUP_ICON/RT_ICON"):
                verify_icon(file)

    def test_missing_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            file = Path(directory) / "calculator.exe"
            binary = executable_fixture()
            struct.pack_into("<II", binary, 0x98 + 112 + 16, 0, 0)
            file.write_bytes(binary)
            with self.assertRaisesRegex(ValueError, "RT_GROUP_ICON/RT_ICON"):
                verify_icon(file)


if __name__ == "__main__":
    unittest.main()
