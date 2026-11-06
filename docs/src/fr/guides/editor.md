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
    "kit": "target/slint-editor/vendor/slint-kit-slint/ui/kit.slint"
  }
}
```

La clé est `kit`, sans `@` ; la valeur désigne le fichier `kit.slint` lui-même, car l’import `@kit` ne contient aucun nom de fichier. L’extension Slint résout les chemins relatifs depuis la racine du workspace. Conservez les autres alias déjà présents dans cette table. Consultez le [réglage des bibliothèques de l’extension Slint](https://github.com/slint-ui/slint/blob/v1.18.1/editors/vscode/package.json).

## Zed

Fusionnez ces arguments du serveur de langage dans `.zed/settings.json` à la racine du workspace :

```json
{
  "lsp": {
    "slint": {
      "binary": {
        "arguments": [
          "-L",
          "kit=target/slint-editor/vendor/slint-kit-slint/ui/kit.slint"
        ]
      }
    }
  }
}
```

Conservez les arguments existants et ajoutez cette paire `-L`. Zed démarre le serveur dans le worktree du projet : le chemin est donc relatif à cette racine. L’[extension Slint pour Zed](https://github.com/slint-ui/slint/blob/v1.18.1/editors/zed/src/slint.rs) transmet `binary.arguments` à `slint-lsp`. Les autres éditeurs peuvent transmettre les mêmes arguments en utilisant la racine du workspace comme répertoire de travail du serveur.

## Vérifier et actualiser

Redémarrez le serveur de langage Slint ou rechargez l’éditeur, puis ouvrez un fichier `.slint` qui importe `KitButton` depuis `@kit`. L’import doit être résolu et Aller à la définition doit ouvrir le composant copié. Continuez de compiler avec `slint_kit_build::compile` dans `build.rs` : les réglages de l’éditeur ne configurent pas la compilation Cargo.

Relancez la commande de vendoring après une mise à jour de `Cargo.lock` ou un `cargo clean`, qui supprime `target/`. Ne modifiez pas les fichiers copiés : actualisez-les depuis la dépendance résolue. Vous pouvez versionner les réglages relatifs de l’éditeur ; `target/` reste généré et ignoré par Git.

## Applications utilisant un checkout local

Cargo ne copie pas les dépendances locales déclarées par chemin. Si l’application utilise `slint-kit` et `slint-kit-build` depuis un checkout voisin `../slint-kit`, utilisez `../slint-kit/crates/kit-slint/ui/kit.slint` comme chemin de bibliothèque dans VS Code, ou `kit=../slint-kit/crates/kit-slint/ui/kit.slint` dans l’argument `-L` de Zed. Adaptez ce chemin relatif à votre workspace. Pour ouvrir le dépôt Slint Kit lui-même, utilisez `crates/kit-slint/ui/kit.slint`.

Si l’éditeur affiche encore « Cannot find requested import », vérifiez que le fichier configuré est bien `kit.slint` et existe, que le workspace ouvert correspond à la racine du chemin et que la version de la bibliothèque correspond à votre dépendance Cargo. Un simple chemin d’inclusion n’enregistre pas l’alias de bibliothèque `@kit`. Consultez les [bibliothèques de composants Slint](https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/#component-libraries).

## Imports supplémentaires avec Slint 1.18.1

Le serveur de langage résout un alias seul comme un fichier et ajoute le suffixe d’un import au chemin configuré. Un alias unique ne peut donc pas résoudre à la fois `@kit` et `@kit/fonts/inter.slint` ou `@kit/icons/lucide/all.slint`. Faire pointer `kit` vers le dossier `ui` résout ces imports supplémentaires, mais laisse `@kit` seul introuvable. Le réglage vers le fichier ci-dessus privilégie l’API des composants.

Conservez les imports supplémentaires documentés dans votre application : la compilation Cargo via `slint-kit-build` les résout correctement grâce aux métadonnées de module et à son alias de dossier. Avec le réglage éditeur vers le fichier, Slint 1.18.1 peut encore signaler ces imports supplémentaires comme introuvables. Il s’agit d’une limite du serveur de langage, pas d’un défaut de configuration Cargo ; ne remplacez pas les imports racine de l’application par l’ancien chemin `@kit/kit.slint`.
