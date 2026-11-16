const {test,expect}=require('@playwright/test');
async function ready(page){await page.goto('/');await expect(page.locator('#loading')).toHaveCount(0);await expect(page.locator('#calculator-status')).toContainText('0:00');await expect.poll(()=>page.evaluate(async()=>{const registration=await navigator.serviceWorker.getRegistration();return registration?.active?.state;})).toBe('activated');await expect.poll(()=>page.evaluate(()=>!!navigator.serviceWorker.controller)).toBe(true);await page.locator('canvas').focus();}
test('keyboard calculation, reload and offline shell',async({page,context})=>{
 await ready(page);await page.keyboard.type('45-130');await page.keyboard.press('Enter');
 await expect(page.locator('#calculator-status')).toContainText('−0:45');
 await page.reload();await expect(page.locator('#loading')).toHaveCount(0);
 await expect(page.locator('#calculator-status')).toContainText('−0:45');
 await page.evaluate(async()=>{await navigator.serviceWorker.ready;});
 await expect.poll(()=>page.evaluate(()=>!!navigator.serviceWorker.controller)).toBe(true);
 await context.setOffline(true);await page.reload();await expect(page.locator('#loading')).toHaveCount(0);
 await expect(page.locator('#calculator-status')).toContainText('−0:45');
 await context.setOffline(false);
});
for(const viewport of [{width:320,height:568},{width:400,height:760},{width:844,height:390}]){
 test(`canvas fits ${viewport.width}x${viewport.height}`,async({page})=>{
  await page.setViewportSize(viewport);await ready(page);
  const rect=await page.locator('canvas').boundingBox();expect(rect.width).toBeLessThanOrEqual(viewport.width);expect(rect.height).toBeLessThanOrEqual(viewport.height);
  await page.keyboard.type('1:30*2');await page.keyboard.press('Enter');await expect(page.locator('#calculator-status')).toContainText('3:00');
 });
}
test('storage unavailable keeps calculation working',async({page})=>{
 await page.addInitScript(()=>{Storage.prototype.setItem=function(){throw new DOMException('Denied','SecurityError');};});
 await ready(page);await page.keyboard.type('1:30*2');await page.keyboard.press('Enter');await expect(page.locator('#calculator-status')).toContainText('3:00');
});

test('visual evidence and pointer keypad',async({page})=>{
 await page.setViewportSize({width:400,height:760});await ready(page);
 await page.waitForTimeout(500);await page.screenshot({path:'.cache/calculator-original.png'});
 await page.mouse.click(380,20);await page.waitForTimeout(200);await page.screenshot({path:'.cache/calculator-settings.png'});
});

for(const viewport of [{width:320,height:568},{width:400,height:760}]){
 test(`bottom row pointer input ${viewport.width}`,async({page})=>{
  await page.setViewportSize(viewport);await ready(page);await page.keyboard.type('1+2');await expect(page.locator('#calculator-status')).toContainText('0:02');
  const canvas=await page.locator('canvas').boundingBox();await page.mouse.click(canvas.x+canvas.width*0.875,canvas.y+canvas.height-35,{delay:30});
  await expect(page.locator('#calculator-status')).toContainText('0:03');
 });
}

