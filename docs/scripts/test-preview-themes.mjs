// SPDX-License-Identifier: GPL-3.0-or-later
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';

class Element {
  constructor() {
    this.listeners = new Map(); this.dataset = {}; this.style = {}; this.children = [];
    this.classList = {add() {}, remove() {}}; this.clientWidth = 500;
    this.options = [{textContent:'Light'}, {textContent:'Dark'}];
  }
  addEventListener(name, callback) { this.listeners.set(name, callback); }
  emit(name, event = {}) { this.listeners.get(name)?.(event); }
  setAttribute() {}
  append(child) { this.children.push(child); }
  remove() { this.removed = true; }
  getBoundingClientRect() { return {width:520, height:300, bottom:344}; }
}
const siteSource = fs.readFileSync(new URL('../src/assets/site.js', import.meta.url), 'utf8');
const shellTemplate = fs.readFileSync(new URL('../src/preview.njk', import.meta.url), 'utf8');
const storageKey = 'slint-kit-preview-theme';
function environment({saved = null, search = '', dark = false, french = false, blockedStorage = false} = {}) {
  const values = new Map(saved === null ? [] : [[storageKey, saved]]);
  const location = {origin:'https://slint-kit.test', pathname:'/preview/', search};
  const window = new Element();
  const media = new Map();
  const context = {
    window, location, URLSearchParams, console,
    navigator:{}, ResizeObserver:class {observe() {}},
    setTimeout() {return 1;}, clearTimeout() {},
    requestAnimationFrame(callback) {callback();},
    matchMedia(query) {if(!media.has(query)){const result=new Element();result.matches=query.includes('color-scheme')?dark:false;media.set(query,result);}return media.get(query);},
    localStorage:{
      getItem(key) {if (blockedStorage) throw Error('Storage unavailable'); return values.get(key) ?? null;},
      setItem(key, value) {if (blockedStorage) throw Error('Storage unavailable'); values.set(key, value);},
    },
    history:{replaceState(_state, _title, url) {location.search = new URL(url, location.origin).search;}},
  };
  window.matchMedia = context.matchMedia;
  context.document = {documentElement:{lang:french ? 'fr' : 'en', dataset:{}}};
  return {context, values, window, media};
}
function page(options) {
  const state = environment(options);
  const preview = new Element();
  preview.dataset = {component:'button', width:'520', height:'300'};
  const elements = Object.fromEntries(['.run-preview','.preview-status','.preview-frame','.preview-theme','figure'].map(name => [name, new Element()]));
  elements['.preview-theme'].disabled = true;
  preview.querySelector = name => elements[name];
  state.context.document.querySelector = () => null;
  state.context.document.querySelectorAll = name => name === '.live-preview' ? [preview] : [];
  const frames = [];
  state.context.document.createElement = () => {
    const frame = new Element();
    frame.messages = [];
    frame.contentWindow = {postMessage(data, origin) {frame.messages.push({data, origin});}};
    frames.push(frame); return frame;
  };
  vm.runInNewContext(siteSource, state.context);
  return {...state, elements, frames, message(data, source = frames.at(-1)?.contentWindow, origin = state.context.location.origin) {
    state.window.emit('message', {data, source, origin});
  }};
}

test('saved theme is available before launch and does not load a frame', () => {
  const state = page({saved:'dark', french:true});
  assert.equal(state.elements['.preview-theme'].value, 'dark');
  assert.equal(state.elements['.preview-theme'].disabled, false);
  assert.equal(state.frames.length, 0);
  state.elements['.run-preview'].emit('click');
  const parameters = new URL(state.frames[0].src, state.context.location.origin).searchParams;
  assert.equal(parameters.get('theme'), 'dark');
  assert.equal(parameters.get('lang'), 'fr');
});

test('explicit theme wins, invalid values fall back, and blocked storage still works', () => {
  assert.equal(page({saved:'light', search:'?theme=dark'}).elements['.preview-theme'].value, 'dark');
  assert.equal(page({saved:'dark', search:'?theme=invalid'}).elements['.preview-theme'].value, 'dark');
  assert.equal(page({blockedStorage:true, dark:true}).elements['.preview-theme'].value, 'dark');
});

