---
layout: base.njk
locale: en
title: "Project setup"
translationKey: "guides/project-setup/"
permalink: "{{ 'guides/project-setup/' | localeUrl(locale) }}index.html"
---

# Project setup

## Create a project with one command

The `slint-kit` CLI creates an independent Cargo project with matching kit dependencies, the public build helper, a themed window, one connected button and per-window `install`. Install the CLI from the matching source revision; it is not a crates.io package:

```sh
cargo install --git https://github.com/damione1/slint-kit --{% if project.registryExamples %}tag v{{ project.version }}{% else %}branch develop{% endif %} slint-kit-cli --locked
slint-kit new my-app
cd my-app
```

{% if not project.registryExamples %}
This guide follows the `develop` branch. Before building, copy the `slint-kit` and `slint-kit-build` entries from the [Cargo manifest below](#cargo-dependencies) into the generated `Cargo.toml`. Keep the generated Slint dependency and its features.
{% endif %}

```sh
cargo run
```

`slint-kit new` never overwrites an existing path, including an empty directory or symlink. Its parent directory must exist. A package name has at most 64 characters, starts with an ASCII letter and contains only ASCII letters, digits, `-` or `_`; Rust keywords, standard-library crate names and Windows device names are rejected. Use `slint-kit new "folder with spaces" --name my-app` to choose the directory separately from its portable Cargo name. The generator neither runs Cargo nor installs dependencies; the first `cargo run` resolves the configured dependencies and writes `Cargo.lock`. Commit that lockfile for reproducible application builds.

The generated `src/main.rs` calls the small `src/lib.rs` factory/entry point; together they contain at most 15 Rust lines. The factory uses the kit's weak-window action adapter. Click **Run action** or focus it and press Space/Enter: Rust updates the visible status. Edit `ui/app.slint` for the layout and theme, and the Rust callback for application behavior. The generated sources use GPL-3.0-or-later; review the [license and adoption limits](/guides/why-slint-kit/) before distributing your application.

<h3 id="include-a-browser-preview">Include a browser preview</h3>

{% if not project.registryExamples %}Apply the same source dependency replacements to the generated Web project before building it.{% endif %}

```sh
slint-kit new my-web-app --web
cd my-web-app
rustup target add wasm32-unknown-unknown
cargo build --release --lib --target wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version '=0.2.129' --locked
wasm-bindgen target/wasm32-unknown-unknown/release/my_web_app.wasm --target web --out-dir pkg
python3 -m http.server 8090 --bind 127.0.0.1
```

Open `http://localhost:8090/`. This project keeps the desktop `cargo run` entry point, adds the WASM library/start function, `index.html` and the matching `wasm-bindgen` dependency, and uses Slint's FemtoVG renderer for desktop and browser. The desktop-only default uses Skia. Serve the generated JS/WASM over HTTP; opening the HTML as a local file is insufficient. Browser preview does not qualify native IME, mobile input or screen readers.

The scaffolder and component downloads share their Cargo project shell and `build.rs` template. Downloads retain the component's own UI, typed callbacks, optional dependencies and lockfile. The manual setup below remains available for existing applications.


Configure a Rust application once, then use each component through an `@kit` import, properties and typed callbacks. Component pages provide `ui/app.slint` and `src/main.rs`; their downloadable projects also include the manifest, build script and lockfile.

The build helper uses Slint’s experimental module compilation to share the kit's Rust/Slint contracts. Keep runtime and build packages on the same release and follow the supported Slint patch range. See [build compatibility](https://docs.rs/crate/slint-kit-build/latest) and [adoption limits](/guides/why-slint-kit/).

<h2 id="cargo-dependencies">Dependencies</h2>

Use Rust **{{ project.rust }}+**, edition **2024**, and Slint **{{ project.slint }}**. Start with `cargo new my-app` and add the runtime and build packages to `Cargo.toml`:

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

The manifest above pins the example to Slint {{ project.slint }} for reproducibility. The kit itself accepts Slint 1.19 patch releases from 1.19.0 (`~1.19.0`). Keep the Slint compiler and runtime aligned when updating a patch release.

The build helper keeps application Slint language edition 1.18 by default. The `@kit` library has its own edition 1.19 project; use `config().with_edition((1, 19))` to opt the application into 1.19 too. Rust 1.95 or later is required.


Keep the kit's runtime and build packages on matching releases. Preserve an existing application's compatible backend and renderer. This desktop example uses winit and Skia; see [Getting started](/guides/getting-started/) for platform prerequisites and [Native integration](/guides/native/) for optional host services.

These examples target Slint Kit **{{ project.version }}**. To build against a local checkout, replace the two kit dependencies above with paths. For an application beside a `slint-kit` checkout:

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Keep the Slint dependency from the first manifest. {% if project.registryExamples %}The downloadable projects use the same registry pins.{% else %}For a downloaded project, replace the two kit registry pins with the Git or local path entries above, then run `cargo run` to refresh its lockfile.{% endif %} Keep the Cargo dependency name `slint-kit` because Slint’s generated module code references `slint_kit::ui`.

<h2 id="desktop-and-mobile-targets">Desktop, mobile, web and embedded targets</h2>

Slint supplies the platform backends, renderers and deployment tools. Slint Kit supplies Rust/Slint components and controllers. Reuse `@kit` imports, the build helper and per-window `slint_kit::install(&window)` on compatible targets; adapt the application's backend, renderer, entry point and packaging for each platform.

| Target | Where to start |
| --- | --- |
| macOS, Windows, Linux | The winit/Skia manifest above is a desktop starting point. Install the [system prerequisites](/guides/getting-started/), then follow Slint's [desktop packaging guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/packaging/general/). |
| Android | Use Slint's [Android guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/): SDK/NDK setup, Rust targets, `cdylib`, `android_main`, backend initialization and APK build/deployment tools. |
| iOS | Follow Slint's [iOS guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/) for Xcode on macOS, device/simulator targets, the application entry point and signing. |
| WebAssembly | The CLI's `--web` option creates a browser preview. See the [local steps above](#include-a-browser-preview) and Slint's [Web guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/web/) for browser builds and deployment. |
| Embedded Linux | Slint's [embedded guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/embedded/#embedded-linux) covers deployments such as Yocto, Buildroot and Torizon. Choose the board's Rust target, cross-toolchain and renderer; see [LinuxKMS](https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backend_linuxkms/) for direct display access without a desktop compositor. |
| Microcontrollers / bare metal | Slint supports microcontrollers through board support, input integration and a software renderer. Follow the [official microcontroller instructions](https://docs.slint.dev/latest/docs/slint/guide/platforms/embedded/#microcontrollers). Slint Kit currently uses Rust's standard library and does not provide a `no_std` microcontroller port. |

### Build and package for the chosen target

`cargo build --release --locked` builds a desktop binary after the application's first dependency resolution. Packaging that binary into an installer, app bundle or distribution package is a separate step covered by Slint's desktop guide. Changing `--target` alone does not create an Android APK, an iOS application or board firmware. Use the official platform guide for the required entry point, linker/sysroot, assets and deployment tools.

For Android, enable `backend-android-activity-06` and initialize the backend with `slint::android::init(app)` in `android_main` before creating the window and installing kit services. The official guide describes the SDK/NDK versions and packaging tools for the selected Slint version. Keep platform-specific dependencies separate from the desktop manifest instead of copying its winit/Skia features unchanged.

For embedded Linux, verify the board's display/input drivers, font resources, renderer and available memory with Slint's examples first, then integrate the kit through the same Rust build helper. Slint's support for a board is distinct from a tested deployment of this kit. For a `no_std` board, start from Slint's own board example; the kit's desktop starter is not firmware.

On mobile, adapt layouts, touch targets, scrolling, safe areas and the virtual keyboard using [Slint's mobile guidance](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/general/). The optional kit `native` file/directory dialogs return `SelectionOutcome::Failed(HostError::Unsupported)` on Android and iOS; use an application platform integration if needed.

The downloadable starters use desktop or desktop/Web entry points. The site's captures and headless tests do not qualify mobile or embedded devices: verify touch, text input, IME and accessibility on the actual target.

## Build the UI

Create `build.rs` at the project root:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

The build package registers the kit's shared Slint types and verifies their contracts. Use the public root import `@kit` for components, enums and structs. Additional font and icon files retain their documented imports. Configure the [editor](/guides/editor/) separately so its language server can resolve these imports too.

On Windows, use Slint Kit 0.7.1 or later: 0.7.0 incorrectly escaped canonical source paths in its generated import. Both `compile` and `compile_with_config` accept relative, absolute and canonical paths, including spaces and Unicode. For custom settings, pass a configuration from `slint_kit_build::config()` or `config_with_libraries(...)` to `compile_with_config`; application include paths and library aliases are preserved.

## Import, set properties, receive values

Create `ui/app.slint`:

```slint
import { KitButton } from "@kit";

export component App inherits Window {
    callback saved();
    KitButton { text: "Save"; activated => { root.saved(); } }
}
```

Create `src/main.rs`:

```rust
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    app.on_saved(|| println!("Saved"));
    app.run()
}
```

Install the kit's services once per window before showing it. Each component owns its interaction state; add another instance by declaring it in Slint. Your callbacks handle application actions and accepted domain values. The kit supplies validation, navigation, theme recipes and the conversion of its UI contracts. See [Forms and validation](/guides/forms/) for submission rules.

When an application callback needs to update its window, wrap it with `slint_kit::action(&app, |app| { /* application action */ })`. For a callback receiving one value, use `slint_kit::event(&app, |app, value| { /* application action */ })`; `event2` receives two values and `event3` receives three. These helpers retain a weak window handle and ignore callbacks after that window is destroyed. A handler may use the temporary window handle to update application state; do not store it inside the window. Ordinary callbacks that only save or send a value need no wrapper.

Run `cargo run`. If you replaced registry dependencies with local source paths, first run `cargo generate-lockfile` (or run once without `--locked`) to resolve that source graph. Afterward use `cargo run --locked` to preserve it. An unchanged downloaded project uses its supplied lockfile. Browse the [component catalogue](/components/) for props and callbacks, or the [workspace example](/guides/workspace/) for a larger application.
