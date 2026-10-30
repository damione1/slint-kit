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
  for(const icon of catalogue.icons)for(const mode of ['standalone','button','pressable']) {
    const snippet=iconSnippet(icon.name,mode);
    assert(snippet.includes(`LucideIcons.${icon.name}()`));
    if(mode!=='standalone')assert(snippet.includes('callback action-requested();'));
  }
  assert.throws(()=>iconSnippet('search','unknown'));
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
