const {test}=require('node:test');const assert=require('node:assert/strict');const fs=require('node:fs');
test('audio is unlocked synchronously by a gesture and shared with later WASM clicks',async(t)=>{
 const prior={window:global.window,document:global.document};t.after(()=>Object.assign(global,prior));
 let gesture=false,created=0,started=0;const handlers={};
 class AudioContext {
  constructor(){created++;assert(gesture,'AudioContext created outside user activation');this.state='suspended';this.currentTime=0;this.sampleRate=48000;this.destination={};}
  resume(){assert(gesture);this.state='running';return Promise.resolve();}
  createBuffer(){return {};}
  createBufferSource(){return {connect(){},start(){}};}
  createOscillator(){return {frequency:{value:0},connect(){},start(){started++;},stop(){},disconnect(){}};}
  createGain(){return {gain:{value:0},connect(){},disconnect(){}};}
 }
 global.window={AudioContext};global.document={addEventListener:(event,fn)=>handlers[event]=fn};
 const source=fs.readFileSync('crates/calculator-app/src/audio.js','utf8');const audio=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64'));
 audio.configureSound(true);assert.equal(created,0);
 assert.doesNotThrow(()=>audio.playClick());assert.equal(created,0);
 gesture=true;handlers.pointerup();gesture=false;
 audio.playClick();assert.equal(created,1);assert.equal(started,1);
 audio.configureSound(false);audio.playClick();assert.equal(started,1);
});

test('unavailable audio does not throw during gesture or calculation',async(t)=>{
 const prior={window:global.window,document:global.document};t.after(()=>Object.assign(global,prior));const handlers={};
 global.window={AudioContext:class{constructor(){throw Error('Audio device unavailable');}}};global.document={addEventListener:(event,fn)=>handlers[event]=fn};
 const source=fs.readFileSync('crates/calculator-app/src/audio.js','utf8');const audio=await import('data:text/javascript;base64,'+Buffer.from(source).toString('base64')+'#unavailable');
 audio.configureSound(true);assert.doesNotThrow(()=>handlers.pointerup());assert.doesNotThrow(()=>audio.playClick());
});
