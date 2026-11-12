---
layout: base.njk
locale: fr
title: "Configurer le projet"
translationKey: "guides/project-setup/"
permalink: "{{ 'guides/project-setup/' | localeUrl(locale) }}index.html"
---

# Configurer le projet

## Créer un projet en une commande

Le CLI `slint-kit` crée un projet Cargo indépendant avec les dépendances du kit correspondantes, le build helper public, une fenêtre thémée, un bouton connecté et `install` par fenêtre. Le CLI est disponible depuis la branche source `develop` ; aucune publication de son paquet n’est supposée :

```sh
cargo install --git https://github.com/damione1/slint-kit --branch develop slint-kit-cli --locked
slint-kit new my-app
cd my-app
cargo run
```

`slint-kit new` n’écrase jamais un chemin existant, même un dossier vide ou un lien symbolique. Le dossier parent doit exister. Un nom de paquet comporte au plus 64 caractères, commence par une lettre ASCII et contient seulement des lettres ASCII, chiffres, `-` ou `_` ; mots-clés Rust, noms de crates de la bibliothèque standard et noms de périphériques Windows sont refusés. `slint-kit new "dossier avec espaces" --name my-app` sépare le chemin du nom Cargo portable. Le générateur n’exécute pas Cargo et n’installe aucune dépendance ; le premier `cargo run` résout les versions du registre et écrit `Cargo.lock`. Versionnez ce verrou pour reproduire les builds de l’application.

Le `src/main.rs` généré appelle la petite fabrique et le point d’entrée de `src/lib.rs` ; leur total ne dépasse pas 15 lignes Rust. La fabrique utilise l’adaptateur d’action à fenêtre faible du kit. Cliquez sur **Run action** ou focalisez-le et appuyez sur Espace/Entrée : Rust actualise le statut visible. Modifiez `ui/app.slint` pour la présentation et le thème, puis le callback Rust pour le comportement métier. Les sources générées utilisent GPL-3.0-or-later ; consultez [la licence et les limites d’adoption](/fr/guides/why-slint-kit/) avant de distribuer l’application.

### Ajouter un aperçu navigateur

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

Le manifeste ci-dessus épingle l’exemple sur Slint {{ project.slint }} pour sa reproductibilité. Le kit accepte les versions correctives de Slint 1.18 à partir de la 1.18.1 (`~1.18.1`). Gardez le compilateur et le runtime Slint alignés lors d’une mise à jour corrective.

Gardez les paquets d’exécution et de compilation du kit sur des versions correspondantes. Conservez le backend et le moteur de rendu compatibles d’une application existante. Cet exemple de bureau utilise winit et Skia ; consultez [Bien démarrer](/fr/guides/getting-started/) pour les prérequis système et l’[intégration native](/fr/guides/native/) pour les services hôtes optionnels.

Ces exemples ciblent Slint Kit **{{ project.version }}**. Pour développer depuis les sources, remplacez les deux dépendances du kit ci-dessus par des chemins locaux. Pour une application voisine d’un checkout `slint-kit` :

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Conservez la dépendance Slint du premier manifeste. Les téléchargements ciblent la version correspondante du registre ; les chemins locaux permettent de développer depuis les sources. Gardez le nom Cargo `slint-kit` : le code de modules généré par Slint référence `slint_kit::ui`.

<h2 id="desktop-and-mobile-targets">Cibles bureau et mobile</h2>

Les composants Slint Kit peuvent servir dans des applications Rust de bureau et mobiles. Choisissez le backend et le moteur de rendu Slint de l’application pour chaque cible ; conservez les imports `@kit`, le helper de compilation et l’initialisation `slint_kit::install(&window)`.

| Cible | Configuration de l’application |
| --- | --- |
| macOS, Windows, Linux | Le manifeste winit/Skia ci-dessus constitue un point de départ pour le bureau. Installez les [prérequis système](/fr/guides/getting-started/). |
| Android | Activez la feature Slint `backend-android-activity-06`, compilez l’application en `cdylib` et appelez `slint::android::init(app)` dans `android_main` avant de créer la fenêtre. Installez les SDK/NDK Android et une cible Rust Android ; suivez le [guide Android de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/) pour créer l’APK. |
| iOS | Utilisez winit et Skia avec les cibles Rust pour appareil et simulateur iOS. La compilation et l’empaquetage nécessitent macOS et Xcode ; le [guide iOS de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/) détaille le point d’entrée, le projet Xcode et la signature. |

Les projets téléchargeables utilisent des points d’entrée et manifestes de bureau ; adaptez-les avec le guide de la plateforme avant de compiler pour mobile.

Sur mobile, adaptez la mise en page, les cibles tactiles, le défilement, les zones sûres et le clavier virtuel en suivant les [conseils mobiles de Slint](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/general/). Les sélecteurs de fichiers/répertoires de la feature optionnelle `native` renvoient `SelectionOutcome::Failed(HostError::Unsupported)` sur Android et iOS ; prévoyez une intégration de plateforme dans l’application si ces services sont nécessaires.

La compilation repose sur les backends de Slint. Les captures publiées et les tests headless du kit ne qualifient pas les appareils Android/iOS : vérifiez les interactions tactiles, la saisie native, l’IME et les lecteurs d’écran sur vos appareils cibles.

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
