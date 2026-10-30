---
layout: base.njk
pagination:
  data: locales
  size: 1
  addAllPagesToCollections: true
  alias: locale
permalink: "{{ 'components/' | localeUrl(locale) }}index.html"
translationKey: "components/"
eleventyComputed:
  title: "{% if locale == 'fr' %}Composants{% else %}Components{% endif %}"
---
# {{ ui[locale].components }}

{% if locale == 'fr' %}Trouvez un composant par usage, consultez sa capture et ses branchements. Pour un premier essai fonctionnel : [bouton](/fr/components/button/) ou [liste avec recherche](/fr/components/combobox/).{% else %}Find a component by purpose, inspect its screenshot and wiring. For a first working example, try a [button](/components/button/) or a [searchable combobox](/components/combobox/).{% endif %}

{% if locale == 'fr' %}Les champs prêts à l’emploi intègrent leurs options de libellé et de message. Les [briques de formulaire](#field) permettent une composition personnalisée ; leurs exemples indiquent quels contrôles offrent déjà l’option correspondante.{% else %}Ready-to-use fields include their label and message options. [Form primitives](#field) support custom composition; their examples list the controls that already provide the corresponding option.{% endif %}

<div class="catalogue-search"><label for="component-filter">{{ ui[locale].search }}</label><input type="search" id="component-filter" placeholder="KitButton, slider…" autocomplete="off" aria-controls="component-catalogue"><p role="status" id="filter-status" data-empty="{{ ui[locale].noResults }}"></p></div>
<div id="component-catalogue">
{% for family, label in ui.families %}
<section class="catalogue-family" id="{{ family }}">
<div class="catalogue-family-heading"><h2>{{ label[locale] }}</h2></div>
<div class="catalogue-grid">
{% for component in components | whereFamily(family) %}<a class="catalogue-item" href="{{ ('components/' + component.slug + '/') | localeUrl(locale) }}" data-search="{{ component.name | lower }} {{ component | componentLabel(locale) | lower }} {{ component[locale].summary | lower }}"><canvas class="component-mockup" data-component-mockup="{{ component.slug }}" width="640" height="336" aria-hidden="true"></canvas><strong>{{ component | componentLabel(locale) }}</strong><code>{{ component.name }}</code><span>{{ component[locale].summary }}</span></a>{% endfor %}
</div>
</section>
{% endfor %}
</div>
<script type="module" src="{{ '/assets/component-mockups.js' | versionedAsset }}"></script>
