import importlib.util, json, pathlib, tempfile, unittest
ROOT=pathlib.Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('build_web', ROOT/'scripts/build-web.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
class WebBuildTests(unittest.TestCase):
    def test_external_loader_and_complete_cache(self):
        with tempfile.TemporaryDirectory() as temp:
            dist=pathlib.Path(temp)
            (dist/'index.html').write_text('<script type="module">import init from "/app-abcdef1234.js";await init();</script>')
            (dist/'app-abcdef1234.js').write_text('export default async function init(){}')
            (dist/'app-abcdef1234_bg.wasm').write_bytes(b'wasm')
            first=module.finalize(dist)
            html=(dist/'index.html').read_text()
            self.assertNotIn('await init',html)
            self.assertIn('loader-',html)
            info=json.loads((dist/'build-info.json').read_text())
            self.assertIn('/app-abcdef1234_bg.wasm',info['assets'])
            self.assertNotIn('/sw.js',info['assets'])
            self.assertEqual(first,module.finalize(dist))
            (dist/'app-abcdef1234_bg.wasm').write_bytes(b'next version')
            self.assertNotEqual(first,module.finalize(dist))
    def test_cloudflare_cache_directives_do_not_conflict(self):
        import fnmatch
        def effective_cache(text, url):
            pattern = None
            values = []
            for line in text.splitlines():
                if line and not line.startswith(' '): pattern = line
                elif pattern and fnmatch.fnmatchcase(url, pattern):
                    header = line.strip()
                    if header == '! Cache-Control': values = []
                    elif header.startswith('Cache-Control:'): values.append(header.split(':', 1)[1].strip())
            return ', '.join(values)
        with tempfile.TemporaryDirectory() as temp:
            dist = pathlib.Path(temp)
            (dist/'index.html').write_text('<html></html>')
            (dist/'app-abcdef1234_bg.wasm').write_bytes(b'wasm')
            module.finalize(dist)
            headers = (dist/'_headers').read_text()
            self.assertEqual(effective_cache(headers, '/app-abcdef1234_bg.wasm'), 'public, max-age=31536000, immutable')
            self.assertEqual(effective_cache(headers, '/sw.js'), 'no-cache, no-store, must-revalidate')
            self.assertEqual(effective_cache(headers, '/index.html'), 'public, max-age=0, must-revalidate')
    def test_ios_status_bar_matches_working_web_version(self):
        from html.parser import HTMLParser
        class MetaParser(HTMLParser):
            def handle_starttag(parser, tag, attrs):
                attrs = dict(attrs)
                if tag == 'meta' and attrs.get('name') == 'apple-mobile-web-app-status-bar-style':
                    parser.status_bar = attrs.get('content')
        parser = MetaParser()
        parser.feed((ROOT/'index.html').read_text(encoding='utf-8'))
        self.assertEqual(parser.status_bar, 'black')

    def test_tag_only_publication(self):
        text=(ROOT/'.github/workflows/release.yml').read_text()
        self.assertIn('tags:',text)
        self.assertNotIn('branches:',text)
        self.assertIn('needs: [verify, build]',text)
if __name__=='__main__':unittest.main()
