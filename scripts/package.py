import pathlib,sys,tarfile,tomllib,zipfile
from licenses import collect
from windows_binary import verify_standalone
root=pathlib.Path(__file__).resolve().parents[1]
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
platform=sys.argv[1]
if platform not in ('windows-x64','linux-x64'):raise SystemExit('Unsupported platform')
name=f'calculator-{version}-{platform}'
out=root/'release';out.mkdir(exist_ok=True)
exe='calculator.exe' if platform=='windows-x64' else 'calculator'
license_path=root/'.cache/THIRD_PARTY_LICENSES.txt'
license_path.parent.mkdir(exist_ok=True)
license_path.write_text(collect(),encoding='utf-8')
files=[(license_path,'THIRD_PARTY_LICENSES.txt'),(root/'target/release'/exe,exe),(root/'LICENSE','LICENSE'),(root/'docs/desktop.md','README.md')]
if platform=='windows-x64':
    verify_standalone(root/'target/release'/exe)
    with zipfile.ZipFile(out/(name+'.zip'),'w',compression=zipfile.ZIP_DEFLATED) as archive:
        for path,filename in files:archive.write(path,name+'/'+filename)
else:
    with tarfile.open(out/(name+'.tar.gz'),'w:gz') as archive:
        for path,filename in files:archive.add(path,arcname=name+'/'+filename)
