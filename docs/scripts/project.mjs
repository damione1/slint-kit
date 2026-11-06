// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';

export function projectMetadata(source) {
  const sections = new Map();
  let section;
  for (const line of source.split('\n')) {
    const heading = line.match(/^\s*\[([^\]]+)\]\s*(?:#.*)?$/);
    if (heading) {
      section = heading[1];
      sections.set(section, '');
    } else if (section) {
      sections.set(section, sections.get(section) + line + '\n');
    }
  }
  function required(section, pattern, name) {
    const value = sections.get(section)?.match(pattern)?.[1];
    if (!value) throw new Error(`Workspace manifest is missing ${name}`);
    return value;
  }
  const version = required('workspace.package', /^version\s*=\s*"([^"]+)"/m, 'version');
  return {
    version,
    rust: required('workspace.package', /^rust-version\s*=\s*"([^"]+)"/m, 'rust-version'),
    slint: required('workspace.dependencies', /^slint\s*=\s*\{[^\n]*?version\s*=\s*"=([^"]+)"/m, 'exact Slint version'),
    preOne: version.startsWith('0.'),
  };
}

export function loadProject() {
  return projectMetadata(fs.readFileSync(new URL('../../Cargo.toml', import.meta.url), 'utf8'));
}
