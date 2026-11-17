---
layout: base.njk
locale: fr
title: "Configurer l’éditeur"
translationKey: "guides/editor/"
permalink: "{{ 'guides/editor/' | localeUrl(locale) }}index.html"
---

# Résoudre les imports `@kit` dans l’éditeur

`slint-kit-build` enregistre `@kit` pour la compilation Cargo. Le serveur de langage Slint ne lit pas votre `build.rs` : configurez séparément ses chemins de bibliothèques pour activer les diagnostics, la complétion et la navigation des imports comme `@kit`.

Utilisez l’extension Slint de votre éditeur et un serveur de langage correspondant à la version Slint de l’application (actuellement **{{ project.slint }}**).

## Applications utilisant crates.io

Ouvrez la racine du workspace Cargo de l’application dans l’éditeur. Après avoir créé `Cargo.lock` avec `cargo check`, lancez cette commande depuis cette racine :

```sh
cargo vendor --locked target/slint-editor/vendor
```

Cargo copie les dépendances distantes choisies par votre fichier de verrouillage dans ce dossier du projet, y compris `slint-kit-slint/ui`, ses polices, ses icônes et ses notices de licence. Aucun clonage du kit ni chemin de cache Cargo propre à une machine n’est nécessaire. La commande copie **toutes les dépendances distantes** : prévoyez de l’espace disque et du temps pour la première exécution.

Pour un usage limité à l’éditeur, vous pouvez laisser de côté la configuration de remplacement des sources Cargo affichée par cette commande : `build.rs` continue de résoudre la même version verrouillée du kit avec Cargo. Pour compiler également depuis les sources copiées, fusionnez la configuration affichée dans `.cargo/config.toml`, comme l’explique le [guide Cargo sur le vendoring](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html).

Les réglages ci-dessous supposent une seule version résolue de `slint-kit-slint` et les noms de dossiers par défaut de Cargo. Si vos dépendances contiennent plusieurs versions, inspectez `target/slint-editor/vendor/` et choisissez le dossier correspondant au kit utilisé par l’application ; Cargo ajoute des suffixes de version si nécessaire. N’utilisez pas `--versioned-dirs` sans adapter les chemins ci-dessous.

## VS Code

Fusionnez ce réglage dans `.vscode/settings.json` à la racine du workspace :

```json
{
  "slint.libraryPaths": {
    "kit": "target/slint-editor/vendor/slint-kit-slint/ui/slint-project.json",
    "kit-icons": "target/slint-editor/vendor/slint-kit-slint/ui/icons/lucide",
    "kit-fonts": "target/slint-editor/vendor/slint-kit-slint/ui/fonts"
  }
}
```

La clé est `kit`, sans `@` ; le fichier projet sélectionne l’édition 1.19 de la bibliothèque et son point d’entrée `kit.slint`. Les icônes et polices optionnelles utilisent leurs alias de dossiers distincts.

## Zed

Fusionnez ces arguments du serveur de langage dans `.zed/settings.json` à la racine du workspace :

```json
{
  "lsp": {
    "slint": {
      "binary": {
        "arguments": [
          "-L",
          "kit=target/slint-editor/vendor/slint-kit-slint/ui/slint-project.json",
          "-L", "kit-icons=target/slint-editor/vendor/slint-kit-slint/ui/icons/lucide",
          "-L", "kit-fonts=target/slint-editor/vendor/slint-kit-slint/ui/fonts"
        ]
      }
    }
  }
}
```

