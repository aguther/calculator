const {test,expect}=require('@playwright/test');
async function ready(page){await page.goto('/');await expect(page.locator('#loading')).toHaveCount(0);await expect(page.locator('#calculator-status')).toContainText('0:00');await page.evaluate(async()=>{await navigator.serviceWorker.ready;});await expect.poll(()=>page.evaluate(()=>!!navigator.serviceWorker.controller)).toBe(true);await page.locator('canvas').focus();}
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
 await page.waitForTimeout(100);await page.mouse.click(40,153);
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
  const old=await page.evaluate(()=>caches.keys());
  const second=await context.newPage();await second.goto('/');await expect(second.locator('#loading')).toHaveCount(0);
  await request.post('/__test/build-next');
  await page.evaluate(async()=>{const r=await navigator.serviceWorker.ready;await r.update();});
  await expect(page.locator('#update')).toBeVisible();await expect.poll(async()=>{const box=await page.locator('canvas').boundingBox();return box.height;}).toBeLessThan(720);await page.locator('#update').click();
  await expect.poll(()=>page.evaluate(()=>caches.keys()).catch(()=>old)).not.toEqual(old);
  await expect(page.locator('#loading')).toHaveCount(0);await expect(second.locator('#loading')).toHaveCount(0);
  await expect(page.locator('#calculator-status')).toContainText('0:45');
  await context.setOffline(true);await second.reload();await expect(second.locator('#loading')).toHaveCount(0);
  await expect(second.locator('#calculator-status')).toContainText('0:45');await context.setOffline(false);
 } finally {await request.post('/__test/build-current');}
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
