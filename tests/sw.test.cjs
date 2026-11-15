const {test}=require('node:test');const assert=require('node:assert/strict');const vm=require('node:vm');const fs=require('node:fs');
function worker(fail=false){
 const events={},deleted=[];let skipped=false;
 const self={addEventListener:(name,handler)=>events[name]=handler,location:{origin:'https://example.test'},clients:{claim:async()=>{}},skipWaiting:()=>{skipped=true;}};
 const caches={open:async()=>({addAll:async assets=>{assert(assets.length);if(fail)throw Error('Network unavailable');}}),delete:async name=>deleted.push(name),keys:async()=>['calculator-old','unrelated','calculator-next']};
 vm.runInNewContext(fs.readFileSync('web/sw.template.js','utf8').replace('__VERSION__','"next"').replace('__ASSETS__','["/index.html","/app.wasm"]'),{self,caches,Request:class{constructor(url){this.url=url;}},URL,fetch:async()=>{}});
 return {events,deleted,skipped:()=>skipped};
}
test('incomplete installation removes new cache and keeps old version',async()=>{
 const w=worker(true);let completion;w.events.install({waitUntil:p=>completion=p});await assert.rejects(completion,/Network/);assert.deepEqual(w.deleted,['calculator-next']);assert.equal(w.skipped(),false);
});
test('update activates only when requested and cleans only calculator caches',async()=>{
 const w=worker();let completion;w.events.install({waitUntil:p=>completion=p});await completion;assert.equal(w.skipped(),false);
 w.events.message({data:{type:'ACTIVATE_UPDATE'}});assert.equal(w.skipped(),true);
 w.events.activate({waitUntil:p=>completion=p});await completion;assert.deepEqual(w.deleted,['calculator-old']);
});


test('navigation unwraps a cached index redirect from a static host',async()=>{
 const events={};const redirected=new Response('<html>calculator</html>',{headers:{'Content-Type':'text/html'}});
 Object.defineProperty(redirected,'redirected',{value:true});
 const caches={open:async()=>({match:async key=>key==='/index.html'?redirected:undefined})};
 const self={addEventListener:(name,handler)=>events[name]=handler,location:{origin:'https://example.test'}};
 vm.runInNewContext(fs.readFileSync('web/sw.template.js','utf8').replace('__VERSION__','"next"').replace('__ASSETS__','[]'),{self,caches,URL,Response,fetch:async()=>{throw Error('Offline');}});
 let completion;events.fetch({request:{method:'GET',url:'https://example.test/',mode:'navigate'},respondWith:p=>completion=p});
 const result=await completion;assert.equal(result.redirected,false);assert.equal(result.status,200);
 assert.equal(result.headers.get('Content-Type'),'text/html');assert.equal(await result.text(),'<html>calculator</html>');
});
