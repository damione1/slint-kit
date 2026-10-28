// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const docsDirectory = fileURLToPath(new URL('../', import.meta.url));
export const starterFiles = ['Cargo.toml', 'Cargo.lock', 'build.rs', 'src/main.rs', 'ui/app.slint', 'preview@2x.png'];

export function loadStarters() {
  const entries = JSON.parse(fs.readFileSync(path.join(docsDirectory, 'content/starters.json'), 'utf8'));
  const captures = JSON.parse(fs.readFileSync(path.join(docsDirectory, 'content/starter-captures.json'), 'utf8'));
  return Object.fromEntries(Object.entries(entries).map(([slug, entry]) => {
    if (!/^[a-z]+(?:-[a-z]+)*$/.test(slug)) throw new Error(`Invalid starter slug: ${slug}`);
    const image = fs.readFileSync(path.join(docsDirectory, 'quickstart', slug, 'preview@2x.png'));
    const capture = captures[slug];
    const width = image.readUInt32BE(16), height = image.readUInt32BE(20);
    const scale = capture.environment.scale_factor || 2;
    return [slug, { ...entry, width, height, displayWidth: width / scale }];
  }));
}
