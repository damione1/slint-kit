// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';

const entities = {amp:'&',lt:'<',gt:'>',quot:'"',apos:"'",nbsp:' ',rsquo:'’',lsquo:'‘',rdquo:'”',ldquo:'“',ndash:'–',mdash:'—',hellip:'…'};
function text(html) {
  return html.replace(/<[^>]*>/g,' ').replace(/&(#x[\da-f]+|#\d+|\w+);/gi, (match, entity) => {
    if (entity.startsWith('#')) {
      const point = entity[1].toLowerCase()==='x' ? parseInt(entity.slice(2),16) : Number(entity.slice(1));
      return Number.isInteger(point)&&point>0&&point<=0x10ffff ? String.fromCodePoint(point) : ' ';
    }
    return entities[entity] ?? match;
  }).replace(/\s+/g,' ').trim();
}

// Index reader-facing prose and API tables from the actual emitted pages.
// Downloadable programs, navigation and interactive/noindex shells are excluded.
export function searchDocument(html, url) {
  if (/<meta\b[^>]*name=["']robots["'][^>]*content=["'][^"']*noindex/i.test(html)) return null;
  const main = html.match(/<main\b[^>]*>([\s\S]*?)<\/main>/i)?.[1];
  const title = main?.match(/<h1\b[^>]*>([\s\S]*?)<\/h1>/i)?.[1];
  const locale = html.match(/<html\b[^>]*lang=["'](en|fr)["']/i)?.[1];
  if (!main || !title || !locale) return null;
  const prose = main.replace(/<section\b[^>]*class=["']live-preview["'][^>]*>[\s\S]*?<\/section>/gi,' ').replace(/<p\b[^>]*class=["'](?:component-identifier|eyebrow)["'][^>]*>[\s\S]*?<\/p>/gi,' ').replace(/<(pre|script|style|nav)\b[^>]*>[\s\S]*?<\/\1>/gi,' ').replace(/<h1\b[^>]*>[\s\S]*?<\/h1>/i,' ');
  const description = html.match(/<meta\b[^>]*name=["']description["'][^>]*content=["']([^"']*)["']/i)?.[1] || '';
  const api = main.match(/<p\b[^>]*class=["']component-identifier["'][^>]*>\s*<code>([^<]+)<\/code>/i)?.[1] || '';
  return {locale,url,title:text(title),api:text(api),summary:text(description),text:text(prose)};
}

export function writeSearchIndexes(directory) {
  const documents = {en:[],fr:[]};
  function walk(folder) {
    for (const entry of fs.readdirSync(folder,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))) {
      const file = path.join(folder,entry.name);
      if (entry.isDirectory()) walk(file);
      else if (entry.name==='index.html') {
        const url = '/'+path.relative(directory,file).split(path.sep).join('/').replace(/index\.html$/,'');
        const document = searchDocument(fs.readFileSync(file,'utf8'),url);
        if (document) { const {locale,...record}=document; documents[locale].push(record); }
      }
    }
  }
  walk(directory);
  for (const [locale,entries] of Object.entries(documents)) {
    fs.mkdirSync(path.join(directory,'assets'),{recursive:true});
    fs.writeFileSync(path.join(directory,'assets',`search-index-${locale}.json`),JSON.stringify({schema:1,locale,entries}));
  }
}
