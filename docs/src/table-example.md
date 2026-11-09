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

Cette application Rust et Slint complète raccorde le tri des colonnes, un filtre, la navigation clavier et la sélection d’un `KitDataTable` grâce à `KeyedModel<K, Row>`. Rust conserve les projets et leurs clés ; le modèle attribue les jetons, avance les générations et publie les lignes ; le tableau virtualise leur présentation. L’exemple utilise Slint Kit {{ project.version }}, Rust {{ project.rust }}+ et Slint {{ project.slint }}.

Choisissez **Atlas**, puis triez la colonne **Tâches** : Atlas reste sélectionné même s’il change de ligne. Masquez les projets archivés : Atlas reste sélectionné. **Cedar** est archivé et ne peut pas être sélectionné. Un événement provenant d’une ancienne génération est rejeté.

Cet exemple détaille explicitement les clés, générations et publications du modèle pour montrer leur fonctionnement. Pour réutiliser ces mécanismes dans votre application, `KeyedModel<K, Row>` fournit `sorting_by`, `navigation` et `activation` : consultez le [guide des modèles typés](/fr/guides/controllers/). `KitDataTable` affiche des cellules texte ; une grille éditable demande une composition supplémentaire.

## Essayer l’application

Créez un projet neuf, puis remplacez ses fichiers par les quatre fichiers complets ci-dessous. Les paquets viennent de crates.io ; aucun checkout du kit n’est nécessaire. Les [prérequis natifs](/fr/guides/getting-started/) sont les mêmes que pour le guide de démarrage.
{% else %}
# Sortable data table with stable identity

This complete Rust and Slint application connects column sorting, a filter, keyboard navigation and selection in a `KitDataTable` through `KeyedModel<K, Row>`. Rust owns projects and their keys; the model assigns tokens, advances generations and publishes rows; the table virtualizes their presentation. The example uses Slint Kit {{ project.version }}, Rust {{ project.rust }}+ and Slint {{ project.slint }}.

Choose **Atlas**, then sort the **Tasks** column: Atlas stays selected even as it moves to another row. Hide archived projects: Atlas stays selected. **Cedar** is archived and cannot be selected. An event from an old generation is rejected.

This example spells out keys, generations and model publication to show how they work. To reuse those mechanisms in your application, `KeyedModel<K, Row>` supplies `sorting_by`, `navigation` and `activation`: see the [typed model guide](/guides/controllers/). `KitDataTable` displays text cells; an editable grid requires additional composition.

## Try the application

Create a fresh project, then replace its files with the four complete files below. Packages come from crates.io; no kit checkout is needed. The [native prerequisites](/guides/getting-started/) match the getting-started guide.
{% endif %}

```sh
cargo new slint-kit-table-example
cd slint-kit-table-example
```

{% set capture = tableCaptures[locale] %}
<figure class="component-preview"><img src="/assets/components/{{ capture.filename }}" srcset="/assets/components/{{ capture.retinaFilename }} 2x" alt="{% if locale == 'fr' %}Application native avec quatre projets, deux colonnes triables, une ligne archivée et un interrupteur qui affiche les projets archivés{% else %}Native application with four projects, two sortable columns, one archived row and a switch that shows archived projects{% endif %}" width="{{ capture.width }}" height="{{ capture.height }}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>

{% if locale == 'fr' %}
1. Cliquez sur Atlas pour sélectionner le projet `#101`.
2. Cliquez sur Tâches : les valeurs sont triées numériquement, pas comme du texte. Un deuxième clic inverse l’ordre. La sélection reste `#101` et la ligne active la suit : le jeton représente le projet, jamais sa position visuelle.
3. Désactivez **Afficher les projets archivés** : Cedar disparaît, l’ordre et la sélection restent. Réactivez-le : Cedar revient à sa place.
4. Donnez le focus au tableau, puis utilisez les flèches, Début, Fin et Entrée. Cedar est ignoré. Les touches de page déplacent le focus de dix projets sélectionnables au maximum.

## Le chemin court

`KeyedModel<ProjectId, TableRow>` reçoit des paires `(clé, ligne)` et fait le reste :

- `bind(&app, set_rows, set_generation, set_active_index)` publie les lignes, la génération et l’index actif vers la fenêtre, à travers une référence faible.
- `navigation()` est le handler de `navigate` : parcours au clavier qui ignore les lignes désactivées.
- `activation(handler)` est le handler de `activate(token, generation)` : il rejette une génération périmée ou une ligne désactivée, marque la ligne sélectionnée, puis appelle le métier avec la `ProjectId`.
- `sorting_by(compare, changed)` est l’action d’un en-tête triable : ordre croissant puis décroissant, clés, sélection et ligne active conservées, génération avancée ; `changed` reçoit la direction à afficher sur l’en-tête.
- `replace(rows)` sert au filtre : les clés qui survivent gardent leur jeton, leur sélection et leur position active ; la génération avance.

L’application ne fournit que ses données, son comparateur, la clé sélectionnée, l’ordre affiché et ses libellés dans les deux langues. Les cellules viennent de `CellContent::text` et `table_cells` ; la [référence KitDataTable](/fr/components/data-table/) décrit les badges, les cases à cocher, les cellules éditables et les largeurs de colonne.

