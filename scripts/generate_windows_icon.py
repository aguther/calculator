"""Generate the Windows ICO from the unchanged shared application artwork.

Install scripts/requirements-icons.txt, then run this script from any directory. Use --check in CI to
verify the checked-in asset without writing it. Each ICO frame is a PNG, retaining
alpha transparency at all Windows shell scale factors.
"""

import argparse
import io
from pathlib import Path
import struct
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
SIZES = (16, 20, 24, 32, 40, 48, 64, 96, 128, 256)
ICON = ROOT / "assets/windows/calculator.ico"


def generate_icon():
    """Return a deterministic ICO with all supported shell image sizes."""
    with Image.open(ROOT / "web/icon-512.png") as source:
        source = source.convert("RGBA")
        frames = []
        for size in SIZES:
            output = io.BytesIO()
            source.resize((size, size), Image.Resampling.LANCZOS).save(
                output, format="PNG"
            )
            frames.append(output.getvalue())
    offset = 6 + 16 * len(SIZES)
    directory = bytearray(struct.pack("<HHH", 0, 1, len(SIZES)))
    for size, frame in zip(SIZES, frames):
        directory.extend(
            struct.pack(
                "<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(frame), offset
            )
        )
        offset += len(frame)
    return bytes(directory) + b"".join(frames)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    generated = generate_icon()
    if args.check:
        if ICON.read_bytes() != generated:
            raise SystemExit(
                "Windows icon is stale; run scripts/generate_windows_icon.py"
            )
        print("Windows icon matches the shared application artwork")
    else:
        ICON.parent.mkdir(parents=True, exist_ok=True)
        ICON.write_bytes(generated)
        print(f"Generated {ICON.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
