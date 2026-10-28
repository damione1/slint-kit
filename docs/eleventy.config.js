// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import MarkdownIt from 'markdown-it';
import { scanComponents, componentLabel } from './scripts/inventory.mjs';
import { localePrefix, localeUrl, siteOrigin } from './scripts/routes.mjs';
import { structuredData, scriptJson } from './scripts/metadata.mjs';
export default function (eleventyConfig) {
  eleventyConfig.addPassthroughCopy('src/assets');
  eleventyConfig.addPassthroughCopy({ 'src/_redirects': '_redirects' });
  eleventyConfig.addPassthroughCopy({ 'src/_headers': '_headers' });
  eleventyConfig.addPassthroughCopy({ quickstart: 'quickstart' });
  eleventyConfig.addFilter('componentLabel', componentLabel);
  eleventyConfig.addFilter('quickstartSource', (file, slug) => fs.readFileSync(path.join('quickstart', slug, file), 'utf8'));
  eleventyConfig.addGlobalData('quickstarts', {
    button: { component: 'KitButton', width: 840, height: 440, displayWidth: 420 },
    combobox: { component: 'KitCombobox', width: 960, height: 960, displayWidth: 480 },
  });
  const versionedAssets = new Map();
  eleventyConfig.on('eleventy.before', () => versionedAssets.clear());
  eleventyConfig.addFilter('versionedAsset', sourceUrl => {
    if (!versionedAssets.has(sourceUrl)) {
      const content = fs.readFileSync(path.join('src', sourceUrl.slice(1)));
      const hash = createHash('sha256').update(content).digest('hex').slice(0, 16);
      const file = path.parse(sourceUrl);
      versionedAssets.set(sourceUrl, {
        url: `/assets/versioned/${file.name}.${hash}${file.ext}`, content,
      });
    }
    return versionedAssets.get(sourceUrl).url;
  });
  eleventyConfig.on('eleventy.after', () => {
    for (const { url, content } of versionedAssets.values()) {
      const target = path.join('_site', url.slice(1));
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.writeFileSync(target, content);
    }
  });
  eleventyConfig.addFilter('localePrefix', localePrefix);
  eleventyConfig.addFilter('localeUrl', localeUrl);
  eleventyConfig.addFilter('structuredData', structuredData);
  eleventyConfig.addFilter('scriptJson', scriptJson);
  eleventyConfig.addFilter('sortByUrl', items => [...items].sort((a, b) => a.url.localeCompare(b.url)));
  eleventyConfig.addFilter('xmlEscape', value => String(value).replace(/[&<>"']/g, character => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' })[character]));
  eleventyConfig.addPassthroughCopy({ '../assets/branding': 'assets/branding' });
  eleventyConfig.addPassthroughCopy({ examples: 'examples' });
  eleventyConfig.addPassthroughCopy({ consumer: 'consumer' });
  for (const file of ['Cargo.toml', 'build.rs', 'ui/table.slint', 'src/main.rs']) {
    eleventyConfig.addPassthroughCopy({ [`table-consumer/${file}`]: `table-consumer/${file}` });
  }
  eleventyConfig.addWatchTarget('../crates/kit-slint/ui');
  eleventyConfig.addWatchTarget('examples');
  eleventyConfig.addWatchTarget('content');
  let inventory = scanComponents();
  eleventyConfig.on('eleventy.before', () => { inventory = scanComponents(); });
  eleventyConfig.addGlobalData('components', () => inventory);
  eleventyConfig.addGlobalData('site', {
    name: 'Slint Kit', repository: 'https://github.com/damione1/slint-kit',
    sourceRef: 'v0.2.0',
    url: siteOrigin(),
  });
  const manifest = fs.readFileSync('../Cargo.toml', 'utf8');
  eleventyConfig.addGlobalData('project', {
    version: manifest.match(/^version = "([^"]+)"/m)[1],
    rust: manifest.match(/^rust-version = "([^"]+)"/m)[1],
    slint: manifest.match(/^slint = \{ version = "=([^"]+)"/m)[1],
    preOne: manifest.match(/^version = "([^"]+)"/m)[1].startsWith('0.'),
  });
  eleventyConfig.addGlobalData('locales', ['en', 'fr']);
  const markdown = new MarkdownIt({ html: false, linkify: false });
  eleventyConfig.addFilter('consumerSource', file => fs.readFileSync(path.join('consumer', file), 'utf8'));
  eleventyConfig.addFilter('tableSource', file => fs.readFileSync(path.join('table-consumer', file), 'utf8'));
  eleventyConfig.addWatchTarget('consumer');
  eleventyConfig.addWatchTarget('table-consumer');
  eleventyConfig.addGlobalData('typedCaptures', () => Object.fromEntries(['en', 'fr'].map(locale => {
    const filename = `typed-integration${locale === 'fr' ? '-fr' : ''}.png`;
    const image = fs.readFileSync(path.join('src/assets/components', filename));
    return [locale, { filename, retinaFilename: filename.replace('.png', '@2x.png'), width: image.readUInt32BE(16), height: image.readUInt32BE(20) }];
  })));
  eleventyConfig.addGlobalData('tableCaptures', () => Object.fromEntries(['en', 'fr'].map(locale => {
    const filename = `table-integration${locale === 'fr' ? '-fr' : ''}.png`;
    const image = fs.readFileSync(path.join('src/assets/components', filename));
    return [locale, { filename, retinaFilename: filename.replace('.png', '@2x.png'), width: image.readUInt32BE(16), height: image.readUInt32BE(20) }];
  })));
  eleventyConfig.addFilter('prose', text => markdown.render(text || ''));
  eleventyConfig.addFilter('componentByName', name => inventory.find(c => c.name === name));
  eleventyConfig.addFilter('example', (slug, locale = 'en') => fs.readFileSync(path.join('examples', locale === 'fr' ? 'fr' : '', `${slug}.slint`), 'utf8'));
  eleventyConfig.addFilter('whereFamily', (components, family) => components.filter(c => c.family === family));
  eleventyConfig.addFilter('mdEscape', text => String(text).replaceAll('|', '\\|').replaceAll('\n', ' '));
  eleventyConfig.addFilter('localeComponent', (component, locale) => component[locale]);
  return { dir: { input: 'src', output: '_site' }, markdownTemplateEngine: 'njk', htmlTemplateEngine: 'njk' };
}
