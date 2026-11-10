// SPDX-License-Identifier: GPL-3.0-or-later
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {heroProjects, orderedProjects, nextDirection, SortColumn, SortDirection} from '../src/assets/landing.js';

test('the hero uses the real Rust table example records, including the disabled project', () => {
  const source = fs.readFileSync(new URL('../table-consumer/src/main.rs', import.meta.url), 'utf8');
  const records = [...source.matchAll(/\(ProjectId\((\d+)\),\s*project\("([^"]+)",\s*(\d+),\s*(true|false)\)\)/g)]
    .map(([,key,name,tasks,archived]) => ({key:Number(key),name,tasks:Number(tasks),archived:archived==='true'}));
  assert.equal(records.length, 4);
  assert.deepEqual(heroProjects, records);
});

test('numeric sorting moves stable records through the native example ascending/descending cycle', () => {
  let direction = SortDirection.Unsorted;
  const original = [...heroProjects];
  for (const expected of [[203,409,101,307], [307,101,409,203], [203,409,101,307]]) {
    direction = nextDirection(direction);
    const rows = orderedProjects(SortColumn.Tasks, direction);
    assert.deepEqual(rows.map(row=>row.key), expected);
    assert.equal(rows.find(row=>row.key===101), original[0], 'sorting retains record identity');
  }
  assert.deepEqual(heroProjects, original, 'sorting does not mutate the source order');
  assert.throws(()=>nextDirection('random'));
  assert.throws(()=>orderedProjects('random', SortDirection.Unsorted));
  assert.throws(()=>orderedProjects(SortColumn.Tasks,'random'));
});
