// SPDX-License-Identifier: GPL-3.0-or-later
import assert from 'node:assert/strict';
import test from 'node:test';
import { lucideCatalogue } from './lucide-catalogue.mjs';
import { scanComponents } from './inventory.mjs';
import { drawMockup, mockups } from '../src/assets/component-mockups.js';
import { iconSnippet } from '../src/assets/lucide-gallery.js';

test('the gallery covers every generated Slint glyph, including fill layers and licenses', () => {
  const catalogue=lucideCatalogue();
  assert.equal(catalogue.icons.length,1999);
  assert.equal(new Set(catalogue.icons.map(icon=>icon.name)).size,1999);
  assert(catalogue.sprite.includes('ISC License'));
  assert(catalogue.sprite.includes('MIT License'));
  const tag=catalogue.icons.find(icon=>icon.name==='tag');
  assert(tag.fill.length>0);
  assert(catalogue.sprite.includes(`<path d="${tag.fill}" fill="currentColor"/>`));
  assert(catalogue.icons.every(icon=>icon.stroke&&icon.categories.length));
  for(const icon of catalogue.icons)for(const mode of ['standalone','button','text-button','pressable']) {
    const snippet=iconSnippet(icon.name,mode);
    assert(snippet.includes(`LucideIcons.${icon.name}()`));
    if(mode!=='standalone')assert(snippet.includes('callback action-requested();'));
  }
  assert.throws(()=>iconSnippet('search','unknown'));
  assert(iconSnippet('download','text-button').includes('leading-icon: LucideIcons.download()'));
});

test('every public component has a distinct explicit drawing with finite coordinates', () => {
  const components=scanComponents();
  assert.deepEqual(Object.keys(mockups).sort(),components.map(c=>c.slug).sort());
  const drawings=new Map();
  for(const component of components) {
    const operations=[];
    const context=new Proxy({}, {set(target,key,value){operations.push([key,value]);target[key]=value;return true;}, get(target,key){return target[key]??((...args)=>{assert(args.every(arg=>typeof arg!=='number'||Number.isFinite(arg)));operations.push([key,...args]);});}});
    drawMockup(context,component.slug,{canvas:'#fff',paper:'#eee',soft:'#ddd',ink:'#222',accent:'#76a'});
    const signature=JSON.stringify(operations);
    assert(!drawings.has(signature),`${component.slug} duplicates ${drawings.get(signature)}`);
    drawings.set(signature,component.slug);
  }
  assert.throws(()=>drawMockup({},'unknown',{}));
});

test('catalogue indicators are circular, stepper arrows mirror, and the card action fits its surface', () => {
  const recipe = slug => {
    const calls=[];
    mockups[slug](new Proxy({}, {get:(_,name)=>(...args)=>calls.push({name,args})}));
    return calls;
  };
  for (const slug of ['radial-progress','spinner']) {
    const arcs=recipe(slug).filter(call=>call.name==='arc');
    assert.equal(arcs.length,2,`${slug} needs a circular track and circular foreground`);
    const [track,foreground]=arcs.map(call=>call.args);
    assert.deepEqual(foreground.slice(0,3),track.slice(0,3),'Foreground must share the track center and radius');
    assert.equal(track[4]-track[3],Math.PI*2);
    if(slug==='radial-progress')assert(Math.abs((foreground[4]-foreground[3])/(Math.PI*2)-.65)<1e-9,'The arc must represent its 65% label');
  }
  const arrows=recipe('number-stepper').filter(call=>call.name==='path').map(call=>call.args[0]);
  assert.equal(arrows.length,2);
  const middle=(arrows[0][1][1]+arrows[1][1][1])/2;
  assert.deepEqual(arrows[1],arrows[0].map(([x,y])=>[x,2*middle-y]),'Use the same chevron geometry reflected vertically');
  const card=recipe('card'),surface=card.find(call=>call.name==='rect').args,button=card.find(call=>call.name==='button').args;
  assert(button[0]>surface[0]&&button[0]+button[2]<surface[0]+surface[2]);
  assert(button[1]>surface[1]&&button[1]+34<surface[1]+surface[3],'Keep the entire action and bottom spacing inside the card');
});
