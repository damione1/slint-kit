// SPDX-License-Identifier: GPL-3.0-or-later
// Exercise the real page and Worker handler; external Turnstile/email are test doubles.
import fs from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { contact } from '../contact-worker.mjs';

const docs = fileURLToPath(new URL('../', import.meta.url));
const out = path.join(docs, '.verification/contact');
await fs.mkdir(out, { recursive: true });
const messages = [], proofs = [], errors = [];
let failEmail = false, rateLimited = false, configured = true, postCount = 0;
const env = {
  TURNSTILE_SITE_KEY: 'test-site-key', TURNSTILE_SECRET_KEY: 'test-secret',
  CONTACT_TO: 'private-inbox@example.test', CONTACT_FROM: 'notifications@example.test',
  CONTACT_EMAIL: { send: async message => { if (failEmail) throw Error('private-inbox@example.test provider error'); messages.push(message); } },
  CONTACT_RATE_LIMITER: { limit: async () => ({ success: !rateLimited }) },
};
const server = http.createServer(async (req, res) => {
  try {
    const url = new URL(req.url, `http://${req.headers.host}`);
    if (url.pathname.startsWith('/api/contact')) {
      const chunks = []; for await (const chunk of req) chunks.push(chunk);
      if (req.method === 'POST') postCount++;
      const request = new Request(url, { method: req.method, headers: { ...req.headers, 'CF-Connecting-IP': '192.0.2.1' }, ...(req.method === 'POST' ? { body: Buffer.concat(chunks) } : {}) });
      const response = await contact(request, configured ? env : {}, async () => Response.json({ success: true, hostname: url.hostname, action: 'contact' }));
      res.writeHead(response.status, Object.fromEntries(response.headers)); res.end(Buffer.from(await response.arrayBuffer())); return;
    }
    const target = path.resolve(docs, '_site', '.' + url.pathname, url.pathname.endsWith('/') ? 'index.html' : '');
    assert(target.startsWith(path.join(docs, '_site') + path.sep));
    const mime = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.json': 'application/json', '.svg': 'image/svg+xml' };
    res.writeHead(200, { 'Content-Type': mime[path.extname(target)] || 'application/octet-stream' }); res.end(await fs.readFile(target));
  } catch { res.writeHead(404); res.end(); }
});
server.listen(0, '127.0.0.1'); await once(server, 'listening');
const origin = `http://127.0.0.1:${server.address().port}`;
const profile = await fs.mkdtemp(path.join(os.tmpdir(), 'slint-contact-chrome-'));
const chrome = spawn(process.env.CHROME_PATH || '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', ['--headless=new', '--no-first-run', '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank'], { stdio: 'ignore' });
const pause = ms => new Promise(resolve => setTimeout(resolve, ms));
let socket, sequence = 0; const pending = new Map();
function call(method, params = {}) {
  const id = ++sequence;
  return new Promise((resolve, reject) => { const timer = setTimeout(() => { pending.delete(id); reject(Error(method + ' timed out')); }, 20_000); pending.set(id, { resolve, reject, timer }); socket.send(JSON.stringify({ id, method, params })); });
}
async function evaluate(expression) {
  const result = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails));
  return result.result.value;
}
async function waitFor(expression) { for (let i = 0; i < 100; i++) { if (await evaluate(expression).catch(() => false)) return; await pause(50); } throw Error('Timed out: ' + expression); }
async function navigate(route) { await call('Page.navigate', { url: origin + route }); await waitFor(`location.pathname===${JSON.stringify(route.split('?')[0])}&&document.readyState==='complete'`); }
async function click(selector) {
  const point = await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.scrollIntoView({block:'center'});const r=e.getBoundingClientRect();return{x:r.x+r.width/2,y:r.y+r.height/2}})()`);
  for (const type of ['mousePressed', 'mouseReleased']) await call('Input.dispatchMouseEvent', { type, button: 'left', clickCount: 1, ...point });
}
async function capture(name) { await fs.writeFile(path.join(out, name + '.png'), Buffer.from((await call('Page.captureScreenshot', { format: 'png' })).data, 'base64')); proofs.push(name); }
async function fill() { await evaluate(`(()=>{const f=document.querySelector('[data-contact-form]');for(const [name,value] of Object.entries({name:'Ada',email:'ada@example.test',message:'A native Rust application, with typed fields.'}))f.elements.namedItem(name).value=value})()`); }
const widgetScript = `window.turnstile={render:(element,options)=>{window.widgetOptions=options;window.challengeExpired=()=>options['expired-callback']();element.textContent='Spam check (test double)';element.style.minHeight='65px';options.callback('test-token');return 'test-widget';},reset:()=>window.widgetOptions.callback('fresh-test-token'),remove:()=>{}};`;
try {
  let port; for (let i = 0; i < 100; i++) { try { port = (await fs.readFile(path.join(profile, 'DevToolsActivePort'), 'utf8')).split('\n')[0]; break; } catch { await pause(50); } }
  assert(port);
  const tabs = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
  socket = new WebSocket(tabs.find(tab => tab.type === 'page').webSocketDebuggerUrl);
  socket.addEventListener('message', event => {
    const data = JSON.parse(event.data);
    if (data.method === 'Runtime.exceptionThrown') errors.push(data.params.exceptionDetails.text);
    if (data.method === 'Fetch.requestPaused') void call('Fetch.fulfillRequest', { requestId: data.params.requestId, responseCode: 200, responseHeaders: [{ name: 'Content-Type', value: 'text/javascript' }], body: Buffer.from(widgetScript).toString('base64') });
    const item = pending.get(data.id);
    if (item) { pending.delete(data.id); clearTimeout(item.timer); data.error ? item.reject(Error(JSON.stringify(data.error))) : item.resolve(data.result); }
  });
  await once(socket, 'open'); await call('Page.enable'); await call('Runtime.enable');
  await call('Fetch.enable', { patterns: [{ urlPattern: 'https://challenges.cloudflare.com/turnstile/*' }] });
  for (const locale of ['en', 'fr']) {
    const route = `${locale === 'fr' ? '/fr' : ''}/contact/`;
    for (const [theme, width, text200] of [['light', 1280, false], ['dark', 1280, false], ['dark', 375, false], ['light', 375, true], ['dark', 375, true]]) {
      await call('Emulation.setDeviceMetricsOverride', { width, height: 1000, deviceScaleFactor: 1, mobile: false });
      await navigate(route + '?theme=' + theme); await waitFor('!document.querySelector(".contact-submit").disabled');
      if (text200) await evaluate('document.documentElement.style.fontSize="200%"');
      assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'), 'no overflow: ' + [locale, theme, width, text200]);
      assert.equal(await evaluate('window.widgetOptions.theme'), theme);
      assert.equal(await evaluate('window.widgetOptions.language'), locale);
      assert(await evaluate('[...document.querySelectorAll(".contact-field input,.contact-field textarea,.contact-field select")].every(e=>e.labels.length===1)'));
      assert(!await evaluate('document.documentElement.outerHTML.includes("private-inbox@example.test")'));
      await evaluate('document.querySelector("h1").scrollIntoView({block:"start"})');
      await capture(`${locale}-${theme}-${width}-${text200 ? 'text200' : 'normal'}`);
    }
    await call('Emulation.setDeviceMetricsOverride', { width: 1280, height: 1000, deviceScaleFactor: 1, mobile: false });
    await navigate(route + '?theme=dark'); await waitFor('!document.querySelector(".contact-submit").disabled');
    const emptyCount = postCount; await click('.contact-submit'); assert.equal(postCount, emptyCount, 'native required fields block empty submission');
    await fill(); await evaluate('document.querySelector("#contact-email").value="invalid"');
    const before = postCount; await click('.contact-submit'); assert.equal(postCount, before, 'native email validation blocks malformed email');
    await fill(); await evaluate('window.challengeExpired()'); assert(await evaluate('document.querySelector(".contact-submit").disabled'));
    await evaluate('window.turnstile.reset()');
    failEmail = true; await click('.contact-submit'); await waitFor('document.querySelector("[data-contact-status]").textContent.includes(' + JSON.stringify(locale === 'fr' ? 'indisponible' : 'unavailable') + ')');
    assert.equal(await evaluate('document.querySelector("#contact-message").value'), 'A native Rust application, with typed fields.');
    rateLimited = true; await click('.contact-submit'); await waitFor('document.querySelector("[data-contact-status]").textContent.includes(' + JSON.stringify(locale === 'fr' ? 'Trop de tentatives' : 'Too many attempts') + ')');
    assert.equal(await evaluate('document.querySelector("#contact-email").value'), 'ada@example.test');
    rateLimited = false; failEmail = false;
    await evaluate('document.querySelector("#contact-message").value="          "');
    await click('.contact-submit'); await waitFor('document.querySelector("#contact-message").getAttribute("aria-invalid")==="true"');
    assert.equal(await evaluate('document.activeElement.id'), 'contact-message');
    assert.equal(await evaluate('document.querySelector("#contact-email").value'), 'ada@example.test');
    await fill(); await click('.contact-submit'); await waitFor('document.querySelector("[data-contact-status]").textContent.includes(' + JSON.stringify(locale === 'fr' ? 'a été envoyé' : 'has been sent') + ')');
    assert.equal(await evaluate('document.querySelector("#contact-message").value'), '');
    await evaluate('document.querySelector("[data-contact-status]").scrollIntoView({block:"center"})'); await capture(`${locale}-success`);
    await evaluate('document.querySelector(".site-theme").value="light";document.querySelector(".site-theme").dispatchEvent(new Event("change"))');
    await waitFor('window.widgetOptions.theme==="light"');
  }
  assert.equal(messages.length, 2); assert(messages.every(message => message.to === env.CONTACT_TO));
  configured = false; await navigate('/contact/'); await waitFor('document.querySelector("[data-contact-status]").textContent.includes("unavailable")'); assert(await evaluate('document.querySelector(".contact-submit").disabled'));
  await call('Emulation.setScriptExecutionDisabled', { value: true });
  await navigate('/fr/contact/');
  assert(await evaluate('document.querySelector("noscript").textContent.includes("JavaScript")'));
  assert(await evaluate('document.querySelector(".contact-fallback a").href.endsWith("/discussions")'));
  assert(await evaluate('document.querySelector(".contact-submit").disabled'));
} finally {
  if (socket?.readyState === WebSocket.OPEN) {
    await call('Emulation.setScriptExecutionDisabled', { value: false }).catch(() => {});
  }
  socket?.close(); const stopped = once(chrome, 'close'); chrome.kill(); await stopped; await new Promise(resolve => server.close(resolve));
  await fs.rm(profile, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}
assert.deepEqual(errors, []);
await fs.writeFile(path.join(out, 'results.json'), JSON.stringify({ passed: true, backend: 'Chrome headless; Turnstile/email test doubles, no real email sent', proofs, messages: messages.length }, null, 2));
console.log('Contact PASS: EN/FR, themes, mobile/text200, native validation, expiry, preserved drafts, rate limit and successful submission.');
