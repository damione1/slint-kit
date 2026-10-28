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

<div class="catalogue-search"><label for="component-filter">{{ ui[locale].search }}</label><input type="search" id="component-filter" placeholder="KitButton, slider…" autocomplete="off" aria-controls="component-catalogue"><p role="status" id="filter-status" data-empty="{{ ui[locale].noResults }}"></p></div>
<div id="component-catalogue">
{% for family, label in ui.families %}
<section class="catalogue-family" id="{{ family }}">
<h2>{{ label[locale] }}</h2>
<div class="catalogue-grid">
{% for component in components | whereFamily(family) %}<a class="catalogue-item" href="{{ ('components/' + component.slug + '/') | localeUrl(locale) }}" data-search="{{ component.name | lower }} {{ component | componentLabel(locale) | lower }} {{ component[locale].summary | lower }}"><strong>{{ component | componentLabel(locale) }}</strong><code>{{ component.name }}</code><span>{{ component[locale].summary }}</span></a>{% endfor %}
</div>
</section>
{% endfor %}
</div>