## Lecture avancée : ce que le modèle garantit

`ProjectId` est une clé métier stable. Le jeton `int` d’une ligne est une projection attribuée par le modèle ; un jeton retiré n’est jamais recyclé. La génération identifie une projection publiée : chaque tri ou `replace` l’avance, et `accept(token, generation)` rejette tout geste portant une génération antérieure, puis vérifie que la ligne existe encore et est active. `active-index` n’est qu’une projection destinée au focus et au défilement ; l’identité active est une clé.

Le `VecModel` derrière `model()` est conservé entre les publications. Le tableau a un viewport borné et utilise la vue virtualisée du kit. Le modèle libère son emprunt d’état avant de notifier le modèle ou la fenêtre ; les liaisons ne gardent qu’une référence faible vers celle-ci, et fermer la fenêtre ne retient rien. Les callbacks de l’exemple capturent le modèle partagé et le catalogue, jamais la fenêtre ; seul le handler de sélection remonte une référence faible pour mettre à jour le libellé.

Quatre projets rendent le mécanisme lisible. Pour des données volumineuses, filtrez et triez côté hôte avant `replace`, et gardez le comparateur peu coûteux : le modèle le rappelle à chaque tri.
{% else %}
1. Click Atlas to select project `#101`.
2. Click Tasks: values sort numerically, not as strings. A second click reverses the order. Selection stays on `#101` and the active row follows it: the token identifies the project, never its visual row position.
3. Turn off **Show archived projects**: Cedar disappears, the order and the selection stay. Turn it on again: Cedar returns to its place.
4. Focus the table, then use arrows, Home, End and Enter. Cedar is skipped. Page keys move focus by up to ten enabled projects.

## The short path

`KeyedModel<ProjectId, TableRow>` takes `(key, row)` pairs and does the rest:

- `bind(&app, set_rows, set_generation, set_active_index)` publishes rows, generation and active index to the window through a weak reference.
- `navigation()` is the `navigate` handler: keyboard traversal that skips disabled rows.
- `activation(handler)` is the `activate(token, generation)` handler: it rejects a stale generation or a disabled row, marks the selected row, then calls the business handler with the `ProjectId`.
- `sorting_by(compare, changed)` is a sortable header's action: ascending then descending order, keys, selection and active row preserved, generation advanced; `changed` receives the direction to show on the header.
- `replace(rows)` serves the filter: surviving keys keep their token, selection and active position; the generation advances.

The application supplies only its data, its comparator, the selected key, the displayed order and its labels in both languages. Cells come from `CellContent::text` and `table_cells`; the [KitDataTable reference](/components/data-table/) covers badges, check marks, editable cells and column widths.

## Advanced reading: what the model guarantees

`ProjectId` is a stable domain key. A row's `int` token is a projection assigned by the model; a removed token is never recycled. The generation identifies one published projection: every sort or `replace` advances it, and `accept(token, generation)` rejects any gesture carrying an earlier generation, then checks that the row still exists and is enabled. `active-index` is only a projection used for focus and scrolling; the active identity is a key.

The `VecModel` behind `model()` remains alive across publications. The table has a bounded viewport and uses the kit's virtualized view. The model releases its state borrow before notifying the model or the window; bindings keep only a weak window reference, and closing the window retains nothing. The example's callbacks capture the shared model and the catalogue, never the window; only the selection handler upgrades a weak reference to update the label.

Four projects make the mechanism readable. For large datasets, filter and sort on the host side before `replace`, and keep the comparator cheap: the model calls it on every sort.
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
La comparaison de noms utilise l’ordre des chaînes Rust ; elle n’implémente pas une collation linguistique. Cedar garde son nom dans les deux langues. Les libellés accessibles de tri (`sort-ascending-label`, `sort-descending-label`, `unnamed-row-label`), l’interrupteur et le libellé de sélection sont fournis par l’application dans chaque langue ; le rendu et l’interaction sont les mêmes en français et en anglais. Cette fenêtre winit/Skia et ses captures ne qualifient pas un lecteur d’écran ou toutes les plateformes.

Consultez la [référence KitDataTable](/fr/components/data-table/) pour l’API complète et le [guide des contrôleurs](/fr/guides/controllers/) pour `KeyedModel` et les autres modèles d’état. L’[application de champ validé et sélection](/fr/guides/typed-example/) complète cet exemple.
{% else %}
Name comparison uses Rust string ordering; it does not implement linguistic collation. Cedar keeps its name in both languages. The accessible sort labels (`sort-ascending-label`, `sort-descending-label`, `unnamed-row-label`), the switch and the selection label are supplied by the application in each language; rendering and interaction are the same in French and English. This winit/Skia window and its screenshots do not qualify a screen reader or every platform.

See the [KitDataTable reference](/components/data-table/) for the full API and the [controller guide](/guides/controllers/) for `KeyedModel` and other state models. The [validated field and selection application](/guides/typed-example/) complements this example.
{% endif %}
