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
