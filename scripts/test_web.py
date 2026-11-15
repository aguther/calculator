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
    def test_tag_only_publication(self):
        text=(ROOT/'.github/workflows/release.yml').read_text()
        self.assertIn('tags:',text)
        self.assertNotIn('branches:',text)
        self.assertIn('needs: [verify, build]',text)
if __name__=='__main__':unittest.main()
