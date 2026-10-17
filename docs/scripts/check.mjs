// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';
import { scanComponents } from './inventory.mjs';
const docs = fileURLToPath(new URL('../', import.meta.url));
const output = path.join(docs, '_site');
const errors = [];
const check = (condition, message) => { if (!condition) errors.push(message); };
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const decode = text => text.replaceAll('&quot;', '"').replaceAll('&#39;', "'").replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&amp;', '&');
const capturesPath = path.join(docs, 'content/captures.json');
const provenance = fs.existsSync(capturesPath) ? JSON.parse(fs.readFileSync(capturesPath, 'utf8')) : { components: {} };
check(provenance.environment?.backend === 'winit' && provenance.environment?.renderer === 'skia', 'Capture manifest must identify its native backend and renderer');
const kitUi = path.join(docs, '../crates/kit-slint/ui');
const kitHash = crypto.createHash('sha256');
for (const file of fs.readdirSync(kitUi).filter(file => file.endsWith('.slint')).sort()) {
  kitHash.update(file + '\0');
  kitHash.update(fs.readFileSync(path.join(kitUi, file)));
  kitHash.update('\0');
}
check(provenance.environment?.kitUiSha256 === kitHash.digest('hex'), 'Kit UI sources changed: regenerate native screenshots and provenance');
const components = scanComponents();
const slugs = new Set(components.map(c => c.slug));
check(slugs.size === components.length, 'Component route slugs must be unique');
const metadata = JSON.parse(fs.readFileSync(path.join(docs, 'content/components.json'), 'utf8'));
check(Object.keys(metadata).length === components.length, 'Component prose inventory differs from the public exports');
function pngDetails(buffer) {
  if (buffer.subarray(0, 8).toString('hex') !== '89504e470d0a1a0a') throw Error('Invalid PNG signature');
  const width = buffer.readUInt32BE(16), height = buffer.readUInt32BE(20);
  const depth = buffer[24], color = buffer[25], interlace = buffer[28];
  if (depth !== 8 || ![2, 6].includes(color) || interlace !== 0) throw Error('Expected noninterlaced RGB(A)8 native PNG');
  const bpp = color === 6 ? 4 : 3, chunks = [];
  for (let cursor = 8; cursor < buffer.length;) {
    const length = buffer.readUInt32BE(cursor), type = buffer.toString('ascii', cursor + 4, cursor + 8);
    if (type === 'IDAT') chunks.push(buffer.subarray(cursor + 8, cursor + 8 + length));
    cursor += length + 12;
  }
  const raw = zlib.inflateSync(Buffer.concat(chunks));
  const stride = width * bpp, pixels = Buffer.alloc(stride * height);
  const paeth = (a, b, c) => { const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c); return pa <= pb && pa <= pc ? a : pb <= pc ? b : c; };
  let index = 0;
  for (let y = 0; y < height; y++) {
    const filter = raw[index++];
    if (filter > 4) throw Error('Invalid PNG row filter');
    for (let x = 0; x < stride; x++) {
      const at = y * stride + x, left = x >= bpp ? pixels[at - bpp] : 0, up = y ? pixels[at - stride] : 0, upperLeft = y && x >= bpp ? pixels[at - stride - bpp] : 0;
      const predictor = [0, left, up, Math.floor((left + up) / 2), paeth(left, up, upperLeft)][filter];
      pixels[at] = (raw[index++] + predictor) & 255;
    }
  }
  const colors = new Set();
  for (let i = 0; i < pixels.length; i += bpp) colors.add(`${pixels[i]},${pixels[i + 1]},${pixels[i + 2]}`);
  return { width, height, colors: colors.size };
}
for (const component of components) {
  const { name, slug } = component;
  for (const locale of ['fr', 'en']) {
    const examplePath = path.join(docs, 'examples', locale === 'fr' ? 'fr' : '', `${slug}.slint`);
    const imageFilename = `${slug}${locale === 'fr' ? '-fr' : ''}.png`;
    const imagePath = path.join(docs, 'src/assets/components', imageFilename);
    check(fs.existsSync(examplePath), `${name}/${locale}: missing standalone example`);
    check(fs.existsSync(imagePath), `${name}/${locale}: missing native capture`);
    if (!fs.existsSync(examplePath) || !fs.existsSync(imagePath)) continue;
    const example = fs.readFileSync(examplePath, 'utf8'), image = fs.readFileSync(imagePath);
    check(/from\s+"@kit\//.test(example), `${name}/${locale}: example must consume public @kit imports`);
    check(/export component Example inherits Window/.test(example), `${name}/${locale}: example must provide a runnable Window`);
    check(new RegExp(`\\b${name}\\s*\\{`).test(example), `${name}/${locale}: example must instantiate its documented component`);
    const record = locale === 'fr' ? provenance.components[slug]?.locales?.fr : provenance.components[slug];
    check(record, `${name}/${locale}: missing capture provenance`);
    if (record) {
      check(record.exampleSha256 === sha(example), `${name}/${locale}: regenerate screenshot after example changes`);
      check(record.screenshotSha256 === sha(image), `${name}/${locale}: screenshot differs from the capture manifest`);
      for (const variant of record.variants || []) {
        const variantPath = path.join(docs, 'src/assets/components', variant.file);
        check(fs.existsSync(variantPath), `${name}/${locale}: missing native variant ${variant.file}`);
        if (fs.existsSync(variantPath)) {
          const pixels = fs.readFileSync(variantPath);
          check(sha(pixels) === variant.sha256, `${name}/${locale}: native variant changed ${variant.file}`);
          check(pixels.length >= 24 && pixels.readUInt32BE(16) === variant.width && pixels.readUInt32BE(20) === variant.height, `${name}/${locale}: invalid native variant dimensions ${variant.file}`);
        }
      }
    }
    try {
      const info = pngDetails(image);
      check(info.width >= 240 && info.height >= 120 && info.colors > 40, `${name}/${locale}: blank or undersized image`);
      if (record) check(info.width === record.width && info.height === record.height, `${name}/${locale}: incorrect capture dimensions`);
    } catch (error) { errors.push(`${name}/${locale}: ${error.message}`); }
    const text = component[locale];
    for (const key of ['summary', 'usage', 'behavior']) check(typeof text?.[key] === 'string' && text[key].trim().length >= (key === 'summary' ? 20 : 35), `${name}/${locale}: missing meaningful ${key}`);
    const page = path.join(output, locale, 'components', slug, 'index.html');
    check(fs.existsSync(page), `${name}/${locale}: missing rendered page`);
    if (!fs.existsSync(page)) continue;
    const html = fs.readFileSync(page, 'utf8');
    check(html.includes(`<html lang="${locale}">`), `${name}/${locale}: incorrect HTML language`);
    const alternate = locale === 'fr' ? 'en' : 'fr';
    check(html.includes(`href="/${alternate}/components/${slug}/"`), `${name}/${locale}: language link must retain the current component`);
    check(html.includes(`<h1>${name}</h1>`), `${name}/${locale}: missing component heading`);
    const code = html.match(/<pre><code class="language-slint">([\s\S]*?)<\/code><\/pre>/)?.[1];
    check(code && decode(code).trim() === example.trim(), `${name}/${locale}: displayed code differs from the native capture source`);
    check(html.includes(`/assets/components/${imageFilename}`), `${name}/${locale}: missing matching capture`);
    check(text?.summary && html.includes(decode(text.summary)), `${name}/${locale}: missing localized summary`);
  }
}
// Check every emitted HTML link/image, including fragment destinations. This
// catches navigation breakage outside the component catalogue and both locales.
function walk(directory) { return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]); }
const htmlFiles = fs.existsSync(output) ? walk(output).filter(file => file.endsWith('.html')) : [];
for (const file of htmlFiles) {
  const html = fs.readFileSync(file, 'utf8');
  check(!/\{[{%]|undefined|\[object Object\]/.test(html), `${path.relative(output, file)}: unresolved template content`);
  for (const match of html.matchAll(/\b(?:href|src|srcset)="([^"]+)"/g)) {
    const href = decode(match[1]);
    if (/^(?:https?:|mailto:|data:)/.test(href)) continue;
    const [route, fragment] = href.split('#');
    let target = route ? (route.startsWith('/') ? path.join(output, decodeURIComponent(route)) : path.resolve(path.dirname(file), decodeURIComponent(route))) : file;
    if (fs.existsSync(target) && fs.statSync(target).isDirectory()) target = path.join(target, 'index.html');
    check(fs.existsSync(target), `${path.relative(output, file)}: broken local URL ${href}`);
    if (fragment && fs.existsSync(target) && target.endsWith('.html')) {
      const targetHtml = fs.readFileSync(target, 'utf8');
      check(targetHtml.includes(`id="${decodeURIComponent(fragment)}"`), `${path.relative(output, file)}: missing fragment ${href}`);
    }
  }
}
check(htmlFiles.length >= components.length * 2 + 12, 'Incomplete site output');
if (errors.length) { console.error(errors.join('\n')); process.exitCode = 1; }
else console.log(`Documentation checks passed: ${components.length} public components, ${components.length * 2} localized references, ${components.length * 2} localized native screenshots, verbatim examples, ${htmlFiles.length} HTML pages, local links and image provenance.`);
