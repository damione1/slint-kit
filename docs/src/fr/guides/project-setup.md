---
layout: base.njk
locale: fr
title: "Configurer le projet"
translationKey: "guides/project-setup/"
permalink: "{{ 'guides/project-setup/' | localeUrl(locale) }}index.html"
---

# Configurer le projet

Configurez l’application Rust une seule fois, puis utilisez chaque composant avec un import `@kit`, des propriétés et des callbacks typés. Les fiches fournissent `ui/app.slint` et `src/main.rs` ; les projets téléchargeables incluent aussi le manifeste, le script de compilation et le fichier de verrouillage.

## Dépendances

Utilisez Rust **{{ project.rust }}+**, l’édition **2024** et Slint **{{ project.slint }}**. Lancez `cargo new my-app` et ajoutez les paquets d’exécution et de compilation dans `Cargo.toml` :

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "{{ project.rust }}"

[dependencies]
slint-kit = "={{ project.version }}"
slint = { version = "={{ project.slint }}", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "={{ project.version }}"
```

Gardez les paquets d’exécution et de compilation du kit sur des versions correspondantes. Conservez le backend et le moteur de rendu compatibles d’une application existante. Cet exemple de bureau utilise winit et Skia ; consultez [Bien démarrer](/fr/guides/getting-started/) pour les prérequis système et l’[intégration native](/fr/guides/native/) pour les services hôtes optionnels.

Ces exemples ciblent Slint Kit **{{ project.version }}**. Pour développer depuis les sources, remplacez les deux dépendances du kit ci-dessus par des chemins locaux. Pour une application voisine d’un checkout `slint-kit` :

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Conservez la dépendance Slint du premier manifeste. Les téléchargements ciblent la version correspondante du registre ; les chemins locaux permettent de développer depuis les sources. Gardez le nom Cargo `slint-kit` : le code de modules généré par Slint référence `slint_kit::ui`.

## Compiler l’interface

Créez `build.rs` à la racine du projet :

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Le paquet de compilation enregistre les types Slint partagés du kit et vérifie leurs contrats. Utilisez l’import public racine `@kit` pour les composants, enums et structs. Les fichiers de polices et d’icônes supplémentaires gardent leurs imports documentés. Configurez l’[éditeur](/fr/guides/editor/) séparément pour que son serveur de langage résolve également ces imports.

Sous Windows, utilisez Slint Kit 0.7.1 ou une version ultérieure : la 0.7.0 échappait incorrectement les chemins canonicalisés dans l’import généré. `compile` et `compile_with_config` acceptent les chemins relatifs, absolus et canonicalisés, avec espaces et Unicode. Pour personnaliser la compilation, transmettez une configuration issue de `slint_kit_build::config()` ou de `config_with_libraries(...)` à `compile_with_config` ; les chemins d’inclusion et les alias de bibliothèques de l’application sont conservés.

## Importer, définir les propriétés, recevoir les valeurs

Créez `ui/app.slint` :

```slint
import { KitButton } from "@kit";

export component App inherits Window {
    callback saved();
    KitButton { text: "Enregistrer"; activated => { root.saved(); } }
}
```

Créez `src/main.rs` :

```rust
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_saved(|| println!("Enregistré"));
    app.run()
}
```

Installez les services du kit une fois par fenêtre avant son affichage. Chaque composant possède son état d’interaction ; ajoutez une autre instance en la déclarant dans Slint. Vos callbacks traitent les actions de l’application et les valeurs métier acceptées. Le kit fournit la validation, la navigation, les recettes de thème et la conversion de ses contrats d’interface. Les [formulaires et validations](/fr/guides/forms/) détaillent les règles de soumission.

Lorsqu’un callback de l’application doit modifier sa fenêtre, utilisez `slint_kit::action(&app, |app| { /* action métier */ })`. Pour un callback recevant une valeur, utilisez `slint_kit::event(&app, |app, value| { /* action métier */ })` ; `event2` reçoit deux valeurs et `event3` en reçoit trois. Ces helpers conservent une référence faible et ignorent les callbacks après la destruction de la fenêtre. Le gestionnaire peut employer la référence temporaire pour modifier l’état de l’application ; ne la stockez pas dans la fenêtre. Un callback qui enregistre ou transmet simplement une valeur n’a pas besoin de ce wrapper.

Lancez `cargo run`. Après remplacement des dépendances registry par des chemins locaux, exécutez d’abord `cargo generate-lockfile` (ou lancez une fois sans `--locked`) pour résoudre ce graphe de sources. Utilisez ensuite `cargo run --locked` pour le conserver. Un projet téléchargé inchangé utilise son fichier de verrouillage fourni. Le [catalogue](/fr/components/) présente les propriétés et callbacks ; l’[exemple d’espace de travail](/fr/guides/workspace/) montre une application plus complète.
