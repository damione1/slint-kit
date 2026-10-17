// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import MarkdownIt from 'markdown-it';
import { scanComponents } from './scripts/inventory.mjs';
import { localePrefix, localeUrl, siteOrigin } from './scripts/routes.mjs';
export default function (eleventyConfig) {
  eleventyConfig.addPassthroughCopy('src/assets');
  eleventyConfig.addPassthroughCopy({ 'src/_redirects': '_redirects' });
  eleventyConfig.addFilter('localePrefix', localePrefix);
  eleventyConfig.addFilter('localeUrl', localeUrl);
  eleventyConfig.addFilter('sortByUrl', items => [...items].sort((a, b) => a.url.localeCompare(b.url)));
  eleventyConfig.addFilter('xmlEscape', value => String(value).replace(/[&<>"']/g, character => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&apos;' })[character]));
  eleventyConfig.addPassthroughCopy({ '../assets/branding': 'assets/branding' });
  eleventyConfig.addPassthroughCopy({ examples: 'examples' });
  eleventyConfig.addPassthroughCopy({ consumer: 'consumer' });
  eleventyConfig.addWatchTarget('../crates/kit-slint/ui');
  eleventyConfig.addWatchTarget('examples');
  eleventyConfig.addWatchTarget('content');
  let inventory = scanComponents();
  eleventyConfig.on('eleventy.before', () => { inventory = scanComponents(); });
  eleventyConfig.addGlobalData('components', () => inventory);
  eleventyConfig.addGlobalData('site', {
    name: 'Slint Kit', repository: 'https://github.com/damione1/slint-kit',
    url: siteOrigin(),
  });
  eleventyConfig.addGlobalData('locales', ['en', 'fr']);
  const markdown = new MarkdownIt({ html: false, linkify: false });
  eleventyConfig.addFilter('consumerSource', file => fs.readFileSync(path.join('consumer', file), 'utf8'));
  eleventyConfig.addWatchTarget('consumer');
  eleventyConfig.addFilter('prose', text => markdown.render(text || ''));
  eleventyConfig.addFilter('componentByName', name => inventory.find(c => c.name === name));
  eleventyConfig.addFilter('example', (slug, locale = 'en') => fs.readFileSync(path.join('examples', locale === 'fr' ? 'fr' : '', `${slug}.slint`), 'utf8'));
  eleventyConfig.addFilter('whereFamily', (components, family) => components.filter(c => c.family === family));
  eleventyConfig.addFilter('mdEscape', text => String(text).replaceAll('|', '\\|').replaceAll('\n', ' '));
  eleventyConfig.addFilter('localeComponent', (component, locale) => component[locale]);
  return { dir: { input: 'src', output: '_site' }, markdownTemplateEngine: 'njk', htmlTemplateEngine: 'njk' };
}
