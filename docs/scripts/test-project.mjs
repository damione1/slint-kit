// SPDX-License-Identifier: GPL-3.0-or-later
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { projectMetadata, loadProject } from './project.mjs';

test('documentation versions come from the workspace, not another package table', () => {
  const source = `[package]\nversion = "9.8.7"\n[workspace.package]\nversion = "0.7.0"\nrust-version = "1.92"\n[workspace.dependencies]\nslint = { version = "=1.18.1", default-features = false }\n`;
  assert.deepEqual(projectMetadata(source), {version: '0.7.0', rust: '1.92', slint: '1.18.1', preOne: true});
  assert.equal(projectMetadata(source.replace('0.7.0', '1.2.3')).preOne, false);
  assert.equal(projectMetadata(source.replace('=1.18.1', '~1.18.1')).slint, '1.18.1');
  assert.throws(() => projectMetadata(source.replace('=1.18.1', '1.18.1')), /pinned Slint version/);
  assert.throws(() => projectMetadata(source.replace('=1.18.1', '^1.18.1')), /pinned Slint version/);
});

test('published guide dependency snippets use the central version', () => {
  for (const locale of ['en', 'fr']) {
    for (const guide of ['project-setup', 'native', 'workspace']) {
      const source = fs.readFileSync(new URL(`../src/${locale}/guides/${guide}.md`, import.meta.url), 'utf8');
      const dependencies = source.split('\n').filter(line => /^slint-kit(?:-build)?\s*=/.test(line) && !line.includes('path ='));
      assert.ok(dependencies.length, `${locale}/${guide}: dependency snippet`);
      for (const dependency of dependencies) assert.match(dependency, /\{\{ project\.version \}\}/);
    }
  }
  assert.match(loadProject().version, /^\d+\.\d+\.\d+$/);
});


test('password examples keep accepted secrets out of visible status and log sinks', () => {
  for (const locale of ['', 'fr/']) {
    const source = fs.readFileSync(new URL(`../examples/${locale}password-field.slint`, import.meta.url), 'utf8');
    assert.doesNotMatch(source, /root\.status\s*=\s*[^;]*(?:value\.value\.text|accepted-text)/);
    assert.match(source, /root\.status\s*=\s*"(?:Password accepted|Mot de passe accepté)"/);
  }
  const ui = fs.readFileSync(new URL('../quickstart/password-field/ui/app.slint', import.meta.url), 'utf8');
  assert.doesNotMatch(ui, /accepted-text|value\.value\.text/);
  assert.match(ui, /text:\s*root\.accepted\s*\?/);
  const rust = fs.readFileSync(new URL('../quickstart/password-field/src/main.rs', import.meta.url), 'utf8');
  assert.match(rust, /spec\.accept\(commit\)/);
  assert.match(rust, /Some\(value\)/);
  assert.doesNotMatch(rust, /(?:println!|eprintln!|dbg!)\s*\([^;]*(?:format\s*\(|&value|\{(?:value|password)[^}]*\})/);
});
