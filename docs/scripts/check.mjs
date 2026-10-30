// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import zlib from 'node:zlib';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { scanComponents } from './inventory.mjs';
import { localeUrl, siteOrigin } from './routes.mjs';
import { scriptJson } from './metadata.mjs';
import { loadStarters } from './starters.mjs';
const docs = fileURLToPath(new URL('../', import.meta.url));
const output = path.join(docs, '_site');
const errors = [];
const check = (condition, message) => { if (!condition) errors.push(message); };
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const decode = text => text.replaceAll('&quot;', '"').replaceAll('&#39;', "'").replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&amp;', '&');
const capturesPath = path.join(docs, 'content/captures.json');
const provenance = fs.existsSync(capturesPath) ? JSON.parse(fs.readFileSync(capturesPath, 'utf8')) : { components: {} };
check(provenance.environment?.backend === 'winit' && provenance.environment?.renderer === 'skia', 'Capture manifest must identify its native backend and renderer');
for (const arguments_ of [['scripts/capture_pipeline.py','check'], ['scripts/build-web-previews.py','--check'], ['scripts/build-workspace-preview.py','--check']]) {
  const result = spawnSync('python3', arguments_, {cwd:docs,stdio:'inherit'});
  check(result.status === 0, `Individual documentation artifacts are stale: ${arguments_[0]}`);
}
const renderCaptures = JSON.parse(fs.readFileSync(path.join(docs, 'content/render-captures.json'),'utf8'));
check(renderCaptures.schema === 2, 'Individual capture provenance is required');
const components = scanComponents();
const starters = loadStarters();
const starterCaptures = JSON.parse(fs.readFileSync(path.join(docs, 'content/starter-captures.json'), 'utf8'));
check(Object.keys(starters).length === components.length, 'Every public component needs a complete runnable starter');
const slugs = new Set(components.map(c => c.slug));
check(slugs.size === components.length, 'Component route slugs must be unique');
const metadata = JSON.parse(fs.readFileSync(path.join(docs, 'content/components.json'), 'utf8'));
check(Object.keys(metadata).length === components.length, 'Component prose inventory differs from the public exports');
function pngDetails(buffer) {
  if (buffer.subarray(0, 8).toString('hex') !== '89504e470d0a1a0a') throw Error('Invalid PNG signature');
  const width = buffer.readUInt32BE(16), height = buffer.readUInt32BE(20);
  const depth = buffer[24], color = buffer[25], interlace = buffer[28];
  if (depth !== 8 || ![2, 6].includes(color) || interlace !== 0) throw Error('Expected noninterlaced RGB(A)8 PNG');
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
    check(fs.existsSync(imagePath), `${name}/${locale}: missing capture`);
    if (!fs.existsSync(examplePath) || !fs.existsSync(imagePath)) continue;
    const example = fs.readFileSync(examplePath, 'utf8'), image = fs.readFileSync(imagePath);
    check(/from\s+"@kit\//.test(example), `${name}/${locale}: example must consume public @kit imports`);
    check(/export component Example inherits Window/.test(example), `${name}/${locale}: example must provide a runnable Window`);
    check(new RegExp(`\\b${name}\\s*\\{`).test(example), `${name}/${locale}: example must instantiate its documented component`);
    const record = locale === 'fr' ? provenance.components[slug]?.locales?.fr : provenance.components[slug];
    check(record, `${name}/${locale}: missing capture provenance`);
    if (record) {
      // Semantic identities above allow non-rendering source edits without recapturing.
      check(record.screenshotSha256 === sha(image), `${name}/${locale}: screenshot differs from the capture manifest`);
      for (const variant of record.variants || []) {
        const variantPath = path.join(docs, 'src/assets/components', variant.file);
        check(fs.existsSync(variantPath), `${name}/${locale}: missing capture variant ${variant.file}`);
        if (fs.existsSync(variantPath)) {
          const pixels = fs.readFileSync(variantPath);
          check(sha(pixels) === variant.sha256, `${name}/${locale}: capture variant changed ${variant.file}`);
          check(pixels.length >= 24 && pixels.readUInt32BE(16) === variant.width && pixels.readUInt32BE(20) === variant.height, `${name}/${locale}: invalid capture variant dimensions ${variant.file}`);
        }
      }
    }
    try {
      const info = pngDetails(image);
      check(info.width >= 240 && info.height >= 120 && info.colors > 40, `${name}/${locale}: blank or undersized image`);
      if (record) check(info.width === record.width && info.height === record.height, `${name}/${locale}: incorrect capture dimensions`);
    } catch (error) { errors.push(`${name}/${locale}: ${error.message}`); }
    const retinaPath = imagePath.replace(/\.png$/, '@2x.png');
    check(fs.existsSync(retinaPath), `${name}/${locale}: missing Retina capture`);
    if (fs.existsSync(retinaPath)) {
      const retina = fs.readFileSync(retinaPath);
      check(retina.readUInt32BE(16) === image.readUInt32BE(16) * 2 && retina.readUInt32BE(20) === image.readUInt32BE(20) * 2, `${name}/${locale}: 2x capture must retain the logical viewport`);
      check(record?.variants?.some(variant => variant.file === path.basename(retinaPath) && variant.pixelDensity === 2 && variant.sha256 === sha(retina)), `${name}/${locale}: missing Retina provenance`);
    }
    const text = component[locale];
    for (const key of ['summary', 'usage', 'behavior']) check(typeof text?.[key] === 'string' && text[key].trim().length >= (key === 'summary' ? 20 : 35), `${name}/${locale}: missing meaningful ${key}`);
    const page = path.join(output, localeUrl(`components/${slug}/`, locale), 'index.html');
    check(fs.existsSync(page), `${name}/${locale}: missing rendered page`);
    if (!fs.existsSync(page)) continue;
    const html = fs.readFileSync(page, 'utf8');
    check(html.includes(`<html lang="${locale}">`), `${name}/${locale}: incorrect HTML language`);
    const alternate = locale === 'fr' ? 'en' : 'fr';
    check(html.includes(`href="${localeUrl(`components/${slug}/`, alternate)}"`), `${name}/${locale}: language link must retain the current component`);
    check(/<h1>[^<]*Slint<\/h1>/.test(html) && html.includes(`<code>${name}</code>`), `${name}/${locale}: heading must describe the Slint component and retain its API name`);
    const code = [...html.matchAll(/<pre><code class="language-slint">([\s\S]*?)<\/code><\/pre>/g)].map(match => decode(match[1]).trim());
    check(code.includes(example.trim()), `${name}/${locale}: displayed code differs from the capture source`);
    if (starters[slug]) {
      const starter = path.join(docs, '.generated/consumers', slug);
      const capture = starterCaptures[slug];
      check(starters[slug].component === name, `${name}: starter must match its component`);
      check(['winit','browser'].includes(capture.environment?.backend) && ['skia','femtovg'].includes(capture.environment?.renderer), `${name}: starter must identify its actual capture backend`);
      // Package locks and README changes do not invalidate a rendered scene.
      // The individual render key above covers UI, host and resolved runtime.
      check(sha(fs.readFileSync(path.join(docs, 'quickstart', slug, capture.image.file))) === capture.image.sha256, `${name}: starter screenshot differs from its capture proof`);
      const blocks = [...html.matchAll(/<pre><code class="language-(?:toml|rust|slint)">([\s\S]*?)<\/code><\/pre>/g)].map(match => decode(match[1]).trim());
      for (const file of ['Cargo.toml', 'build.rs', 'ui/app.slint', 'src/main.rs']) {
        const source = fs.readFileSync(path.join(docs, 'quickstart', slug, file), 'utf8');
        check(blocks.includes(source.trim()), `${name}/${locale}: working example must display the complete ${file}`);
        check(fs.readFileSync(path.join(output, 'quickstart', slug, file), 'utf8') === source, `${name}/${locale}: downloaded ${file} must match displayed code`);
      }
    }
    check(html.includes(`srcset="/assets/components/${imageFilename.replace('.png', '@2x.png')} 2x"`), `${name}/${locale}: Retina candidate is required`);
    check(html.includes(`/assets/components/${imageFilename}`), `${name}/${locale}: missing matching capture`);
    check(text?.summary && html.includes(decode(text.summary)), `${name}/${locale}: missing localized summary`);
  }
}
// Check every emitted HTML link/image, including fragment destinations. This
// catches navigation breakage outside the component catalogue and both locales.
function walk(directory) { return fs.readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? walk(path.join(directory, entry.name)) : [path.join(directory, entry.name)]); }
const demoPages = ['preview/index.html', 'examples/workspace/index.html'];
const htmlFiles = fs.existsSync(output) ? walk(output).filter(file => file.endsWith('.html') && !demoPages.some(name => file === path.join(output, name))) : [];
const browserPreview = fs.readFileSync(path.join(output, 'preview/index.html'), 'utf8');
check(browserPreview.includes('<meta name="robots" content="noindex">'), 'Interactive preview shell must stay out of search results');
const workspacePage = path.join(output, 'examples/workspace/index.html');
check(fs.existsSync(workspacePage), 'Missing workspace demonstration page');
if (fs.existsSync(workspacePage)) {
  check(fs.readFileSync(workspacePage, 'utf8').includes('<meta name="robots" content="noindex">'), 'Workspace demonstration must stay out of search results');
}
const workspaceManifest = path.join(output, 'assets/workspace-preview/manifest.json');
check(fs.existsSync(workspaceManifest), 'Missing workspace module manifest');
if (fs.existsSync(workspaceManifest)) {
  const entry = JSON.parse(fs.readFileSync(workspaceManifest, 'utf8'));
  const validBase = /^\/assets\/workspace-preview\/[a-f0-9]{20}$/.test(entry.base || '');
  check(entry.schema === 1 && validBase, 'Workspace assets need a content-addressed base URL');
  check(entry.module === `${entry.base}/workspace.js`, 'Workspace module URL must match its verified base');
  for (const [kind, required] of [['assets',['workspace.js','workspace_bg.wasm']], ['downloads',['source.tar.gz','LICENSE','NOTICE.txt','THIRD_PARTY_NOTICES.txt']]]) {
    check(required.every(name => entry[kind]?.[name]), `Missing workspace ${kind}`);
    if (!validBase) continue;
    for (const [name, record] of Object.entries(entry[kind] || {})) {
      check(!name.includes('/') && name !== '..', `Invalid workspace asset filename: ${name}`);
      if (name.includes('/') || name === '..') continue;
      const file = path.join(output, entry.base, name);
      const content = fs.existsSync(file) ? fs.readFileSync(file) : null;
      check(content && content.length === record.bytes && sha(content) === record.sha256, `Workspace ${kind} integrity mismatch: ${name}`);
    }
  }
}
const webPreviews = JSON.parse(fs.readFileSync(path.join(docs, '.generated/web/manifest.json'), 'utf8'));
check(webPreviews.schema === 2 && webPreviews.defaultTheme === 'tailwind', 'Browser previews need individual theme variants with Tailwind as default');
check(webPreviews.components.length === components.length && components.every(c => webPreviews.components.includes(c.slug)), 'Browser preview must cover every public component');
for (const component of components) {
  for (const theme of ['kit', 'tailwind']) {
    const entry = webPreviews.entries[component.slug]?.[theme];
    check(entry && new RegExp(`/assets/previews/${component.slug}/${theme}/[a-f0-9]{20}$`).test(entry.base), `Missing independent preview ${component.slug}/${theme}`);
    if (!entry) continue;
    for (const [name,checksum] of Object.entries(entry.assets)) {
      const file=path.join(output,entry.base,name);
      check(fs.existsSync(file) && sha(fs.readFileSync(file))===checksum, `Preview integrity mismatch: ${component.slug}/${theme}/${name}`);
    }
    for (const [name,checksum] of Object.entries(entry.sourceAssets)) {
      const file=path.join(output,entry.sources[name]);
      check(fs.existsSync(file) && sha(fs.readFileSync(file))===checksum, `Preview source integrity mismatch: ${component.slug}/${theme}/${name}`);
    }
  }
}
const origin = siteOrigin();
check(!fs.existsSync(path.join(output, 'en')), 'Legacy /en pages must redirect, not remain duplicate HTML output');
const redirectFile = path.join(output, '_redirects');
check(fs.existsSync(redirectFile), 'Cloudflare Pages must receive the legacy English redirect rules');
const redirects = fs.existsSync(redirectFile) ? fs.readFileSync(redirectFile, 'utf8').split('\n').map(line => line.trim()).filter(line => line && !line.startsWith('#')).map(line => line.split(/\s+/)) : [];
function redirectTarget(route) {
  for (const [source, destination, status] of redirects) {
    if (source === route) return { destination, status };
    if (source?.endsWith('*') && route.startsWith(source.slice(0, -1))) {
      return { destination: destination.replace(':splat', route.slice(source.length - 1)), status };
    }
  }
}
check(redirectTarget('/en')?.destination === '/' && redirectTarget('/en')?.status === '301', 'Old /en homepage must permanently redirect to /');
const expectedRoutes = new Set();
for (const locale of ['en', 'fr']) {
  const generalPages = Object.keys(JSON.parse(fs.readFileSync(path.join(docs, 'content/pages.json'), 'utf8')));
  for (const key of [...generalPages, ...components.map(component => `components/${component.slug}/`)]) {
    expectedRoutes.add(localeUrl(key, locale));
  }
}
const expectedPageCount = expectedRoutes.size;
const generalDescriptions = new Set();
for (const file of htmlFiles) {
  const html = fs.readFileSync(file, 'utf8');
  const route = '/' + path.relative(output, file).replaceAll(path.sep, '/').replace(/index\.html$/, '');
  const locale = route.startsWith('/fr/') ? 'fr' : 'en';
  const key = locale === 'fr' ? route.slice('/fr/'.length) : route.slice(1);
  check(expectedRoutes.delete(route), `Unexpected or duplicate HTML route: ${route}`);
  check(html.includes(`<html lang="${locale}">`), `${route}: incorrect HTML language`);
  check(html.includes(`<link rel="canonical" href="${origin}${route}">`), `${route}: canonical URL must use the configured public origin`);
  for (const language of ['fr', 'en']) {
    check(html.includes(`<link rel="alternate" hreflang="${language}" href="${origin}${localeUrl(key, language)}">`), `${route}: incorrect corresponding-page hreflang for ${language}`);
  }
  check(html.includes(`<link rel="alternate" hreflang="x-default" href="${origin}${localeUrl(key, 'en')}">`), `${route}: default-language hreflang must point to English`);
  const other = locale === 'fr' ? 'en' : 'fr';
  check(html.includes(`class="language" href="${localeUrl(key, other)}"`), `${route}: visible language switch must preserve the page`);
  check(!/http-equiv="refresh"/i.test(html), `${route}: content pages must not use a meta redirect`);
  const pageTitle = decode(html.match(/<title>([^<]+)<\/title>/)?.[1] || '');
  const description = decode(html.match(/<meta name="description" content="([^"]+)">/)?.[1] || '');
  check(pageTitle.length > 0 && description.length > 0, `${route}: title and description are required`);
  if (!/^components\/[^/]+\/$/.test(key)) {
    check(!generalDescriptions.has(description), `${route}: general pages need distinct descriptions`);
    generalDescriptions.add(description);
  }
  check(html.includes(`<meta property="og:url" content="${origin}${route}">`), `${route}: sharing URL must match canonical`);
  const shareImage = `${origin}/assets/social/slint-kit-${locale}.png`;
  check(html.includes(`<meta property="og:image" content="${shareImage}">`) && html.includes(`<meta name="twitter:image" content="${shareImage}">`), `${route}: localized sharing image is required`);
  check(html.includes('<meta name="twitter:card" content="summary_large_image">'), `${route}: large sharing card is required`);
  for (const image of html.matchAll(/<img\b[^>]*>/g)) {
    check(/\balt="[^"]*"/.test(image[0]) && /\bwidth="\d+"/.test(image[0]) && /\bheight="\d+"/.test(image[0]), `${route}: every image needs alt and intrinsic dimensions`);
    const src = image[0].match(/\bsrc="(\/[^"?#]+\.png)"/)?.[1];
    if (src && fs.existsSync(path.join(output, src.slice(1)))) {
      const pixels = fs.readFileSync(path.join(output, src.slice(1)));
      check(Number(image[0].match(/\bwidth="(\d+)"/)?.[1]) === pixels.readUInt32BE(16) && Number(image[0].match(/\bheight="(\d+)"/)?.[1]) === pixels.readUInt32BE(20), `${route}: declared PNG dimensions must match the actual capture`);
      if (src.startsWith('/assets/components/')) {
        const retinaSrc = src.replace(/\.png$/, '@2x.png');
        check(image[0].includes(`srcset="${retinaSrc} 2x"`), `${route}: preview needs a 2x source`);
        const retinaPath = path.join(output, retinaSrc.slice(1));
        check(fs.existsSync(retinaPath), `${route}: missing Retina preview`);
        if (fs.existsSync(retinaPath)) {
          const retina = fs.readFileSync(retinaPath);
          check(retina.readUInt32BE(16) === pixels.readUInt32BE(16) * 2 && retina.readUInt32BE(20) === pixels.readUInt32BE(20) * 2, `${route}: Retina image must preserve the logical preview size`);
        }
      }
    }
  }
  if (key) {
    check(/<details class="mobile-nav">/.test(html), `${route}: mobile menu must start closed without JavaScript`);
    check(html.includes('<nav class="desktop-nav"'), `${route}: desktop navigation must be available without script initialization`);
  }
  const jsonBlocks = [...html.matchAll(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/g)];
  check(jsonBlocks.length === 1, `${route}: one structured graph is required`);
  try {
    const graph = JSON.parse(jsonBlocks[0]?.[1] || 'null')?.['@graph'];
    const page = graph?.find(item => item['@id'] === `${origin}${route}#webpage`);
    const source = graph?.find(item => item['@type'] === 'SoftwareSourceCode');
    check(page?.url === origin + route && page?.inLanguage === locale && page?.name === pageTitle && page?.description === description, `${route}: structured page facts must match visible metadata`);
    check(source?.codeRepository === 'https://github.com/damione1/slint-kit' && source?.license.endsWith('/LICENSE'), `${route}: source identity must be factual`);
    if (key) {
      const crumbs = graph?.find(item => item['@type'] === 'BreadcrumbList')?.itemListElement;
      check(crumbs?.at(-1)?.item === origin + route && crumbs?.every((crumb, index) => crumb.position === index + 1), `${route}: breadcrumb must end at the actual page with consecutive positions`);
      const visible = html.match(/<nav class="breadcrumbs"[\s\S]*?<\/nav>/)?.[0] || '';
      check(crumbs?.every(crumb => visible.includes(crumb.name)), `${route}: structured breadcrumb names must be visible`);
    }
  } catch (error) { check(false, `${route}: invalid structured data: ${error.message}`); }
  if (locale === 'en') {
    const legacy = route === '/' ? '/en/' : '/en' + route;
    const redirect = redirectTarget(legacy);
    check(redirect?.destination === route && redirect?.status === '301', `${legacy}: old English URL must permanently redirect to ${route}`);
  }
  check(!/\{[{%]|undefined|\[object Object\]/.test(html), `${path.relative(output, file)}: unresolved template content`);
  const linkedUrls = [...html.matchAll(/\b(href|src|srcset)="([^"]+)"/g)].flatMap(([, attribute, value]) => attribute === 'srcset' ? value.split(',').map(candidate => candidate.trim().split(/\s+/)[0]) : [value]);
  for (const linkedUrl of linkedUrls) {
    let href = decode(linkedUrl);
    if (/^(?:mailto:|data:)/.test(href)) continue;
    if (/^https?:/.test(href)) {
      const url = new URL(href);
      if (url.origin !== origin) continue;
      href = url.pathname + url.hash;
    }
    check(!/^\/en(?:\/|$)/.test(href), `${route}: public links must use canonical English root routes, not ${href}`);
    const [location, fragment] = href.split('#');
    const destination = location.split('?')[0];
    let target = destination ? (destination.startsWith('/') ? path.join(output, decodeURIComponent(destination)) : path.resolve(path.dirname(file), decodeURIComponent(destination))) : file;
    if (fs.existsSync(target) && fs.statSync(target).isDirectory()) target = path.join(target, 'index.html');
    check(fs.existsSync(target), `${path.relative(output, file)}: broken local URL ${href}`);
    if (fragment && fs.existsSync(target) && target.endsWith('.html')) {
      const targetHtml = fs.readFileSync(target, 'utf8');
      check(targetHtml.includes(`id="${decodeURIComponent(fragment)}"`), `${path.relative(output, file)}: missing fragment ${href}`);
    }
  }
}
// Script raw-text serialization must round-trip arbitrary editorial copy
// without allowing a closing script tag, an HTML entity or a Unicode separator.
const hostileCopy = { description: '</script><script>alert("&")</script>\u2028\u2029' };
const serializedCopy = scriptJson(hostileCopy);
check(!/[<>&\u2028\u2029]/.test(serializedCopy) && JSON.parse(serializedCopy).description === hostileCopy.description, 'JSON-LD must safely preserve raw-text special characters');
for (const locale of ['en', 'fr']) {
  const image = fs.readFileSync(path.join(docs, 'src/assets/social', `slint-kit-${locale}.png`));
  check(image.subarray(0, 8).toString('hex') === '89504e470d0a1a0a' && image.readUInt32BE(16) === 1200 && image.readUInt32BE(20) === 630, `${locale}: sharing image must be a 1200×630 PNG`);
  const page = fs.readFileSync(path.join(output, localeUrl('guides/table-example/', locale), 'index.html'), 'utf8');
  const blocks = [...page.matchAll(/<pre><code class="language-(?:toml|rust|slint)">([\s\S]*?)<\/code><\/pre>/g)].map(match => decode(match[1]).trim());
  for (const file of ['Cargo.toml', 'build.rs', 'ui/table.slint', 'src/main.rs']) {
    const source = fs.readFileSync(path.join(docs, 'table-consumer', file), 'utf8');
    check(blocks.includes(source.trim()), `${locale}: table guide must show the complete ${file}`);
    check(fs.readFileSync(path.join(output, 'table-consumer', file), 'utf8') === source, `${locale}: table download must match displayed source: ${file}`);
  }
}
const tableProvenance = JSON.parse(fs.readFileSync(path.join(docs, 'content/table-captures.json'), 'utf8'));
check(tableProvenance.backend === 'winit' && tableProvenance.renderer === 'skia', 'Table captures must use the documented native backend');
for (const file of ['Cargo.toml', 'build.rs', 'ui/table.slint', 'src/main.rs']) {
  check(tableProvenance.sources[file] === sha(fs.readFileSync(path.join(docs, 'table-consumer', file))), `Table consumer changed: recapture native example after ${file}`);
}
for (const locale of ['en', 'fr']) {
  const capture = tableProvenance.captures[locale];
  const image = fs.readFileSync(path.join(docs, 'src/assets/components', capture.file));
  check(capture.sha256 === sha(image) && capture.width === image.readUInt32BE(16) && capture.height === image.readUInt32BE(20), `${locale}: table screenshot must match its provenance`);
  const retina = fs.readFileSync(path.join(docs, 'src/assets/components', capture.retina.file));
  check(capture.retina.pixelDensity === 2 && capture.retina.sha256 === sha(retina) && capture.retina.width === capture.width * 2 && capture.retina.height === capture.height * 2, `${locale}: Retina table capture must match its provenance`);
}
const tableManifest = fs.readFileSync(path.join(docs, 'table-consumer/Cargo.toml'), 'utf8');
check(!/\bpath\s*=|\bworkspace\s*=|\b(?:version|edition|license|rust-version)\.workspace\s*=/.test(tableManifest), 'Public table consumer must resolve registry dependencies without workspace inheritance');
check(expectedRoutes.size === 0, `Missing documentation routes: ${[...expectedRoutes].join(', ')}`);
for (const locale of ['en', 'fr']) {
  const page = path.join(output, localeUrl('guides/typed-example/', locale), 'index.html');
  const html = fs.existsSync(page) ? fs.readFileSync(page, 'utf8') : '';
  const blocks = [...html.matchAll(/<pre><code class="language-(?:toml|rust|slint)">([\s\S]*?)<\/code><\/pre>/g)].map(match => decode(match[1]).trim());
  for (const file of ['Cargo.toml', 'build.rs', 'ui/app.slint', 'src/main.rs']) {
    const source = fs.readFileSync(path.join(docs, 'consumer', file), 'utf8');
    check(blocks.includes(source.trim()), `${locale}: typed guide must display the complete downloadable ${file}`);
    const downloadable = path.join(output, 'consumer', file);
    check(fs.existsSync(downloadable) && fs.readFileSync(downloadable, 'utf8') === source, `${locale}: typed guide download differs from displayed ${file}`);
  }
}
const consumerManifest = fs.readFileSync(path.join(docs, 'consumer/Cargo.toml'), 'utf8');
check(!/\bpath\s*=|\bworkspace\s*=|\b(?:version|edition|license|rust-version)\.workspace\s*=/.test(consumerManifest), 'Public consumer manifest must use registry dependencies without workspace inheritance');
const consumerMain = fs.readFileSync(path.join(docs, 'consumer/src/main.rs'), 'utf8');
check(!/mod verify|verify::|--verify/.test(consumerMain), 'Public consumer must run without a private verification module');
check(fs.readFileSync(path.join(docs, 'consumer/ui/app.slint'), 'utf8') === fs.readFileSync(path.join(docs, 'integration/ui/app.slint'), 'utf8'), 'Public consumer UI must match the screenshot-verified integration UI');
const sitemapFile = path.join(output, 'sitemap.xml');
check(fs.existsSync(sitemapFile), 'Missing sitemap.xml');
const sitemap = fs.existsSync(sitemapFile) ? fs.readFileSync(sitemapFile, 'utf8') : '';
check(sitemap.startsWith('<?xml version="1.0" encoding="UTF-8"?>'), 'Sitemap must declare UTF-8 XML');
check(sitemap.includes('xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"'), 'Sitemap must use the standard sitemap namespace');
const sitemapEntries = [...sitemap.matchAll(/<url>\s*([\s\S]*?)\s*<\/url>/g)].map(match => match[1]);
const canonicalUrls = new Set(htmlFiles.map(file => origin + '/' + path.relative(output, file).replaceAll(path.sep, '/').replace(/index\.html$/, '')));
check(sitemapEntries.length === canonicalUrls.size, 'Sitemap must include every emitted canonical HTML page, including all paginated references');
const seenUrls = new Set();
let frenchUrls = 0;
let englishUrls = 0;
for (const entry of sitemapEntries) {
  const locations = [...entry.matchAll(/<loc>([^<]+)<\/loc>/g)];
  check(locations.length === 1, 'Each sitemap entry must declare exactly one canonical URL');
  if (locations.length !== 1) continue;
  const location = decode(locations[0][1]).replaceAll('&apos;', "'");
  check(!seenUrls.has(location), `Duplicate sitemap URL: ${location}`);
  seenUrls.add(location);
  check(canonicalUrls.delete(location), `Sitemap URL does not match an emitted canonical HTML page: ${location}`);
  let url;
  try { url = new URL(location); } catch { check(false, `Invalid sitemap URL: ${location}`); continue; }
  check(url.origin === origin && !url.search && !url.hash, `Sitemap URL must use the configured origin without query or fragment: ${location}`);
  check(!/^\/(?:en|assets|examples|consumer)(?:\/|$)/.test(url.pathname), `Sitemap must exclude legacy routes and downloadable files: ${location}`);
  const french = url.pathname.startsWith('/fr/');
  if (french) frenchUrls++; else englishUrls++;
  const key = french ? url.pathname.slice('/fr/'.length) : url.pathname.slice(1);
  for (const language of ['en', 'fr', 'x-default']) {
    const target = origin + localeUrl(key, language === 'x-default' ? 'en' : language);
    const alternate = [...entry.matchAll(/<xhtml:link\s+rel="alternate"\s+hreflang="([^"]+)"\s+href="([^"]+)"\s*\/>/g)].find(match => match[1] === language);
    check(alternate && decode(alternate[2]).replaceAll('&apos;', "'") === target, `Sitemap hreflang must retain the corresponding page for ${language}: ${location}`);
  }
}
check(canonicalUrls.size === 0, `Canonical HTML pages missing from sitemap: ${[...canonicalUrls].join(', ')}`);
check(englishUrls > 0 && frenchUrls > 0 && englishUrls === frenchUrls, 'Sitemap must include the complete English and French page sets');
const robotsFile = path.join(output, 'robots.txt');
check(fs.existsSync(robotsFile), 'Missing robots.txt');
const robots = fs.existsSync(robotsFile) ? fs.readFileSync(robotsFile, 'utf8') : '';
check(/^User-agent:\s*\*\s*$/m.test(robots) && /^Allow:\s*\/\s*$/m.test(robots), 'Robots rules must allow the public documentation site');
check(!/^Disallow:\s*\S/m.test(robots), 'Public documentation must not be blocked in robots.txt');
check(robots.split('\n').filter(line => line.startsWith('Sitemap:')).join('\n') === `Sitemap: ${origin}/sitemap.xml`, 'Robots must reference the sitemap at the configured public origin');

check(htmlFiles.length === expectedPageCount, 'Incomplete or duplicated site output');
if (errors.length) { console.error(errors.join('\n')); process.exitCode = 1; }
else console.log(`Documentation checks passed: ${components.length} public components, ${components.length * 2} localized references, ${components.length * 2} localized screenshots, verbatim examples, ${htmlFiles.length} canonical HTML pages and sitemap URLs, English root/French routes, legacy redirects, standalone consumer, local links and image provenance.`);
