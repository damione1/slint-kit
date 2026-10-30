// SPDX-License-Identifier: GPL-3.0-or-later
import test from 'node:test';
import assert from 'node:assert/strict';
import {families} from './inventory.mjs';
import {familyIllustration} from './family-illustrations.mjs';
test('every family has an original deterministic SVG with its own filter identity',()=>{
 const scenes=Object.keys(families).map(family=>{
  const svg=familyIllustration(family);
  assert.equal(svg,familyIllustration(family));
  assert(svg.includes('viewBox="0 0 360 200"'));
  assert(svg.includes('aria-hidden="true"'));
  assert(svg.includes(`id="shadow-${family}"`));
  assert(!svg.includes('<image'));assert(!svg.includes('<script'));assert(!svg.includes('<text'));
  return svg;
 });
 assert.equal(new Set(scenes).size,Object.keys(families).length);
 assert.throws(()=>familyIllustration('unregistered-family'));
});
