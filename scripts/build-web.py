"""Finalize Trunk output: external CSP-safe loader and atomic offline shell."""

import hashlib
import json
import os
import re
import subprocess
import sys
from pathlib import Path

from licenses import collect

ROOT = Path(__file__).resolve().parents[1]


def finalize(dist: Path) -> str:
    """Externalize module loaders and generate content-versioned offline assets."""
    (dist / "THIRD_PARTY_LICENSES.txt").write_text(collect(), encoding="utf-8")
    html_path = dist / "index.html"
    html = html_path.read_text(encoding="utf-8")

    def externalize(match):
        """Save an inline module under a content-derived filename."""
        code = match.group(1)
        name = "loader-" + hashlib.sha256(code.encode()).hexdigest()[:16] + ".js"
        (dist / name).write_text(code, encoding="utf-8")
        return '<script type="module" src="/' + name + '"></script>'

    html = re.sub('<script type="module">(.*?)</script>', externalize, html, flags=re.S)
    html_path.write_text(html, encoding="utf-8")
    template_path = dist / "web/sw.template.js"
    if template_path.exists():
        template_path.unlink()
    assets = sorted(
        (
            "/" + file.relative_to(dist).as_posix()
            for file in dist.rglob("*")
            if file.is_file()
            and file.name
            not in ("sw.js", "_headers", "build-info.json", "THIRD_PARTY_LICENSES.txt")
        )
    )
    digest = hashlib.sha256()
    for asset in assets:
        digest.update(asset.encode())
        digest.update((dist / asset[1:]).read_bytes())
    version = digest.hexdigest()[:20]
    template = (ROOT / "web/sw.template.js").read_text(encoding="utf-8")
    (dist / "sw.js").write_text(
        template.replace("__VERSION__", json.dumps(version)).replace(
            "__ASSETS__", json.dumps(assets)
        ),
        encoding="utf-8",
    )
    headers = (ROOT / "_headers").read_text(encoding="utf-8")
    for asset in assets:
        if re.search("-[a-f0-9]{8,}", asset):
            headers += f"{asset}\n  ! Cache-Control\n  Cache-Control: public, max-age=31536000, immutable\n"
    (dist / "_headers").write_text(headers, encoding="utf-8")
    (dist / "build-info.json").write_text(
        json.dumps({"build": version, "assets": assets}, indent=2) + "\n",
        encoding="utf-8",
    )
    return version


if __name__ == "__main__":
    if "--finalize-only" not in sys.argv:
        subprocess.run(
            ["trunk", "build", "--release", "--locked"],
            cwd=ROOT,
            check=True,
            env={**os.environ, "NO_COLOR": "true"},
        )
    print("PWA build:", finalize(ROOT / "dist"))
