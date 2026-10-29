// SPDX-License-Identifier: GPL-3.0-or-later
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../../', import.meta.url));
export const slugify = name => name.replace(/^Kit/, '').replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
export const componentLabel = (component, locale) => component[locale].label
  || component.name.replace(/^Kit/, '').replace(/([a-z0-9])([A-Z])/g, '$1 $2');
export const families = {
  button: { fr: 'Boutons et actions', en: 'Buttons and actions' },
  field: { fr: 'Structure des formulaires', en: 'Form structure' },
  input: { fr: 'Saisie de texte', en: 'Text input' },
  number: { fr: 'Nombres et montants', en: 'Numbers and money' },
  select: { fr: 'Sélection', en: 'Selection' },
  choice: { fr: 'Choix et valeurs', en: 'Choices and values' },
  calendar: { fr: 'Dates', en: 'Dates' },
  datetime: { fr: 'Heures et fuseaux', en: 'Times and zones' },
  files: { fr: 'Fichiers', en: 'Files' },
  color: { fr: 'Couleurs', en: 'Colors' },
  layout: { fr: 'Mise en page', en: 'Layout' },
  display: { fr: 'Présentation', en: 'Display' },
  feedback: { fr: 'Retour et progression', en: 'Feedback and progress' },
  navigation: { fr: 'Navigation', en: 'Navigation' },
  dialog: { fr: 'Dialogues', en: 'Dialogs' },
  data: { fr: 'Données virtualisées', en: 'Virtualized data' },
  focus: { fr: 'Gestion du focus', en: 'Focus management' },
  icons: { fr: 'Icônes', en: 'Icons' },
};
// Preserve positions while removing comments and quoted strings, so declarations
// inside children or implementation functions never enter the public reference.
function mask(source) {
  return source.replace(/\/\*[\s\S]*?\*\/|\/\/[^\n]*|"(?:\\.|[^"\\])*"/g, x => x.replace(/[^\n]/g, ' '));
}
export function scanComponents() {
  const dir = path.join(root, 'crates/kit-slint/ui');
  const metadataPath = path.join(root, 'docs/content/components.json');
  const metadata = fs.existsSync(metadataPath) ? JSON.parse(fs.readFileSync(metadataPath, 'utf8')) : {};
  const types = new Map();
  for (const file of fs.readdirSync(dir).filter(x => x.endsWith('.slint'))) {
    const source = fs.readFileSync(path.join(dir, file), 'utf8');
    for (const match of source.matchAll(/export\s+(enum|struct)\s+(\w+)\s*\{([^}]+)\}/g)) {
      types.set(match[2], { kind: match[1], name: match[2], definition: match[3].trim().replace(/\s+/g, ' ') });
    }
  }
  const components = [];
  for (const file of fs.readdirSync(dir).filter(x => x.endsWith('.slint'))) {
    const source = fs.readFileSync(path.join(dir, file), 'utf8');
    const masked = mask(source);
    for (const match of masked.matchAll(/export\s+component\s+(\w+)\s+inherits\s+(\w+)\s*\{/g)) {
      const [declaration, name, base] = match;
      let depth = 1, end = match.index + declaration.length;
      const start = end;
      while (depth && end < masked.length) { if (masked[end] === '{') depth++; if (masked[end] === '}') depth--; end++; }
      const body = source.slice(start, end - 1);
      const bodyMask = mask(body);
      let nesting = 0, top = '';
      for (let i = 0; i < bodyMask.length; i++) {
        if (bodyMask[i] === '{') nesting++;
        if (!nesting) top += body[i]; else top += bodyMask[i] === '\n' ? '\n' : ' ';
        if (bodyMask[i] === '}') nesting--;
      }
      const properties = Array.from(top.matchAll(/\b(in-out|in|out)\s+property\s*<([^;]+?)>\s*([\w-]+)/g), m => ({ direction: m[1], type: m[2], name: m[3] }));
      const callbacks = Array.from(top.matchAll(/\bcallback\s+([\w-]+)\s*\(([^)]*)\)\s*(?:->\s*([^;]+))?\s*;/g), m => ({ name: m[1], signature: `${m[1]}(${m[2].trim()})${m[3] ? ` → ${m[3].trim()}` : ''}` }));
      const functions = Array.from(top.matchAll(/\bpublic\s+(?:pure\s+)?function\s+([\w-]+)\s*\(([^)]*)\)\s*(?:->\s*([^{}\n]+))?/g), m => ({ signature: `${m[1]}(${m[2].trim()})${m[3] ? ` → ${m[3].trim()}` : ''}` }));
      const signatures = [...properties.map(p => p.type), ...callbacks.map(c => c.signature), ...functions.map(f => f.signature)].join(' ');
      const relatedTypes = new Map();
      function collectTypes(text) {
        for (const name of text.match(/\b[A-Z]\w+/g) || []) {
          if (!types.has(name) || relatedTypes.has(name) || ['KitTheme', 'ButtonRecipe'].includes(name)) continue;
          const definition = types.get(name);
          relatedTypes.set(name, definition);
          collectTypes(definition.definition);
        }
      }
      collectTypes(signatures);
      const publicTypes = [...relatedTypes.values()].sort((a, b) => a.name.localeCompare(b.name));
      const family = file.replace('.slint', '');
      const screenshots = {};
      for (const locale of ['fr', 'en']) {
        const filename = `${slugify(name)}${locale === 'fr' ? '-fr' : ''}.png`;
        const imagePath = path.join(root, 'docs/src/assets/components', filename);
        const image = fs.existsSync(imagePath) ? fs.readFileSync(imagePath) : undefined;
        screenshots[locale] = { filename, retinaFilename: filename.replace('.png', '@2x.png'), width: image?.length >= 24 ? image.readUInt32BE(16) : undefined, height: image?.length >= 24 ? image.readUInt32BE(20) : undefined };
      }
      components.push({ screenshots, name, slug: slugify(name), base, family, familyLabel: families[family], types: publicTypes, source: `crates/kit-slint/ui/${file}`, line: source.slice(0, match.index).split('\n').length, properties, callbacks, functions, ...metadata[name] });
    }
  }
  const byName = new Map(components.map(component => [component.name, component]));
  const inherited = (component, seen = new Set()) => {
    if (seen.has(component.name)) throw new Error(`Cyclic component inheritance: ${component.name}`);
    seen.add(component.name);
    const parent = byName.get(component.base);
    const properties = new Map((parent ? inherited(parent, seen) : []).map(property => [property.name, property]));
    for (const property of component.properties) properties.set(property.name, { ...property, owner: component.name });
    return [...properties.values()];
  };
  for (const component of components) component.effectiveProperties = inherited(component);
  const optionPages = components.filter(component => component.sharedOption);
  for (const component of components) {
    component.optionDemos = optionPages.filter(page => component.effectiveProperties.some(property => property.name === page.sharedOption));
    component.optionConsumers = component.sharedOption ? components.filter(candidate => candidate.effectiveProperties.some(property => property.name === component.sharedOption)).map(candidate => ({name:candidate.name,slug:candidate.slug})) : [];
    component.inheritedProperties = component.effectiveProperties.filter(property => property.owner !== component.name);
  }
  return components.sort((a, b) => a.name.localeCompare(b.name));
}
if (process.argv[1] === fileURLToPath(import.meta.url)) console.log(JSON.stringify(scanComponents(), null, 2));
