---
layout: base.njk
pagination:
  data: locales
  size: 1
  addAllPagesToCollections: true
  alias: locale
permalink: "{{ '' | localeUrl(locale) }}index.html"
translationKey: ""
home: true
eleventyComputed:
  title: "{% if locale == 'fr' %}Documentation{% else %}Documentation{% endif %}"
---
{% if locale == 'fr' %}
<p class="eyebrow">Rust + Slint</p>

# Construire avec Slint Kit

Des composants natifs, des contrôleurs typés et des thèmes partagés pour vos applications Rust et Slint.

<div class="home-actions"><a class="primary-link" href="/fr/guides/getting-started/">Bien démarrer <span aria-hidden="true">→</span></a><a href="/fr/components/">Parcourir les {{ components.length }} composants</a></div>

<div class="project-facts"><p><strong>Version {{ project.version }}</strong>{% if project.preOne %} · avant la version 1.0{% endif %} · <a href="https://crates.io/crates/slint-kit">Paquets sur crates.io</a></p><p>Rust {{ project.rust }}+ · Slint {{ project.slint }} · <a href="/fr/guides/getting-started/">Prérequis et installation</a></p><p>Projet indépendant, code et ressources sous <a href="{{ site.repository }}/blob/main/LICENSE">GPL-3.0-or-later</a>. Maintenance et contributions sur <a href="{{ site.repository }}">GitHub</a>.</p></div>

## La documentation

<div class="guide-grid"><a href="/fr/guides/getting-started/"><strong>Installer et composer</strong><span>Brancher le kit, importer un composant et lancer une fenêtre.</span></a><a href="/fr/guides/controllers/"><strong>Garder un état typé</strong><span>Saisie, validation, sélection stable et générations.</span></a><a href="/fr/guides/themes/"><strong>Thèmes et accessibilité</strong><span>Couleurs sémantiques, tailles et vérification native.</span></a><a href="/fr/guides/native/"><strong>Intégration native</strong><span>Fichiers, capacités du système et résultats explicites.</span></a></div>

## Les composants

Chaque référence présente une capture native, un exemple Slint complet et l’API publique. Commencez par une famille :
{% else %}
<p class="eyebrow">Rust + Slint</p>

# Build with Slint Kit

Native components, typed controllers and shared themes for your Rust and Slint applications.

<div class="home-actions"><a class="primary-link" href="/guides/getting-started/">Get started <span aria-hidden="true">→</span></a><a href="/components/">Browse all {{ components.length }} components</a></div>

<div class="project-facts"><p><strong>Version {{ project.version }}</strong>{% if project.preOne %} · pre-1.0 series{% endif %} · <a href="https://crates.io/crates/slint-kit">Packages on crates.io</a></p><p>Rust {{ project.rust }}+ · Slint {{ project.slint }} · <a href="/guides/getting-started/">Prerequisites and installation</a></p><p>Independent project, code and assets under <a href="{{ site.repository }}/blob/main/LICENSE">GPL-3.0-or-later</a>. Maintenance and contributions on <a href="{{ site.repository }}">GitHub</a>.</p></div>

## The documentation

<div class="guide-grid"><a href="/guides/getting-started/"><strong>Install and compose</strong><span>Connect the kit, import a component and open a window.</span></a><a href="/guides/controllers/"><strong>Keep state typed</strong><span>Editing, validation, stable selection and generations.</span></a><a href="/guides/themes/"><strong>Themes and accessibility</strong><span>Semantic colors, sizes and native verification.</span></a><a href="/guides/native/"><strong>Native integration</strong><span>Files, host capabilities and explicit outcomes.</span></a></div>

## Components

Each reference includes a native screenshot, a complete Slint example and the public API. Start with a family:
{% endif %}

<div class="family-grid">{% for family, label in ui.families %}<a href="{{ 'components/' | localeUrl(locale) }}#{{ family }}">{{ label[locale] }} <span>{{ (components | whereFamily(family)).length }}</span></a>{% endfor %}</div>
