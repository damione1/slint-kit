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
const browserDensity = Number(process.env.BROWSER_DPR || 1);
assert([1, 2].includes(browserDensity), 'BROWSER_DPR must be 1 or 2');
const output = path.join(docs, browserDensity === 2 ? '.verification/browser-retina' : '.verification/browser');
await fs.mkdir(output, { recursive: true });
const chrome = process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome';
await fs.access(chrome);
const pause = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.wasm': 'application/wasm', '.svg': 'image/svg+xml', '.png': 'image/png', '.xml': 'application/xml', '.txt': 'text/plain' };
const server = http.createServer(async (request, response) => {
  try {
    const route = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
    const target = path.resolve(docs, '_site', '.' + route, route.endsWith('/') ? 'index.html' : '');
    assert(target.startsWith(path.join(docs, '_site') + path.sep));
    const content = await fs.readFile(target);
    // Force the old delayed-script bug to surface even on a fast local machine.
    if (/^\/assets\/(?:versioned\/)?site(?:\.[a-f0-9]{16})?\.js$/.test(route)) await pause(1200);
    if (/\/assets\/components\/button(?:-fr)?(?:@2x)?\.png$/.test(route) || /^\/assets\/versioned\/button\.[a-f0-9]{16}\.png$/.test(route)) await pause(1500);
    response.writeHead(200, { 'Content-Type': mime[path.extname(target)] || 'text/plain', 'Cache-Control': route.startsWith('/assets/previews/') ? 'public, max-age=31536000, immutable' : 'no-store' });
    response.end(content);
  } catch {
    response.writeHead(404); response.end();
  }
});
server.listen(0, '127.0.0.1');
await once(server, 'listening');
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await fs.mkdtemp(path.join(os.tmpdir(), 'slint-kit-browser-'));
// CDP's deviceScaleFactor alone changes devicePixelRatio but leaves
// ResizeObserver.devicePixelContentBoxSize at the host density. Winit uses
// that physical box, so launch with a matching density for canvas checks.
const processChrome = spawn(chrome, ['--headless=new', `--force-device-scale-factor=${browserDensity}`, '--no-first-run', '--no-default-browser-check', '--remote-debugging-port=0', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore' });
let socket;
let sourceId;
const pending = new Map();
const results = [];
let passed = false;
let sequence = 0;
let navigationSequence = 0;
const requests = [];
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
  const url = new URL(route, origin);
  url.searchParams.set('verification', ++navigationSequence);
  const targetUrl = url.href;
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
    if (data.method === 'Network.requestWillBeSent') requests.push(data.params.request.url);
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
  await call('Emulation.setDeviceMetricsOverride', { width: 900, height: 900, deviceScaleFactor: browserDensity, mobile: false });
  sourceId = (await call('Page.addScriptToEvaluateOnNewDocument', { source: `window.auditShifts=[];new PerformanceObserver(list=>{for(const item of list.getEntries())if(!item.hadRecentInput)window.auditShifts.push(item.value)}).observe({type:'layout-shift',buffered:true});` })).identifier;
  // Inspect meaningful open states as well as constructors: an unopened
  // dialog's canvas only shows its launcher.
  for (const [slug, x, y] of [['dialog-surface', 90, 44], ['drawer', 90, 44],
    ['combobox', 140, 44], ['select', 140, 44], ['select-trigger', 140, 44],
    ['multi-select', 140, 84], ['tooltip', 90, 44], ['tooltip-layer', 90, 44]]) {
    await navigate(`/preview/?component=${slug}`);
    for (let count = 0; count < 300; count++) {
      if (await evaluate('Boolean(window.previewReady || window.previewError)')) break;
      await pause(100);
    }
    assert(await evaluate('Boolean(window.previewReady && !window.previewError)'), `${slug}: open-state startup`);
    await call('Input.dispatchMouseEvent', { type: 'mouseMoved', x, y });
    await call('Input.dispatchMouseEvent', { type: 'mousePressed', button: 'left', clickCount: 1, x, y });
    await call('Input.dispatchMouseEvent', { type: 'mouseReleased', button: 'left', clickCount: 1, x, y });
    await pause(500);
    const canvas = await evaluate('document.querySelector("canvas").getBoundingClientRect().toJSON()');
    const capture = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true,
      clip: { x: canvas.x, y: canvas.y, width: canvas.width, height: canvas.height, scale: 1 } });
    const visualOutput = path.join(docs, browserDensity === 2 ? '.verification/visual-qa/web-retina' : '.verification/visual-qa/web');
    await fs.mkdir(visualOutput, { recursive: true });
    await fs.writeFile(path.join(visualOutput, `${slug}-open.png`), Buffer.from(capture.data, 'base64'));
    results.push({ name: 'browser-preview-open-state', component: slug });
  }
  for (const locale of ['en', 'fr']) {
    const prefix = locale === 'fr' ? '/fr' : '';
    for (const [name, route, width, dark, text200, pixelDensity = 1] of [
      ['home-mobile', `${prefix}/`, 375, false, false],
      ['catalogue-mobile', `${prefix}/components/`, 375, false, false],
      ['catalogue-dark', `${prefix}/components/`, 1350, true, false],
      ['catalogue-text200', `${prefix}/components/`, 375, false, true],
      ['button-mobile', `${prefix}/components/button/`, 375, false, false],
      ['button-dark', `${prefix}/components/button/`, 375, true, false],
      ['button-text200', `${prefix}/components/button/`, 375, false, true],
      ['button-desktop', `${prefix}/components/button/`, 1350, false, false],
      ['combobox-mobile', `${prefix}/components/combobox/`, 375, false, false],
      ['combobox-desktop', `${prefix}/components/combobox/`, 1350, false, false],
      ['table-guide', `${prefix}/guides/table-example/`, 375, false, false],
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
      if (pixelDensity === 2) {
        if (route.includes('/components/')) {
          assert(await evaluate('(()=>{const image=document.querySelector(".component-preview img");return image.currentSrc.endsWith("@2x.png") && Number(image.getAttribute("width")) === parseFloat(image.style.width)})()'), `${locale}/${name}: poster must retain native 2x pixels`);
        } else assert(imageSource?.endsWith('@2x.png'), `${locale}/${name}: browser must load the native 2x source`);
      }
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
    // The status counts the matches against the whole rendered catalogue.
    const catalogueSize = await evaluate('document.querySelectorAll(".catalogue-item").length');
    assert.equal(await evaluate('document.querySelector("#filter-status").innerText'), `2 / ${catalogueSize}`);
    assert.deepEqual(await evaluate('[...document.querySelectorAll(".catalogue-item")].filter(item=>!item.hidden).map(item=>item.querySelector("code").innerText)'), ['KitButton', 'KitButtonGroup']);
  }
  // Prove that the browser backend can construct every documented application.
  // Native interaction proofs remain separate from browser startup coverage.
  await call('Network.setCacheDisabled', { cacheDisabled: false });
  await call('Emulation.setDeviceMetricsOverride', { width: 900, height: 900, deviceScaleFactor: browserDensity, mobile: false });
  const starters = JSON.parse(await fs.readFile(path.join(docs, 'content/starters.json'), 'utf8'));
  const previewSlugs = process.env.BROWSER_COMPONENTS?.split(',').filter(Boolean) || Object.keys(starters);
  assert(previewSlugs.length && previewSlugs.every(slug => Object.hasOwn(starters, slug)), 'BROWSER_COMPONENTS must list known component slugs');
  for (const slug of previewSlugs) {
    await navigate(`/preview/?component=${slug}`);
    for (let count = 0; count < 300; count++) {
      if (await evaluate('Boolean(window.previewReady || window.previewError)')) break;
      await pause(100);
    }
    const state = await evaluate('({ready:window.previewReady,error:window.previewError,width:document.querySelector("canvas").width,height:document.querySelector("canvas").height})');
    assert(state.ready && !state.error && state.width > 0 && state.height > 0, `${slug}: browser startup ${JSON.stringify(state)}`);
    const canvas = await evaluate('document.querySelector("canvas").getBoundingClientRect().toJSON()');
    assert(Math.abs(state.width - canvas.width * browserDensity) <= 1 && Math.abs(state.height - canvas.height * browserDensity) <= 1,
      `${slug}: canvas backing pixels must match its CSS size and actual browser density`);
    const capture = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: true,
      clip: { x: canvas.x, y: canvas.y, width: canvas.width, height: canvas.height, scale: 1 } });
    const visualOutput = path.join(docs, browserDensity === 2 ? '.verification/visual-qa/web-retina' : '.verification/visual-qa/web');
    await fs.mkdir(visualOutput, { recursive: true });
    await fs.writeFile(path.join(visualOutput, `${slug}.png`), Buffer.from(capture.data, 'base64'));
    results.push({ name: 'browser-preview-startup', component: slug, ...state });
  }
  // Winit's DOM paste handler bypasses TextInput key callbacks. Exercise the
  // real browser event, including formatted paste and leaving the Slint window.
  await navigate('/preview/?component=telephone-field');
  for (let count = 0; count < 300; count++) {
    if (await evaluate('Boolean(window.previewReady || window.previewError)')) break;
    await pause(100);
  }
  assert(await evaluate('Boolean(window.previewReady && !window.previewError)'));
  await call('Input.dispatchMouseEvent', { type: 'mousePressed', button: 'left', clickCount: 1, x: 120, y: 70 });
  await call('Input.dispatchMouseEvent', { type: 'mouseReleased', button: 'left', clickCount: 1, x: 120, y: 70 });
  await pause(100);
  const phonePaste = await evaluate(`(() => {
    const target = document.activeElement;
    const paste = text => {
      const data = new DataTransfer(); data.setData('text/plain', text);
      const event = new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true });
      let propagated = false; const observe = () => { propagated = true; };
      document.addEventListener('paste', observe, true);
      target.dispatchEvent(event); document.removeEventListener('paste', observe, true);
      return { prevented: event.defaultPrevented, propagated };
    };
    return { rejected: paste('+1 ABC 555-0123'), formatted: paste('+1 (438) 555-0123') };
  })()`);
  assert(phonePaste.rejected.prevented && !phonePaste.rejected.propagated, 'Phone must reject letters before the browser backend inserts them');
  assert(phonePaste.formatted.propagated, 'Phone must let the native editor handle supported formatted paste');
  assert(await evaluate(`(async () => {
    const input = document.createElement('input'); document.body.append(input); input.focus();
    await new Promise(resolve => setTimeout(resolve, 200));
    const data = new DataTransfer(); data.setData('text/plain', 'Ada');
    const event = new ClipboardEvent('paste', { clipboardData: data, bubbles: true, cancelable: true });
    input.dispatchEvent(event); input.remove(); return !event.defaultPrevented;
  })()`), 'Phone paste guard must leave external HTML inputs alone');
  results.push({ name: 'phone-browser-paste-guard', ...phonePaste, externalInputPreserved: true });
  for (const locale of ['en', 'fr']) {
    await call('Emulation.setDeviceMetricsOverride', { width: 375, height: 900, deviceScaleFactor: browserDensity, mobile: false });
    const start = requests.length;
    await navigate(`${locale === 'fr' ? '/fr' : ''}/components/button/`);
    assert(!requests.slice(start).some(url => /\.(?:wasm)(?:\?|$)/.test(url)), 'Landing on a component must not fetch WebAssembly');
    assert.deepEqual(await evaluate('[...document.querySelector(".preview-theme").options].map(option=>option.value)'), ['light', 'dark'], 'The built-in presentation supports both color modes');
    await click('.run-preview');
    for (let count = 0; count < 300; count++) {
      if (await evaluate('document.querySelector(".component-preview").hidden')) break;
      await pause(100);
    }
    assert(await evaluate('document.querySelector(".component-preview").hidden && !document.querySelector(".run-preview").disabled'), 'Run preview must replace the poster with a ready iframe');
    assert(await evaluate('document.documentElement.scrollWidth <= innerWidth'), 'Live preview must fit mobile width');
    await evaluate('document.querySelector(".preview-frame").scrollIntoView({block:"center"})');
    const point = await evaluate('(()=>{const f=document.querySelector("iframe");const r=f.getBoundingClientRect();const c=f.contentDocument.querySelector("canvas");return {x:r.x+80*r.width/c.getBoundingClientRect().width,y:r.y+44*r.width/c.getBoundingClientRect().width}})()');
    for (let i = 0; i < 3; i++) {
      await call('Input.dispatchMouseEvent', { type:'mousePressed', button:'left', clickCount:1, ...point });
      await call('Input.dispatchMouseEvent', { type:'mouseReleased', button:'left', clickCount:1, ...point });
    }
    await pause(200);
    const capture = await call('Page.captureScreenshot', { format:'png', captureBeyondViewport:false });
    await fs.writeFile(path.join(output, `${locale}-live-button.png`), Buffer.from(capture.data, 'base64'));
    await click('.run-preview');
    assert(await evaluate('!document.querySelector("iframe") && !document.querySelector(".component-preview").hidden'), 'Stop preview must release the browser app and restore its poster');
    results.push({ name:'on-demand-mobile-preview', locale, loaded:true, stopped:true });
  }
  // An iframe can fit the page while its canvas/credits still overflow inside.
  // Check the desktop gutter, retina, mobile scaling and enlarged credits.
  for (const [width, density] of [[1500, browserDensity], [375, browserDensity]]) {
    await call('Emulation.setDeviceMetricsOverride', { width, height: 1000, deviceScaleFactor: density, mobile: false });
    await navigate('/fr/components/input-group/');
    await click('.run-preview');
    for (let count = 0; count < 300; count++) {
      if (await evaluate('document.querySelector(".component-preview").hidden')) break;
      await pause(100);
    }
    await evaluate('document.querySelector("iframe").contentDocument.querySelector("footer").style.fontSize="22px"');
    await pause(300);
    const geometry = await evaluate(`(() => {
      const host = document.querySelector('.preview-frame').getBoundingClientRect();
      const frame = document.querySelector('iframe'); const bounds = frame.getBoundingClientRect();
      const innerDocument = frame.contentDocument;
      const canvas = innerDocument.querySelector('canvas'); const canvasBounds = canvas.getBoundingClientRect();
      return { centered: Math.abs(bounds.x + bounds.width / 2 - host.x - host.width / 2) < 1,
        fitted: bounds.width <= host.width && bounds.height <= host.height,
        backingPixelsMatch: Math.abs(canvas.width - canvasBounds.width * frame.contentWindow.devicePixelRatio) <= 1 && Math.abs(canvas.height - canvasBounds.height * frame.contentWindow.devicePixelRatio) <= 1,
        noHorizontalOverflow: innerDocument.documentElement.scrollWidth <= frame.contentWindow.innerWidth,
        noVerticalOverflow: innerDocument.documentElement.scrollHeight <= frame.contentWindow.innerHeight,
        footerVisible: innerDocument.querySelector('footer').getBoundingClientRect().bottom <= frame.contentWindow.innerHeight + 1 };
    })()`);
    assert(Object.values(geometry).every(Boolean), `Preview geometry ${width}/${density}: ${JSON.stringify(geometry)}`);
    results.push({ name: 'preview-canvas-and-credits-fit', width, density, ...geometry });
    await evaluate('document.querySelector(".preview-frame").scrollIntoView({block:"center"})');
    const capture = await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false });
    await fs.writeFile(path.join(output, `input-group-${width}-${density}.png`), Buffer.from(capture.data, 'base64'));
  }
  // Canvas cards stay independent of Slint compilation. Gallery glyphs and
  // copyable snippets must follow the selected icon in both languages.
  for (const locale of ['en','fr']) {
    await call('Emulation.setDeviceMetricsOverride', { width: 1440, height: 1050, deviceScaleFactor: browserDensity, mobile: false });
    await navigate(locale==='fr'?'/fr/components/':'/components/');
    assert.equal(await evaluate('document.querySelectorAll("[data-component-mockup]").length'),124);
    assert(await evaluate(`(()=>{const c=document.querySelector('[data-component-mockup="button"]');return c.getContext('2d').getImageData(10,10,1,1).data[3]>0})()`), 'Visible mockup must be painted');
    assert.equal(await evaluate('document.querySelectorAll(".family-illustration").length'),0);
    const catalogue = await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});
    await fs.writeFile(path.join(output,`${locale}-component-mockups.png`),Buffer.from(catalogue.data,'base64'));
    const galleryRequests=requests.length;
    await navigate(locale==='fr'?'/fr/components/icon/':'/components/icon/');
    assert.equal(await evaluate('document.querySelectorAll(".icon-tile").length'),1999);
    assert(!requests.slice(galleryRequests).some(url=>url.includes('.wasm')), 'The icon gallery must not start a WASM preview');
    await evaluate(`(()=>{const s=document.querySelector('#icon-search');s.value='arrow up';s.dispatchEvent(new Event('input'));const c=document.querySelector('#icon-category');c.value='arrows';c.dispatchEvent(new Event('change'))})()`);
    assert(await evaluate(`(()=>{const tiles=[...document.querySelectorAll('.icon-tile:not([hidden])')];return tiles.length>0&&tiles.every(t=>t.dataset.search.includes('arrow')&&t.dataset.search.includes('up')&&t.dataset.categories.split(' ').includes('arrows'))})()`));
    await click('[data-icon="arrow-up"]');
    assert.equal(await evaluate('document.querySelector("#selected-icon-name").textContent'),'arrow-up');
    assert(await evaluate('document.querySelector("#icon-snippet").textContent.includes("LucideIcons.arrow-up()")'));
    await evaluate('document.querySelector(\'[data-icon="arrow-up-right"]\').focus()');
    await keyEnter();
    assert.equal(await evaluate('document.querySelector("#selected-icon-name").textContent'),'arrow-up-right');
    await evaluate(`(()=>{const m=document.querySelector('#icon-integration');m.value='pressable';m.dispatchEvent(new Event('change'));navigator.clipboard.writeText=async text=>window.copiedIcon=text})()`);
    await click('.icon-inspector .code-actions button');
    assert(await evaluate('window.copiedIcon===document.querySelector("#icon-snippet").textContent&&window.copiedIcon.includes("KitPressable")&&window.copiedIcon.includes("callback action-requested();")'));
    await evaluate(`(()=>{const s=document.querySelector('#icon-search');s.value='no-such-lucide-icon';s.dispatchEvent(new Event('input'))})()`);
    assert(await evaluate('!document.querySelector("#icon-empty").hidden'));
    await evaluate(`(()=>{const s=document.querySelector('#icon-search');s.value='';s.dispatchEvent(new Event('input'));const c=document.querySelector('#icon-category');c.value='';c.dispatchEvent(new Event('change'));document.querySelector('#lucide-title').scrollIntoView()})()`);
    for(const [variant,width,dark,textScale] of [['light',1440,false,1],['dark',1440,true,1],['mobile',375,false,1],['text200',1100,false,2]]) {
      await call('Emulation.setDeviceMetricsOverride',{width,height:1100,deviceScaleFactor:browserDensity,mobile:false});
      await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-color-scheme',value:dark?'dark':'light'}]});
      await evaluate(`document.documentElement.style.fontSize='${16*textScale}px';document.querySelector('#lucide-title').scrollIntoView()`);
      await pause(300);
      assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'),`${locale} icon gallery overflows at ${variant}`);
      const screenshot=await call('Page.captureScreenshot',{format:'png',captureBeyondViewport:false});
      await fs.writeFile(path.join(output,`${locale}-lucide-${variant}.png`),Buffer.from(screenshot.data,'base64'));
    }
    results.push({name:'visual-catalogues',locale,icons:1999,mockups:124,search:true,categories:true,copy:true});
    await evaluate("document.documentElement.style.fontSize=''");
  }
  passed = true;
  console.log(`Browser checks PASS: ${results.filter(row => row.route).length} page captures, ${results.filter(row => row.name === "browser-preview-startup").length} preview startups, EN/FR on-demand mobile previews, delayed script/image CLS, light/dark, text200, keyboard/pointer, no-JS menu and filter.`);
} finally {
  await fs.writeFile(path.join(output, 'results.json'), JSON.stringify({ passed, chrome, results }, null, 2) + '\n');
  if (sourceId && socket?.readyState === WebSocket.OPEN) await call('Page.removeScriptToEvaluateOnNewDocument', { identifier: sourceId });
  socket?.close();
  processChrome.kill();
  await once(processChrome, 'exit').catch(() => {});
  server.close();
  await fs.rm(profile, { recursive: true, force: true });
}
