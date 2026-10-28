// SPDX-License-Identifier: GPL-3.0-or-later
// Run after native verification of the exact current table consumer sources.
import fs from 'node:fs';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';
const docs = fileURLToPath(new URL('../', import.meta.url));
const sha = value => crypto.createHash('sha256').update(value).digest('hex');
const sources = Object.fromEntries(['Cargo.toml', 'build.rs', 'ui/table.slint', 'src/main.rs'].map(file => [file, sha(fs.readFileSync(`${docs}/table-consumer/${file}`))]));
const captures = Object.fromEntries(['en', 'fr'].map(locale => {
  const file = `table-integration${locale === 'fr' ? '-fr' : ''}.png`;
  const image = fs.readFileSync(`${docs}/src/assets/components/${file}`);
  return [locale, { file, sha256: sha(image), width: image.readUInt32BE(16), height: image.readUInt32BE(20) }];
}));
fs.writeFileSync(`${docs}/content/table-captures.json`, JSON.stringify({ backend: 'winit', renderer: 'skia', snapshotAPI: 'slint::Window::take_snapshot', scaleFactor: 1, sources, captures }, null, 2) + '\n');
console.log('Table consumer source and native capture hashes recorded.');
