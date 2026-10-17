---
layout: base.njk
pagination:
  data: locales
  size: 1
  alias: locale
permalink: "/{{ locale }}/guides/typed-example/index.html"
translationKey: "guides/typed-example/"
eleventyComputed:
  title: "{% if locale == 'fr' %}Application avec contrôleurs typés{% else %}Typed controller application{% endif %}"
---
{% if locale == 'fr' %}
# Application avec contrôleurs typés

Une application Rust complète avec un champ natif validé et une sélection par clé stable. Tous les signaux de ces deux contrôles sont raccordés à leurs contrôleurs.

Depuis la racine de ce dépôt, lancez :
{% else %}
# Typed controller application

A complete Rust application with a validated native field and a selection identified by stable keys. Every signal on these two controls is connected to its controller.

From this repository's root, run:
{% endif %}

```sh
cargo run -p kit-docs-integration{% if locale == "fr" %} -- --fr{% endif %}
```

<figure class="component-preview"><img src="/assets/components/typed-integration{% if locale == "fr" %}-fr{% endif %}.png" alt="{% if locale == 'fr' %}Fenêtre native Project settings avec un nom de projet et une sélection Workspace{% else %}Native Project settings window with a project name and Workspace selection{% endif %}" loading="eager"><figcaption>{{ ui[locale].snapshot }}</figcaption></figure>

{% if locale == 'fr' %}
## Essayer le comportement

1. Modifiez le nom et appuyez sur Entrée, cliquez sur « Accepter le nom » ou quittez le champ : le contrôleur accepte le brouillon valide.
2. Entrez plus de 40 graphèmes : la validation affiche un message et conserve le dernier nom accepté.
3. Ouvrez Espace de travail, utilisez les flèches puis Entrée, ou cliquez sur un choix. La ligne Archives désactivée est ignorée.
4. Fermez la liste avec Échap ; Tab revient au champ. Le nom choisi provient de `WorkspaceKey`, jamais d’un index de ligne.

L’exemple choisit `Ownership::Local`. Pour une persistance contrôlée, passez à `Ownership::Controlled` et accusez réception après la décision métier avec `acknowledge` pour le champ et `set_selected` pour la sélection. Gardez les callbacks de publication faibles et libérez tout emprunt d’état avant une mise à jour de fenêtre.

Les surfaces modales demandent aussi une pile de superpositions et un registre de focus. L’[exemple complet de navigation](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs) et sa [composition Slint](https://github.com/damione1/slint-kit/blob/main/showcase/ui/base-navigation.slint) montrent le tiroir modal avec restauration de l’ouvreur exact, trap de focus et cibles réelles enregistrées.

## Composition Slint

[Télécharger ui/app.slint](/integration/ui/app.slint).
{% else %}
## Try the behavior

1. Edit the name and press Enter, click “Accept name” or leave the field: the controller accepts a valid draft.
2. Enter more than 40 graphemes: validation displays an error and preserves the last accepted name.
3. Open Workspace, use arrow keys then Enter, or click an option. The disabled Archive row is skipped.
4. Close the list with Escape; Tab returns to the field. The accepted choice comes from `WorkspaceKey`, never a row index.

The example chooses `Ownership::Local`. For controlled persistence, use `Ownership::Controlled` and acknowledge the business decision with `acknowledge` for the field and `set_selected` for the selection. Keep publisher callbacks weak and release any state borrow before updating a window.

Modal surfaces also need an overlay stack and a focus registry. The [complete navigation example](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs) and its [Slint composition](https://github.com/damione1/slint-kit/blob/main/showcase/ui/base-navigation.slint) demonstrate a modal drawer with exact opener restoration, focus trapping and registered real targets.

## Slint composition

[Download ui/app.slint](/integration/ui/app.slint).
{% endif %}

```slint
{{ 'ui/app.slint' | integrationSource | safe }}
```

## {% if locale == 'fr' %}Contrôleurs Rust{% else %}Rust controllers{% endif %}

{% if locale == 'fr' %}Le parseur construit `ProjectName` ; le validateur compte les graphèmes. Le signal d’édition alimente le brouillon sans remplacer l’éditeur natif. `bind_select!` projette les options avec leur génération et raccorde tous les signaux de sélection. [Télécharger src/main.rs](/integration/src/main.rs).{% else %}The parser constructs `ProjectName`; the validator counts graphemes. The edit signal updates the draft while preserving the native editor. `bind_select!` projects options with their generation and wires every selection signal. [Download src/main.rs](/integration/src/main.rs).{% endif %}

```rust
{{ 'src/main.rs' | integrationSource | safe }}
```

{% if locale == 'fr' %}
## Construire et vérifier

Le paquet [Cargo.toml](/integration/Cargo.toml) utilise les dépendances du workspace. Son [build.rs](/integration/build.rs) compile l’interface via `kit_build::compile`. Pour l’utiliser dans un autre projet, adaptez les chemins comme dans [Bien démarrer](/fr/guides/getting-started/).

La commande suivante ouvre une vraie fenêtre winit/Skia et transmet des événements pointeur/clavier au champ et à la sélection. Elle vérifie édition, acceptation, conservation d’une valeur lors d’un échec, ouverture de liste et validation d’une clé stable. Le [module de vérification](/integration/src/verify.rs) reste séparé du comportement applicatif. Cette vérification ne qualifie pas IME ou lecteur d’écran.
{% else %}
## Build and verify

The [Cargo.toml](/integration/Cargo.toml) package uses workspace dependencies. Its [build.rs](/integration/build.rs) compiles the interface with `kit_build::compile`. For another project, adjust paths as shown in [Getting started](/en/guides/getting-started/).

The command below opens a real winit/Skia window and dispatches pointer/keyboard events to the field and selection. It checks editing, acceptance, preservation of an accepted value after failure, list opening and a stable keyed commit. The [verification module](/integration/src/verify.rs) remains separate from application behavior. This verification does not qualify IME or a screen reader.
{% endif %}

```sh
cargo run -p kit-docs-integration -- --verify{% if locale == "fr" %} --fr{% endif %}
```
