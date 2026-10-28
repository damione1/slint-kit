---
layout: base.njk
pagination:
  data: locales
  size: 1
  addAllPagesToCollections: true
  alias: locale
permalink: "{{ 'guides/table-example/' | localeUrl(locale) }}index.html"
translationKey: "guides/table-example/"
eleventyComputed:
  title: "{% if locale == 'fr' %}Tableau triable{% else %}Sortable data table{% endif %}"
---
{% if locale == 'fr' %}
# Tableau triable avec identité stable

Cette application Rust et Slint complète raccorde le tri des colonnes, la navigation clavier et la sélection d’un `KitDataTable`. Rust conserve les projets et leurs identifiants ; le tableau virtualise leur présentation. L’exemple utilise Slint Kit {{ project.version }}, Rust {{ project.rust }}+ et Slint {{ project.slint }}.

Choisissez **Atlas**, puis triez la colonne **Tâches** : Atlas reste sélectionné même s’il change de ligne. **Cedar** est archivé et ne peut pas être sélectionné. Un événement provenant d’une ancienne génération est rejeté.

## Essayer l’application

Créez un projet neuf, puis remplacez ses fichiers par les quatre fichiers complets ci-dessous. Les paquets viennent de crates.io ; aucun checkout du kit n’est nécessaire. Les [prérequis natifs](/fr/guides/getting-started/) sont les mêmes que pour le guide de démarrage.
{% else %}
# Sortable data table with stable identity

This complete Rust and Slint application connects column sorting, keyboard navigation and selection in a `KitDataTable`. Rust owns projects and their identities; the table virtualizes their presentation. The example uses Slint Kit {{ project.version }}, Rust {{ project.rust }}+ and Slint {{ project.slint }}.

Choose **Atlas**, then sort the **Tasks** column: Atlas stays selected even as it moves to another row. **Cedar** is archived and cannot be selected. An event from an old generation is rejected.

## Try the application

Create a fresh project, then replace its files with the four complete files below. Packages come from crates.io; no kit checkout is needed. The [native prerequisites](/guides/getting-started/) match the getting-started guide.
{% endif %}

```sh
cargo new slint-kit-table-example
cd slint-kit-table-example
```

{% set capture = tableCaptures[locale] %}
<figure class="component-preview"><img src="/assets/components/{{ capture.filename }}" srcset="/assets/components/{{ capture.retinaFilename }} 2x" alt="{% if locale == 'fr' %}Application native avec quatre projets, deux colonnes triables et une ligne archivée{% else %}Native application with four projects, two sortable columns and one archived row{% endif %}" width="{{ capture.width }}" height="{{ capture.height }}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>

{% if locale == 'fr' %}
1. Cliquez sur Atlas pour sélectionner le projet `#101`.
2. Cliquez sur Tâches : les valeurs sont triées numériquement, pas comme du texte. Un deuxième clic inverse le tri, un troisième rétablit l’ordre d’origine.
3. La sélection reste `#101` après le tri. Le jeton représente le projet, jamais sa position visuelle.
4. Donnez le focus au tableau, puis utilisez les flèches, Début, Fin et Entrée. Cedar est ignoré. Les touches de page déplacent le focus de cinq projets sélectionnables au maximum.

## Pourquoi la génération change

`ProjectId` est une clé métier stable. `ViewGeneration` identifie l’ordre projeté. Chaque tri incrémente cette génération ; `activate(token, generation)` vérifie la génération, l’existence du projet et son état avant d’accepter la sélection. La ligne active suit elle aussi une clé. `active-index` n’est qu’une projection destinée au focus et au défilement.

Le modèle `VecModel` est conservé entre les publications. Le tableau a un viewport borné et utilise la vue virtualisée du kit. Le contrôleur libère ses emprunts d’état avant de notifier le modèle ou de mettre à jour la fenêtre ; ses callbacks ne conservent qu’une référence faible vers celle-ci. Les quatre projets rendent le mécanisme lisible ; pour des données volumineuses, adaptez aussi le filtrage et le tri côté hôte.
{% else %}
1. Click Atlas to select project `#101`.
2. Click Tasks: values sort numerically, not as strings. A second click reverses sorting, and a third restores the original order.
3. Selection stays on `#101` after sorting. The token identifies the project, never its visual row position.
4. Focus the table, then use arrows, Home, End and Enter. Cedar is skipped. Page keys move focus by up to five enabled projects.

## Why the generation changes

`ProjectId` is a stable domain key. `ViewGeneration` identifies the projected order. Every sort increments that generation; `activate(token, generation)` checks the generation, project existence and availability before accepting selection. The active row also follows a key. `active-index` is only a projection used for focus and scrolling.

The `VecModel` remains alive across publications. The table has a bounded viewport and uses the kit's virtualized view. The controller releases state borrows before model notifications or window updates; callbacks keep only weak window references. Four projects make the mechanism readable; for large datasets, also adapt host-side filtering and sorting.
{% endif %}

## Cargo.toml

[{{ ui[locale].download }} — Cargo.toml](/table-consumer/Cargo.toml)

```toml
{{ 'Cargo.toml' | tableSource | safe }}
```

## build.rs

[{{ ui[locale].download }} — build.rs](/table-consumer/build.rs)

```rust
{{ 'build.rs' | tableSource | safe }}
```

## ui/table.slint

[{{ ui[locale].download }} — ui/table.slint](/table-consumer/ui/table.slint)

```slint
{{ 'ui/table.slint' | tableSource | safe }}
```

## src/main.rs

[{{ ui[locale].download }} — src/main.rs](/table-consumer/src/main.rs)

```rust
{{ 'src/main.rs' | tableSource | safe }}
```

## {% if locale == 'fr' %}Exécuter et adapter{% else %}Run and adapt{% endif %}

```sh
cargo run{% if locale == 'fr' %} -- --fr{% endif %}
```

{% if locale == 'fr' %}
La comparaison de noms utilise l’ordre des chaînes Rust ; elle n’implémente pas une collation linguistique. Cedar garde son nom dans les deux langues. Les libellés accessibles internes de tri du composant suivent la version du kit utilisée. Cette fenêtre winit/Skia et ses captures ne qualifient pas un lecteur d’écran ou toutes les plateformes.

Consultez la [référence KitDataTable](/fr/components/data-table/) pour l’API complète et le [guide des contrôleurs](/fr/guides/controllers/) pour les autres modèles d’état. L’[application de champ validé et sélection](/fr/guides/typed-example/) complète cet exemple.
{% else %}
Name comparison uses Rust string ordering; it does not implement linguistic collation. Cedar keeps its name in both languages. Internal accessible sort labels follow the kit version in use. This winit/Skia window and its screenshots do not qualify a screen reader or every platform.

See the [KitDataTable reference](/components/data-table/) for the full API and the [controller guide](/guides/controllers/) for other state models. The [validated field and selection application](/guides/typed-example/) complements this example.
{% endif %}
