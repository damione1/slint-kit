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

function assertSourceDependency(dependency, context) {
  const manifest = fs.readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8');
  const repository = manifest.match(/^repository\s*=\s*"([^"]+)"/m)?.[1];
  assert.ok(repository, 'the workspace declares its repository');
  assert.equal(dependency.match(/git\s*=\s*"([^"]+)"/)?.[1], repository, `${context}: repository`);
  assert.equal(dependency.match(/branch\s*=\s*"([^"]+)"/)?.[1], 'develop', `${context}: branch`);
  assert.doesNotMatch(dependency, /\b(?:version|rev|tag|path)\s*=/);
}

test('published guide dependency snippets use the central version', () => {
  for (const locale of ['en', 'fr']) {
    for (const guide of ['project-setup', 'native', 'workspace']) {
      const source = fs.readFileSync(new URL(`../src/${locale}/guides/${guide}.md`, import.meta.url), 'utf8');
      const dependencies = source.split('\n').filter(line => /^slint-kit(?:-build)?\s*=/.test(line) && !line.includes('path ='));
      assert.ok(dependencies.length, `${locale}/${guide}: dependency snippet`);
      for (const dependency of dependencies) {
        if (/\bgit\s*=/.test(dependency)) assertSourceDependency(dependency, `${locale}/${guide}`);
        else assert.match(dependency, /\{\{ project\.version \}\}/);
      }
    }
  }
  assert.match(loadProject().version, /^\d+\.\d+\.\d+$/);
});


