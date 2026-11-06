---
layout: base.njk
locale: fr
title: "Bien démarrer"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Bien démarrer


Installez Slint Kit depuis crates.io avec **Rust {{ project.rust }}+** et **Slint {{ project.slint }}**. Aucun clonage du kit n’est nécessaire.

Les applications utilisent l’édition Rust 2024. Le compilateur et le runtime Slint doivent correspondre aux versions du kit.

## Créer une application de bureau

Suivez une fois la [configuration du projet](/fr/guides/project-setup/) pour Cargo et `build.rs`. Utilisez l’import public racine `@kit` et appelez `slint_kit::install(&app)` avant d’afficher chaque fenêtre. Le kit fournit le comportement, le thème et les conversions des contrats typés ; les callbacks de l’application traitent les actions métier.

Cette documentation cible Slint Kit **{{ project.version }}**. Utilisez des versions correspondantes des paquets runtime et build. Configurer le projet décrit aussi les dépendances sources locales. Une application utilisant une ancienne version suit la documentation correspondante.

## Utiliser un exemple documenté

Pour une application fonctionnelle, commencez par le [bouton](/fr/components/button/) ou la [liste avec recherche](/fr/components/combobox/) : chaque fiche montre le composant Slint et ses callbacks métier, avec un lien vers la configuration commune.

Chaque fiche propose un projet complet téléchargeable avec `Cargo.lock`, Rust, Slint et `build.rs`. Décompressez le ZIP, puis lancez `cargo run --locked`. La capture et la preview interactive utilisent ce même exemple. Pour les règles communes des champs, consultez les [formulaires et validations](/fr/guides/forms/).

Les captures montrent l’état déclaré dans l’exemple. Les actions système comme l’ouverture d’un sélecteur de fichiers nécessitent l’[adaptateur natif](/fr/guides/native/). Pour une édition validée et une sélection interactive, consultez l’[application avec contrôleurs typés](/fr/guides/typed-example/).

Importez composants, structs, enums et globals depuis l’unique point d’entrée `@kit`. Les fichiers supplémentaires de polices et d’icônes Lucide gardent leurs chemins documentés. Évitez les composants auxiliaires privés. Transmettez le même thème aux contrôles composés, préservez la taille intrinsèque et choisissez `ControlWidth.fill` quand le parent demande un contrôle étiré.

## Garder le domaine en Rust

Une chaîne Slint est un brouillon éditable. Pour les noms acceptés, montants, dates et choix identifiés par clé, utilisez des newtypes, des enums et les [liaisons typées](/fr/guides/controllers/). Les grandes collections utilisent un modèle et une vue, plutôt qu’une liste de contrôles construite en entier.

## Compiler le showcase

Le showcase est une application fonctionnelle qui présente les composants publics. Compilez-le depuis le dépôt source avec Git et **Rust {{ project.rust }}+**.

Préparez les dépendances de bureau de votre système : outils de ligne de commande Xcode sur macOS, chaîne Rust MSVC et outils de compilation Visual Studio sur Windows, ou bibliothèques de développement requises sur Linux. Le [guide de compilation de Slint {{ project.slint }}](https://github.com/slint-ui/slint/blob/v{{ project.slint }}/docs/building.md#prerequisites) détaille la configuration par plateforme.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

Le code et les ressources incluses sont sous [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE) ; conservez les notices des dépendances.
