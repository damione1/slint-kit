---
layout: base.njk
pagination:
  data: locales
  size: 1
  addAllPagesToCollections: true
  alias: locale
permalink: "{{ 'guides/typed-example/' | localeUrl(locale) }}index.html"
translationKey: "guides/typed-example/"
eleventyComputed:
  title: "{% if locale == 'fr' %}Application avec contrôleurs typés{% else %}Typed controller application{% endif %}"
---
{% if locale == 'fr' %}
# Application avec contrôleurs typés

Une application Rust complète avec un champ natif validé et une sélection par clé stable. Tous les signaux de ces deux contrôles sont raccordés à leurs contrôleurs.

Créez une application avec les paquets publiés sur crates.io. Les quatre fichiers complets ci-dessous s’utilisent dans un projet neuf, sans copier le dépôt du kit :
{% else %}
# Typed controller application

A complete Rust application with a validated native field and a selection identified by stable keys. Every signal on these two controls is connected to its controller.

Create an application using the packages published on crates.io. The four complete files below work in a fresh project without copying the kit repository:
{% endif %}

```sh
cargo new slint-kit-typed-example
cd slint-kit-typed-example
```

<figure class="component-preview"><img src="/assets/components/typed-integration{% if locale == "fr" %}-fr{% endif %}.png" alt="{% if locale == 'fr' %}Fenêtre native Project settings avec un nom de projet et une sélection Workspace{% else %}Native Project settings window with a project name and Workspace selection{% endif %}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>

{% if locale == 'fr' %}
## Cargo.toml

Remplacez le manifeste créé par Cargo. [Télécharger Cargo.toml](/consumer/Cargo.toml).
{% else %}
## Cargo.toml

Replace the manifest created by Cargo. [Download Cargo.toml](/consumer/Cargo.toml).
{% endif %}

```toml
{{ 'Cargo.toml' | consumerSource | safe }}
```

## build.rs

{% if locale == 'fr' %}Ce script compile l’interface et enregistre les imports publics `@kit`. [Télécharger build.rs](/consumer/build.rs).{% else %}This script compiles the interface and registers the public `@kit` imports. [Download build.rs](/consumer/build.rs).{% endif %}

```rust
{{ 'build.rs' | consumerSource | safe }}
```

{% if locale == 'fr' %}
## Essayer le comportement

1. Modifiez le nom et appuyez sur Entrée, cliquez sur « Accepter le nom » ou quittez le champ : le contrôleur accepte le brouillon valide.
2. Entrez plus de 40 graphèmes : la validation affiche un message et conserve le dernier nom accepté.
3. Ouvrez Espace de travail, utilisez les flèches puis Entrée, ou cliquez sur un choix. La ligne Archives désactivée est ignorée.
4. Fermez la liste avec Échap ; Tab revient au champ. Le nom choisi provient de `WorkspaceKey`, jamais d’un index de ligne.

L’exemple choisit `Ownership::Local`. Pour une persistance contrôlée, passez à `Ownership::Controlled` et accusez réception après la décision métier avec `acknowledge` pour le champ et `set_selected` pour la sélection. Gardez les callbacks de publication faibles et libérez tout emprunt d’état avant une mise à jour de fenêtre.

Les surfaces modales demandent aussi une pile de superpositions et un registre de focus. L’[exemple complet de navigation](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs) et sa [composition Slint](https://github.com/damione1/slint-kit/blob/main/showcase/ui/base-navigation.slint) montrent le tiroir modal avec restauration de l’ouvreur exact, trap de focus et cibles réelles enregistrées.

## Composition Slint

[Télécharger ui/app.slint](/consumer/ui/app.slint).
{% else %}
## Try the behavior

1. Edit the name and press Enter, click “Accept name” or leave the field: the controller accepts a valid draft.
2. Enter more than 40 graphemes: validation displays an error and preserves the last accepted name.
3. Open Workspace, use arrow keys then Enter, or click an option. The disabled Archive row is skipped.
4. Close the list with Escape; Tab returns to the field. The accepted choice comes from `WorkspaceKey`, never a row index.

The example chooses `Ownership::Local`. For controlled persistence, use `Ownership::Controlled` and acknowledge the business decision with `acknowledge` for the field and `set_selected` for the selection. Keep publisher callbacks weak and release any state borrow before updating a window.

Modal surfaces also need an overlay stack and a focus registry. The [complete navigation example](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs) and its [Slint composition](https://github.com/damione1/slint-kit/blob/main/showcase/ui/base-navigation.slint) demonstrate a modal drawer with exact opener restoration, focus trapping and registered real targets.

## Slint composition

[Download ui/app.slint](/consumer/ui/app.slint).
{% endif %}

```slint
{{ 'ui/app.slint' | consumerSource | safe }}
```

## {% if locale == 'fr' %}Contrôleurs Rust{% else %}Rust controllers{% endif %}

{% if locale == 'fr' %}Le parseur construit `ProjectName` ; le validateur compte les graphèmes. Le signal d’édition alimente le brouillon sans remplacer l’éditeur natif. `bind_select!` projette les options avec leur génération et raccorde tous les signaux de sélection. [Télécharger src/main.rs](/consumer/src/main.rs).{% else %}The parser constructs `ProjectName`; the validator counts graphemes. The edit signal updates the draft while preserving the native editor. `bind_select!` projects options with their generation and wires every selection signal. [Download src/main.rs](/consumer/src/main.rs).{% endif %}

```rust
{{ 'src/main.rs' | consumerSource | safe }}
```

{% if locale == 'fr' %}
## Exécuter l’application

Enregistrez les quatre fichiers aux emplacements indiqués dans le projet créé au début de ce guide. Cargo installe `slint-kit` et `slint-kit-build` depuis crates.io ; aucun chemin local vers le kit n’est nécessaire. Lancez la version française :
{% else %}
## Run the application

Save the four files at the shown paths in the project created at the start of this guide. Cargo installs `slint-kit` and `slint-kit-build` from crates.io; no local path to the kit is needed. Run the English version:
{% endif %}

```sh
cargo run{% if locale == "fr" %} -- --fr{% endif %}
```

{% if locale == 'fr' %}Cet exemple utilise une fenêtre native winit/Skia. Les captures présentent cette même interface ; elles ne qualifient pas IME ou lecteur d’écran.{% else %}This example uses a native winit/Skia window. The screenshots show this same interface; they do not qualify IME or a screen reader.{% endif %}
