// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import { createHash } from 'node:crypto';

// Read the public generated glyphs: the gallery paints the exact same stroke
// and fill layers as KitIcon, including Lucide's filled dots.
export function lucideCatalogue() {
  const root = new URL('../../', import.meta.url);
  const source = fs.readFileSync(new URL('crates/kit-slint/ui/icons/lucide/all.slint', root), 'utf8');
  const version = source.match(/\(Lucide ([\d.]+),/)[1];
  const snapshot = new URL(`vendor/lucide/${version}/`, root);
  const icons = [...source.matchAll(/public pure function ([\w-]+)\(\) -> IconGlyph \{ return \{ stroke: "([^"]*)", fill: "([^"]*)", viewbox: (\d+) \}; \}/g)].map(([, name, stroke, fill, viewbox]) => {
    const metadata = JSON.parse(fs.readFileSync(new URL(`icons/${name}.json`, snapshot), 'utf8'));
    return { name, stroke, fill, viewbox, categories: metadata.categories,
      search: [name, ...metadata.tags, ...metadata.categories].join(' ').toLowerCase(),
      deprecated: metadata.deprecated || false };
  });
  if (icons.length !== Number(source.match(/all (\d+) icons/)[1])) throw new Error('Incomplete Lucide gallery');
  const license = fs.readFileSync(new URL('LICENSE', snapshot), 'utf8');
  const sprite = `<!-- Lucide ${version}. Original license notices:\n${license.replaceAll('--', '—')}\n-->\n<svg xmlns="http://www.w3.org/2000/svg">${icons.map(icon => `<symbol id="${icon.name}" viewBox="0 0 ${icon.viewbox} ${icon.viewbox}"><path d="${icon.stroke}" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>${icon.fill ? `<path d="${icon.fill}" fill="currentColor"/>` : ''}</symbol>`).join('')}</svg>`;
  return { version, icons, categories: [...new Set(icons.flatMap(icon => icon.categories))].sort(),
    sprite, url: `/assets/versioned/lucide.${createHash('sha256').update(sprite).digest('hex').slice(0, 16)}.svg` };
}
