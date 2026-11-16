"""Collect registry license material for built distributions, including embedded fonts."""

import os
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def collect() -> str:
    """Return available Cargo package licenses and bundled asset licenses."""
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    registry_directories = list((cargo_home / "registry/src").glob("*"))
    lockfile = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
    sections = [
        "THIRD-PARTY LICENSE MATERIAL\nCalculator uses the following Cargo registry packages.\nIncludes available target-specific packages and embedded font licenses.\n"
    ]
    for package in sorted(
        lockfile["package"], key=lambda package: (package["name"], package["version"])
    ):
        if not package.get("source", "").startswith("registry+"):
            continue
        name = package["name"] + "-" + package["version"]
        source = next(
            (
                registry_directory / name
                for registry_directory in registry_directories
                if (registry_directory / name).is_dir()
            ),
            None,
        )
        if source is None:
            # Packages for targets not built need not be present locally.
            continue
        metadata = tomllib.loads((source / "Cargo.toml").read_text(encoding="utf-8"))[
            "package"
        ]
        license_paths = set()
        for file in source.iterdir():
            if file.is_file() and file.name.upper().startswith(
                ("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT")
            ):
                license_paths.add(file)
        license_file = metadata.get("license-file")
        if license_file and (source / license_file).is_file():
            license_paths.add(source / license_file)
        if package["name"] == "epaint_default_fonts":
            license_paths.update(source.rglob("*.txt"))
        sections.append(
            "\n"
            + "=" * 72
            + "\n"
            + name
            + "\nLicense: "
            + metadata.get("license", "See license file")
            + "\n"
        )
        for file in sorted(license_paths):
            sections.append(
                "\n--- "
                + file.relative_to(source).as_posix()
                + " ---\n"
                + file.read_text(encoding="utf-8", errors="replace")
                + "\n"
            )
    sections.append(
        "\nNoto Sans (Regular and SemiBold)\n"
        + (ROOT / "assets/fonts/OFL.txt").read_text(encoding="utf-8")
    )
    sections.append(
        "\nLucide Icons\n" + (ROOT / "assets/icons/LICENSE").read_text(encoding="utf-8")
    )
    return "\n".join(sections)


if __name__ == "__main__":
    print(collect())
