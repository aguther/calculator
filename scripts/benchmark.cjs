// Local release comparison; all data is synthetic. Servers bind loopback only.
const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),zlib=require('node:zlib');
const {spawn}=require('node:child_process');const {chromium}=require('@playwright/test');
const reference=process.env.CALCULATOR_REFERENCE_ROOT||path.resolve('../simple-calculator');
function files(dir){return fs.readdirSync(dir,{withFileTypes:true}).flatMap(f=>f.isDirectory()?files(path.join(dir,f.name)):[path.join(dir,f.name)]);}
function size(root){const assets=files(root).filter(f=>!['_headers','build-info.json','asset-manifest.json','THIRD_PARTY_LICENSES.txt'].includes(path.basename(f)));return {raw_bytes:assets.reduce((n,f)=>n+fs.statSync(f).size,0),gzip_bytes:assets.reduce((n,f)=>n+zlib.gzipSync(fs.readFileSync(f),{level:9}).length,0)};}
const median=values=>[...values].sort((a,b)=>a-b)[Math.floor(values.length/2)];
async function main(){
 if(!fs.existsSync(path.join(reference,'dist/index.html')))throw Error('Reference production build missing; set CALCULATOR_REFERENCE_ROOT');
 const servers=[spawn(process.execPath,[path.join(reference,'test/server.js'),path.resolve('dist'),'8875'],{windowsHide:true,stdio:'ignore'}),spawn(process.execPath,[path.join(reference,'test/server.js'),path.join(reference,'dist'),'8876'],{windowsHide:true,stdio:'ignore'})];
 let browser;
 try{
  for(const port of [8875,8876])for(let i=0;i<50;i++){try{if((await fetch(`http://127.0.0.1:${port}`)).ok)break;}catch{}await new Promise(r=>setTimeout(r,100));}
  browser=await chromium.launch({args:['--use-gl=angle','--use-angle=swiftshader','--enable-webgl','--ignore-gpu-blocklist']});
  const result={environment:{date:new Date().toISOString(),platform:os.platform(),release:os.release(),cpu:os.cpus()[0].model,browser:browser.version(),viewport:'400x760',transport:'loopback HTTP; no throttling',samples:10},variants:{}};
  for(const [name,port,root] of [['rust',8875,path.resolve('dist')],['javascript',8876,path.join(reference,'dist')]]){
   const cold=[],warm=[],reaction=[];let lastContext;
   const status=name==='rust'?'#calculator-status':'#current';
   async function ready(page){if(name==='rust'){await page.waitForFunction(()=>!document.querySelector('#loading'));await page.locator('canvas').focus();}else{await page.waitForFunction(()=>!!window.CalculatorCore&&!!window.CalculatorStateStore);}}
   for(let i=0;i<10;i++){
    const context=await browser.newContext({viewport:{width:400,height:760}});const page=await context.newPage();
    let start=performance.now();await page.goto(`http://127.0.0.1:${port}`);await ready(page);await page.keyboard.type('45-130');await page.keyboard.press('Enter');await page.waitForFunction(s=>document.querySelector(s).textContent.includes('−0:45'),status);cold.push(performance.now()-start);
    await page.evaluate(async()=>{await navigator.serviceWorker.ready;});
    await page.waitForFunction(()=>!!navigator.serviceWorker.controller);
    start=performance.now();await page.reload();await ready(page);await page.waitForFunction(s=>document.querySelector(s).textContent.includes('−0:45'),status);warm.push(performance.now()-start);
    start=performance.now();await page.keyboard.type('7');await page.waitForFunction(s=>document.querySelector(s).textContent.startsWith('0:07'),status);reaction.push(performance.now()-start);
    if(i===9)lastContext=context;else await context.close();
   }
   const page=lastContext.pages()[0];const cdp=await lastContext.newCDPSession(page);await cdp.send('Performance.enable');
   const metrics=async()=>Object.fromEntries((await cdp.send('Performance.getMetrics')).metrics.map(m=>[m.name,m.value]));
   const before=await metrics();await page.keyboard.press('Escape');const workloadStart=performance.now();await page.keyboard.type('1+'.repeat(900));await page.waitForFunction(name=>{try{const state=JSON.parse(localStorage.getItem(name==='rust'?'calculator-v1':'zeitrechner-state-v5'));return (name==='rust'?state.calculator.time.steps:state.saved.time.steps).length===900;}catch{return false;}},name);
   const workloadWall=performance.now()-workloadStart;
   const after=await metrics();const longReaction=[];
   for(let i=0;i<10;i++){let start=performance.now();await page.keyboard.type('2');await page.waitForFunction(s=>document.querySelector(s).textContent.startsWith('0:02'),status);longReaction.push(performance.now()-start);await page.keyboard.press('Delete');await page.waitForFunction(s=>document.querySelector(s).textContent.startsWith('0:00'),status);}
   result.variants[name]={...size(root),cold_ready_and_first_calculation_ms:median(cold),warm_reload_ms:median(warm),short_input_roundtrip_ms:median(reaction),long_input_roundtrip_ms:median(longReaction),history_rows_verified:900,history_workload_wall_ms:workloadWall,history_renderer_task_ms:(after.TaskDuration-before.TaskDuration)*1000,js_heap_after_history_bytes:after.JSHeapUsedSize,notes:'Roundtrip includes Playwright and rendering. JS heap excludes full WASM/GPU/process memory. Warm reload includes persisted state and worker cache.'};
   await lastContext.close();
  }
  fs.mkdirSync('.cache',{recursive:true});fs.writeFileSync('.cache/benchmark.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result,null,2));
 }finally{if(browser)await browser.close();for(const server of servers)server.kill();}
}
main().catch(error=>{console.error(error);process.exitCode=1;});
