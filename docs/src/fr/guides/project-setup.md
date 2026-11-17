---
layout: base.njk
locale: fr
title: "Configurer le projet"
translationKey: "guides/project-setup/"
permalink: "{{ 'guides/project-setup/' | localeUrl(locale) }}index.html"
---

# Configurer le projet

## Créer un projet en une commande

Le CLI `slint-kit` crée un projet Cargo indépendant avec les dépendances du kit correspondantes, le build helper public, une fenêtre thémée, un bouton connecté et `install` par fenêtre. Installez le CLI depuis la révision source correspondante ; ce n’est pas un paquet crates.io :

```sh
cargo install --git https://github.com/damione1/slint-kit --{% if project.registryExamples %}tag v{{ project.version }}{% else %}branch develop{% endif %} slint-kit-cli --locked
slint-kit new my-app
cd my-app
```

{% if not project.registryExamples %}
Ce guide suit la branche `develop`. Avant de compiler, copiez les entrées `slint-kit` et `slint-kit-build` du [manifeste Cargo ci-dessous](#cargo-dependencies) dans le `Cargo.toml` généré. Conservez la dépendance Slint générée et ses fonctionnalités.
{% endif %}

```sh
cargo run
```

`slint-kit new` n’écrase jamais un chemin existant, même un dossier vide ou un lien symbolique. Le dossier parent doit exister. Un nom de paquet comporte au plus 64 caractères, commence par une lettre ASCII et contient seulement des lettres ASCII, chiffres, `-` ou `_` ; mots-clés Rust, noms de crates de la bibliothèque standard et noms de périphériques Windows sont refusés. `slint-kit new "dossier avec espaces" --name my-app` sépare le chemin du nom Cargo portable. Le générateur n’exécute pas Cargo et n’installe aucune dépendance ; le premier `cargo run` résout les dépendances configurées et écrit `Cargo.lock`. Versionnez ce verrou pour reproduire les builds de l’application.

Le `src/main.rs` généré appelle la petite fabrique et le point d’entrée de `src/lib.rs` ; leur total ne dépasse pas 15 lignes Rust. La fabrique utilise l’adaptateur d’action à fenêtre faible du kit. Cliquez sur **Run action** ou focalisez-le et appuyez sur Espace/Entrée : Rust actualise le statut visible. Modifiez `ui/app.slint` pour la présentation et le thème, puis le callback Rust pour le comportement métier. Les sources générées utilisent GPL-3.0-or-later ; consultez [la licence et les limites d’adoption](/fr/guides/why-slint-kit/) avant de distribuer l’application.

<h3 id="ajouter-un-apercu-navigateur">Ajouter un aperçu navigateur</h3>

{% if not project.registryExamples %}Appliquez les mêmes dépendances source au projet Web généré avant de le compiler.{% endif %}

```sh
slint-kit new my-web-app --web
cd my-web-app
rustup target add wasm32-unknown-unknown
cargo build --release --lib --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version '=0.2.129' --locked
wasm-bindgen target/wasm32-unknown-unknown/release/my_web_app.wasm --target web --out-dir pkg
python3 -m http.server 8090 --bind 127.0.0.1
```

Ouvrez `http://localhost:8090/`. Ce projet conserve `cargo run` sur desktop, ajoute la bibliothèque et le démarrage WASM, `index.html` et la dépendance `wasm-bindgen` correspondante, et utilise le moteur FemtoVG de Slint sur desktop et navigateur. Le mode desktop seul utilise Skia. Servez JS/WASM en HTTP ; ouvrir le fichier HTML directement ne suffit pas. Cet aperçu ne qualifie pas l’IME natif, la saisie mobile ou les lecteurs d’écran.

Le générateur et les téléchargements de composants partagent le modèle de projet Cargo et `build.rs`. Chaque téléchargement garde l’UI, les callbacks typés, les dépendances optionnelles et le verrou propres à son composant. La configuration manuelle ci-dessous reste disponible pour les applications existantes.


Configurez l’application Rust une seule fois, puis utilisez chaque composant avec un import `@kit`, des propriétés et des callbacks typés. Les fiches fournissent `ui/app.slint` et `src/main.rs` ; les projets téléchargeables incluent aussi le manifeste, le script de compilation et le fichier de verrouillage.

Le helper build utilise la compilation expérimentale de modules de Slint pour partager les contrats Rust/Slint du kit. Gardez les paquets runtime et build sur la même version et suivez la plage corrective Slint prise en charge. Consultez la [compatibilité du build](https://docs.rs/crate/slint-kit-build/latest) et les [limites d’adoption](/fr/guides/why-slint-kit/).

<h2 id="cargo-dependencies">Dépendances</h2>

Utilisez Rust **{{ project.rust }}+**, l’édition **2024** et Slint **{{ project.slint }}**. Lancez `cargo new my-app` et ajoutez les paquets d’exécution et de compilation dans `Cargo.toml` :

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "{{ project.rust }}"

[dependencies]
slint-kit = {% if project.registryExamples %}"={{ project.version }}"{% else %}{ git = "https://github.com/damione1/slint-kit", branch = "develop" }{% endif %}
slint = { version = "={{ project.slint }}", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = {% if project.registryExamples %}"={{ project.version }}"{% else %}{ git = "https://github.com/damione1/slint-kit", branch = "develop" }{% endif %}
```

Le manifeste ci-dessus épingle l’exemple sur Slint {{ project.slint }} pour sa reproductibilité. Le kit accepte les versions correctives de Slint 1.19 à partir de la 1.19.0 (`~1.19.0`). Gardez le compilateur et le runtime Slint alignés lors d’une mise à jour corrective.

Le helper conserve l’édition 1.18 du langage Slint pour l’application par défaut. La bibliothèque `@kit` possède son propre projet en édition 1.19 ; `config().with_edition((1, 19))` fait aussi passer l’application en 1.19. Rust 1.95 ou ultérieur est requis.


Gardez les paquets d’exécution et de compilation du kit sur des versions correspondantes. Conservez le backend et le moteur de rendu compatibles d’une application existante. Cet exemple de bureau utilise winit et Skia ; consultez [Bien démarrer](/fr/guides/getting-started/) pour les prérequis système et l’[intégration native](/fr/guides/native/) pour les services hôtes optionnels.

Ces exemples ciblent Slint Kit **{{ project.version }}**. Pour compiler avec un checkout local, remplacez les deux dépendances du kit ci-dessus par des chemins. Pour une application voisine d’un checkout `slint-kit` :

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Conservez la dépendance Slint du premier manifeste. {% if project.registryExamples %}Les projets téléchargeables utilisent les mêmes versions du registre.{% else %}Dans un projet téléchargé, remplacez les deux versions du kit provenant du registre par les entrées Git ou les chemins locaux ci-dessus, puis lancez `cargo run` pour actualiser le fichier de verrouillage.{% endif %} Gardez le nom de dépendance Cargo `slint-kit`, car le code de modules généré par Slint référence `slint_kit::ui`.

<h2 id="desktop-and-mobile-targets">Cibles bureau, mobile, web et embarquées</h2>

Slint fournit les backends de plateforme, les moteurs de rendu et les outils de déploiement. Slint Kit fournit des composants et contrôleurs Rust/Slint. Réutilisez les imports `@kit`, le helper build et `slint_kit::install(&window)` par fenêtre sur les cibles compatibles ; adaptez le backend, le moteur, le point d’entrée et le packaging de l’application à chaque plateforme.

| Cible | Par où commencer |
| --- | --- |
| macOS, Windows, Linux | Le manifeste winit/Skia ci-dessus est un point de départ desktop. Installez les [prérequis système](/fr/guides/getting-started/), puis suivez le [guide de packaging desktop de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/packaging/general/). |
| Android | Suivez le [guide Android de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/) : SDK/NDK, cibles Rust, `cdylib`, `android_main`, initialisation du backend et outils de compilation/déploiement APK. |
| iOS | Le [guide iOS de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/) couvre Xcode sur macOS, les cibles appareil/simulateur, le point d’entrée et la signature. |
| WebAssembly | L’option `--web` du CLI crée un aperçu navigateur. Consultez les [étapes locales ci-dessus](#ajouter-un-apercu-navigateur) et le [guide Web de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/web/) pour la compilation et le déploiement web. |
| Linux embarqué | Le [guide embarqué de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/embedded/#embedded-linux) couvre notamment Yocto, Buildroot et Torizon. Choisissez la cible Rust de la carte, la chaîne de compilation croisée et le moteur ; voir [LinuxKMS](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_linuxkms/) pour l’accès direct à l’affichage sans compositeur desktop. |
| Microcontrôleurs / bare metal | Slint prend en charge des microcontrôleurs via le support de carte, l’intégration des entrées et un moteur logiciel. Suivez les [instructions officielles pour microcontrôleurs](https://docs.slint.dev/latest/docs/slint/guide/platforms/embedded/#microcontrollers). Slint Kit utilise actuellement la bibliothèque standard Rust et ne fournit pas de portage `no_std` pour microcontrôleur. |

### Compiler et empaqueter pour la cible choisie

`cargo build --release --locked` compile un binaire desktop après la première résolution des dépendances de l’application. La création d’un installateur, d’un bundle d’application ou d’un paquet de distribution est une étape distincte, décrite par le guide desktop de Slint. Changer seulement `--target` ne produit pas un APK Android, une application iOS ou le firmware d’une carte. Suivez le guide officiel de la plateforme pour le point d’entrée, le linker/sysroot, les ressources et les outils de déploiement requis.

Sur Android, activez `backend-android-activity-06` et initialisez le backend avec `slint::android::init(app)` dans `android_main`, avant de créer la fenêtre et d’installer les services du kit. Le guide officiel précise les versions de SDK/NDK et les outils de packaging pour la version Slint choisie. Séparez les dépendances propres à la plateforme du manifeste desktop au lieu de reprendre ses features winit/Skia sans adaptation.

Sur Linux embarqué, vérifiez d’abord les pilotes d’affichage et d’entrée, les polices, le moteur et la mémoire disponible avec les exemples de Slint, puis intégrez le kit par le même helper build Rust. Le support d’une carte par Slint est distinct d’un déploiement testé de ce kit. Pour une carte `no_std`, partez de l’exemple de carte de Slint ; le starter desktop du kit n’est pas un firmware.

Sur mobile, adaptez layouts, cibles tactiles, défilement, zones sûres et clavier virtuel avec les [recommandations mobiles de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/general/). Les dialogues fichiers/dossiers optionnels `native` du kit renvoient `SelectionOutcome::Failed(HostError::Unsupported)` sur Android et iOS ; utilisez au besoin une intégration de plateforme dans l’application.

Les starters téléchargeables utilisent des points d’entrée desktop ou desktop/Web. Les captures du site et les tests headless ne qualifient pas les appareils mobiles ou embarqués : vérifiez le tactile, la saisie, l’IME et l’accessibilité sur la cible réelle.

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
