// SPDX-License-Identifier: GPL-3.0-or-later
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
const source = fs.readFileSync(new URL('../src/assets/analytics.js', import.meta.url), 'utf8');
function environment(url = 'https://slint-kit.dev/fr/components/button/', debug = false) {
  const events = [], listeners = new Map();
  const document = {documentElement:{lang:'fr'},addEventListener(name, handler) {listeners.set(name, handler);}};
  const window = {location:new URL(url),sessionStorage:{getItem:() => debug ? '1' : null},gtag(...args) {events.push(JSON.parse(JSON.stringify(args)));}};
  vm.runInNewContext(source, {document, window, URL});
  return {events, listeners, track:window.slintKitAnalytics.track};
}
function link(href, position = 'content', results = null) {
  const element = {href,closest(selector) {return selector === '.doc-search-results' ? results : selector === position ? {} : null;}};
  return {target:{closest:() => element}, element};
}
test('events carry page context without clipboard, query or unknown fields', () => {
  const state = environment();
  state.track('code_copy', {snippet_language:'rust',clipboard:'private code',search_term:'private query',value:NaN});
  assert.deepEqual(state.events, [['event','code_copy',{page_locale:'fr',content_group:'component',component_name:'button',snippet_language:'rust'}]]);
  state.track('arbitrary_event'); assert.equal(state.events.length, 1);
});
test('local builds send no analytics and debug applies only to the current session', () => {
  const local = environment('http://localhost:8080/'); local.track('code_copy'); assert.equal(local.events.length, 0);
  const debug = environment('https://slint-kit.dev/', true); debug.track('code_copy'); assert.equal(debug.events[0][2].debug_mode, true);
  const normal = environment(); normal.track('code_copy'); assert.equal(normal.events[0][2].debug_mode, undefined);
});
test('repository and starter intents ignore right clicks, foreign downloads and other repositories', () => {
  const state = environment(), click = state.listeners.get('click'), aux = state.listeners.get('auxclick');
  click({type:'click',button:0,...link('https://github.com/damione1/slint-kit?secret=ignored','header')});
  aux({type:'auxclick',button:1,...link('https://slint-kit.dev/quickstart/projects/text-field.zip')});
  aux({type:'auxclick',button:2,...link('https://github.com/damione1/slint-kit')});
  click({type:'click',button:0,...link('https://github.com/damione1/slint-kit-other')});
  click({type:'click',button:0,...link('https://other.test/quickstart/projects/button.zip')});
  assert.deepEqual(state.events.map(e => e[1]), ['github_visit','starter_download']);
  assert.equal(state.events[0][2].link_placement, 'header'); assert.equal(state.events[0][2].content_type, 'repository');
  assert.equal(state.events[1][2].component_name, 'text-field'); assert(!JSON.stringify(state.events).includes('secret'));
});
test('search selection records destination and rank without a query', () => {
  const state = environment();
  const results = {querySelectorAll:() => [first.element, second.element]};
  const first = link('https://slint-kit.dev/fr/components/button/', 'content', results);
  const second = link('https://slint-kit.dev/fr/guides/forms/', 'content', results);
  state.listeners.get('click')({type:'click',button:0,...second});
  assert.equal(state.events[0][1], 'search_result_click'); assert.equal(state.events[0][2].result_position, 2);
  assert.equal(state.events[0][2].destination_path, '/fr/guides/forms/');
});
test('workspace intent needs a trusted interaction on the ready canvas and counts once', () => {
  const state = environment('https://slint-kit.dev/examples/workspace/');
  const pointer = state.listeners.get('pointerdown'), keyboard = state.listeners.get('keydown');
  pointer({isTrusted:true,target:{tagName:'INPUT',dataset:{ready:'true'}}});
  pointer({isTrusted:false,target:{tagName:'CANVAS',dataset:{ready:'true'}}});
  keyboard({isTrusted:true,target:{tagName:'CANVAS',dataset:{ready:'false'}}});
  pointer({isTrusted:true,target:{tagName:'CANVAS',dataset:{ready:'true'}}});
  keyboard({isTrusted:true,target:{tagName:'CANVAS',dataset:{ready:'true'}}});
  assert.deepEqual(state.events.map(e => e[1]), ['workspace_interaction']);
});
async function copyCode(succeeds) {
  const events = [], elements = [];
  const code = {className:'language-rust',textContent:'private code must never be sent',parentElement:{prepend() {}}};
  const document = {querySelector:() => null,querySelectorAll:selector => selector === 'pre > code' ? [code] : [],documentElement:{lang:'en',dataset:{}},createElement() {
    const element = {listeners:new Map(),setAttribute() {},append() {},addEventListener(name,handler) {this.listeners.set(name,handler);}}; elements.push(element); return element;
  }};
  const window = {slintKitAnalytics:{track:(...args) => events.push(args)},addEventListener() {},dispatchEvent() {},matchMedia:() => ({matches:false,addEventListener() {}})};
  vm.runInNewContext(fs.readFileSync(new URL('../src/assets/site.js', import.meta.url),'utf8'),{document,window,location:new URL('https://slint-kit.test/'),URLSearchParams,CustomEvent:class {},localStorage:{getItem:() => null,setItem() {}},navigator:{clipboard:{async writeText() {if (!succeeds) throw Error('Permission denied');}}}});
  await elements.find(e => e.type === 'button').listeners.get('click')(); return JSON.parse(JSON.stringify(events));
}
test('a successful clipboard write records one copy and a denied write records none', async () => {
  assert.deepEqual(await copyCode(true), [['code_copy',{snippet_language:'rust',content_type:'snippet'}]]);
  assert.deepEqual(await copyCode(false), []);
});

test('search waits for settled results and cancels stale or closed queries without sending text', async () => {
  const {setupSearch} = await import('../src/assets/search.js');
  const events = [], previousWindow = globalThis.window;
  globalThis.window = {slintKitAnalytics:{track:(name, values) => events.push({name, values})}};
  const element = () => ({listeners:new Map(),addEventListener(name, handler) {this.listeners.set(name,handler);},focus() {}});
  const trigger = {...element(),dataset:{index:'/assets/search-index-en.json'}}, input = {...element(),value:''};
  const list = {replaceChildren() {},querySelector:() => null}, status = {}, form = element(), close = element();
  const dialog = {...element(),open:false,showModal() {this.open=true;},querySelector(selector) {
    return {'input':input,'.doc-search-results':list,'[role="status"]':status,'.doc-search-close':close,'form':form}[selector];
  }};
  const document = {...element(),documentElement:{lang:'en'},activeElement:trigger,querySelector(selector) {return {'.doc-search-trigger':trigger,'#doc-search':dialog}[selector];}};
  try {
    setupSearch(document,async () => ({ok:true,json:async () => ({schema:1,locale:'en',entries:[]})}));
    trigger.listeners.get('click')();
    input.value='private@example.com'; await input.listeners.get('input')();
    input.value='private query'; await input.listeners.get('input')();
    await new Promise(resolve => setTimeout(resolve,650));
    assert.deepEqual(events.map(e=>e.name),['search_open','view_search_results','search_no_results']);
    assert.equal(events[1].values.result_count,0);
    assert(!JSON.stringify(events).includes('private'));
    input.value='closed query'; await input.listeners.get('input')();
    dialog.open=false;dialog.listeners.get('close')();
    await new Promise(resolve => setTimeout(resolve,650));
    assert.equal(events.length,3);
  } finally {globalThis.window=previousWindow;}
});
