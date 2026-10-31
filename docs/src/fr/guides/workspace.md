---
layout: base.njk
locale: fr
title: "Composer une application de travail"
translationKey: "guides/workspace/"
permalink: "{{ 'guides/workspace/' | localeUrl(locale) }}index.html"
---

# Composer une application de travail

L’exemple assemble des paramètres de compte, une liste de projets et un dialogue d’édition avec les composants publics de Slint Kit. La composition utilise la présentation Tailwind intégrée en modes clair et sombre. L’hôte Rust possède les données, leur filtrage et les modifications en mémoire.

## Essayer la composition complète

- [Paramètres du compte](/examples/workspace/?mode=light&scale=1&scene=account&lang=fr) : modifiez le profil, choisissez une langue et activez les notifications.
- [Projets](/examples/workspace/?mode=light&scale=1&scene=projects&lang=fr) : recherchez, filtrez, sélectionnez un projet, créez-en un ou modifiez-le.
- [Dialogue d’édition](/examples/workspace/?mode=light&scale=1&scene=dialog&lang=fr) : examinez les champs, la validation, l’état d’enregistrement et le retour du focus.

Les contrôles de la démo permettent de changer de thème, de présentation claire/sombre et d’échelle du texte. Comparez les mêmes données et la même structure, avec une personnalisation locale, plutôt que deux jeux de composants. Les modifications de la démo sont conservées en mémoire.

## Lancer l’application native

Téléchargez [workspace.slint](/assets/examples/workspace.slint) comme `ui/app.slint` et [workspace.rs](/assets/examples/workspace.rs) comme `src/main.rs`. Ajoutez ces deux fichiers dans le même projet :

### Cargo.toml

```toml
[package]
name = "workspace-example"
version = "0.1.0"
edition = "2024"

[dependencies]
slint-kit = "=0.5.0"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "=0.5.0"
```

### build.rs

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile workspace");
}
```

Lancez `cargo run` dans le dossier du projet. Les sources incluent le point d’entrée, l’initialisation de la validation, le modèle à clés stables, le filtrage et les callbacks d’enregistrement. L’application utilise la présentation Tailwind intégrée avec Inter.

Choisissez `KitThemes.light` ou `.dark` pour changer de mode. Les sources importent `{ InterFont }` depuis `@kit/fonts/inter.slint` et choisissent cette police avec `default-font-family: InterFont.family`. [Configurer le thème et les réglages locaux](/fr/guides/themes/).

## Garder la responsabilité des données dans l’application

`ProductWorkspace.projects` est une projection contrôlée fournie par l’hôte. Les demandes de recherche et de filtre remontent à Rust ; l’identité des projets ne dépend pas de leur position. Les callbacks valident avant d’appliquer les changements et signalent leur achèvement. Remplacez le stockage en mémoire par vos services applicatifs en préservant cette répartition.

Le composite transmet explicitement son thème à ses enfants et au dialogue. Un rayon, un padding ou une icône locale ne modifie pas les régions voisines. Le dialogue désactive les contrôles d’arrière-plan pendant son ouverture, conserve l’ouvreur et lui rend le focus à la fermeture ou après l’enregistrement.

Lors de l’adaptation, conservez les éditeurs natifs, le clavier, les contraintes de champ et les noms accessibles des actions. Testez le vrai backend avec des libellés longs et du texte agrandi ; la démo navigateur ne qualifie ni l’IME ni les lecteurs d’écran des plateformes.
