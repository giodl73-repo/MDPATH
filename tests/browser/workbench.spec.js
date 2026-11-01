import {test,expect} from '@playwright/test';
import {readFile,mkdir} from 'node:fs/promises';
const fixtures=JSON.parse(await readFile('.cache/native-references.json','utf8'));
const examples=JSON.parse(await readFile('dist/MDPATH/examples.json','utf8'));
async function open(page){await page.goto('./');await expect(page.locator('#results')).toBeVisible();}
async function download(page,id){const waiting=page.waitForEvent('download');await page.locator('#'+id).click();return readFile(await(await waiting).path(),'utf8');}
test('real WASM exactly matches all native response fixtures',async({page})=>{
 await page.goto('./');const actual=await page.evaluate(async cases=>{
  const worker=new Worker(new URL('worker.js',location.href),{type:'module'});await new Promise((resolve,reject)=>{worker.onmessage=e=>e.data.type==='ready'?resolve():reject(e.data);worker.onerror=reject;});
  const results=[];for(let id=0;id<cases.length;id++)results.push(await new Promise((resolve,reject)=>{worker.onmessage=e=>resolve(e.data.canonical);worker.onerror=reject;worker.postMessage({type:'resolve',id,request:cases[id].input});}));worker.terminate();return results;
 },fixtures);expect(actual).toEqual(fixtures.map(f=>f.canonical));
});
for(const name of Object.keys(examples))test('example '+name,async({page})=>{
 await open(page);await page.locator(`[data-example="${name}"]`).click();await page.locator('#run').click();await expect(page.locator('#report-download')).toBeEnabled();const canonical=fixtures.find(f=>f.name===name).canonical;expect(await download(page,'report-download')).toBe(canonical);
 const report=JSON.parse(canonical);if(report.ok){await expect(page.locator('#content')).toHaveText(report.resolved.content);expect(await download(page,'content-download')).toBe(report.resolved.content);}else{await expect(page.locator('#diagnostic')).toContainText(report.diagnostic.code);await expect(page.locator('#results')).toBeHidden();}
});
test('own Markdown and path binding, then edit invalidates stale exports',async({page})=>{
 await open(page);await page.locator('#markdown').fill('# Own\n\nMy own paragraph.\n');await page.locator('#path').fill('mine.md');await page.locator('#uri').fill('md://mine.md#own:text:0');
 await expect(page.locator('#results')).toBeHidden();await expect(page.locator('#report-download')).toBeDisabled();await page.locator('#run').click();await expect(page.locator('#content')).toHaveText('My own paragraph.');
 const req=JSON.parse(await download(page,'request-download'));expect(req.path).toBe('mine.md');await page.locator('#uri').fill('md://other.md#own:text:0');await page.locator('#run').click();await expect(page.locator('#diagnostic')).toContainText('DocumentMismatch');
});
test('opened original retains CRLF and Unicode through failed replacement',async({page})=>{
 await open(page);const source='# Notes\r\n\r\nCafé 🌿\r\n';await page.locator('#document-file').setInputFiles({name:'notes.md',mimeType:'text/markdown',buffer:Buffer.from(source)});await expect.poll(()=>page.locator('#markdown').inputValue()).toContain('Café');
 expect(await download(page,'original-download')).toBe(source);await page.locator('#document-file').setInputFiles({name:'bad.md',mimeType:'text/markdown',buffer:Buffer.from([255])});await expect(page.locator('#error')).toContainText('valid UTF-8');expect(await download(page,'original-download')).toBe(source);
});
test('oversize file retains draft',async({page})=>{
 await open(page);const before=await download(page,'original-download');await page.locator('#document-file').setInputFiles({name:'big.md',mimeType:'text/markdown',buffer:Buffer.alloc(131073,32)});await expect(page.locator('#error')).toContainText('128 KiB');expect(await download(page,'original-download')).toBe(before);
});
test('late file cannot overwrite an edit or reset',async({page})=>{
 await open(page);await page.evaluate(()=>{File.prototype.arrayBuffer=()=>new Promise(resolve=>window.finishRead=()=>resolve(new TextEncoder().encode('stale').buffer));});
 await page.locator('#document-file').setInputFiles({name:'held.md',mimeType:'text/markdown',buffer:Buffer.from('held')});await page.locator('#markdown').fill('newer draft');await page.evaluate(()=>window.finishRead());expect(await page.locator('#markdown').inputValue()).toBe('newer draft');await expect(page.locator('#run')).toBeEnabled();
 await page.locator('#document-file').setInputFiles({name:'held.md',mimeType:'text/markdown',buffer:Buffer.from('held')});await page.locator('#reset').click();await page.evaluate(()=>window.finishRead());expect(await page.locator('#markdown').inputValue()).toBe(examples.named.markdown);
});
test('file deadline recovers and retains draft',async({page})=>{
 await open(page);await page.clock.install();const before=await page.locator('#markdown').inputValue();await page.evaluate(()=>{File.prototype.arrayBuffer=()=>new Promise(()=>{});});await page.locator('#document-file').setInputFiles({name:'held.md',mimeType:'text/markdown',buffer:Buffer.from('held')});await expect(page.locator('#run')).toBeDisabled();await page.clock.fastForward(5001);await expect(page.locator('#error')).toContainText('five seconds');await expect(page.locator('#run')).toBeEnabled();expect(await page.locator('#markdown').inputValue()).toBe(before);
});
test('startup timeout retains independent downloads',async({page})=>{
 await page.addInitScript(()=>{window.Worker=class{postMessage(){}terminate(){}};});await page.clock.install();await page.goto('./');await page.locator('#markdown').fill('retained');await page.clock.fastForward(12001);await expect(page.locator('#retry')).toBeVisible();expect(await download(page,'original-download')).toBe('retained');expect(JSON.parse(await download(page,'request-download')).markdown).toBe('retained');await expect(page.locator('a[href="./source.zip"]')).toBeVisible();
});
test('compute timeout survives edits and fences late result',async({page})=>{
 await page.addInitScript(()=>{window.Worker=class{constructor(){window.fakeWorker=this;setTimeout(()=>this.onmessage?.({data:{type:'ready'}}),20);}postMessage(data){this.last=data;}terminate(){}};});await page.clock.install();await page.goto('./');await page.clock.fastForward(21);await expect(page.locator('#run')).toBeDisabled();await page.locator('#markdown').fill('retained');await page.clock.fastForward(10001);await expect(page.locator('#retry')).toBeVisible();await page.evaluate(()=>window.fakeWorker.onmessage({data:{id:window.fakeWorker.last.id,canonical:'{}'}}));await expect(page.locator('#results')).toBeHidden();await expect(page.locator('#report-download')).toBeDisabled();expect(await download(page,'original-download')).toBe('retained');
});
test('keyboard chooses an error example and restores named address',async({page})=>{
 await open(page);await page.locator('[data-example="numeric-stale"]').focus();await page.keyboard.press('Enter');await page.locator('#run').focus();await page.keyboard.press('Enter');await expect(page.locator('#diagnostic')).toContainText('NumericUriStale');await page.locator('#reset').focus();await page.keyboard.press('Enter');await expect(page.locator('#uri')).toBeFocused();await page.locator('#run').focus();await page.keyboard.press('Enter');await expect(page.locator('#results')).toBeVisible();
});
test('HTML and Unicode remain inert resolved text',async({page})=>{
 await open(page);await page.locator('#markdown').fill('# Notes\n\nCafé 🌿 <img src=x onerror=alert(1)>\n');await page.locator('#uri').fill('md://guide.md#notes:text:0');await page.locator('#run').click();await expect(page.locator('#content')).toHaveText('Café 🌿 <img src=x onerror=alert(1)>');expect(await page.locator('#content img').count()).toBe(0);
});
test('without JavaScript source and examples remain available',async({browser,baseURL})=>{
 const context=await browser.newContext({javaScriptEnabled:false});const page=await context.newPage();await page.goto(baseURL);await expect(page.locator('#static-help')).toBeVisible();await expect(page.locator('#markdown')).toContainText('request-router');const response=await page.request.get(new URL('source.zip',baseURL).href);expect((await response.body()).subarray(0,2).toString()).toBe('PK');await context.close();
});
test('WASM magic, MIME and independent artifact routes',async({request,baseURL})=>{
 const response=await request.get(new URL('pkg/mdpath_web_bg.wasm',baseURL).href);expect(response.headers()['content-type']).toContain('application/wasm');expect([...((await response.body()).subarray(0,4))]).toEqual([0,97,115,109]);
 for(const path of ['BUILD.md','LICENSE','THIRD-PARTY-NOTICES.txt','rust/COPYRIGHT-library.html','examples.json','source/src/resolver.rs','source/web/adapter/Cargo.lock'])expect((await request.get(new URL(path,baseURL).href)).ok(),path).toBeTruthy();
});
for(const width of [1440,390])test('readable results at '+width,async({page})=>{
 await page.setViewportSize({width,height:1000});await open(page);expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBeTruthy();await mkdir('.cache/review',{recursive:true});await page.screenshot({path:`.cache/review/results-${width}.png`,fullPage:true});
});
