// SPDX-License-Identifier: GPL-3.0-or-later
import {fileURLToPath} from 'node:url';
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import assert from 'node:assert/strict';
const docs=fileURLToPath(new URL('../',import.meta.url));
const root=path.join(docs,'_site');
const out=path.join(docs,'.verification/landing');
await fs.mkdir(out,{recursive:true});
const server=http.createServer(async(req,res)=>{try{const route=new URL(req.url,'http://localhost').pathname;const file=path.resolve(root,'.'+route,route.endsWith('/')?'index.html':'');assert(file.startsWith(root+'/'));const types={'.html':'text/html','.css':'text/css','.js':'text/javascript','.png':'image/png','.json':'application/json','.svg':'image/svg+xml','.wasm':'application/wasm'};const content=await fs.readFile(file);res.writeHead(200,{'Content-Type':types[path.extname(file)]||'application/octet-stream'});res.end(content);}catch{console.log('HTTP 404',req.url);res.writeHead(404);res.end();}});
server.listen(0,'127.0.0.1');await once(server,'listening');
const origin=`http://127.0.0.1:${server.address().port}`;
const profile=await fs.mkdtemp(path.join(os.tmpdir(),'slint-kit-docs-review-'));
const chrome=spawn(process.env.CHROME_PATH||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--no-first-run','--no-default-browser-check','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
const pause=ms=>new Promise(r=>setTimeout(r,ms));let socket;let seq=0;const pending=new Map();const results=[];
function call(method,params={}){const id=++seq;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(new Error(method+' timed out'));},20000);pending.set(id,{resolve,reject,timer});socket.send(JSON.stringify({id,method,params}));});}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(r.exceptionDetails)throw Error(JSON.stringify(r.exceptionDetails));return r.result.value;}
async function navigate(route){await call('Page.navigate',{url:origin+route});for(let n=0;n<100;n++){if(await evaluate(`location.pathname===${JSON.stringify(route.split('?')[0])}&&document.readyState==='complete'`).catch(()=>false))return;await pause(50);}throw Error('Navigation timeout '+route);}
try{
 let port;for(let n=0;n<100;n++){try{port=(await fs.readFile(path.join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];break;}catch{await pause(50);}}
 assert(port);const tabs=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();socket=new WebSocket(tabs.find(t=>t.type==='page').webSocketDebuggerUrl);socket.addEventListener('message',e=>{const d=JSON.parse(e.data),p=pending.get(d.id);if(p){clearTimeout(p.timer);pending.delete(d.id);d.error?p.reject(Error(JSON.stringify(d.error))):p.resolve(d.result);}});await once(socket,'open');await call('Page.enable');

 const color = async selector => evaluate(`(()=>{const s=getComputedStyle(document.querySelector(${JSON.stringify(selector)}));return {background:s.backgroundColor,text:s.color}})()`);
 const luminance = rgb => rgb.match(/[\d.]+/g).slice(0,3).map(Number).map(v=>{v/=255;return v<=0.04045?v/12.92:((v+0.055)/1.055)**2.4}).reduce((sum,v,i)=>sum+v*[.2126,.7152,.0722][i],0);
 const order = () => evaluate('[...document.querySelectorAll("[data-hero-table] tbody tr")].map(row=>Number(row.dataset.projectKey))');
 const click = async selector => {const p=await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.scrollIntoView({block:'center'});const r=e.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2}})()`);await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...p});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...p});};
 await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
 for(const locale of ['en','fr']) {
  const prefix=locale==='fr'?'/fr':'';
  for(const [theme,width,text200] of [['light',1280,false],['dark',1280,false],['dark',375,false],['light',375,true],['dark',375,true]]) {
   await call('Emulation.setDeviceMetricsOverride',{width,height:1000,deviceScaleFactor:1,mobile:false});await navigate(`${prefix}/?theme=${theme}`);
   if(text200)await evaluate('document.documentElement.style.fontSize="32px"');
   assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'),`${locale}/${theme}/${width}/${text200}: overflow`);
   assert.equal(await evaluate('document.querySelector("[data-hero-table]").dataset.playback'),'paused','reduced motion must prevent autoplay');
   assert.equal(await evaluate('document.querySelectorAll("main img, main canvas").length'),0,'illustrations must use HTML/CSS');
   assert(await evaluate('[...document.querySelectorAll(".landing-contract code")].every(c=>c.children.length===0&&c.textContent.includes("\\n    "))'),'code preserves indentation and contains no injected paragraph markup');
   if (width>=1000) {
    const overlap=await evaluate(`(()=>{const a=document.querySelector('.landing-table').getBoundingClientRect(),b=document.querySelector('.landing-money').getBoundingClientRect(),stage=document.querySelector('.landing-stage').getBoundingClientRect();return{intersects:Math.min(a.right,b.right)>Math.max(a.left,b.left)&&Math.min(a.bottom,b.bottom)>Math.max(a.top,b.top),compact:stage.height<a.height+b.height}})()`);
    assert(overlap.intersects&&overlap.compact,'desktop hero cards must overlap in a compact composition');
    for (const selector of ['[data-hero-selection]','[data-hero-playback]','.landing-table figcaption a']) {
     assert(await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.scrollIntoView({block:'center'});const range=document.createRange();range.selectNodeContents(e);const r=range.getBoundingClientRect();const hit=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);return hit===e||e.contains(hit)})()`),`${selector} must stay visible above the overlapping card`);
    }
    await evaluate('window.scrollTo(0,0)');
   }
   const shot=await call('Page.captureScreenshot',{format:'png'});await fs.writeFile(path.join(out,`${locale}-landing-${theme}-${width}-${text200?'text200':'normal'}.png`),Buffer.from(shot.data,'base64'));
   if (text200) {
    const height=await evaluate('document.querySelector(".hero-animation-controls").getBoundingClientRect().height');
    for (let n=0;n<3;n++) {await click('[data-sort-column="tasks"]');assert.equal(await evaluate('document.querySelector(".hero-animation-controls").getBoundingClientRect().height'),height,'narration must not shift layout at 200% text');}
   }
   const license=await color('.landing-license');const foreground=luminance(license.text),background=luminance(license.background);assert((Math.max(foreground,background)+.05)/(Math.min(foreground,background)+.05)>=4.5,'license text must remain readable');
   results.push({locale,theme,width,text200,noBitmaps:true,codeIndented:true});
  }
  await call('Emulation.setDeviceMetricsOverride',{width:1280,height:1000,deviceScaleFactor:1,mobile:false});await navigate(`${prefix}/?theme=light`);
  const lightLicense=await color('.landing-license'),lightMoney=await color('.mock-money .mock-input');
  await evaluate(`(()=>{const e=document.querySelector('.site-theme');e.value='dark';e.dispatchEvent(new Event('change'))})()`);
  const darkLicense=await color('.landing-license'),darkMoney=await color('.mock-money .mock-input');
  assert(luminance(darkLicense.background)<luminance(lightLicense.background)*.3);assert(luminance(darkMoney.background)<luminance(lightMoney.background)*.3);
  await evaluate('window.originalAtlas=document.querySelector("[data-project-key=\\"101\\"]")');
  await click('[data-sort-column="tasks"]');assert.deepEqual(await order(),[203,409,101,307]);
  assert.equal(await evaluate('document.querySelector("[data-hero-table]").dataset.selectedKey'),'101');assert(await evaluate('window.originalAtlas===document.querySelector("[data-project-key=\\"101\\"]")'),'sort must retain the exact row node');
  await click('[data-sort-column="tasks"]');assert.deepEqual(await order(),[307,101,409,203]);
  await click('[data-sort-column="tasks"]');assert.deepEqual(await order(),[101,203,307,409]);
  await click('[data-project-key="203"] button');assert.equal(await evaluate('document.querySelector("[data-hero-table]").dataset.selectedKey'),'203');
  assert(await evaluate('document.querySelector("[data-project-key=\\"307\\"] button").disabled'),'Cedar stays disabled');
  await click('[data-sort-column="tasks"]');assert.equal(await evaluate('document.querySelector("[data-hero-table]").dataset.selectedKey'),'203');assert(await evaluate('document.querySelector("[data-hero-caption]").textContent.includes("Boreal")'),'caption must describe the actual selected project');
  await evaluate('Object.defineProperty(navigator,"clipboard",{configurable:true,value:{writeText:async text=>{window.copiedCode=text}}})');
  for (const index of [1,2]) {
    const expected=await evaluate(`document.querySelector('.landing-contract>div:nth-child(${index}) code').textContent`);
    await click(`.landing-contract>div:nth-child(${index}) .code-actions button`);
    assert.equal(await evaluate('window.copiedCode'),expected);
    assert(expected.includes(index===1?'\n        root.contact-changed(value);':'\n        println!'));
  }
  await evaluate('document.querySelector(".landing-contract").scrollIntoView({block:"center"})');
  await fs.writeFile(path.join(out,`${locale}-contract-dark.png`),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
  await evaluate('document.querySelector(".landing-license").scrollIntoView({block:"center"})');
  await fs.writeFile(path.join(out,`${locale}-license-dark.png`),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
  results.push({locale,keyedSort:true,disabledRow:true,caption:true,clipboardIndentation:true,liveTheme:true});
 }
 await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'no-preference'}]});await navigate('/?theme=dark');await evaluate('window.scrollTo(0,0)');
 for(let n=0;n<60;n++){if(await evaluate('document.querySelector("[data-hero-table]").dataset.sortDirection==="ascending"'))break;await pause(100);}
 assert.equal(await evaluate('document.querySelector("[data-hero-table]").dataset.sortDirection'),'ascending','visible hero must demonstrate a real numeric sort');
 await click('[data-hero-playback]');const paused=await order();await pause(2900);assert.deepEqual(await order(),paused,'pause must stop the timer');
 await click('[data-hero-playback]');await evaluate('window.scrollTo(0,document.body.scrollHeight)');await pause(150);const offscreen=await order();await pause(2900);assert.deepEqual(await order(),offscreen,'offscreen hero must stop scheduling steps');
 results.push({autoplay:true,pause:true,offscreenPause:true});
 await call('Emulation.setScriptExecutionDisabled',{value:true});await navigate('/');assert(await evaluate('document.querySelector("main").textContent.includes("shared interaction logic")'));await navigate('/fr/');assert(await evaluate('document.querySelector("main").textContent.includes("logique d’interaction")'));assert(await evaluate('[...document.querySelectorAll("[data-sort-column]")].every(b=>b.disabled)'));results.push({check:'content-readable-with-javascript-disabled',passed:true});
 await fs.writeFile(path.join(out,'results.json'),JSON.stringify({passed:true,backend:'Chrome headless HTML/CSS; no native Slint qualification',results},null,2));console.log('PASS: EN/FR landing themes, HTML illustrations, keyed sort, reduced motion, playback, offscreen pause and code copying.');
}catch(e){await fs.writeFile(path.join(out,'results.json'),JSON.stringify({passed:false,error:String(e),results},null,2));throw e;}finally{socket?.close();chrome.kill();await once(chrome,'exit');server.close();await fs.rm(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100});}
