// SPDX-License-Identifier: GPL-3.0-or-later
// Optional native Chrome check. No browser packages or user profile are used.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
const docs = fileURLToPath(new URL('../', import.meta.url));
const output = path.join(docs, '.verification/browser');
await fs.mkdir(output, { recursive: true });
const chrome = process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
await fs.access(chrome);
const pause = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.png': 'image/png', '.xml': 'application/xml', '.txt': 'text/plain' };
const server = http.createServer(async (request, response) => {
  try {
    const route = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const target = path.resolve(docs, '_site', '.' + route, route.endsWith('/') ? 'index.html' : '');
    assert(target.startsWith(path.join(docs, '_site') + path.sep));
    const content = await fs.readFile(target);
    // Force the old delayed-script bug to surface even on a fast local machine.
    if (/^\/assets\/(?:versioned\/)?site(?:\.[a-f0-9]{16})?\.js$/.test(route)) await pause(1200);
    if (/\/assets\/components\/button(?:-fr)?(?:@2x)?\.png$/.test(route)) await pause(1500);
    response.writeHead(200, { 'Content-Type': mime[path.extname(target)] || 'text/plain', 'Cache-Control': 'no-store' });
    response.end(content);
  } catch {
    response.writeHead(404); response.end();
  }
});
server.listen(0, '127.0.0.1');
await once(server, 'listening');
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await fs.mkdtemp(path.join(os.tmpdir(), 'slint-kit-browser-'));
const processChrome = spawn(chrome, ['--headless=new', '--no-first-run', '--no-default-browser-check', '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore' });
let socket;
let sourceId;
const pending = new Map();
const results = [];
let passed = false;
let sequence = 0;
let navigationSequence = 0;
function call(method, params = {}) {
  const id = ++sequence;
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timeout: ${method}`)); }, 30000);
    pending.set(id, { resolve, reject, timeout });
    socket.send(JSON.stringify({ id, method, params }));
  });
}
async function evaluate(expression) {
  const result = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (result.exceptionDetails) throw new Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
async function navigate(route) {
  const targetUrl = origin + route + '?verification=' + (++navigationSequence);
  await call('Page.navigate', { url: targetUrl });
  for (let count = 0; count < 150; count++) {
    if (await evaluate(`location.href === ${JSON.stringify(targetUrl)} && document.readyState === 'complete'`).catch(() => false)) break;
    await pause(100);
  }
  assert.equal(await evaluate('document.readyState'), 'complete');
  assert.equal(await evaluate('location.href'), targetUrl);
  await pause(400);
}
async function click(selector) {
  const point = await evaluate(`(() => {const e=document.querySelector(${JSON.stringify(selector)});e.scrollIntoView({block:'center'});const r=e.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}})()`);
  await call('Input.dispatchMouseEvent', { type: 'mousePressed', button: 'left', clickCount: 1, ...point });
  await call('Input.dispatchMouseEvent', { type: 'mouseReleased', button: 'left', clickCount: 1, ...point });
}
async function keyEnter() {
  await call('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, text: '\r' });
  await call('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
}
try {
  let port;
  for (let count = 0; count < 100; count++) {
    try { port = (await fs.readFile(path.join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]; break; } catch { await pause(100); }
  }
  assert(port, 'Chrome must expose its temporary debugging port');
  const tabs = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  socket = new WebSocket(tabs.find(tab => tab.type === 'page').webSocketDebuggerUrl);
  socket.addEventListener('message', event => {
    const data = JSON.parse(event.data);
    const entry = pending.get(data.id);
    if (!entry) return;
    pending.delete(data.id); clearTimeout(entry.timeout);
    if (data.error) entry.reject(new Error(JSON.stringify(data.error))); else entry.resolve(data.result);
  });
  await once(socket, 'open');
  await call('Page.enable');
  await call('Network.enable');
  await call('Network.setCacheDisabled', { cacheDisabled: true });
  await call('Page.bringToFront');
  sourceId = (await call('Page.addScriptToEvaluateOnNewDocument', { source: `window.auditShifts=[];new PerformanceObserver(list=>{for(const item of list.getEntries())if(!item.hadRecentInput)window.auditShifts.push(item.value)}).observe({type:'layout-shift',buffered:true});` })).identifier;
  for (const locale of ['en', 'fr']) {
    const prefix = locale === 'fr' ? '/fr' : '';
    for (const [name, route, width, dark, text200, pixelDensity = 1] of [
      ['home-mobile', `${prefix}/`, 375, false, false],
      ['button-mobile', `${prefix}/components/button/`, 375, false, false],
      ['button-dark', `${prefix}/components/button/`, 375, true, false],
      ['button-text200', `${prefix}/components/button/`, 375, false, true],
      ['button-desktop', `${prefix}/components/button/`, 1350, false, false],
      ['combobox-mobile', `${prefix}/components/combobox/`, 375, false, false],
      ['combobox-desktop', `${prefix}/components/combobox/`, 1350, false, false],
      ['table-guide', `${prefix}/guides/table-example/`, 375, false, false],
      ['source-guide', `${prefix}/guides/source-updates/`, 375, false, false],
      ['source-guide-text200', `${prefix}/guides/source-updates/`, 375, true, true],
      ['button-retina', `${prefix}/components/button/`, 375, false, false, 2],
      ['table-retina', `${prefix}/guides/table-example/`, 375, false, false, 2],
      ['typed-retina', `${prefix}/guides/typed-example/`, 375, false, false, 2],
    ]) {
      await call('Emulation.setDeviceMetricsOverride', { width, height: 900, deviceScaleFactor: pixelDensity, mobile: false });
      await call('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-color-scheme', value: dark ? 'dark' : 'light' }] });
      await navigate(route);
      const cls = await evaluate('window.auditShifts.reduce((sum,value)=>sum+value,0)');
      assert(cls < 0.001, `${locale}/${name}: unstable initial layout (CLS ${cls})`);
      if (route.includes('/components/button/')) {
        assert.equal(await evaluate('document.querySelector(".mobile-nav").open'), false);
        assert.equal(await evaluate('getComputedStyle(document.querySelector(".desktop-nav")).display !== "none"'), width > 720);
      }
      if (text200) await evaluate('document.documentElement.style.fontSize="32px"');
      assert(await evaluate('document.documentElement.scrollWidth <= innerWidth'), `${locale}/${name}: document overflow`);
      const imageSource = await evaluate('document.querySelector(".component-preview img")?.currentSrc || null');
      if (pixelDensity === 2) assert(imageSource?.endsWith('@2x.png'), `${locale}/${name}: browser must load the native 2x source`);
      const capture = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
      await fs.writeFile(path.join(output, `${locale}-${name}.png`), Buffer.from(capture.data, 'base64'));
      results.push({ locale, name, route, width, dark, text200, pixelDensity, imageSource, cls });
    }
    await call('Emulation.setDeviceMetricsOverride', { width: 375, height: 900, deviceScaleFactor: 1, mobile: false });
    await navigate(`${prefix}/components/button/`);
    const copyResult = await evaluate(`(async () => {
      let copied;
      const writeText = navigator.clipboard.writeText;
      navigator.clipboard.writeText = async text => { copied = text; };
      try {
        const pre = document.querySelector('pre');
        const expected = pre.querySelector('code').textContent;
        pre.querySelector('.code-actions button').click();
        await new Promise(resolve => setTimeout(resolve, 0));
        return { matches: copied === expected, status: pre.querySelector('[role=status]').textContent };
      } finally { navigator.clipboard.writeText = writeText; }
    })()`);
    assert(copyResult.matches, `${locale}: Copy must preserve the complete displayed source`);
    assert.equal(copyResult.status, locale === 'fr' ? 'Copié' : 'Copied');
    await evaluate('document.querySelector(".mobile-nav>summary").focus()');
    await keyEnter();
    assert.equal(await evaluate('document.querySelector(".mobile-nav").open'), true, 'keyboard opens native disclosure');
    await click('.mobile-nav>summary');
    assert.equal(await evaluate('document.querySelector(".mobile-nav").open'), false, 'pointer closes disclosure');
    await call('Emulation.setScriptExecutionDisabled', { value: true });
    await navigate(`${prefix}/components/button/`);
    assert.equal(await evaluate('document.querySelector(".mobile-nav").open'), false);
    await click('.mobile-nav>summary');
    assert.equal(await evaluate('document.querySelector(".mobile-nav").open'), true, 'menu works without JavaScript');
    await call('Emulation.setDeviceMetricsOverride', { width: 1350, height: 900, deviceScaleFactor: 1, mobile: false });
    assert.equal(await evaluate('getComputedStyle(document.querySelector(".desktop-nav")).display !== "none"'), true, 'desktop navigation works without JavaScript');
    await call('Emulation.setScriptExecutionDisabled', { value: false });
    await call('Emulation.setDeviceMetricsOverride', { width: 375, height: 900, deviceScaleFactor: 1, mobile: false });
    await navigate(`${prefix}/components/`);
    await evaluate('document.querySelector("#component-filter").focus()');
    await call('Input.insertText', { text: 'KitButton' });
    assert.equal(await evaluate('document.querySelector("#filter-status").innerText'), '2 / 123');
    assert.deepEqual(await evaluate('[...document.querySelectorAll(".catalogue-item")].filter(item=>!item.hidden).map(item=>item.querySelector("code").innerText)'), ['KitButton', 'KitButtonGroup']);
  }
  passed = true;
  console.log(`Browser checks PASS: ${results.length} captures, delayed script/image CLS, EN/FR, light/dark, text200, keyboard/pointer, no-JS menu and filter.`);
} finally {
  await fs.writeFile(path.join(output, 'results.json'), JSON.stringify({ passed, chrome, results }, null, 2) + '\n');
  if (sourceId && socket?.readyState === WebSocket.OPEN) await call('Page.removeScriptToEvaluateOnNewDocument', { identifier: sourceId });
  socket?.close();
  processChrome.kill();
  await once(processChrome, 'exit').catch(() => {});
  server.close();
  await fs.rm(profile, { recursive: true, force: true });
}
