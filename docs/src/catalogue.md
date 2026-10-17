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

{% if locale == 'fr' %}Les {{ components.length }} composants publics du kit, regroupés par usage. Chaque page contient un exemple complet et sa capture native.{% else %}All {{ components.length }} public kit components, grouped by purpose. Each page includes a complete example and its native screenshot.{% endif %}

<div class="catalogue-search"><label for="component-filter">{{ ui[locale].search }}</label><input type="search" id="component-filter" placeholder="KitButton, slider…" autocomplete="off" aria-controls="component-catalogue"><p role="status" id="filter-status" data-empty="{{ ui[locale].noResults }}"></p></div>
<div id="component-catalogue">
{% for family, label in ui.families %}
<section class="catalogue-family" id="{{ family }}">
<h2>{{ label[locale] }}</h2>
<div class="catalogue-grid">
{% for component in components | whereFamily(family) %}<a class="catalogue-item" href="{{ ('components/' + component.slug + '/') | localeUrl(locale) }}" data-search="{{ component.name | lower }} {{ component[locale].summary | lower }}"><strong>{{ component.name }}</strong><span>{{ component[locale].summary }}</span></a>{% endfor %}
</div>
</section>
{% endfor %}
</div>
