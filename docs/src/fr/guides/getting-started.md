---
layout: base.njk
locale: fr
title: "Bien démarrer"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Bien démarrer


Installez Slint Kit depuis crates.io avec **Rust 1.92+** et **Slint 1.18.1**. Aucun clonage du kit n’est nécessaire.

Les applications utilisent l’édition Rust 2024. Le compilateur et le runtime Slint doivent correspondre aux versions du kit.

## Créer une application de bureau

Lancez `cargo new my-app`, puis ajoutez ces dépendances dans `my-app/Cargo.toml`. Cargo télécharge automatiquement le kit et sa bibliothèque de composants.

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.92"

[dependencies]
slint-kit = "0.3"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.3"
```

Créez `build.rs` à la racine de l’application. `slint-kit-build` enregistre la bibliothèque `@kit` et vérifie les contrats partagés générés avant de compiler l’interface.

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Créez `ui/app.slint` :

```slint
import { KitButton, KitTextField, KitThemes } from "@kit/kit.slint";

export { KitFormValidation, HtmlInputKind, NumberInputFormat, KitInputConstraints, KitValidationResult, KitValidityState } from "@kit/kit.slint";

export component App inherits Window {
    title: "Slint Kit";
    preferred-width: 420px;
    preferred-height: 240px;
    max-width: 100000px;
    max-height: 100000px;
    background: KitThemes.light.canvas;
    in-out property <string> name;
    callback save(string);

    VerticalLayout {
        padding: 24px;
        spacing: 16px;
        alignment: start;
        KitTextField { label: "Nom"; text <=> root.name; }
        KitButton {
            text: "Enregistrer";
            activated => { root.save(root.name); }
        }
    }
}
```

Créez `src/main.rs`. Le signal reçoit le brouillon courant lors de l’activation de Enregistrer ; l’application doit le valider avant de l’accepter dans son état métier.

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    app.on_save(|name| println!("Enregistré : {name}"));
    app.run()
}
```

Lancez `cargo run` depuis `my-app`. Cette configuration utilise le **backend winit et le moteur Skia**. Conservez le backend compatible d’une application existante lors d’une intégration.

## Utiliser un exemple documenté

Pour une application fonctionnelle, commencez par le [bouton](/fr/components/button/) ou la [liste avec recherche](/fr/components/combobox/) : chaque fiche fournit les quatre fichiers et leurs callbacks Rust raccordés.

Les autres [fiches de composant](/fr/components/) contiennent la composition visuelle de la capture. Pour l’afficher, copiez le fichier `.slint` dans `ui/app.slint` et conservez `build.rs`. Ces fichiers exportent `Example` ; utilisez ce point d’entrée :

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    Example::new()?.run()
}
```

Les captures montrent l’état déclaré dans l’exemple. Les actions système comme l’ouverture d’un sélecteur de fichiers nécessitent l’[adaptateur natif](/fr/guides/native/). Pour une édition validée et une sélection interactive, consultez l’[application avec contrôleurs typés](/fr/guides/typed-example/).

Les imports publics passent par `@kit/kit.slint`, ou un module public de famille comme `@kit/button.slint`. Évitez les composants auxiliaires privés. Transmettez le même thème aux contrôles composés, préservez la taille intrinsèque et choisissez `ControlWidth.fill` quand le parent demande un contrôle étiré.

## Garder le domaine en Rust

Une chaîne Slint est un brouillon éditable. Pour les noms acceptés, montants, dates et choix identifiés par clé, utilisez des newtypes, des enums et les [liaisons typées](/fr/guides/controllers/). Les grandes collections utilisent un modèle et une vue, plutôt qu’une liste de contrôles construite en entier.

## Compiler le showcase

Le showcase est une application fonctionnelle qui présente les composants publics. Compilez-le depuis le dépôt source avec Git et **Rust 1.92+**.

Préparez les dépendances de bureau de votre système : outils de ligne de commande Xcode sur macOS, chaîne Rust MSVC et outils de compilation Visual Studio sur Windows, ou bibliothèques de développement requises sur Linux. Le [guide de compilation de Slint 1.18.1](https://github.com/slint-ui/slint/blob/v1.18.1/docs/building.md#prerequisites) détaille la configuration par plateforme.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

Le code et les ressources incluses sont sous [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE) ; conservez les notices des dépendances.
