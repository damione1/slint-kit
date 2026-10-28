---
layout: base.njk
locale: en
title: "Getting started"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Getting started

The reference follows `main`. For new localized labels, composite focus and native modal traversal, see [updates and compatibility](/guides/source-updates/). These additions require sources after crates.io 0.1.0.

Install Slint Kit from crates.io with **Rust 1.92+** and **Slint 1.18.1**. No toolkit checkout is required.

Applications use Rust edition 2024. The Slint compiler and runtime must match the kit’s versions.

## Create a desktop application

Run `cargo new my-app`, then add these dependencies to `my-app/Cargo.toml`. Cargo downloads the kit and its component library automatically.

```toml
[package]
name = "my-app"
version = "0.1.0"
edition = "2024"
rust-version = "1.92"

[dependencies]
slint-kit = "0.1"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.1"
```

Create `build.rs` at the application root. `slint-kit-build` registers the `@kit` library and verifies the generated shared contracts before compiling the UI.

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Create `ui/app.slint`:

```slint
import { KitButton, KitTextField } from "@kit/kit.slint";

export component App inherits Window {
    title: "Slint Kit";
    preferred-width: 420px;
    preferred-height: 240px;
    in-out property <string> name;
    callback save(string);

    VerticalLayout {
        padding: 24px;
        spacing: 16px;
        alignment: start;
        KitTextField { label: "Name"; text <=> root.name; }
        KitButton {
            text: "Save";
            activated => { root.save(root.name); }
        }
    }
}
```

Create `src/main.rs`. The callback receives the current draft when Save is activated; an application should validate that draft before accepting it into its domain state.

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    app.on_save(|name| println!("Saved: {name}"));
    app.run()
}
```

Run `cargo run` from `my-app`. This configuration uses the **winit backend and Skia renderer**. Preserve an existing compatible backend when integrating into another app.

## Use a documented example

Each [component page](/components/) provides a complete `.slint` file and its native screenshot. Copy that file to `ui/app.slint` and keep the same `build.rs`. The reference file exports `Example`; replace the entire contents of `src/main.rs` with this entry point:

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    Example::new()?.run()
}
```

The screenshots show the declared example state. Host-dependent actions such as opening a file picker require the [native adapter](/guides/native/). For validated editing and interactive selection, see the [typed controller application](/guides/typed-example/).

Public imports use `@kit/kit.slint`, or a public family module such as `@kit/button.slint`. Do not depend on private helper components. Pass the same theme into composed controls, preserve intrinsic sizing and choose `ControlWidth.fill` only when the parent requests a stretched control.

## Keep the domain in Rust

A Slint string is an editable draft. For accepted names, money, dates and keyed choices, use domain newtypes, enums and [typed bindings](/guides/controllers/). Large collections belong in a model/view, not an eagerly created list of controls.

## Build the showcase

The showcase is a working application that demonstrates the public components. Build it from the source repository with Git and **Rust 1.92+**.

Prepare the desktop dependencies for your OS: Xcode Command Line Tools on macOS, an MSVC Rust toolchain and Visual Studio Build Tools on Windows, or the required development libraries on Linux. See the [Slint 1.18.1 build guide](https://github.com/slint-ui/slint/blob/v1.18.1/docs/building.md#prerequisites) for platform setup.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

Code and bundled assets are [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE); preserve dependency notices.
