// SPDX-License-Identifier: GPL-3.0-or-later
// Capture the actual WASM canvas, using a fresh isolated Chrome profile.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const docs=fileURLToPath(new URL('../',import.meta.url));
const root=path.dirname(docs.slice(0,-1));
const plan=JSON.parse(await fs.readFile(process.argv[2],'utf8'));
const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm'};
const server=http.createServer(async(request,response)=>{
  try {
    const url=new URL(request.url,'http://localhost');
    if(url.pathname==='/capture/') {
      const scene=plan.find(item=>item.id===url.searchParams.get('scene'));
      assert(scene,'Unknown capture scene');
      const module=JSON.stringify(scene.base+'/slint_kit_previews.js');
      response.writeHead(200,{'Content-Type':'text/html'});
      response.end(`<!doctype html><html><meta charset="utf-8"><style>html,body{margin:0;padding:0;background:transparent}canvas{display:block;outline:none}</style><canvas id="canvas" tabindex="0"></canvas><script type="module">try {const {default:init}=await import(${module});try{await init()}catch(error){if(!String(error).includes("Using exceptions for control flow, don't mind me."))throw error}await document.fonts.ready;await new Promise(requestAnimationFrame);await new Promise(requestAnimationFrame);window.captureReady=true}catch(error){window.captureError=String(error)}</script></html>`);
      return;
    }
    const file=path.resolve(docs,'.generated/web','.'+decodeURIComponent(url.pathname));
    assert(file.startsWith(path.resolve(docs,'.generated/web')+path.sep));
    const content=await fs.readFile(file);
    response.writeHead(200,{'Content-Type':mime[path.extname(file)]||'application/octet-stream'});
    response.end(content);
  }catch{response.writeHead(404);response.end();}
});
server.listen(0,'127.0.0.1');await once(server,'listening');
const origin=`http://127.0.0.1:${server.address().port}`;
const profile=await fs.mkdtemp(path.join(os.tmpdir(),'slint-doc-capture-'));
const chrome=process.env.CHROME_PATH||(process.platform==='darwin'?'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome':'google-chrome');
const child=spawn(chrome,['--headless=new','--force-device-scale-factor=2','--no-first-run','--no-default-browser-check','--remote-debugging-port=0','--use-angle=swiftshader','--enable-unsafe-swiftshader',`--user-data-dir=${profile}`,'about:blank'],{stdio:['ignore','ignore','pipe']});
let errors='';child.stderr.on('data',chunk=>{errors=(errors+String(chunk)).slice(-4000)});
let socket;let sequence=0;const pending=new Map();
function call(method,params={}){
 const id=++sequence;
 return new Promise((resolve,reject)=>{const timeout=setTimeout(()=>{pending.delete(id);reject(new Error(`Chrome timeout: ${method}`))},30000);pending.set(id,{resolve,reject,timeout});socket.send(JSON.stringify({id,method,params}));});
}
async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;}
try{
 let port;
 for(let i=0;i<150;i++){try{port=(await fs.readFile(path.join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];break}catch{}if(child.exitCode!==null)throw new Error(errors);await pause(100)}
 assert(port,'Chrome did not start: '+errors);
 const targets=await(await fetch(`http://127.0.0.1:${port}/json`)).json();
 socket=new WebSocket(targets.find(target=>target.type==='page').webSocketDebuggerUrl);
 await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true})});
 socket.addEventListener('message',event=>{const message=JSON.parse(event.data),request=pending.get(message.id);if(!request)return;clearTimeout(request.timeout);pending.delete(message.id);message.error?request.reject(new Error(JSON.stringify(message.error))):request.resolve(message.result)});
 await call('Page.enable');await call('Runtime.enable');
 await call('Emulation.setDeviceMetricsOverride',{width:1400,height:1200,deviceScaleFactor:2,mobile:false});
 const version=await call('Browser.getVersion');
 for(const scene of plan){
  const url=`${origin}/capture/?scene=${encodeURIComponent(scene.id)}&component=${encodeURIComponent(scene.slug)}`;
  await call('Page.navigate',{url});
  let ready=false;
  for(let i=0;i<600;i++){
   const state=await evaluate(`location.href===${JSON.stringify(url)}?({ready:window.captureReady,error:window.captureError}):({})`).catch(()=>({}));
   if(state.error)throw new Error(`${scene.id}: ${state.error}`);
   if(state.ready){ready=true;break}await pause(100);
  }
  assert(ready,`Canvas not ready: ${scene.id}`);
  await pause(150);
  const rectangle=await evaluate('(()=>{const c=document.querySelector("canvas"),r=c.getBoundingClientRect();return{x:r.x,y:r.y,width:r.width,height:r.height,pixels:[c.width,c.height],density:devicePixelRatio}})()');
  assert.equal(rectangle.density,2);assert(rectangle.width>0&&rectangle.height>0);
  assert.equal(rectangle.pixels[0],Math.round(rectangle.width*2),'Canvas backing pixels must be native ×2');
  if(scene.activation){
    // Overlay posters use a declared point inside their known opener. Regular
    // interaction qualification remains in check-browser.mjs.
    const activation=Array.isArray(scene.activation)?{action:'click',point:scene.activation}:scene.activation;
    assert(['click','hover'].includes(activation.action),'Unknown poster action');
    assert(Array.isArray(activation.point)&&activation.point.length===2&&activation.point.every(Number.isFinite),'Invalid poster point');
    const [x,y]=activation.point;
    await call('Input.dispatchMouseEvent',{type:'mouseMoved',x,y});
    if(activation.action==='click'){
      await call('Input.dispatchMouseEvent',{type:'mousePressed',x,y,button:'left',clickCount:1});
      await call('Input.dispatchMouseEvent',{type:'mouseReleased',x,y,button:'left',clickCount:1});
    }
    await pause(activation.action==='hover'?750:200);
  }
  const screenshot=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:true,clip:{x:rectangle.x,y:rectangle.y,width:rectangle.width,height:rectangle.height,scale:1}});
  const destination=path.resolve(root,scene.output);await fs.mkdir(path.dirname(destination),{recursive:true});
  const png=Buffer.from(screenshot.data,'base64');
  assert.equal(png.readUInt32BE(16),Math.round(rectangle.width*2));assert.equal(png.readUInt32BE(20),Math.round(rectangle.height*2));
  await fs.writeFile(destination,png);
  await fs.writeFile(destination+'.environment.json',JSON.stringify({sceneKey:scene.sceneKey,key:scene.key,capturedAt:new Date().toISOString(),backend:'browser',renderer:'femtovg',chrome:version.product,platform:process.platform,density:2,canvasPixels:rectangle.pixels}));
  console.log(`Browser ×2: ${scene.id}`);
 }
}finally{
 for(const item of pending.values())clearTimeout(item.timeout);
 socket?.close();child.kill('SIGTERM');server.close();
 await Promise.race([once(child,'exit'),pause(3000)]).catch(()=>{});
 if(child.exitCode===null)child.kill('SIGKILL');
 await fs.rm(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100});
}
