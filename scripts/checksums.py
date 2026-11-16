"""Write SHA-256 checksums for the Windows and Linux release archives."""

import hashlib
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1] / "release"


def main() -> None:
    """Run the command-line workflow."""
    files = sorted(
        (
            package
            for package in ROOT.iterdir()
            if package.name.endswith((".zip", ".tar.gz"))
        )
    )
    if len(files) != 2:
        raise SystemExit("Expected Windows and Linux release archives")
    (ROOT / "SHA256SUMS.txt").write_text(
        "".join(
            (
                hashlib.sha256(package.read_bytes()).hexdigest()
                + "  "
                + package.name
                + "\n"
                for package in files
            )
        )
    )


if __name__ == "__main__":
    main()
