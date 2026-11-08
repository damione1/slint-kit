---
layout: base.njk
locale: en
title: "Project setup"
translationKey: "guides/project-setup/"
permalink: "{{ 'guides/project-setup/' | localeUrl(locale) }}index.html"
---

# Project setup

Configure a Rust application once, then use each component through an `@kit` import, properties and typed callbacks. Component pages provide `ui/app.slint` and `src/main.rs`; their downloadable projects also include the manifest, build script and lockfile.

## Dependencies

Use Rust **{{ project.rust }}+**, edition **2024**, and Slint **{{ project.slint }}**. Start with `cargo new my-app` and add the runtime and build packages to `Cargo.toml`:

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

Keep the kit's runtime and build packages on matching releases. Preserve an existing application's compatible backend and renderer. This desktop example uses winit and Skia; see [Getting started](/guides/getting-started/) for platform prerequisites and [Native integration](/guides/native/) for optional host services.

These examples target Slint Kit **{{ project.version }}**. To develop against a source checkout instead, replace the two kit dependencies above with local paths. For an application beside a `slint-kit` checkout:

```toml
[dependencies]
slint-kit = { path = "../slint-kit/crates/kit" }

[build-dependencies]
slint-kit-build = { path = "../slint-kit/crates/kit-build" }
```

Keep the Slint dependency from the first manifest. Downloads target the matching registry release; local paths are an alternative for source development. Keep the Cargo dependency name `slint-kit`: Slint's generated module code references `slint_kit::ui`.

<h2 id="desktop-and-mobile-targets">Desktop and mobile targets</h2>

Slint Kit components can be used in desktop and mobile Rust applications. Select the application's Slint backend and renderer for each target; keep the same `@kit` imports, build helper and `slint_kit::install(&window)` initialization.

| Target | Application setup |
| --- | --- |
| macOS, Windows, Linux | The winit/Skia manifest above is a desktop starting point. Install the [system prerequisites](/guides/getting-started/). |
| Android | Enable Slint's `backend-android-activity-06` feature, build the application as a `cdylib` and initialize `slint::android::init(app)` in `android_main` before creating the window. Install the Android SDK/NDK and a Rust Android target; follow the [Slint Android guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/android/) for APK packaging. |
| iOS | Use winit and Skia with the Rust iOS device/simulator targets. Building and packaging require macOS and Xcode; follow the [Slint iOS guide](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/ios/) for the application entry point, Xcode project and signing. |

The downloadable starters use desktop entry points and manifests; adapt them using the platform guide before building for mobile.

On mobile, adapt layouts, touch targets, scrolling, safe areas and the virtual keyboard using [Slint's mobile guidance](https://docs.slint.dev/latest/docs/slint/guide/platforms/mobile/general/). The kit's optional `native` file/directory dialogs return `SelectionOutcome::Failed(HostError::Unsupported)` on Android and iOS; use an application platform integration if those services are needed.

Compilation support comes from Slint's backends. The kit's published screenshots and headless tests do not qualify Android/iOS devices: verify touch interaction, native text input, IME and screen readers on your target devices.

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
