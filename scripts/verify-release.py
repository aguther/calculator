"""Validate the stable release tag, release notes, and main ancestry."""

import pathlib
import re
import subprocess
import sys
import tomllib

ROOT = pathlib.Path(__file__).resolve().parents[1]


def main() -> None:
    """Run the command-line workflow."""
    tag = sys.argv[1]
    version = tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]["package"][
        "version"
    ]
    if not re.fullmatch("v\\d+\\.\\d+\\.\\d+", tag) or tag != "v" + version:
        raise SystemExit("Tag must match the stable Cargo workspace version")
    notes = ROOT / "docs" / "release-notes" / (version + ".md")
    if not notes.is_file() or not notes.read_text(encoding="utf-8").strip():
        raise SystemExit("Missing release notes for " + tag)
    subprocess.run(
        ["git", "merge-base", "--is-ancestor", "HEAD", "origin/main"],
        cwd=ROOT,
        check=True,
    )
    print("Release version and main ancestry verified:", tag)


if __name__ == "__main__":
    main()
