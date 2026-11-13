// SPDX-License-Identifier: GPL-3.0-or-later
import test from 'node:test';
import assert from 'node:assert/strict';
import { declaredDefault, propertyDeclaration, propertyDocumentation, scanComponents } from './inventory.mjs';

const declarations = source => Array.from(source.matchAll(propertyDeclaration), m => ({ name: m[3], default: declaredDefault(m[4], m[5]) }));

test('declared defaults come from the source: inline initializers verbatim, zero values and composite expressions named', () => {
  const [plain, inline, bound, multiline, text] = declarations([
    'in property <string> label;',
    'in property <int> active-index: -1;',
    'in-out property <string> query <=> selection.query;',
    'in property <length> row-height: max(48px,\n        32px * root.theme.text-scale);',
    'in property <string> empty-label: "No matching records";',
  ].join('\n'));
  assert.deepEqual(plain, { name: 'label', default: { kind: 'none' } });
  assert.deepEqual(inline, { name: 'active-index', default: { kind: 'inline', text: '-1' } });
  assert.deepEqual(bound, { name: 'query', default: { kind: 'elided' } });
  assert.deepEqual(multiline, { name: 'row-height', default: { kind: 'elided' } });
  assert.deepEqual(text, { name: 'empty-label', default: { kind: 'inline', text: '"No matching records"' } });
  assert.deepEqual(declaredDefault(':', ' root.purpose == TextPurpose.search ? root.theme.icons.search :        ', true), { kind: 'elided' }, 'a masked struct literal is never shown truncated');
});

const component = (name, en, fr) => ({ name, en: { properties: en }, fr: { properties: fr } });
const declared = new Map([['label', {}], ['required', {}]]);

test('property documentation must describe declared or inherited properties consistently in both languages', () => {
  const valid = component('KitSample', { label: { effect: 'Names the editor.', example: 'label: "Name";' } }, { label: { effect: 'Nomme l’éditeur.', example: 'label: "Nom";' } });
  const documentation = propertyDocumentation(valid, declared);
  assert.deepEqual([...documentation.keys()], ['label']);
  assert.equal(documentation.get('label').documentedOn, 'KitSample');
  assert.equal(documentation.get('label').fr.effect, 'Nomme l’éditeur.');
  assert.throws(() => propertyDocumentation(component('KitSample', { text: { effect: 'Draft.' } }, { text: { effect: 'Brouillon.' } }), declared), /neither declared nor inherited/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: 'Names.' } }, { required: { effect: 'Exige.' } }), declared), /differ between English and French/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: 'Names.', default: 'Empty.' } }, { label: { effect: 'Nomme.' } }), declared), /documented keys differ/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: 'Names.', summary: 'x' } }, { label: { effect: 'Nomme.', summary: 'x' } }), declared), /unknown documentation key/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { default: 'Empty.' } }, { label: { default: 'Vide.' } }), declared), /effect is required/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: 'Names.', example: 'text: "x";' } }, { label: { effect: 'Nomme.', example: 'text: "x";' } }), declared), /example must use the property/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: 'Names.', example: 'label: "a";\n\nlabel: "b";' } }, { label: { effect: 'Nomme.', example: 'label: "a";\n\nlabel: "b";' } }), declared), /blank line/);
  assert.throws(() => propertyDocumentation(component('KitSample', { label: { effect: '' } }, { label: { effect: '' } }), declared), /nonempty string/);
});

test('the reference explains the central field, selection and table properties and children inherit the explanations', () => {
  const components = scanComponents();
  const byName = new Map(components.map(entry => [entry.name, entry]));
  for (const name of ['KitTextField', 'KitNumberStepper', 'KitSelect', 'KitCombobox', 'KitMultiSelect', 'KitRadioGroup', 'KitCheckbox', 'KitDataTable', 'KitContentList']) {
    assert.ok(byName.get(name).documentedProperties.some(property => property.owner === name), `${name} documents its own properties`);
  }
  for (const name of ['label', 'text', 'access', 'required', 'commit-policy', 'spec-key', 'validation-timing']) {
    assert.ok(byName.get('KitTextField').documentedProperties.some(property => property.name === name), `KitTextField documents ${name}`);
  }
  for (const name of ['columns', 'rows', 'generation', 'active-index', 'active-column', 'edit-error']) {
    assert.ok(byName.get('KitDataTable').documentedProperties.some(property => property.name === name), `KitDataTable documents ${name}`);
  }
  // The three central components leave no declared or inherited property unexplained.
  for (const name of ['KitTextField', 'KitSelect', 'KitDataTable']) {
    const undocumented = byName.get(name).effectiveProperties.filter(property => !property.documentation).map(property => property.name);
    assert.deepEqual(undocumented, [], `${name} explains every property`);
  }
  const email = byName.get('KitEmailField').documentedProperties.find(property => property.name === 'purpose');
  assert.equal(email.owner, 'KitTextField');
  assert.equal(email.documentation.documentedOn, 'KitTextField');
  assert.equal(email.documentation.en.effect, byName.get('KitTextField').en.properties.purpose.effect);
  // Composition links come from what the owner renders, not from a property name.
  const fieldLabel = byName.get('KitMoneyField').documentedProperties.find(property => property.name === 'label');
  assert.equal(fieldLabel.compositionPrimitive?.name, 'KitFieldLabel');
  const tableLabel = byName.get('KitDataTable').documentedProperties.find(property => property.name === 'label');
  assert.equal(tableLabel.compositionPrimitive, undefined, 'a table label is an accessible name, not a field label');
  for (const entry of components) {
    for (const property of entry.effectiveProperties) assert.ok(['none', 'inline', 'elided'].includes(property.default.kind), `${entry.name}.${property.name} has a declared default`);
  }
});
