"""Verify that the Windows binary does not need a separately installed VC runtime."""
from pathlib import Path
import struct,sys
def dependencies(file):
    binary=Path(file).read_bytes()
    pe=struct.unpack_from('<I',binary,0x3c)[0]
    count=struct.unpack_from('<H',binary,pe+6)[0]
    optional_size=struct.unpack_from('<H',binary,pe+20)[0]
    optional=pe+24
    magic=struct.unpack_from('<H',binary,optional)[0]
    imports_rva=struct.unpack_from('<I',binary,optional+(112 if magic==0x20b else 96)+8)[0]
    ranges=[]
    for index in range(count):
        size,base,raw_size,raw=struct.unpack_from('<IIII',binary,optional+optional_size+index*40+8)
        ranges.append((base,max(size,raw_size),raw))
    def offset(rva):
        for base,size,raw in ranges:
            if base<=rva<base+size:return raw+rva-base
        raise ValueError('Invalid PE RVA')
    cursor=offset(imports_rva);names=[]
    while any(binary[cursor:cursor+20]):
        start=offset(struct.unpack_from('<I',binary,cursor+12)[0])
        names.append(binary[start:binary.index(0,start)].decode('ascii'))
        cursor+=20
    return names
def verify_standalone(file):
    names=dependencies(file)
    forbidden=[name for name in names if name.upper().startswith(('VCRUNTIME','MSVCP','CONCRT'))]
    if forbidden:raise SystemExit('Non-portable VC runtime dependencies: '+', '.join(forbidden))
    return names
if __name__=='__main__':print('Windows system DLL imports:',', '.join(verify_standalone(sys.argv[1])))
