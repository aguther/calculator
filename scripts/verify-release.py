import pathlib,re,subprocess,sys,tomllib
root=pathlib.Path(__file__).resolve().parents[1]
tag=sys.argv[1]
version=tomllib.loads((root/'Cargo.toml').read_text())['workspace']['package']['version']
if not re.fullmatch(r'v\d+\.\d+\.\d+',tag) or tag!='v'+version:
    raise SystemExit('Tag must match the stable Cargo workspace version')
subprocess.run(['git','merge-base','--is-ancestor','HEAD','origin/main'],cwd=root,check=True)
print('Release version and main ancestry verified:',tag)