test('theme changes survive loading and update the existing frame without restarting it', () => {
  const state = page({saved:'light'});
  state.elements['.run-preview'].emit('click');
  const choice = state.elements['.preview-theme'];
  choice.value = 'dark'; choice.emit('change');
  state.message({type:'slint-kit-preview', state:'ready', theme:'light', width:520, height:344});
  assert.equal(choice.value, 'dark', 'a late ready event must not undo the latest choice');
  assert.equal(state.frames.length, 1);
  assert.deepEqual(JSON.parse(JSON.stringify(state.frames[0].messages.at(-1).data)), {type:'slint-kit-theme', theme:'dark', requestId:2});
  assert.equal(state.values.get(storageKey), 'dark');
  state.message({type:'slint-kit-preview', state:'theme', theme:'light'});
  assert.equal(choice.value, 'light', 'the child footer can change the same preference');
  assert.equal(state.frames.length, 1);
  state.message({type:'slint-kit-preview', state:'theme', theme:'dark'}, {}, 'https://other.test');
  assert.equal(choice.value, 'light', 'untrusted frames cannot change the theme');
});

async function shell(options) {
  const state = environment({search:'?component=button', ...options});
  const elements = Object.fromEntries(['canvas','footer','#preview-theme','#theme-label','#license','#third-party','#source','#error'].map(name => [name, new Element()]));
  state.context.document.querySelector = name => elements[name];
  state.context.document.getElementById = name => elements['#' + name];
  const messages = [], calls = [];
  let starts = 0;
  state.context.parent = {postMessage(data) {messages.push(data);}};
  state.context.testRuntime = {async default() {starts++; calls.push(['start', new URLSearchParams(state.context.location.search).get('theme')]);}, set_theme(theme) {calls.push(['theme', theme]);}};
  let source = shellTemplate.match(/<script type="module">([\s\S]+)<\/script>/)[1];
  source = source.replace('{{ webPreviews.components | scriptJson | safe }}', '["button"]')
    .replace('{{ webPreviews.entries | scriptJson | safe }}', JSON.stringify({button:{tailwind:{base:'/preview-assets', sources:{'NOTICE.txt':'/notice','THIRD_PARTY_NOTICES.txt':'/third-party','source.tar.gz':'/source'}}}}))
    .replace("runtime = await import(entry.base + '/slint_kit_previews.js');", 'runtime = testRuntime;');
  await vm.runInNewContext('(async () => {' + source + '\n})()', state.context);
  return {...state, elements, messages, calls, starts:() => starts};
}

test('direct shell initializes the requested theme and changes it in place', async () => {
  const state = await shell({search:'?component=button&theme=dark'});
  assert.deepEqual(state.calls[0], ['start','dark']);
  assert.equal(state.context.window.previewTheme, 'dark');
  const choice = state.elements['#preview-theme'];
  choice.value = 'light'; choice.emit('change');
  assert.equal(state.context.window.previewTheme, 'light');
  assert.equal(state.context.document.documentElement.dataset.previewTheme, 'light');
  assert.equal(state.starts(), 1);
  assert.equal(state.messages.at(-1).state, 'theme');
  assert.equal(state.messages.at(-1).theme, 'light');
  state.window.emit('message', {origin:state.context.location.origin, source:state.context.parent, data:{type:'slint-kit-theme', theme:'dark'}});
  assert.equal(choice.value, 'dark');
  assert.equal(state.calls.at(-1)[1], 'dark');
  assert.equal(state.starts(), 1);
});

test('motion preference changes update the existing Slint window', async () => {
  const state=await shell({search:'?component=button&theme=dark'});
  const preference=state.media.get('(prefers-reduced-motion: reduce)');
  preference.matches=true;
  preference.emit('change');
  assert.deepEqual(state.calls.at(-1),['theme','dark']);
  assert.equal(state.starts(),1);
  assert.equal(state.context.window.previewTheme,'dark');
});

test('a stale theme acknowledgement cannot replace the latest loading choice', () => {
  const state=page({saved:'light'});
  state.elements['.run-preview'].emit('click');
  const choice=state.elements['.preview-theme'];
  choice.value='dark';choice.emit('change');
  const oldRequest=state.frames[0].messages.at(-1).data;
  choice.value='light';choice.emit('change');
  state.message({type:'slint-kit-preview',state:'theme',theme:'dark',requestId:oldRequest.requestId});
  state.message({type:'slint-kit-preview',state:'ready',theme:'dark',width:520,height:344});
  assert.equal(choice.value,'light');
  assert.equal(state.frames[0].messages.at(-1).data.theme,'light');
});
