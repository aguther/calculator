from pathlib import Path
import importlib.util,json,shutil
root=Path(__file__).resolve().parents[1]
info=json.loads((root/'dist/build-info.json').read_text())
next_dir=root/'.cache'/('pwa-next-'+info['build'])
if not next_dir.exists():
    shutil.copytree(root/'dist',next_dir)
    script=next_dir/'web/app.js'
    script.write_text(script.read_text(encoding='utf-8')+'\n// Integration-test update build.\n',encoding='utf-8')
    spec=importlib.util.spec_from_file_location('build_web',root/'scripts/build-web.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);module.finalize(next_dir)
print(next_dir)
