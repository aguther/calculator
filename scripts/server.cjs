const http=require('node:http'),fs=require('node:fs'),path=require('node:path');
const baseRoot=path.resolve('dist');let root=baseRoot;let holdNext=false;const held=[];let installGate=null;let gateSequence=0;
const nextRoot=require('node:child_process').execFileSync('python',['scripts/prepare-update.py'],{encoding:'utf8'}).trim();
http.createServer((req,res)=>{
 if(req.method==='POST' && req.url==='/__test/hold-next'){root=nextRoot;holdNext=true;installGate='/__test/install-gate?attempt='+ ++gateSequence;res.end('holding');return;}
 if(req.method==='POST' && req.url==='/__test/release-next'){holdNext=false;for(const send of held.splice(0))send();res.end('released');return;}
 if(req.method==='POST' && req.url==='/__test/build-next'){root=nextRoot;res.end('next');return;}
 if(req.method==='POST' && req.url==='/__test/build-current'){root=baseRoot;installGate=null;res.end('current');return;}
 const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 if(pathname==='/__test/install-gate'){const send=()=>{res.setHeader('Cache-Control','no-store');res.end('ready');};if(holdNext)held.push(send);else send();return;}
 // Keep a request tied to its selected build while asynchronous reads finish.
 const requestRoot=root,requestGate=installGate;
 const file=path.resolve(requestRoot,'.'+(pathname==='/'?'/index.html':pathname));
 if(!file.startsWith(requestRoot+path.sep)){res.writeHead(403);res.end();return;}
 const ext=path.extname(file),mime={'.js':'text/javascript','.wasm':'application/wasm','.html':'text/html','.css':'text/css','.webmanifest':'application/manifest+json','.png':'image/png','.svg':'image/svg+xml','.json':'application/json'};
 const send=()=>fs.readFile(file,(err,data)=>{if(err){res.writeHead(404);res.end();return;}
 res.setHeader('Content-Type',mime[ext]||'application/octet-stream');res.setHeader('Cache-Control','no-store');
 res.setHeader('Content-Security-Policy',"default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; worker-src 'self'");if(requestRoot===nextRoot && pathname==='/sw.js' && requestGate)data=Buffer.from(data.toString().replace('const ASSETS = [','const ASSETS = ['+JSON.stringify(requestGate)+','));res.end(data);});
 if(holdNext && requestRoot===nextRoot && ext==='.wasm')held.push(send);else send();
}).listen(8765,'127.0.0.1');
