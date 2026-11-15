"""Collect registry license material for built distributions, including embedded fonts."""
from pathlib import Path
import os,tomllib
ROOT=Path(__file__).resolve().parents[1]
def collect():
    cargo=Path(os.environ.get('CARGO_HOME',Path.home()/'.cargo'))
    registries=list((cargo/'registry/src').glob('*'))
    lock=tomllib.loads((ROOT/'Cargo.lock').read_text(encoding='utf-8'))
    sections=['THIRD-PARTY LICENSE MATERIAL\nCalculator uses the following Cargo registry packages.\nIncludes available target-specific packages and embedded font licenses.\n']
    for package in sorted(lock['package'],key=lambda p:(p['name'],p['version'])):
        if not package.get('source','').startswith('registry+'):continue
        name=package['name']+'-'+package['version']
        source=next((r/name for r in registries if (r/name).is_dir()),None)
        if source is None:continue # Packages for targets not built need not be present.
        metadata=tomllib.loads((source/'Cargo.toml').read_text(encoding='utf-8'))['package']
        texts=set()
        for file in source.iterdir():
            if file.is_file() and file.name.upper().startswith(('LICENSE','LICENCE','COPYING','NOTICE','COPYRIGHT')):texts.add(file)
        license_file=metadata.get('license-file')
        if license_file and (source/license_file).is_file():texts.add(source/license_file)
        if package['name']=='epaint_default_fonts':texts.update(source.rglob('*.txt'))
        sections.append('\n'+'='*72+'\n'+name+'\nLicense: '+metadata.get('license','See license file')+'\n')
        for file in sorted(texts):
            sections.append('\n--- '+file.relative_to(source).as_posix()+' ---\n'+file.read_text(encoding='utf-8',errors='replace')+'\n')
    return '\n'.join(sections)
if __name__=='__main__':print(collect())
