"""Package desktop binaries and license material for a release."""

import pathlib
import sys
import tarfile
import tomllib
import zipfile

from licenses import collect
from windows_binary import verify_standalone

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main() -> None:
    """Run the command-line workflow."""
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"][
        "version"
    ]
    platform = sys.argv[1]
    if platform not in ("windows-x64", "linux-x64"):
        raise SystemExit("Unsupported platform")
    name = f"calculator-{version}-{platform}"
    release_directory = ROOT / "release"
    release_directory.mkdir(exist_ok=True)
    executable_name = "calculator.exe" if platform == "windows-x64" else "calculator"
    license_path = ROOT / ".cache/THIRD_PARTY_LICENSES.txt"
    license_path.parent.mkdir(exist_ok=True)
    license_path.write_text(collect(), encoding="utf-8")
    files = [
        (license_path, "THIRD_PARTY_LICENSES.txt"),
        (ROOT / "target/release" / executable_name, executable_name),
        (ROOT / "LICENSE", "LICENSE"),
        (ROOT / "docs/desktop.md", "README.md"),
    ]
    if platform == "windows-x64":
        verify_standalone(ROOT / "target/release" / executable_name)
        with zipfile.ZipFile(
            release_directory / (name + ".zip"), "w", compression=zipfile.ZIP_DEFLATED
        ) as archive:
            for path, filename in files:
                archive.write(path, name + "/" + filename)
    else:
        with tarfile.open(release_directory / (name + ".tar.gz"), "w:gz") as archive:
            for path, filename in files:
                archive.add(path, arcname=name + "/" + filename)


if __name__ == "__main__":
    main()