test('unpublished source guide dependencies share the workspace repository and develop branch', () => {
  for (const locale of ['en', 'fr']) {
    const source = fs.readFileSync(new URL(`../src/${locale}/guides/project-setup.md`, import.meta.url), 'utf8');
    const blocks = Array.from(source.matchAll(/```toml\n([\s\S]*?)```/g), match => match[1])
      .filter(block => /git\s*=/.test(block));
    assert.equal(blocks.length, 1, `${locale}: one source dependency manifest`);
    const dependencies = blocks[0].split('\n').filter(line => /^slint-kit(?:-build)?\s*=/.test(line));
    assert.deepEqual(dependencies.map(line => line.match(/^(\S+)\s*=/)[1]), ['slint-kit', 'slint-kit-build']);
    for (const dependency of dependencies) assertSourceDependency(dependency, locale);
  }
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
  const starters = JSON.parse(fs.readFileSync(new URL('../content/starters.json', import.meta.url), 'utf8'));
  for (const slug of ['button-group', 'steps', 'timeline', 'date-picker', 'date-time-field', 'file-picker', 'directory-picker', 'otp-field']) assert.equal(starters[slug].development, true);
  for (const slug of ['text-field', 'password-field', 'search-field', 'data-table', 'content-list', 'virtual-text-view', 'button-group', 'steps', 'timeline', 'date-picker', 'date-time-field', 'form-error-summary', 'file-picker', 'directory-picker', 'otp-field']) {
    for (const locale of ['en', 'fr']) {
      for (const registryExamples of [false, true]) {
        for (const version of ['0.7.2', '0.7.3']) {
          const rendered = env.renderString(template, { component: { slug }, locale, project: { registryExamples, version }, quickstart: { ...starters[slug], en: 'English usage.', fr: 'Usage français.' } });
          const development = !registryExamples || version === '0.7.2';
          if (development) {
            assert.doesNotMatch(rendered, /cargo run --locked|\/quickstart\/projects\//);
            assert.match(rendered, /(?:cannot compile|ne compilent pas)/);
            assert.match(rendered, /crates\/kit-build/);
            assert.ok(rendered.includes(`https://github.com/damione1/slint-kit/tree/v0.7.2/docs/quickstart/${slug}`));
            if (['text-field', 'password-field', 'search-field'].includes(slug)) assert.match(rendered, /(?:field composition and action|composition et d’actions de champ)/);
            if (slug === 'form-error-summary') assert.match(rendered, /(?:field direction|direction des champs)/);
          } else {
            assert.match(rendered, /cargo run --locked/);
            assert.ok(rendered.includes(`/quickstart/projects/${slug}.zip`));
          }
        }
      }
    }
  }
  const flagged = env.renderString(template, { component: { slug: 'arbitrary-new-api' }, locale: 'en', project: { registryExamples: false, version: '0.7.2' }, quickstart: { development: true, en: 'Usage.', fr: 'Usage.' } });
  assert.match(flagged, /cannot compile/);
  assert.doesNotMatch(flagged, /cargo run --locked|\/quickstart\/projects\//);
  const existing = env.renderString(template, { component: { slug: 'button' }, locale: 'en', project: { registryExamples: false, version: '0.7.2' }, quickstart: { en: 'Usage.', fr: 'Usage.' } });
  assert.match(existing, /cargo run --locked/);
});

test('verification-code examples retain typed accepted codes without exposing them in status or logs', () => {
  const ui = fs.readFileSync(new URL('../quickstart/otp-field/ui/app.slint', import.meta.url), 'utf8');
  assert.doesNotMatch(ui, /accepted-text|value\.value\.text/);
  for (const name of ['numeric', 'alpha', 'grouped']) assert.ok(ui.includes(`root.${name}-accepted ?`));
  const rust = fs.readFileSync(new URL('../quickstart/otp-field/src/main.rs', import.meta.url), 'utf8');
  for (const name of ['numeric', 'alpha', 'grouped']) {
    assert.ok(rust.includes(`${name}_code = RefCell::new(None::<OtpValue>)`));
    assert.ok(rust.includes(`${name}.accept(commit)`));
    assert.ok(rust.includes(`*${name}_code.borrow_mut() = Some(value)`));
    assert.ok(rust.includes(`set_${name}_accepted(${name}_code.borrow().is_some())`));
  }
  assert.doesNotMatch(rust, /(?:println!|eprintln!|dbg!)\s*\(/);
});

test('registration form declares its typed schema and business code within fifteen Rust lines', async () => {
  const { default: nunjucks } = await import('nunjucks');
  const starters = JSON.parse(fs.readFileSync(new URL('../content/starters.json', import.meta.url), 'utf8'));
  const template = fs.readFileSync(new URL('../src/_includes/component-quickstart.njk', import.meta.url), 'utf8');
  const env = new nunjucks.Environment(null, { autoescape: false, throwOnUndefined: true });
  env.addFilter('quickstartSource', (file, slug) => `source:${slug}/${file}`);
  for (const locale of ['en', 'fr']) {
    const rendered = env.renderString(template, {component: {slug: 'form'}, locale, project: {registryExamples: false, version: '0.7.2'}, quickstart: starters.form});
    assert.doesNotMatch(rendered, /tree\/v0\.7\.2\/docs\/quickstart\/form/);
  }
  const main = fs.readFileSync(new URL('../quickstart/form/src/main.rs', import.meta.url), 'utf8');
  assert.ok(main.trim().split('\n').length <= 15);
  assert.deepEqual(fs.readdirSync(new URL('../quickstart/form/src/', import.meta.url), {recursive: true}).filter(name => name.endsWith('.rs')), ['main.rs']);
  assert.match(main, /form_schema!\(Registration, App, RegistrationField/);
  assert.equal((main.match(/: required\(/g) || []).length, 3);
  assert.match(main, /consent: checked/);
  assert.doesNotMatch(main, /FormFieldPorts|on_field_edited|register_view|\.accept\(|pub fn parse/);
});

test('released starters offer registry projects while unpublished examples retain source guidance', async () => {
  const { default: nunjucks } = await import('nunjucks');
  const env = new nunjucks.Environment();
  env.addFilter('quickstartSource', (file, slug) => `source:${slug}/${file}`);
  const template = fs.readFileSync(new URL('../src/_includes/component-quickstart.njk', import.meta.url), 'utf8');
  const starters = JSON.parse(fs.readFileSync(new URL('../content/starters.json', import.meta.url), 'utf8'));
  assert.equal(starters['avatar-group'].registryExample, true);
  for (const locale of ['en', 'fr']) {
    const project = { registryExamples: true, version: loadProject().version };
    const released = env.renderString(template, { component: { slug: 'avatar-group' }, locale, project, quickstart: starters['avatar-group'] });
    assert(released.includes('cargo run --locked'));
    assert(released.includes('/quickstart/projects/avatar-group.zip'));
    assert(!released.includes('tree/v0.7.2/'));
    const pending = env.renderString(template, { component: { slug: 'unpublished-component' }, locale, project, quickstart: { ...starters['avatar-group'], registryExample: false } });
    assert(!pending.includes('cargo run --locked'));
    assert(pending.includes('/quickstart/projects/unpublished-component.zip'));
    assert(pending.includes(locale === 'fr' ? 'Aucun exemple de ce nouveau composant' : 'This new component has no example'));
    assert(!pending.includes('tree/v0.7.2/'));
  }
});
