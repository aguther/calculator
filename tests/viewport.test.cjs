const {test}=require('node:test');const assert=require('node:assert/strict');const vm=require('node:vm');const fs=require('node:fs');
test('canvas height follows visible viewport and viewport changes',()=>{
 const values=new Map(),events={};const visualViewport={height:874,offsetTop:0,addEventListener:(event,handler)=>events[event]=handler};
 const window={innerHeight:874,visualViewport,addEventListener:(event,handler)=>events[event]=handler};
 const document={documentElement:{setAttribute:()=>{},style:{setProperty:(key,value)=>values.set(key,value)}},querySelector:()=>({}),addEventListener:()=>{}};
 vm.runInNewContext(fs.readFileSync('web/app.js','utf8'),{window,document,navigator:{}});
 assert.equal(values.get('--viewport-height'),'874px');
 visualViewport.height=500;events.resize();assert.equal(values.get('--viewport-height'),'500px');
 visualViewport.offsetTop=20;events.scroll();assert.equal(values.get('--viewport-top'),'20px');
});
