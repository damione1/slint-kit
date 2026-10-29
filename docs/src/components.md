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

<p class="component-identifier"><code>{{ component.name }}</code> · Slint Kit 0.3.1</p>

{{ text.summary }}

{% if component.sharedOption %}
<p class="component-kind">{% if locale == 'fr' %}Composant de composition · option commune <code>{{ component.sharedOption }}</code>{% else %}Composition primitive · shared <code>{{ component.sharedOption }}</code> option{% endif %}</p>
{% endif %}

{% set quickstart = quickstarts[component.slug] %}
{% if quickstart %}
<section class="live-preview" data-component="{{ component.slug }}" data-width="{{ quickstart.displayWidth }}" data-height="{{ quickstart.height / (quickstart.width / quickstart.displayWidth) }}" aria-label="{% if locale == 'fr' %}Aperçu interactif{% else %}Interactive preview{% endif %}">
<div class="preview-actions"><button type="button" class="run-preview">{% if locale == 'fr' %}Lancer l’aperçu{% else %}Run preview{% endif %}</button><span class="preview-status" role="status"></span></div>
<figure class="component-preview"><img src="{{ component.slug | starterPoster }}" alt="{{ text.summary }}" width="{{ quickstart.width }}" height="{{ quickstart.height }}" style="width:{{ quickstart.displayWidth }}px" loading="eager"><figcaption>{% if locale == 'fr' %}Application native de l’exemple ci-dessous.{% else %}Native app built from the example below.{% endif %}</figcaption></figure>
<div class="preview-frame" hidden></div>
<noscript><p>{% if locale == 'fr' %}Activez JavaScript pour utiliser l’aperçu interactif.{% else %}Enable JavaScript to run the interactive preview.{% endif %}</p></noscript>
{% if quickstart.native_only %}<p class="preview-note">{% if locale == 'fr' %}Les dialogues de fichiers et le dépôt depuis le système nécessitent l’application native ci-dessous.{% else %}File dialogs and OS drag-and-drop require the native application below.{% endif %}</p>{% endif %}
</section>
{% else %}
<figure class="component-preview"><img src="/assets/components/{{ screenshot.filename }}" srcset="/assets/components/{{ screenshot.retinaFilename }} 2x" alt="{{ text.summary }}" width="{{ screenshot.width }}" height="{{ screenshot.height }}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>
{% endif %}

{% if quickstarts[component.slug] %}
{% include "component-quickstart.njk" %}
{% endif %}

{{ text.usage | prose | safe }}

{{ text.behavior | prose | safe }}

{% if component.name == "KitTelephoneField" %}{% include "telephone-options.njk" %}{% endif %}

{% if component.name == 'KitDataTable' %}{% if locale == 'fr' %}Pour relier réellement le tri, les touches et la sélection, utilisez l’[application complète de tableau triable](/fr/guides/table-example/).{% else %}To connect sorting, keyboard input and selection, use the [complete sortable table application](/guides/table-example/).{% endif %}{% endif %}

<p class="source-link"><a href="https://docs.rs/crate/slint-kit-slint/{{ project.version }}/source/{{ component.source | replace("crates/kit-slint/", "") }}#{{ component.line }}">{{ ui[locale].source }} ↗</a></p>

{% if component.optionConsumers.length %}
## {% if locale == 'fr' %}Option commune{% else %}Shared option{% endif %} `{{ component.sharedOption }}`

{% if locale == 'fr' %}La démonstration utilise la brique `{{ component.name }}` séparément. Les contrôles ci-dessous exposent directement une propriété `{{ component.sharedOption }}` : utilisez-la pour leur comportement intégré. Le rendu et les détails de focus dépendent du contrôle choisi.{% else %}This demonstration uses `{{ component.name }}` separately. The controls below expose a built-in `{{ component.sharedOption }}` property: use that property for their integrated behavior. Rendering and focus details depend on the selected control.{% endif %}

<details class="option-consumers"><summary>{% if locale == 'fr' %}Les {{ component.optionConsumers.length }} composants exposant cette propriété{% else %}The {{ component.optionConsumers.length }} components exposing this property{% endif %}</summary>

{% for consumer in component.optionConsumers %}- [{{ consumer.name }}]({{ ('components/' + consumer.slug + '/') | localeUrl(locale) }})
{% endfor %}

</details>
{% endif %}

{% if component.optionDemos.length %}
<p><a href="{{ 'guides/forms/' | localeUrl(locale) }}">{% if locale == 'fr' %}Choisir entre une option intégrée et une composition personnalisée{% else %}Choose between built-in options and custom composition{% endif %}</a></p>

### {% if locale == 'fr' %}Options partagées{% else %}Shared options{% endif %}

{% for demo in component.optionDemos %}- [`{{ demo.sharedOption }}`]({{ ('components/' + demo.slug + '/') | localeUrl(locale) }}) — {% if locale == 'fr' %}propriété intégrée ; exemple de composition personnalisé{% else %}built-in property; custom composition example{% endif %}
{% endfor %}
{% endif %}

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
{% if component.inheritedProperties.length %}
<details class="inherited-api"><summary>{% if locale == 'fr' %}Propriétés héritées disponibles{% else %}Available inherited properties{% endif %}</summary>

| {{ ui[locale].name }} | {{ ui[locale].type }} | {% if locale == 'fr' %}Définie sur{% else %}Defined on{% endif %} |
| --- | --- | --- |
{% for property in component.inheritedProperties %}| `{{ property.name }}` | `{{ property.type | mdEscape }}` | `{{ property.owner }}` |
{% endfor %}

</details>
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

{% for name in text.related %}{% set related = name | componentByName %}{% if related %}- [{{ related | componentLabel(locale) }}]({{ ('components/' + related.slug + '/') | localeUrl(locale) }})
{% endif %}{% endfor %}
{% endif %}
