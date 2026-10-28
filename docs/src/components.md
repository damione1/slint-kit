---
layout: base.njk
pagination:
  data: reference
  size: 1
  addAllPagesToCollections: true
  alias: entry
permalink: "{{ ('components/' + entry.component.slug + '/') | localeUrl(entry.locale) }}index.html"
eleventyComputed:
  title: "{{ entry.component.name }}"
  locale: "{{ entry.locale }}"
  translationKey: "components/{{ entry.component.slug }}/"
---
{% set component = entry.component %}
{% set text = component[locale] %}
{% set screenshot = component.screenshots[locale] %}
<p class="eyebrow"><a href="{{ 'components/' | localeUrl(locale) }}#{{ component.family }}">{{ component.familyLabel[locale] }}</a></p>

# {{ component | componentLabel(locale) }} {% if locale == 'fr' %}pour Slint{% else %}for Slint{% endif %}

<p class="component-identifier"><code>{{ component.name }}</code> · Slint Kit 0.2.0</p>

{{ text.summary }}

{% set quickstart = quickstarts[component.slug] %}
{% if quickstart %}
<figure class="component-preview"><img src="/quickstart/{{ component.slug }}/preview@2x.png" alt="{{ text.summary }}" width="{{ quickstart.width }}" height="{{ quickstart.height }}" style="width:{{ quickstart.displayWidth }}px" loading="eager"><figcaption>{% if component.slug == 'button' %}{% if locale == 'fr' %}Application native après trois clics sur le bouton.{% else %}Native app after three button clicks.{% endif %}{% else %}{% if locale == 'fr' %}La recherche « sha » ne conserve que Shared dans la liste.{% else %}Searching for “sha” leaves only Shared in the list.{% endif %}{% endif %}</figcaption></figure>
{% else %}
<figure class="component-preview"><img src="/assets/components/{{ screenshot.filename }}" srcset="/assets/components/{{ screenshot.retinaFilename }} 2x" alt="{{ text.summary }}" width="{{ screenshot.width }}" height="{{ screenshot.height }}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>
{% endif %}

{% if quickstarts[component.slug] %}
{% include "component-quickstart.njk" %}
{% endif %}

{{ text.usage | prose | safe }}

{{ text.behavior | prose | safe }}

{% if component.name == 'KitDataTable' %}{% if locale == 'fr' %}Pour relier réellement le tri, les touches et la sélection, utilisez l’[application complète de tableau triable](/fr/guides/table-example/).{% else %}To connect sorting, keyboard input and selection, use the [complete sortable table application](/guides/table-example/).{% endif %}{% endif %}

<p class="source-link"><a href="{{ site.repository }}/blob/{{ site.sourceRef }}/{{ component.source }}#L{{ component.line }}">{{ ui[locale].source }} ↗</a></p>

## {% if locale == 'fr' %}Composition visuelle{% else %}Visual composition{% endif %}

{% if locale == 'fr' %}Le fichier ci-dessous produit la capture. Pour les composants pilotés par un contrôleur, il montre l’état affiché sans fournir le branchement Rust. [Créer un projet](/fr/guides/getting-started/).{% else %}The file below produces the screenshot. For controller-driven components, it shows the displayed state without supplying the Rust wiring. [Create a project](/guides/getting-started/).{% endif %}

<details class="visual-example">
<summary>{% if locale == 'fr' %}Afficher le fichier de la capture{% else %}Show the screenshot source{% endif %}</summary>

{% if quickstart %}
<figure class="component-preview"><img src="/assets/components/{{ screenshot.filename }}" srcset="/assets/components/{{ screenshot.retinaFilename }} 2x" alt="{{ text.summary }}" width="{{ screenshot.width }}" height="{{ screenshot.height }}" loading="lazy"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>
{% endif %}

```slint
{{ component.slug | example(locale) | safe }}
```

<a class="download" href="/examples/{% if locale == "fr" %}fr/{% endif %}{{ component.slug }}.slint" download>{{ ui[locale].download }}</a>

</details>

## {{ ui[locale].api }}

{% set parent = component.base | componentByName %}
{% if parent %}{{ ui[locale].inherited }} [{{ parent.name }}]({{ ('components/' + parent.slug + '/') | localeUrl(locale) }}).{% else %}{{ ui[locale].inherited }} `{{ component.base }}`.{% endif %}
{% if component.properties.length %}
### {{ ui[locale].properties }}

| {{ ui[locale].name }} | {{ ui[locale].type }} | {{ ui[locale].direction }} |
| --- | --- | --- |
{% for property in component.properties %}| `{{ property.name }}` | `{{ property.type | mdEscape }}` | `{{ property.direction }}` |
{% endfor %}
{% endif %}
{% if component.types.length %}
### {% if locale == 'fr' %}Types et variantes{% else %}Types and variants{% endif %}

| {{ ui[locale].type }} | {% if locale == 'fr' %}Déclaration{% else %}Declaration{% endif %} |
| --- | --- |
{% for type in component.types %}| `{{ type.kind }} {{ type.name }}` | `{{ type.definition | mdEscape }}` |
{% endfor %}
{% endif %}
{% if component.callbacks.length %}
### {{ ui[locale].callbacks }}

{% for callback in component.callbacks %}- `{{ callback.signature }}`
{% endfor %}
{% endif %}
{% if component.functions.length %}
### {{ ui[locale].methods }}

{% for method in component.functions %}- `{{ method.signature }}`
{% endfor %}
{% endif %}

{% if text.related.length %}
## {% if locale == 'fr' %}Voir aussi{% else %}See also{% endif %}

{% for name in text.related %}{% set related = name | componentByName %}{% if related %}- [{{ related.name }}]({{ ('components/' + related.slug + '/') | localeUrl(locale) }})
{% endif %}{% endfor %}
{% endif %}
