const http=require('node:http'),fs=require('node:fs'),path=require('node:path');
const baseRoot=path.resolve('dist');let root=baseRoot;
const nextRoot=require('node:child_process').execFileSync('python',['scripts/prepare-update.py'],{encoding:'utf8'}).trim();
http.createServer((req,res)=>{
 if(req.method==='POST' && req.url==='/__test/build-next'){root=nextRoot;res.end('next');return;}
 if(req.method==='POST' && req.url==='/__test/build-current'){root=baseRoot;res.end('current');return;}
 const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
 const file=path.resolve(root,'.'+(pathname==='/'?'/index.html':pathname));
 if(!file.startsWith(root+path.sep)){res.writeHead(403);res.end();return;}
 const ext=path.extname(file),mime={'.js':'text/javascript','.wasm':'application/wasm','.html':'text/html','.css':'text/css','.webmanifest':'application/manifest+json','.png':'image/png','.json':'application/json'};
 fs.readFile(file,(err,data)=>{if(err){res.writeHead(404);res.end();return;}
 res.setHeader('Content-Type',mime[ext]||'application/octet-stream');res.setHeader('Cache-Control','no-store');
 res.setHeader('Content-Security-Policy',"default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; worker-src 'self'");res.end(data);});
}).listen(8765,'127.0.0.1');
