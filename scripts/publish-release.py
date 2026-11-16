"""Resume a release safely; assets with different bytes are never overwritten."""
import hashlib,json,pathlib,re,subprocess,sys,tempfile
root=pathlib.Path(__file__).resolve().parents[1]
tag=sys.argv[1]
notes=root/'docs'/'release-notes'/(tag.removeprefix('v')+'.md')
if not notes.is_file() or not notes.read_text(encoding='utf-8').strip():
    raise SystemExit('Missing release notes for '+tag)
def gh(*args,**kwargs):return subprocess.run(['gh',*args],check=True,**kwargs)
# This check runs inside the global production publication lock.
releases=json.loads(gh('release','list','--exclude-pre-releases','--limit','100','--json','tagName',capture_output=True,text=True).stdout)
version=lambda value:tuple(map(int,value.removeprefix('v').split('.')))
for release in releases:
    previous=release['tagName']
    if re.fullmatch(r'v\d+\.\d+\.\d+',previous) and version(previous)>version(tag):
        raise SystemExit('Refusing to deploy an older release than the published or reserved version '+previous)
result=subprocess.run(['gh','release','view',tag],capture_output=True)
if result.returncode:
    gh('release','create',tag,'--verify-tag','--title','Calculator '+tag,'--notes-file',str(notes),'--draft')
else:
    gh('release','edit',tag,'--notes-file',str(notes))
for file in sorted((root/'release').iterdir()):
    with tempfile.TemporaryDirectory() as temp:
        existing=subprocess.run(['gh','release','download',tag,'--pattern',file.name,'--dir',temp],capture_output=True)
        if existing.returncode==0:
            old=pathlib.Path(temp)/file.name
            if hashlib.sha256(old.read_bytes()).digest()!=hashlib.sha256(file.read_bytes()).digest():
                raise SystemExit('Existing asset differs; refusing replacement: '+file.name)
        else:gh('release','upload',tag,str(file))