Conservez les arguments existants et ajoutez cette paire `-L`. Zed démarre le serveur dans le worktree du projet : le chemin est donc relatif à cette racine. L’[extension Slint pour Zed](https://github.com/slint-ui/slint/blob/v1.19.0/editors/zed/src/slint.rs) transmet `binary.arguments` à `slint-lsp`. Les autres éditeurs peuvent transmettre les mêmes arguments en utilisant la racine du workspace comme répertoire de travail du serveur.

## Vérifier et actualiser

Redémarrez le serveur de langage Slint ou rechargez l’éditeur, puis ouvrez un fichier `.slint` qui importe `KitButton` depuis `@kit`. L’import doit être résolu et Aller à la définition doit ouvrir le composant copié. Continuez de compiler avec `slint_kit_build::compile` dans `build.rs` : les réglages de l’éditeur ne configurent pas la compilation Cargo.

Relancez la commande de vendoring après une mise à jour de `Cargo.lock` ou un `cargo clean`, qui supprime `target/`. Ne modifiez pas les fichiers copiés : actualisez-les depuis la dépendance résolue. Vous pouvez versionner les réglages relatifs de l’éditeur ; `target/` reste généré et ignoré par Git.

## Applications utilisant un checkout local

Cargo ne copie pas les dépendances locales déclarées par chemin. Si l’application utilise `slint-kit` et `slint-kit-build` depuis un checkout voisin `../slint-kit`, utilisez `../slint-kit/crates/kit-slint/ui/slint-project.json` comme chemin de bibliothèque dans VS Code, ou `kit=../slint-kit/crates/kit-slint/ui/slint-project.json` dans l’argument `-L` de Zed. Adaptez ce chemin relatif à votre workspace. Pour ouvrir le dépôt Slint Kit lui-même, utilisez `crates/kit-slint/ui/slint-project.json`.

Si l’éditeur affiche encore « Cannot find requested import », vérifiez que le fichier configuré est bien `kit.slint` et existe, que le workspace ouvert correspond à la racine du chemin et que la version de la bibliothèque correspond à votre dépendance Cargo. Un simple chemin d’inclusion n’enregistre pas l’alias de bibliothèque `@kit`. Consultez les [bibliothèques de composants Slint](https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/#component-libraries).

## Imports de composants et de ressources en 0.9

Utilisez `@kit` pour les composants et les types. Un projet de bibliothèque Slint expose uniquement son point d’entrée : les anciens sous-imports comme `@kit/button.slint` deviennent `@kit`. Importez Lucide depuis `@kit-icons/all.slint` ou une catégorie comme `@kit-icons/arrows.slint`, et Inter depuis `@kit-fonts/inter.slint`. Les trois alias ci-dessus fonctionnent dans les aperçus éditeur et correspondent au helper de compilation. Voir les [bibliothèques de projet Slint](https://docs.slint.dev/docs/slint/guide/tooling/project-file/#libraries).

Avec des dépendances locales, faites pointer ces mêmes alias vers les chemins correspondants sous `../slint-kit/crates/kit-slint/ui`. L’application garde son édition indépendante : `slint-kit-build` choisit 1.18 par défaut ; `config().with_edition((1, 19))` fait passer l’application en 1.19 sans modifier la bibliothèque.

<h2 id="static-previews-and-rust-services">Aperçus statiques et services Rust</h2>

Live Preview dans VS Code et `slint-viewer` affichent les options déclarées sans exécuter Rust. Passez les mêmes trois chemins de bibliothèque au viewer avec `-L` ; le fichier d’aperçu conserve ses imports publics `@kit`.

Ces aperçus montrent la disposition et l’état déclaré ; ils n’installent pas les services Rust. Utilisez `cargo run` avec un appel à `slint_kit::install(&app)` par fenêtre, ou la démo navigateur de la fiche composant, pour exercer les comportements fournis par ces services.

| Famille de composants | Aperçu Slint seul | Service nécessaire pour |
| --- | --- | --- |
| Onglets, menus/barres de menus, fils d’Ariane, carrousels | `options`, libellés, état désactivé et sélection simple déclarés | Filtrage, réconciliation, navigation et valeurs `chosen` acceptées ; les carrousels prévisualisent leur première diapositive source |
| Groupes radio et segments | Options et sélection simple déclarées | Sélection au clavier, validation et contrôle des générations |
| Groupes de toggles | Libellés des options ; `rows` explicites pour les marques de sélection exactes | Projection de l’ensemble sélectionné et acceptation des bascules |
| Accordéon | En-têtes des options, repliés ; fournissez `rows: [DisclosureRow]` à la place de `options` pour prévisualiser les corps dépliés | Projection des sections ouvertes et navigation clavier |
| Notation | Échelle et valeur configurées ; `scores` explicites également acceptés | Acceptation des scores et navigation |
| Select, combobox, sélection multiple et saisie de tags | Placeholders/textes déclarés et propriétés de présentation explicites ; options statiques selon les ports exposés | Résolution du libellé sélectionné, chips, filtrage et navigation des popups |
| Calendrier, sélecteurs de date et de période | `days`, `title` et propriétés de navigation explicites | Calcul des grilles mensuelles, bornes, périodes et dates acceptées |
| Champs texte/numériques, couleur et heure/date-heure | Texte natif déclaré ; `text` ou `hex-text` selon le composant | Formatage/analyse typés, règles de commit, validation HTML et résolution des fuseaux |
| Boutons, disposition, feedback et affichage de données | Contenu et présentation déclarés | Actions ou modèles applicatifs utilisés dans votre exemple |

Ne forcez pas la propriété `ready` d’un service : elle signifie que ses callbacks Rust sont installés. La propriété locale ne remplace pas l’installation. Un aperçu statique ne qualifie ni clavier/IME ni technologies d’assistance sur un appareil cible.

## Limite du live-preview Rust dans Slint 1.19.0

La variable d’environnement `SLINT_LIVE_PREVIEW` sélectionne un autre générateur Rust. Celui-ci omet les exports de globals de bibliothèque et les implémentations `Global` nécessaires à Slint Kit ([issue amont #14812](https://github.com/slint-ui/slint/issues/14812)). Toutes les fenêtres utilisant `slint_kit::install` sont concernées, même si leurs contrôles visibles ne dépendent pas de la sélection.

Slint Kit 0.7.2 signale cette limite pendant le build, au lieu de produire du Rust avec des globals introuvables. Supprimez `SLINT_LIVE_PREVIEW` du shell, de `build.rs` et des scripts de développement, puis recompilez. La définir à `0` ou à une chaîne vide sélectionne toujours le générateur live-preview de Slint : retirez entièrement la variable. Activer uniquement la feature Cargo `slint/live-preview` ne le sélectionne pas.

Pour inspecter l’application à l’exécution, conservez `slint/mcp` et `SLINT_EMIT_DEBUG_INFO=1` avec le générateur normal. Live Preview de l’éditeur et `slint-viewer` n’ont pas besoin de `SLINT_LIVE_PREVIEW` ; leur rendu statique suit le tableau ci-dessus.

## Métadonnées d’inspection à l’exécution

Le helper suit la politique par défaut de Slint pour les métadonnées de débogage. Pour inspecter les éléments avec le backend de test ou MCP, définissez `SLINT_EMIT_DEBUG_INFO=1` avant de compiler, ou passez `slint_kit_build::config().with_debug_info(true)` à `compile_with_config`. Les versions de production ordinaires n’imposent pas les noms d’éléments et les chemins source dans l’application générée.

Pour la boucle d’édition, les options d’aperçu et les temps de build mesurés, consultez le [guide de la boucle de développement](../development-loop/).
