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

test('quickstart pages distinguish development APIs from immutable registry examples', async () => {
  const { default: nunjucks } = await import('nunjucks');
  const template = fs.readFileSync(new URL('../src/_includes/component-quickstart.njk', import.meta.url), 'utf8');
  const env = new nunjucks.Environment(null, { autoescape: false, throwOnUndefined: true });
  env.addFilter('quickstartSource', (file, slug) => `source:${slug}/${file}`);
  for (const slug of ['text-field', 'password-field', 'search-field', 'data-table', 'content-list', 'virtual-text-view']) {
    for (const locale of ['en', 'fr']) {
      for (const registryExamples of [false, true]) {
        for (const version of ['0.7.2', '0.7.3']) {
          const rendered = env.renderString(template, { component: { slug }, locale, project: { registryExamples, version }, quickstart: { en: 'English usage.', fr: 'Usage français.' } });
          const development = !registryExamples || version === '0.7.2';
          if (development) {
            assert.doesNotMatch(rendered, /cargo run --locked|\/quickstart\/projects\//);
            assert.match(rendered, /(?:cannot compile|ne compilent pas)/);
            assert.match(rendered, /crates\/kit-build/);
            assert.ok(rendered.includes(`https://github.com/damione1/slint-kit/tree/v0.7.2/docs/quickstart/${slug}`));
            if (['text-field', 'password-field', 'search-field'].includes(slug)) assert.match(rendered, /(?:field composition and action|composition et d’actions de champ)/);
          } else {
            assert.match(rendered, /cargo run --locked/);
            assert.ok(rendered.includes(`/quickstart/projects/${slug}.zip`));
          }
        }
      }
    }
  }
  const existing = env.renderString(template, { component: { slug: 'button' }, locale: 'en', project: { registryExamples: false, version: '0.7.2' }, quickstart: { en: 'Usage.', fr: 'Usage.' } });
  assert.match(existing, /cargo run --locked/);
});
