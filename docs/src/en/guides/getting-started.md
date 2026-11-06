---
layout: base.njk
locale: en
title: "Getting started"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Getting started


Install Slint Kit from crates.io with **Rust {{ project.rust }}+** and **Slint {{ project.slint }}**. No toolkit checkout is required.

Applications use Rust edition 2024. The Slint compiler and runtime must match the kit’s versions.

## Create a desktop application

Follow [Project setup](/guides/project-setup/) once to configure Cargo and `build.rs`. Use the public `@kit` root import and call `slint_kit::install(&app)` before showing each window. The kit supplies the component behavior, theme and typed contract conversions; application callbacks handle business actions.

This documentation targets Slint Kit **{{ project.version }}**. Use matching runtime and build packages. Project setup also describes local source dependencies. Applications using an older release should follow its matching documentation.

## Use a documented example

For a working app, start with the [button](/components/button/) or [searchable combobox](/components/combobox/): each page shows the Slint component and its business callbacks, with a link to shared project setup.

Each page offers a complete downloadable project with `Cargo.lock`, Rust, Slint and `build.rs`. Extract its ZIP, then run `cargo run --locked`. The screenshot and interactive preview use that same example. For shared field rules, see [forms and validation](/guides/forms/).

The screenshots show the declared example state. Host-dependent actions such as opening a file picker require the [native adapter](/guides/native/). For validated editing and interactive selection, see the [typed controller application](/guides/typed-example/).

Import components, structs, enums and globals from the single `@kit` entry point. Supplemental font and Lucide files keep their documented paths. Do not depend on private helper components. Pass the same theme into composed controls, preserve intrinsic sizing and choose `ControlWidth.fill` only when the parent requests a stretched control.

## Keep the domain in Rust

A Slint string is an editable draft. For accepted names, money, dates and keyed choices, use domain newtypes, enums and [typed bindings](/guides/controllers/). Large collections belong in a model/view, not an eagerly created list of controls.

## Build the showcase

The showcase is a working application that demonstrates the public components. Build it from the source repository with Git and **Rust {{ project.rust }}+**.

Prepare the desktop dependencies for your OS: Xcode Command Line Tools on macOS, an MSVC Rust toolchain and Visual Studio Build Tools on Windows, or the required development libraries on Linux. See the [Slint {{ project.slint }} build guide](https://github.com/slint-ui/slint/blob/v{{ project.slint }}/docs/building.md#prerequisites) for platform setup.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

Code and bundled assets are [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE); preserve dependency notices.
