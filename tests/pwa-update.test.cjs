const {test}=require('node:test');
const assert=require('node:assert/strict');
const vm=require('node:vm');
const fs=require('node:fs');
const source=fs.readFileSync('web/app.js','utf8');

async function app({offline=false,failUpdate=false}={}){
 const windowEvents={},documentEvents={},workerEvents={},intervals=[];
 let checks=0,activations=0,reloads=0;
 const update={hidden:true,setAttribute(){},addEventListener(name,fn){this[name]=fn;}};
 const waiting={postMessage(message){assert.equal(message.type,'ACTIVATE_UPDATE');activations++;}};
 const registration={waiting:null,installing:null,addEventListener(name,fn){workerEvents[name]=fn;},async update(){checks++;if(failUpdate)throw Error('Offline');}};
 const navigator={onLine:!offline,serviceWorker:{controller:{},async getRegistration(){return registration;},async register(){return registration;},addEventListener(){}}};
 const document={hidden:false,documentElement:{setAttribute(){},style:{setProperty(){}}},querySelector(){return update;},addEventListener(name,fn){documentEvents[name]=fn;}};
 vm.runInNewContext(source,{
  window:{innerHeight:760,addEventListener(name,fn){windowEvents[name]=fn;}},document,navigator,
  fetch:async()=>{if(offline)throw Error('Offline');return {ok:true,headers:{get:()=> 'text/javascript'}};},
  location:{reload(){reloads++;}},setInterval(fn,ms){intervals.push({fn,ms});},
 });
 async function settle(){for(let i=0;i<12;i++)await Promise.resolve();}
 await settle();
 return {windowEvents,documentEvents,workerEvents,intervals,registration,waiting,navigator,document,update,checks:()=>checks,activations:()=>activations,reloads:()=>reloads,settle};
}

test('checks for releases on resume, focus, reconnect and periodically while visible',async()=>{
 const a=await app();
 const before=a.checks();
 a.windowEvents.pageshow?.();await a.settle();
 assert(a.checks()>before,'Returning to the PWA must check for updates');
 let prior=a.checks();a.windowEvents.focus?.();await a.settle();assert(a.checks()>prior);
 prior=a.checks();a.documentEvents.visibilitychange?.();await a.settle();assert(a.checks()>prior);
 assert.equal(a.intervals.length,1);assert.equal(a.intervals[0].ms,60000);
 prior=a.checks();a.intervals[0].fn();await a.settle();assert(a.checks()>prior);
 a.document.hidden=true;prior=a.checks();a.intervals[0].fn();await a.settle();assert.equal(a.checks(),prior);
 a.document.hidden=false;a.navigator.onLine=false;a.windowEvents.focus();await a.settle();assert.equal(a.checks(),prior);
 a.navigator.onLine=true;a.windowEvents.online?.();await a.settle();assert(a.checks()>prior);
 assert.equal(a.reloads(),0);
});

test('offline startup keeps existing registration connected for later update checks',async()=>{
 const a=await app({offline:true});
 a.navigator.onLine=true;a.windowEvents.online?.();await a.settle();
 assert(a.checks()>0,'An offline launch must still recover update detection when online');
 a.registration.waiting=a.waiting;a.windowEvents.focus?.();await a.settle();
 assert.equal(a.update.hidden,false);assert.equal(a.activations(),0);assert.equal(a.reloads(),0);
 a.update.click();assert.equal(a.activations(),1);
});

test('failed update checks remain retryable and concurrent wake events share one request',async()=>{
 const a=await app({failUpdate:true});let prior=a.checks();
 a.windowEvents.focus?.();await a.settle();assert(a.checks()>prior,'Failed checks must not stop later attempts');
 let finish,calls=0;
 a.registration.update=()=>{calls++;return new Promise(resolve=>{finish=resolve;});};
 a.windowEvents.focus();a.windowEvents.pageshow();a.documentEvents.visibilitychange();await a.settle();
 assert.equal(calls,1,'Concurrent wake events must share the in-flight request');finish();await a.settle();
 assert.equal(a.reloads(),0);
});