test('theme selection persists and blocks calculator keyboard while settings are open',async({page})=>{
 await page.setViewportSize({width:400,height:760});await ready(page);await page.keyboard.press('F1');
 await page.waitForTimeout(100);await page.mouse.click(200,227);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').theme)).toBe('Light');
 await page.keyboard.type('123');await expect(page.locator('#calculator-status')).toContainText('0:00');
 await page.keyboard.press('Escape');await page.reload();await expect(page.locator('#loading')).toHaveCount(0);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').theme)).toBe('Light');
 await page.screenshot({path:'.cache/calculator-light.png'});
});
test('complete second build updates both tabs and remains available offline',async({page,context,request})=>{
 await request.post('/__test/build-current');
 try {
  await ready(page);await page.keyboard.type('45');await page.keyboard.press('Enter');
  await expect(page.locator('#calculator-status')).toContainText('0:45');
  await expect.poll(()=>page.evaluate(()=>!!navigator.serviceWorker.controller)).toBe(true);
  const old=await page.evaluate(()=>{window.__controllerBeforeUpdate=navigator.serviceWorker.controller;return caches.keys();});
  let navigations=0;page.on('framenavigated',frame=>{if(frame===page.mainFrame())navigations++;});
  const second=await context.newPage();await second.goto('/');await expect(second.locator('#loading')).toHaveCount(0);
  // Headed Firefox suspends animation frames in the background tab.
  // Foreground the page before Playwright waits for stable clickable geometry.
  await page.bringToFront();
  await request.post('/__test/hold-next');
  await page.evaluate(async()=>{const r=await navigator.serviceWorker.ready;void r.update();});
  await expect.poll(()=>page.evaluate(async()=>{const r=await navigator.serviceWorker.getRegistration();return r.installing?.state;})).toBe('installing');
  await expect(page.locator('#update')).toBeHidden();
  await page.locator('canvas').focus();await page.keyboard.type('+5=');await expect(page.locator('#calculator-status')).toContainText('0:50');
  expect(navigations).toBe(0);
  await request.post('/__test/release-next');
  await expect(page.locator('#update')).toBeVisible();
  await expect.poll(()=>page.locator('#update img').evaluate(img=>img.complete && img.naturalWidth>0)).toBe(true);
  expect(navigations).toBe(0);expect(await page.evaluate(()=>navigator.serviceWorker.controller===window.__controllerBeforeUpdate)).toBe(true);
  expect(await page.evaluate(()=>caches.keys())).toContain(old[0]);
  await page.locator('canvas').focus();await page.keyboard.type('+10=');await expect(page.locator('#calculator-status')).toContainText('1:00');
  await expect.poll(async()=>{const box=await page.locator('canvas').boundingBox();return box.height;}).toBe(720);await Promise.all([
   page.waitForEvent('framenavigated',{predicate:frame=>frame===page.mainFrame(),timeout:15000}),
   second.waitForEvent('framenavigated',{predicate:frame=>frame===second.mainFrame(),timeout:15000}),
   page.locator('#update').click()
  ]);
  await expect.poll(()=>page.evaluate(()=>caches.keys()).catch(()=>old)).not.toEqual(old);
  await expect(page.locator('#loading')).toHaveCount(0);await expect(second.locator('#loading')).toHaveCount(0);
  await expect(page.locator('#calculator-status')).toContainText('1:00');
  await second.bringToFront();await context.setOffline(true);await second.reload({waitUntil:'domcontentloaded',timeout:15000});await expect(second.locator('#loading')).toHaveCount(0);
  await expect(second.locator('#calculator-status')).toContainText('1:00');await context.setOffline(false);
 } finally {await request.post('/__test/release-next',{timeout:5000}).catch(()=>{});await request.post('/__test/build-current',{timeout:5000}).catch(error=>console.warn('Test server reset failed:',error.message));}
});

test('overlapping touches commit in release order and canceled contacts do nothing',async({browser,browserName})=>{
 test.skip(browserName!=='chromium','CDP multi-touch is Chromium-specific; controller logic has platform-independent tests');
 const context=await browser.newContext({hasTouch:true,viewport:{width:400,height:760}});const page=await context.newPage();
 try{
  await ready(page);const cdp=await context.newCDPSession(page);
  const one={x:55,y:637,id:1},two={x:155,y:637,id:2};
  await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[one]});
  await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[one,two]});
  await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[two]});
  await cdp.send('Input.dispatchTouchEvent',{type:'touchEnd',touchPoints:[]});
  await expect(page.locator('#calculator-status')).toContainText('0:21');
  await page.keyboard.press('Escape');await expect(page.locator('#calculator-status')).toContainText('0:00');
  await cdp.send('Input.dispatchTouchEvent',{type:'touchStart',touchPoints:[one]});
  await cdp.send('Input.dispatchTouchEvent',{type:'touchCancel',touchPoints:[]});
  await page.waitForTimeout(50);await expect(page.locator('#calculator-status')).toContainText('0:00');
 }finally{await context.close();}
});

for(const capability of ['missing','throws']){
 test(`haptics ${capability} never interrupts calculation`,async({page})=>{
  await page.addInitScript(capability=>{
   localStorage.setItem('calculator-v1',JSON.stringify({haptics:true}));
   Object.defineProperty(navigator,'vibrate',{configurable:true,value:capability==='missing'?undefined:()=>{throw new Error('Vibration unavailable');}});
  },capability);
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  await ready(page);await page.keyboard.type('25+36=');
  await expect(page.locator('#calculator-status')).toContainText('1:01');expect(errors).toEqual([]);
 });
}

test('safe-area insets and update banner leave the full canvas visible',async({page})=>{
 await page.setViewportSize({width:390,height:844});await ready(page);
 await page.evaluate(()=>{const style=document.documentElement.style;style.setProperty('--safe-top','59px');style.setProperty('--safe-bottom','34px');style.setProperty('--safe-left','4px');style.setProperty('--safe-right','4px');});
 let canvas=await page.locator('canvas').boundingBox();expect(canvas.y).toBe(59);expect(canvas.x).toBe(4);expect(canvas.height).toBe(751);expect(canvas.width).toBe(382);
 await page.evaluate(()=>{document.querySelector('#update').hidden=false;});
 canvas=await page.locator('canvas').boundingBox();expect(canvas.height).toBe(751);expect(canvas.y+canvas.height).toBe(844-34);
 await page.locator('canvas').focus();await page.keyboard.type('25+36=');await expect(page.locator('#calculator-status')).toContainText('1:01');
});

