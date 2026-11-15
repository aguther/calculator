"""Complete local release gate. Install Trunk/npm/browser engines beforehand."""
import os,subprocess,sys
from pathlib import Path
root=Path(__file__).resolve().parents[1]
commands=[['cargo','fmt','--all','--','--check'],['cargo','clippy','--workspace','--all-targets','--locked','--','-D','warnings'],['cargo','clippy','--locked','-p','calculator','--target','wasm32-unknown-unknown','--','-D','warnings'],['cargo','test','--workspace','--locked'],['node','scripts/reference-fixtures.cjs','--check'],[sys.executable,'scripts/test_web.py'],['node','--test','tests/sw.test.cjs','tests/viewport.test.cjs','tests/audio.test.cjs'],[sys.executable,'scripts/build-web.py'],['cargo','build','--release','--locked','-p','calculator'],['cargo','run','--release','--locked','-p','calculator','--example','smoke']]
for command in commands:
    print('CHECK:', ' '.join(command),flush=True);subprocess.run(command,cwd=root,check=True)
subprocess.run(['npm.cmd' if os.name=='nt' else 'npm','test'],cwd=root,check=True)
print('All local release checks passed.')
