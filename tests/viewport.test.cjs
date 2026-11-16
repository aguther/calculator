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

test('calculator function keys prevent browser shortcuts only on the canvas',()=>{
 const handlers={};
 const document={documentElement:{setAttribute(){},style:{setProperty(){}}},querySelector:()=>({}),addEventListener:(name,fn)=>{handlers[name]=fn;}};
 vm.runInNewContext(fs.readFileSync('web/app.js','utf8'),{window:{innerHeight:760,addEventListener(){}},document,navigator:{}});
 for(const [key,target,ctrlKey,want] of [['F1','calculator-canvas',false,true],['F2','calculator-canvas',false,true],['F3','calculator-canvas',false,true],['F5','calculator-canvas',false,false],['F3','update',false,false],['F3','calculator-canvas',true,false]]){
  let prevented=false;
  handlers.keydown?.({key,target:{id:target},ctrlKey,preventDefault(){prevented=true;}});
  assert.equal(prevented,want,`${key}, ${target}, ctrl=${ctrlKey}`);
 }
});
