// SPDX-License-Identifier: GPL-3.0-or-later
import test from 'node:test';
import assert from 'node:assert/strict';
import {searchDocument} from './search-index.mjs';
import {normalizeSearch,prepareSearch,searchDocuments} from '../src/assets/search.js';

test('global search indexes prose and real API while excluding program listings and noindex demos',()=>{
  const html='<html lang="fr"><head><meta name="description" content="Édition &amp; validation"></head><body><nav>Navigation secrète</nav><main><nav>Fil d’Ariane</nav><h1>Champs &amp; validation</h1><p>Contraintes de formulaire.</p><table><tr><td><code>leading-icon</code></td></tr></table><pre><code>export { KitFormValidation }; very_long_downloadable_program</code></pre></main></body></html>';
  const record=searchDocument(html,'/fr/guides/forms/');
  assert.equal(record.locale,'fr');assert.equal(record.title,'Champs & validation');assert.equal(record.summary,'Édition & validation');
  assert(record.text.includes('leading-icon'));assert(!record.text.includes('very_long'));assert(!record.text.includes('Navigation'));assert(!record.text.includes('Ariane'));
  assert(record.symbols.includes('KitFormValidation'));assert(!record.text.includes('KitFormValidation'));
  assert.equal(searchDocument(html.replace('<head>','<head><meta name="robots" content="noindex">'),'/examples/workspace/'),null);
  assert.equal(searchDocument('<html lang="en"><main>no page title</main></html>','/404/'),null);
});

test('fuzzy search finds typos, accents, camel case APIs and multiple terms across the documentation',()=>{
  const entries=prepareSearch([
    {url:'/fr/guides/forms/',title:'Formulaires et validation',summary:'Valider les valeurs acceptées.',text:'Les contraintes intrinsèques email et URL. Initialiser le moteur une fois.',symbols:['KitFormValidation']},
    {url:'/fr/components/text-field/',title:'Champ de texte pour Slint',api:'KitTextField',summary:'Éditeur de texte',text:'Édition de texte, sélection et propriété leading-icon.'},
    {url:'/fr/components/button/',title:'Bouton pour Slint',api:'KitButton',summary:'KitButton',text:'Action principale et taille.'},
    {url:'/fr/guides/themes/',title:'Thèmes de Slint Kit',summary:'Personnaliser les contrôles',text:'KitThemeContext.current est le défaut des contrôles. KitTextField, KitButton et KitTheme sont personnalisables.'},
    {url:'/fr/components/form-button/',title:'Bouton de formulaire pour Slint',api:'KitFormButton',summary:'Bouton de soumission',text:'KitButton pour les formulaires. Demander la validation avant de soumettre.',symbols:['KitFormValidation']},
  ]);
  assert.equal(normalizeSearch('KitTextField Édition'),'kit text field edition');
  for(const query of ['validaton','valdiation','contraintes email'])assert.equal(searchDocuments(entries,query)[0]?.url,'/fr/guides/forms/',query);
  for(const query of ['KitTextField','kit txt field','edition selection','leading-icon'])assert.equal(searchDocuments(entries,query)[0]?.url,'/fr/components/text-field/',query);
  for(const query of ['KitButton','kitbutton','button'])assert.equal(searchDocuments(entries,query)[0]?.url,'/fr/components/button/',query);
  for(const query of ['KitFormValidation','kitformvalidation'])assert.equal(searchDocuments(entries,query)[0]?.url,'/fr/guides/forms/',query);
  for(const query of ['KitThemeContext','kitthemecontext'])assert.equal(searchDocuments(entries,query)[0]?.url,'/fr/guides/themes/',query);
  assert.deepEqual(searchDocuments(entries,'zzzzzzzzzz'),[]);assert.deepEqual(searchDocuments(entries,'   '),[]);
});
