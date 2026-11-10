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
  keypad: { fr: 'Pavés numériques et codes', en: 'Keypads and codes' },
  select: { fr: 'Sélection', en: 'Selection' },
  choice: { fr: 'Choix et valeurs', en: 'Choices and values' },
  calendar: { fr: 'Dates', en: 'Dates' },
  datetime: { fr: 'Heures et fuseaux', en: 'Times and zones' },
  files: { fr: 'Fichiers', en: 'Files' },
  color: { fr: 'Couleurs', en: 'Colors' },
  layout: { fr: 'Mise en page', en: 'Layout' },
  scroll: { fr: 'Défilement', en: 'Scrolling' },
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
// The initializer of a property declaration, read from the source so the
// reference states the real default. `top` keeps top-level text verbatim and
// blanks nested braces, so a struct literal arrives as spaces: it is reported
// as `elided` rather than invented, like a multi-line expression that would
// not read as one table cell. `none` is the zero value of the type.
export function declaredDefault(operator, initializer, masked = false) {
  if (operator === undefined) return { kind: 'none' };
  const text = initializer.trim();
  if (operator === '<=>' || masked || text === '' || initializer.includes('\n')) return { kind: 'elided' };
  return { kind: 'inline', text: text.replace(/\s+/g, ' ') };
}
export const propertyDeclaration = /\b(in-out|in|out)\s+property\s*<([^;]+?)>\s*([\w-]+)(?:\s*(:|<=>)\s*([^;]*))?/g;
// Hand-written explanations of a property, per locale, from components.json.
// `effect` is required; the other keys are optional and must be given in both
// locales when one has them, so the two references stay consistent.
export const propertyDocumentationKeys = ['effect', 'default', 'states', 'constraints', 'example'];
const locales = ['en', 'fr'];
export function propertyDocumentation(component, declared) {
  const entries = Object.fromEntries(locales.map(locale => [locale, component[locale]?.properties || {}]));
  const names = Object.keys(entries.en);
  if (names.join('\n') !== Object.keys(entries.fr).join('\n')) throw new Error(`${component.name}: documented properties differ between English and French`);
  const documentation = new Map();
  for (const name of names) {
    if (!declared.has(name)) throw new Error(`${component.name}: documented property ${name} is neither declared nor inherited`);
    const keys = locales.map(locale => Object.keys(entries[locale][name]).join('\n'));
    if (keys[0] !== keys[1]) throw new Error(`${component.name}.${name}: documented keys differ between English and French`);
    for (const locale of locales) {
      const entry = entries[locale][name];
      for (const key of Object.keys(entry)) {
        if (!propertyDocumentationKeys.includes(key)) throw new Error(`${component.name}.${name}/${locale}: unknown documentation key ${key}`);
        if (typeof entry[key] !== 'string' || entry[key].trim() === '') throw new Error(`${component.name}.${name}/${locale}: ${key} must be a nonempty string`);
      }
      if (typeof entry.effect !== 'string') throw new Error(`${component.name}.${name}/${locale}: effect is required`);
      if (entry.example !== undefined && !entry.example.includes(name)) throw new Error(`${component.name}.${name}/${locale}: the example must use the property`);
      // The example is emitted inside one HTML block of the page; a blank line would end it.
      if (entry.example !== undefined && /\n\s*\n/.test(entry.example)) throw new Error(`${component.name}.${name}/${locale}: the example must not contain a blank line`);
    }
    documentation.set(name, { en: entries.en[name], fr: entries.fr[name], documentedOn: component.name });
  }
  return documentation;
}
export function scanComponents() {
  const dir = path.join(root, 'crates/kit-slint/ui');
  // Family files also export implementation helpers for sibling modules. Only
  // declarations re-exported by the public @kit module belong in this catalogue.
  const publicModule = fs.readFileSync(path.join(dir, 'kit.slint'), 'utf8');
  const publicNames = new Set(Array.from(mask(publicModule).matchAll(/\bexport\s*\{([^}]+)\}/g))
    .flatMap(match => match[1].split(',').map(name => name.trim())));
  const metadataPath = path.join(root, 'docs/content/components.json');
  const metadata = fs.existsSync(metadataPath) ? JSON.parse(fs.readFileSync(metadataPath, 'utf8')) : {};
  const renderProofPath = path.join(root,'docs/content/render-captures.json');
  const renderProofs = fs.existsSync(renderProofPath) ? JSON.parse(fs.readFileSync(renderProofPath,'utf8')).entries : {};
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
      if (!name.startsWith("Kit") || !publicNames.has(name)) continue;
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
      const properties = Array.from(top.matchAll(propertyDeclaration), m => ({ direction: m[1], type: m[2], name: m[3], default: declaredDefault(m[4], m[5], body.slice(m.index, m.index + m[0].length) !== m[0]) }));
      const callbacks = Array.from(top.matchAll(/\bcallback\s+([\w-]+)\s*\(([^)]*)\)\s*(?:->\s*([^;]+))?\s*;/g), m => ({ name: m[1], signature: `${m[1]}(${m[2].trim()})${m[3] ? ` → ${m[3].trim()}` : ''}` }));
      const functions = Array.from(top.matchAll(/\bpublic\s+(?:pure\s+)?function\s+([\w-]+)\s*\(([^)]*)\)\s*(?:->\s*([^{}\n]+))?/g), m => ({ name: m[1], signature: `${m[1]}(${m[2].trim()})${m[3] ? ` → ${m[3].trim()}` : ''}` }));
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
      const family = file === 'color-wheel.slint' ? 'color' : file.replace('.slint', '');
      const screenshots = {};
      for (const locale of ['fr', 'en']) {
        const filename = `${slugify(name)}${locale === 'fr' ? '-fr' : ''}.png`;
        const imagePath = path.join(root, 'docs/src/assets/components', filename);
        const image = fs.existsSync(imagePath) ? fs.readFileSync(imagePath) : undefined;
        const proof = renderProofs[`catalogue:${slugify(name)}:${locale}:light:1`];
        screenshots[locale] = { backend: proof?.renderProfile?.backend || 'winit', filename, retinaFilename: filename.replace('.png', '@2x.png'), width: image?.length >= 24 ? image.readUInt32BE(16) : undefined, height: image?.length >= 24 ? image.readUInt32BE(20) : undefined };
      }
      // The kit components this one instantiates, read from its body: a
      // shared option is linked to its composition primitive only when the
      // owner really renders that primitive from the option.
      const composes = new Set(Array.from(bodyMask.matchAll(/\b(Kit[A-Z]\w*)\s*\{/g), m => m[1]));
      components.push({ screenshots, name, slug: slugify(name), base, family, familyLabel: families[family], types: publicTypes, source: `crates/kit-slint/ui/${file}`, line: source.slice(0, match.index).split('\n').length, properties, callbacks, functions, composes, ...metadata[name] });
    }
  }
  const byName = new Map(components.map(component => [component.name, component]));
  const inherited = (component, seen = new Set()) => {
    if (seen.has(component.name)) throw new Error(`Cyclic component inheritance: ${component.name}`);
    seen.add(component.name);
    const parent = byName.get(component.base);
    const properties = new Map((parent ? inherited(parent, seen) : []).map(property => [property.name, property]));
    for (const property of component.properties) properties.set(property.name, { ...property, owner: component.name, documentation: undefined });
    // A component documents its own properties and may restate an inherited
    // one (for a changed default); otherwise the owner's explanation follows
    // the property down the inheritance chain.
    for (const [name, documentation] of propertyDocumentation(component, properties)) properties.set(name, { ...properties.get(name), documentation });
    return [...properties.values()];
  };
  for (const component of components) {
    for (const kind of ['callbacks', 'functions']) {
      const callables = new Map(component[kind].map(callable => [callable.name, callable]));
      const en = component.en?.[kind] || {};
      const fr = component.fr?.[kind] || {};
      if (Object.keys(en).sort().join('\n') !== Object.keys(fr).sort().join('\n')) throw new Error(`${component.name}: documented ${kind} differ between English and French`);
      for (const name of Object.keys(en)) {
        if (!callables.has(name)) throw new Error(`${component.name}: unknown documented ${kind} ${name}`);
        callables.get(name).documentation = { en: en[name], fr: fr[name] };
      }
    }
  }
  for (const component of components) component.effectiveProperties = inherited(component);
  const optionPages = components.filter(component => component.sharedOption);
  for (const component of components) {
    component.optionDemos = optionPages.filter(page => component.effectiveProperties.some(property => property.name === page.sharedOption));
    component.optionConsumers = component.sharedOption ? components.filter(candidate => candidate.effectiveProperties.some(property => property.name === component.sharedOption)).map(candidate => ({name:candidate.name,slug:candidate.slug})) : [];
    component.inheritedProperties = component.effectiveProperties.filter(property => property.owner !== component.name);
    // Own declarations keep their table rows; the detail section lists them
    // first, then the inherited ones, with the composition primitive of a
    // shared option derived from the pages that demonstrate it.
    component.properties = component.properties.map(property => component.effectiveProperties.find(candidate => candidate.name === property.name));
    component.documentedProperties = [...component.properties, ...component.inheritedProperties]
      .filter(property => property.documentation)
      .map(property => {
        const page = optionPages.find(candidate => candidate.sharedOption === property.name && byName.get(property.owner).composes.has(candidate.name));
        return { ...property, compositionPrimitive: page ? { name: page.name, slug: page.slug } : undefined };
      });
  }
  return components.sort((a, b) => a.name.localeCompare(b.name));
}
if (process.argv[1] === fileURLToPath(import.meta.url)) console.log(JSON.stringify(scanComponents(), null, 2));
