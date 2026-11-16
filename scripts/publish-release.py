"""Resume a release safely; assets with different bytes are never overwritten."""

import hashlib
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]


def gh(*args, **kwargs):
    """Run the GitHub CLI and stop publication if the command fails."""
    return subprocess.run(["gh", *args], check=True, **kwargs)


def release_version(tag):
    """Parse a stable tag into numeric components for monotonic comparison."""
    return tuple(map(int, tag.removeprefix("v").split(".")))


def main() -> None:
    """Run the command-line workflow."""
    tag = sys.argv[1]
    notes = ROOT / "docs" / "release-notes" / (tag.removeprefix("v") + ".md")
    if not notes.is_file() or not notes.read_text(encoding="utf-8").strip():
        raise SystemExit("Missing release notes for " + tag)
    releases = json.loads(
        gh(
            "release",
            "list",
            "--exclude-pre-releases",
            "--limit",
            "100",
            "--json",
            "tagName",
            capture_output=True,
            text=True,
        ).stdout
    )
    # Publication runs inside the workflow's global production lock.
    for release in releases:
        previous = release["tagName"]
        if re.fullmatch("v\\d+\\.\\d+\\.\\d+", previous) and release_version(
            previous
        ) > release_version(tag):
            raise SystemExit(
                "Refusing to deploy an older release than the published or reserved version "
                + previous
            )
    result = subprocess.run(["gh", "release", "view", tag], capture_output=True)
    if result.returncode:
        gh(
            "release",
            "create",
            tag,
            "--verify-tag",
            "--title",
            "Calculator " + tag,
            "--notes-file",
            str(notes),
            "--draft",
        )
    else:
        gh("release", "edit", tag, "--notes-file", str(notes))
    for file in sorted((ROOT / "release").iterdir()):
        with tempfile.TemporaryDirectory() as temp:
            existing = subprocess.run(
                [
                    "gh",
                    "release",
                    "download",
                    tag,
                    "--pattern",
                    file.name,
                    "--dir",
                    temp,
                ],
                capture_output=True,
            )
            if existing.returncode == 0:
                old = pathlib.Path(temp) / file.name
                if (
                    hashlib.sha256(old.read_bytes()).digest()
                    != hashlib.sha256(file.read_bytes()).digest()
                ):
                    raise SystemExit(
                        "Existing asset differs; refusing replacement: " + file.name
                    )
            else:
                gh("release", "upload", tag, str(file))


if __name__ == "__main__":
    main()
