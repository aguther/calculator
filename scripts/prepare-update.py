"""Prepare a distinct offline build for browser update integration tests."""

import importlib.util
import json
import shutil
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main() -> None:
    """Run the command-line workflow."""
    build_info = json.loads((ROOT / "dist/build-info.json").read_text())
    update_directory = ROOT / ".cache" / ("pwa-next-" + build_info["build"])
    if not update_directory.exists():
        shutil.copytree(ROOT / "dist", update_directory)
        script = update_directory / "web/app.js"
        script.write_text(
            script.read_text(encoding="utf-8")
            + "\n// Integration-test update build.\n",
            encoding="utf-8",
        )
        spec = importlib.util.spec_from_file_location(
            "build_web", ROOT / "scripts/build-web.py"
        )
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        module.finalize(update_directory)
    print(update_directory)


if __name__ == "__main__":
    main()
