// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawn} from 'node:child_process';
import {once} from 'node:events';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {prepareSearch,searchDocuments} from '../src/assets/search.js';
const docs=fileURLToPath(new URL('../',import.meta.url));
const site=path.resolve(docs,process.env.DOCS_SEARCH_SITE||'_site');
for(const locale of ['en','fr']){
 const index=JSON.parse(await fs.readFile(path.join(site,'assets',`search-index-${locale}.json`),'utf8'));
 assert.equal(index.locale,locale);const entries=prepareSearch(index.entries),apis=entries.filter(entry=>entry.api);
 assert(apis.length>=100,'The real index includes the component API catalogue');
 assert(entries.every(entry=>!entry.url.includes('/migration-0-4/')&&!entry.url.includes('/examples/')&&!entry.url.includes('/preview/')),'Removed and noindex pages stay out of global search');
 for(const entry of apis)for(const query of [entry.api,entry.api.toLowerCase()])assert.equal(searchDocuments(entries,query)[0]?.url,entry.url,`${locale}: ${query} must rank its own API reference first`);
}
const output=path.join(docs,'.verification/search');await fs.mkdir(output,{recursive:true});
const mime={'.html':'text/html','.js':'text/javascript','.json':'application/json','.css':'text/css','.svg':'image/svg+xml'};
const server=http.createServer(async(req,res)=>{try{const route=new URL(req.url,'http://localhost').pathname,target=path.resolve(site,'.'+route,route.endsWith('/')?'index.html':'');assert(target.startsWith(site+path.sep));const content=await fs.readFile(target);res.writeHead(200,{'Content-Type':mime[path.extname(target)]||'application/octet-stream'});res.end(content)}catch{res.writeHead(404);res.end()}});
server.listen(0,'127.0.0.1');await once(server,'listening');const origin=`http://127.0.0.1:${server.address().port}`;
const profile=await fs.mkdtemp(path.join(os.tmpdir(),'slint-search-chrome-'));
const chrome=spawn(process.env.CHROME_PATH||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--no-first-run','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
let socket,sequence=0;const pending=new Map(),errors=[],requests=[],proofs=[];const pause=ms=>new Promise(resolve=>setTimeout(resolve,ms));
function call(method,params={}){const id=++sequence;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP timeout '+method))},30000);pending.set(id,{resolve,reject,timer});socket.send(JSON.stringify({id,method,params}))})}
async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});if(result.exceptionDetails)throw Error(JSON.stringify(result.exceptionDetails));return result.result.value}
async function waitFor(expression){for(let index=0;index<100;index++){if(await evaluate(expression).catch(()=>false))return;await pause(100)}throw Error('Condition timed out: '+expression)}
async function navigate(route){await call('Page.navigate',{url:origin+route});await waitFor('Boolean(document.querySelector(".doc-search-trigger:not([hidden])"))')}
async function key(key,code,virtual,modifiers=0){await call('Input.dispatchKeyEvent',{type:'keyDown',key,code,windowsVirtualKeyCode:virtual,modifiers,...(key==='Enter'?{text:'\r'}:{})});await call('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:virtual,modifiers:0})}
async function click(selector){const box=await evaluate(`document.querySelector(${JSON.stringify(selector)}).getBoundingClientRect().toJSON()`);const x=box.x+box.width/2,y=box.y+box.height/2;await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,x,y});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,x,y})}
async function capture(name){const result=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});await fs.writeFile(path.join(output,name+'.png'),Buffer.from(result.data,'base64'));proofs.push(name)}
try{
 let port;for(let i=0;i<100;i++){try{port=(await fs.readFile(path.join(profile,'DevToolsActivePort'),'utf8')).split('\n')[0];break}catch{await pause(100)}}assert(port);
 const tabs=await(await fetch(`http://127.0.0.1:${port}/json/list`)).json();socket=new WebSocket(tabs.find(tab=>tab.type==='page').webSocketDebuggerUrl);
 socket.addEventListener('message',event=>{const data=JSON.parse(event.data);if(data.method==='Runtime.exceptionThrown')errors.push(data.params.exceptionDetails.text);if(data.method==='Network.requestWillBeSent')requests.push(data.params.request.url);const item=pending.get(data.id);if(item){pending.delete(data.id);clearTimeout(item.timer);data.error?item.reject(Error(JSON.stringify(data.error))):item.resolve(data.result)}});await once(socket,'open');
 await call('Page.enable');await call('Runtime.enable');await call('Network.enable');await call('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 await navigate('/components/button/');assert(!requests.some(url=>url.includes('search-index-')),'Index stays unloaded until a query is entered');
 await click('.doc-search-trigger');await waitFor('document.activeElement.id==="doc-search-query"');await call('Input.insertText',{text:'validaton'});
 await waitFor('document.querySelector(".doc-search-results a")?.getAttribute("href")==="/guides/forms/"');
 assert.equal(requests.filter(url=>url.includes('search-index-')).length,1);
 await key('ArrowDown','ArrowDown',40);assert.equal(await evaluate('document.activeElement.getAttribute("href")'),'/guides/forms/');
 await key('ArrowDown','ArrowDown',40);await key('ArrowUp','ArrowUp',38);assert.equal(await evaluate('document.activeElement.getAttribute("href")'),'/guides/forms/');await capture('desktop-fuzzy-guide');
 await key('Escape','Escape',27);await waitFor('!document.querySelector("#doc-search").open');assert.equal(await evaluate('document.activeElement.className'),'doc-search-trigger');
 await key('k','KeyK',75,4);await waitFor('document.querySelector("#doc-search").open');assert.equal(await evaluate('document.activeElement.id'),'doc-search-query');
 await key('Enter','Enter',13);await waitFor('location.pathname==="/guides/forms/"');
 await call('Emulation.setDeviceMetricsOverride',{width:375,height:850,deviceScaleFactor:2,mobile:false});await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:'dark'}]});
 await navigate('/fr/');assert(await evaluate('document.documentElement.scrollWidth<=375'),'Mobile header stays within the viewport');await click('.doc-search-trigger');await call('Input.insertText',{text:'kit txt field'});
 await waitFor('document.querySelector(".doc-search-results a")?.getAttribute("href")==="/fr/components/text-field/"');
 assert(await evaluate('[...document.querySelectorAll(".doc-search-results a")].every(link=>link.getAttribute("href").startsWith("/fr/"))'),'Results stay in the current language');await capture('mobile-fr-dark-fuzzy-api');
 await evaluate('document.documentElement.style.fontSize="200%"');assert(await evaluate('document.documentElement.scrollWidth<=375'),'Mobile topbar wraps when text is enlarged');assert(await evaluate('document.querySelector("#doc-search").getBoundingClientRect().width<=375'),'Search dialog fits enlarged text on mobile');await capture('mobile-fr-text-200');
 await key('ArrowDown','ArrowDown',40);await key('Enter','Enter',13);await waitFor('location.pathname==="/fr/components/text-field/"');
 assert.equal(errors.length,0);await fs.writeFile(path.join(output,'results.json'),JSON.stringify({backend:'Chrome headless',proofs,errors,indexRequests:requests.filter(url=>url.includes('search-index-'))},null,2));console.log('Global search PASS: lazy local indexes, fuzzy guide/API lookup, EN/FR, keyboard navigation, Escape/focus return, mobile dark and text200.');
}catch(error){console.error(await evaluate('JSON.stringify({pathname:location.pathname,active:document.activeElement?.outerHTML,query:document.querySelector("#doc-search-query")?.value,results:[...document.querySelectorAll(".doc-search-results a")].map(a=>a.getAttribute("href")),open:document.querySelector("#doc-search")?.open,status:document.querySelector("#doc-search-status")?.textContent})').catch(()=>''));await capture('failure').catch(()=>{});throw error;}finally{socket?.close();const stopped=once(chrome,'close');chrome.kill('SIGTERM');await stopped;await new Promise(resolve=>server.close(resolve));await fs.rm(profile,{recursive:true,force:true,maxRetries:5,retryDelay:100})}
