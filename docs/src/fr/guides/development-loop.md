---
layout: base.njk
locale: fr
title: "Boucle de développement"
translationKey: "guides/development-loop/"
permalink: "{{ 'guides/development-loop/' | localeUrl(locale) }}index.html"
---

# Boucle de développement : aperçu, recompilation et temps de build

Une application Slint Kit est un projet Cargo ordinaire : `build.rs` compile vos fichiers `.slint`, puis `rustc` compile le Rust généré. Une application en cours d’exécution ne recharge pas un fichier `.slint` modifié. Choisissez la boucle qui correspond à ce que vous modifiez.

## Quelle boucle choisir

| Vous modifiez | Utilisez | Recompilation Rust | Affichage |
| --- | --- | --- | --- |
| La disposition, les espacements, les jetons de thème, le contenu déclaré | Aperçu de l’éditeur (Live Preview de VS Code) ou `slint-viewer --auto-reload` | Non | Déclarations statiques ; sans services Rust |
| Un composant isolé | `slint-viewer` avec `--component` | Non | Le même rendu statique |
| Un comportement fourni par `slint_kit::install` : filtrage, validation, valeurs typées, sélections acceptées | `cargo run` | Oui | L’application réelle |
| Un comportement dans un navigateur | Build WASM de `slint-kit new --web` | Oui | Rendu navigateur ; voir [Configurer le projet](../project-setup/) |
| La logique Rust seule | `cargo check`, puis `cargo run` | Oui | Diagnostics du compilateur d’abord |

Configurez d’abord l’éditeur pour `@kit` : voir [Configurer l’éditeur](../editor/).

## Aperçu de l’éditeur et `slint-viewer`

L’aperçu de l’éditeur et `slint-viewer` compilent directement votre fichier `.slint` et le rechargent à chaque modification. Ils n’exécutent jamais votre code Rust ni `slint_kit::install` : ils montrent donc seulement la disposition et l’état déclaré. Le [guide de l’éditeur](../editor/#apercus-statiques-et-services-rust) indique ce que chaque famille de composants affiche sans services.

Utilisez un viewer et un serveur de langage dont la version correspond à celle de Slint dans l’application (actuellement **{{ project.slint }}**). Un viewer construit pour une version plus ancienne de Slint ne compile pas les fichiers de bibliothèque du kit.

Pour un aperçu autonome d’un fichier, pointez le viewer vers le dossier `ui` du kit. Le fichier d’aperçu doit importer `@kit/kit.slint`, car un alias de dossier résout les imports suffixés :

```sh
slint-viewer --auto-reload -L kit=target/slint-editor/vendor/slint-kit-slint/ui --component App ui/preview.slint
```

Gardez `@kit` comme import dans votre application et utilisez un fichier d’aperçu distinct, comme le décrit le guide de l’éditeur. Ajoutez `--load-data` pour démarrer un aperçu avec des valeurs de propriétés enregistrées.

## `SLINT_LIVE_PREVIEW` n’est pas pris en charge

Slint peut générer du Rust qui recharge le fichier `.slint` à l’exécution lorsque la variable d’environnement `SLINT_LIVE_PREVIEW` est définie. Dans Slint {{ project.slint }}, ce générateur omet les globales de bibliothèque dont `slint_kit::install` a besoin ([ticket amont #14812](https://github.com/slint-ui/slint/issues/14812)) ; Slint Kit interrompt donc le build avec une explication. Ne définissez pas la variable, pas même à `0` ou à une valeur vide. Relisez cette page quand vous mettez Slint à jour.

## Temps de build

Ces chiffres viennent d’une seule machine : servez-vous-en pour comparer des étapes, pas pour prédire vos propres temps. Chaque valeur est une exécution unique, en secondes, du projet créé par `slint-kit new` (moteur Skia, valeur par défaut du bureau).

- Machine : Apple silicon, 12 cœurs, 24 Go de mémoire, macOS.
- Chaîne d’outils : Rust 1.99.1, Slint 1.18.1, Slint Kit 0.7.2 avec le correctif décrit ci-dessous, `cargo build` avec le profil `dev` par défaut et `CARGO_BUILD_JOBS=6`.
- Les dépendances Cargo étaient déjà téléchargées. Les binaires Skia font partie du build à froid.

| Étape | Durée |
| --- | --- |
| Build à froid (`target/` vide) | 48 à 54 s |
| Build sans modification | 1,6 à 1,8 s |
| Build après modification d’un fichier `.slint` | 5,0 à 5,6 s |
| Build après modification d’un fichier Rust | 3,1 à 3,5 s |
| `cargo check` après une modification `.slint` ou Rust | 1,2 à 3,4 s |
| Premier `cargo check` après un `target/` vidé | 12 à 31 s |

Une modification `.slint` coûte plus qu’une modification Rust, car le script de build s’exécute de nouveau : il vérifie les contrats du kit, compile vos fichiers `.slint` et génère le Rust que `rustc` compile ensuite. Le premier build suivant une exécution du script de build recompile encore l’application une fois ; le deuxième build donne donc le temps représentatif « sans modification ».

Les versions antérieures réécrivaient à chaque build un fichier intermédiaire interne, ce qui faisait recompiler l’application par Cargo même sans modification. Avant le correctif, un build sans modification durait 5,0 à 6,5 s et une modification Rust 5,5 à 8,0 s. Passez à une version qui l’inclut.

Deux réglages n’ont produit aucune différence mesurable sur ce petit projet ; il n’y a donc aucune raison de les changer pour la seule vitesse :

- `CARGO_INCREMENTAL=1` ou `0` : les recompilations sont restées à moins de 1 s d’écart.
- `CARGO_PROFILE_DEV_DEBUG=0` au lieu des informations de débogage par défaut : les durées à froid et de recompilation sont restées dans la même plage.

Les temps n’ont pas été comparés à un build antérieur au mode bibliothèque ; cette page n’affirme donc aucun avant/après pour celui-ci.

## Réglages pour votre projet

- **Itérez avec `cargo check`.** Il vérifie les types Rust et exécute `build.rs` sans produire de binaire.
- **Limitez les tâches parallèles quand la mémoire est comptée.** Les grandes interfaces peuvent consommer beaucoup de mémoire quand plusieurs processus `rustc` tournent en même temps. Réduisez leur nombre avec `CARGO_BUILD_JOBS` ou `build.jobs` dans `.cargo/config.toml`, et évitez de lancer plusieurs builds Cargo distincts simultanément.
- **Restez sur le profil `dev` pour le travail courant sur l’interface.** Les mesures ci-dessus l’utilisent. Passez à `--release` seulement pour juger les performances d’animation et de défilement.
- **Conservez votre dossier `target/`.** Un build à froid est l’étape la plus lente de loin : lancez `cargo clean` seulement si nécessaire.
- **Éditeur de liens.** Son choix n’a pas été mesuré ici. Si l’édition de liens domine vos recompilations, mesurez un éditeur de liens plus rapide pour votre plateforme avant de l’adopter.
