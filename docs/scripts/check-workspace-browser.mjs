// SPDX-License-Identifier: GPL-3.0-or-later
// Real WASM consumer, rendered and exercised through Chrome's public CDP API.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
const docs=fileURLToPath(new URL('../',import.meta.url)), dpr=Number(process.env.BROWSER_DPR||2);
const output=path.join(docs,'.verification/product-workspace/browser');await fs.mkdir(output,{recursive:true});
const mime={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.json':'application/json'};
const server=http.createServer(async(req,res)=>{try{const route=new URL(req.url,'http://localhost').pathname,target=path.resolve(docs,'_site','.'+route,route.endsWith('/')?'index.html':'');assert(target.startsWith(path.join(docs,'_site')+path.sep));const content=await fs.readFile(target);res.writeHead(200,{'Content-Type':mime[path.extname(target)]||'application/octet-stream'});res.end(content)}catch{res.writeHead(404);res.end()}});server.listen(0,'127.0.0.1');await once(server,'listening');const origin=`http://127.0.0.1:${server.address().port}`;
const profile=await fs.mkdtemp(path.join(os.tmpdir(),'workspace-chrome-'));
const chrome=spawn(process.env.CHROME_PATH||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new',`--force-device-scale-factor=${dpr}`,'--no-first-run','--remote-debugging-port=0','--use-angle=swiftshader','--enable-unsafe-swiftshader',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
let socket,sequence=0;const pending=new Map(),errors=[],proofs=[];const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
function call(method,params={}){const id=++sequence;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP timeout '+method))},30000);pending.set(id,{resolve,reject,timer});socket.send(JSON.stringify({id,method,params}))})}
async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(result.exceptionDetails)throw Error(JSON.stringify(result.exceptionDetails));return result.result.value}
async function waitFor(expression){for(let index=0;index<200;index++){if(await evaluate(expression).catch(()=>false))return;await pause(100)}throw Error('Condition timed out: '+expression+'; '+await evaluate('JSON.stringify({draft:window.workspacePreview?.editor_draft?.(),state:window.workspacePreview?.editor_state?.(),active:document.activeElement?.tagName})'))}
async function navigate(query){await call('Page.navigate',{url:`${origin}/examples/workspace/?${query}`});await waitFor('Boolean(document.querySelector("canvas")?.dataset.ready)');await pause(250)}
async function capture(name){const result=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});await fs.writeFile(path.join(output,name+'.png'),Buffer.from(result.data,'base64'));proofs.push(name)}
async function click(x,y){await call('Input.dispatchMouseEvent',{type:'mouseMoved',x,y});await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,x,y});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,x,y});await pause(120)}
async function key(key,code,virtual,text){await call('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode:virtual,...(text?{text}:{})});await call('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:virtual})}
async function clearDraft(){
  assert((await evaluate('window.workspacePreview.editor_draft()')).length>0,'Selection test starts with a nonempty draft');
  await key('ArrowRight','ArrowRight',39);
  const apple=await evaluate('/Mac|iPhone|iPad/.test(navigator.platform)');
  const modifier=apple?{key:'Meta',code:'MetaLeft',windowsVirtualKeyCode:91}:{key:'Control',code:'ControlLeft',windowsVirtualKeyCode:17};
  const mask=apple?4:2;
  await call('Input.dispatchKeyEvent',{type:'keyDown',...modifier,modifiers:mask});
  await call('Input.dispatchKeyEvent',{type:'keyDown',key:'a',code:'KeyA',windowsVirtualKeyCode:65,modifiers:mask});
  await call('Input.dispatchKeyEvent',{type:'keyUp',key:'a',code:'KeyA',windowsVirtualKeyCode:65,modifiers:mask});
  await call('Input.dispatchKeyEvent',{type:'keyUp',...modifier,modifiers:0});
  await key('Backspace','Backspace',8);
  await waitFor('window.workspacePreview.editor_draft()===""');
}
try{
let port;for(let index=0;index<100;index++){try{port=(await fs.readFile(path.join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];break}catch{await pause(100)}}assert(port);const tabs=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();socket=new WebSocket(tabs.find(tab=>tab.type==='page').webSocketDebuggerUrl);socket.addEventListener('message',event=>{const data=JSON.parse(event.data);if(data.method==='Runtime.exceptionThrown')errors.push(data.params.exceptionDetails.text);const item=pending.get(data.id);if(item){pending.delete(data.id);clearTimeout(item.timer);data.error?item.reject(Error(JSON.stringify(data.error))):item.resolve(data.result)}});await once(socket,'open');await call('Page.enable');await call('Runtime.enable');await call('Emulation.setDeviceMetricsOverride',{width:1180,height:960,deviceScaleFactor:dpr,mobile:false});
if(!process.env.WORKSPACE_INTERACTIONS_ONLY)for(const scene of ['account','projects','dialog'])for(const mode of ['light','dark'])for(const scale of [1,2]){await navigate(new URLSearchParams({scene,mode,scale}).toString());assert.equal(await evaluate('document.querySelector("#loading") === null'),true);const width=await evaluate('document.querySelector("canvas").getBoundingClientRect().width');assert.equal(width,1180);await capture(`${scene}-tailwind-${mode}-${scale}x@${dpr}x`)}
await navigate('scene=projects');assert.equal(await evaluate('window.workspacePreview.project_count()'),4);const top=await evaluate('document.querySelector("canvas").getBoundingClientRect().top');
await click(406,top+188);assert.equal(await evaluate('window.workspacePreview.project_count()'),1,'Archived filter changes Rust projection');await capture('projects-filtered');
await click(236,top+188);assert.equal(await evaluate('window.workspacePreview.project_count()'),4);
await click(360,top+136);await call('Input.insertText',{text:'no matching project'});await waitFor('window.workspacePreview.project_count()===0');await capture('projects-empty');
await navigate('scene=dialog');assert.equal(await evaluate('window.workspacePreview.editor_state()'),'editing');await evaluate('document.querySelector("canvas").focus()');await key('Escape','Escape',27);await waitFor('window.workspacePreview.editor_state()==="closed"');await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_state()==="editing"');
await capture('dialog-restored');
await key('Tab','Tab',9,'\t');
await clearDraft();
for(let index=0;index<3;index++)await key('Tab','Tab',9,'\t');await key('Enter','Enter',13,'\r');
assert.equal(await evaluate('window.workspacePreview.editor_state()'),'editing','Invalid draft stays in the editor');await capture('dialog-invalid');
await click(500,top+310);for(const character of 'Renamed workspace'){await key(character,character===' '?'Space':'Key'+character.toUpperCase(),character.toUpperCase().charCodeAt(0),character)}await waitFor('window.workspacePreview.editor_draft()==="Renamed workspace"');await capture('dialog-valid-draft');for(let index=0;index<3;index++){await key('Tab','Tab',9,'\t');await pause(40)}await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_state()==="saving"');await key('Escape','Escape',27);assert.equal(await evaluate('window.workspacePreview.editor_state()'),'saving','Busy ignores Escape');await capture('dialog-busy');await waitFor('window.workspacePreview.editor_state()==="saved"');assert.equal(await evaluate('window.workspacePreview.project_name(101)'),'Renamed workspace');
await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_state()==="editing"');await key('Tab','Tab',9,'\t');
async function replaceDraft(text){await clearDraft();for(const character of text)await key(character,character===' '?'Space':'Key'+character.toUpperCase(),character.toUpperCase().charCodeAt(0),character);await waitFor('window.workspacePreview.editor_draft()==='+JSON.stringify(text))}
await replaceDraft('Cancelled draft');await key('Escape','Escape',27);await waitFor('window.workspacePreview.editor_state()==="closed"');await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_draft()==="Renamed workspace"');await capture('dialog-reopened-after-cancel');
await key('Tab','Tab',9,'\t');await replaceDraft('Entered name');await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_state()==="saving"');await waitFor('window.workspacePreview.editor_state()==="saved"');assert.equal(await evaluate('window.workspacePreview.project_name(101)'),'Entered name');await capture('projects-enter-submitted');
await key('Enter','Enter',13,'\r');await waitFor('window.workspacePreview.editor_state()==="editing"');await key('Tab','Tab',9,'\t');await key('Tab','Tab',9,'\t');await key('Enter','Enter',13,'\r');await pause(80);assert.equal(await evaluate('window.workspacePreview.editor_state()'),'editing','Enter in multiline description must not submit');await key('Escape','Escape',27);


await navigate('scene=account&brand=studio&lang=fr');await capture('account-studio-fr');
await call('Emulation.setDeviceMetricsOverride',{width:760,height:700,deviceScaleFactor:dpr,mobile:false});await navigate('scene=dialog&scale=2&lang=fr');await capture('dialog-narrow-fr-200');await evaluate('document.querySelector("canvas").focus()');await key('Tab','Tab',9,'\t');await capture('dialog-narrow-fr-200-field-focus');
assert.equal(errors.length,0,'No browser runtime exceptions');await fs.writeFile(path.join(output,'results.json'),JSON.stringify({backend:'Chrome headless',renderer:'Slint femtovg WebGL via SwiftShader',dpr,proofs,errors},null,2));console.log(`Workspace WASM: ${proofs.length} real renders; filters, empty state, Escape and focus restoration passed.`);
}catch(error){await capture('failure').catch(()=>{});throw error}finally{socket?.close();chrome.kill('SIGTERM');await new Promise(resolve=>server.close(resolve));await fs.rm(profile,{recursive:true,force:true})}
