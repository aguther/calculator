import hashlib,pathlib
root=pathlib.Path(__file__).resolve().parents[1]/'release'
files=sorted(p for p in root.iterdir() if p.name.endswith(('.zip','.tar.gz')))
if len(files)!=2:raise SystemExit('Expected Windows and Linux release archives')
(root/'SHA256SUMS.txt').write_text(''.join(hashlib.sha256(p.read_bytes()).hexdigest()+'  '+p.name+'\n' for p in files))
