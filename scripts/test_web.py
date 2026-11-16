"""Test web build integrity, release publication, and deployment verification."""

import importlib.util
import json
import pathlib
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "build_web", ROOT / "scripts/build-web.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class WebBuildTests(unittest.TestCase):

    def test_external_loader_and_complete_cache(self):
        with tempfile.TemporaryDirectory() as temp:
            dist = pathlib.Path(temp)
            (dist / "index.html").write_text(
                '<script type="module">import init from "/app-abcdef1234.js";await init();</script>'
            )
            (dist / "app-abcdef1234.js").write_text(
                "export default async function init(){}"
            )
            (dist / "app-abcdef1234_bg.wasm").write_bytes(b"wasm")
            first = module.finalize(dist)
            html = (dist / "index.html").read_text()
            self.assertNotIn("await init", html)
            self.assertIn("loader-", html)
            build_info = json.loads((dist / "build-info.json").read_text())
            self.assertIn("/app-abcdef1234_bg.wasm", build_info["assets"])
            self.assertNotIn("/sw.js", build_info["assets"])
            self.assertEqual(first, module.finalize(dist))
            (dist / "app-abcdef1234_bg.wasm").write_bytes(b"next version")
            self.assertNotEqual(first, module.finalize(dist))

    def test_cloudflare_cache_directives_do_not_conflict(self):
        import fnmatch

        def effective_cache(text, url):
            pattern = None
            values = []
            for line in text.splitlines():
                if line and (not line.startswith(" ")):
                    pattern = line
                elif pattern and fnmatch.fnmatchcase(url, pattern):
                    header = line.strip()
                    if header == "! Cache-Control":
                        values = []
                    elif header.startswith("Cache-Control:"):
                        values.append(header.split(":", 1)[1].strip())
            return ", ".join(values)

        with tempfile.TemporaryDirectory() as temp:
            dist = pathlib.Path(temp)
            (dist / "index.html").write_text("<html></html>")
            (dist / "app-abcdef1234_bg.wasm").write_bytes(b"wasm")
            module.finalize(dist)
            headers = (dist / "_headers").read_text()
            self.assertEqual(
                effective_cache(headers, "/app-abcdef1234_bg.wasm"),
                "public, max-age=31536000, immutable",
            )
            self.assertEqual(
                effective_cache(headers, "/sw.js"),
                "no-cache, no-store, must-revalidate",
            )
            self.assertEqual(
                effective_cache(headers, "/index.html"),
                "public, max-age=0, must-revalidate",
            )

    def test_ios_status_bar_matches_working_web_version(self):
        from html.parser import HTMLParser

        class MetaParser(HTMLParser):

            def handle_starttag(parser, tag, attrs):
                attrs = dict(attrs)
                if (
                    tag == "meta"
                    and attrs.get("name") == "apple-mobile-web-app-status-bar-style"
                ):
                    parser.status_bar = attrs.get("content")

        parser = MetaParser()
        parser.feed((ROOT / "index.html").read_text(encoding="utf-8"))
        self.assertEqual(parser.status_bar, "black")

    def test_tag_only_publication(self):
        text = (ROOT / ".github/workflows/release.yml").read_text()
        self.assertIn("tags:", text)
        self.assertNotIn("branches:", text)
        self.assertIn("needs: [verify, build]", text)


class ReleaseNotesTests(unittest.TestCase):

    def publish(self, notes=True, existing=False):
        import runpy
        import shutil
        import subprocess
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as temp:
            fixture_root = pathlib.Path(temp)
            (fixture_root / "scripts").mkdir()
            (fixture_root / "release").mkdir()
            script = fixture_root / "scripts/publish-release.py"
            shutil.copyfile(ROOT / "scripts/publish-release.py", script)
            expected = "## Changes\n- Clear description for users.\n"
            if notes:
                target = fixture_root / "docs/release-notes/0.1.9.md"
                target.parent.mkdir(parents=True)
                target.write_text(expected, encoding="utf-8")
            calls = []

            def run(args, **kwargs):
                calls.append(args)
                if args[:3] == ["gh", "release", "list"]:
                    return subprocess.CompletedProcess(args, 0, stdout="[]")
                if args[:3] == ["gh", "release", "view"]:
                    return subprocess.CompletedProcess(args, 0 if existing else 1)
                if "--notes-file" in args:
                    self.assertEqual(
                        pathlib.Path(args[args.index("--notes-file") + 1]).read_text(
                            encoding="utf-8"
                        ),
                        expected,
                    )
                return subprocess.CompletedProcess(args, 0)

            with patch("sys.argv", [str(script), "v0.1.9"]), patch(
                "subprocess.run", side_effect=run
            ):
                if not notes:
                    with self.assertRaisesRegex(SystemExit, "release notes"):
                        runpy.run_path(str(script), run_name="__main__")
                    self.assertEqual(calls, [])
                    return
                runpy.run_path(str(script), run_name="__main__")
            command = next(
                (
                    args
                    for args in calls
                    if args[:3] == ["gh", "release", "edit" if existing else "create"]
                )
            )
            self.assertIn("--notes-file", command)
            self.assertNotIn("--generate-notes", command)

    def test_release_uses_its_versioned_description(self):
        self.publish()

    def test_resumed_release_keeps_the_description_in_sync(self):
        self.publish(existing=True)

    def test_missing_notes_fail_before_remote_changes(self):
        self.publish(notes=False)


class DeploymentVerificationTests(unittest.TestCase):

    def test_only_the_verified_build_is_accepted(self):
        import http.server
        import threading

        spec = importlib.util.spec_from_file_location(
            "verify_deployment", ROOT / "scripts/verify-deployment.py"
        )
        deploy = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(deploy)
        payloads = [b'{"build":"old"}', b'{"build":"verified"}']
        paths = []

        class Handler(http.server.BaseHTTPRequestHandler):

            def do_GET(self):
                if self.headers.get("User-Agent", "").startswith("Python-urllib"):
                    self.send_response(403)
                    self.end_headers()
                    return
                paths.append(self.path)
                self.send_response(200)
                self.end_headers()
                self.wfile.write(payloads.pop(0))

            def log_message(self, *args):
                pass

        with http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler) as server:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                deploy.verify(
                    "http://127.0.0.1:" + str(server.server_port),
                    "verified",
                    attempts=2,
                    delay=0,
                )
                self.assertEqual(paths, ["/build-info.json", "/build-info.json"])
                payloads.append(b'{"build":"stale"}')
                with self.assertRaisesRegex(
                    SystemExit, "Expected build verified, received stale"
                ):
                    deploy.verify(
                        "http://127.0.0.1:" + str(server.server_port),
                        "verified",
                        attempts=1,
                        delay=0,
                    )
            finally:
                server.shutdown()
                thread.join()


if __name__ == "__main__":
    unittest.main()