test('settings switches toggle and persist',async({page})=>{
 await page.setViewportSize({width:400,height:760});await ready(page);await page.keyboard.press('F1');await page.waitForTimeout(100);
 await page.mouse.click(339,338);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').history_right)).toBe(false);
 await page.mouse.click(339,338);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').history_right)).toBe(true);
 await page.keyboard.press('Escape');await page.reload();await expect(page.locator('#loading')).toHaveCount(0);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').history_right)).toBe(true);
});

test('settings done button remains clickable after scrolling',async({page})=>{
 await page.setViewportSize({width:400,height:760});await ready(page);await page.keyboard.press('F1');await page.waitForTimeout(100);
 await page.mouse.move(200,350);await page.mouse.wheel(0,500);await page.waitForTimeout(100);
 await page.mouse.click(200,666);await page.locator('canvas').focus();await page.keyboard.type('25+36=');
 await expect(page.locator('#calculator-status')).toContainText('1:01');
});

test('sound context is created inside a real pointer gesture',async({page})=>{
 await page.setViewportSize({width:400,height:760});
 await page.addInitScript(()=>{
  localStorage.setItem('calculator-v1',JSON.stringify({sound:true}));window.audioGestures=[];
  for(const name of ['AudioContext','webkitAudioContext']){const Audio=window[name];if(Audio)window[name]=new Proxy(Audio,{construct(target,args){window.audioGestures.push(navigator.userActivation?.isActive);return Reflect.construct(target,args);}});}
 });
 await ready(page);test.skip(!await page.evaluate(()=>!!(window.AudioContext||window.webkitAudioContext)),'This browser runtime does not provide Web Audio; gesture and failure paths have separate unit tests');expect(await page.evaluate(()=>window.audioGestures)).toEqual([]);
 await page.mouse.click(55,500);await expect(page.locator('#calculator-status')).toContainText('0:07');
 await expect.poll(()=>page.evaluate(()=>window.audioGestures.length)).toBe(1);
 expect(await page.evaluate(()=>window.audioGestures[0])).toBe(true);
});

test('opaque iOS reference viewport uses height without a second top inset',async({page})=>{
 await page.setViewportSize({width:402,height:812});await ready(page);
 await page.evaluate(()=>{const s=document.documentElement.style;s.setProperty('--safe-top','0px');s.setProperty('--safe-bottom','34px');});
 await expect.poll(async()=>{const rect=await page.locator('canvas').boundingBox();return [rect.y,rect.height];}).toEqual([0,778]);
 await page.locator('canvas').focus();await page.keyboard.type('1+2');await expect(page.locator('#calculator-status')).toContainText('0:02');
 await page.mouse.click(352,743);await expect(page.locator('#calculator-status')).toContainText('0:03');
});


test('F2 and F3 switch modes while preserving separate calculations',async({page})=>{
 await ready(page);await page.keyboard.type('130=');await expect(page.locator('#calculator-status')).toContainText('1:30');
 await page.keyboard.press('F3');await page.keyboard.type('25=');await expect(page.locator('#calculator-status')).toContainText('25');
 await page.keyboard.press('F2');await expect(page.locator('#calculator-status')).toContainText('1:30');
 await page.keyboard.press('F1');await page.keyboard.press('F3');await expect(page.locator('#calculator-status')).toContainText('1:30');
 await page.keyboard.press('Escape');await page.keyboard.press('F3');await expect(page.locator('#calculator-status')).toContainText('25');
});

test('settings tabs expose help and diagnostics and return to preferences',async({page})=>{
 await page.setViewportSize({width:400,height:760});await ready(page);await page.keyboard.press('F1');await page.waitForTimeout(100);
 await page.mouse.click(200,139);await page.waitForTimeout(100);
 await page.screenshot({path:'.cache/calculator-help.png'});
 await page.mouse.click(320,139);await page.waitForTimeout(100);
 await page.screenshot({path:'.cache/calculator-diagnostics.png'});
 await page.mouse.click(80,139);await page.mouse.click(200,227);
 await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('calculator-v1')||'{}').theme)).toBe('Light');
 await page.keyboard.press('Escape');await page.keyboard.type('25+36=');await expect(page.locator('#calculator-status')).toContainText('1:01');
});

test('canvas function keys suppress browser defaults through actual event propagation',async({page})=>{
 await ready(page);
 for(const key of ['F1','F2','F3']){
  const prevented=await page.locator('canvas').evaluate((canvas,key)=>{
   const event=new KeyboardEvent('keydown',{key,bubbles:true,cancelable:true});canvas.dispatchEvent(event);return event.defaultPrevented;
  },key);
  expect(prevented,`${key} should not trigger browser Help or Find`).toBe(true);
 }
});
