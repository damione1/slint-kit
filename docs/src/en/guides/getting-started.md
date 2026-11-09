---
layout: base.njk
locale: en
title: "Getting started"
translationKey: "guides/getting-started/"
permalink: "{{ 'guides/getting-started/' | localeUrl(locale) }}index.html"
---

# Getting started

Slint Kit combines visual components with shared behavior: draft validation, typed Rust values and stable identity. [Why Slint Kit](/guides/why-slint-kit/) explains the kit and application responsibilities, with examples you can try.


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

## Announced and qualified targets

Slint Kit announces the desktop platforms macOS, Windows and Linux, with Slint's winit backend and Skia renderer as the documented stack. Native file and directory pickers compile on macOS, Windows, Linux and FreeBSD and report `Unsupported` elsewhere. Android, iOS, the web and other Slint backends and renderers are not announced targets; the browser previews on this site compile the kit to WebAssembly for illustration only.

An announced target is one the kit is built for. A qualified target is one with a named proof: a check that runs on a named OS, backend and renderer, and whose result you can read. The table lists every proof the kit publishes. The continuous-integration proofs run on every push to `main` and on every pull request; their current result is the latest run of the [Check workflow](https://github.com/damione1/slint-kit/actions/workflows/check.yml), not a claim made here.

| Target | Backend and renderer | What the proof checks | Where |
| --- | --- | --- | --- |
| macOS, GitHub-hosted `macos-latest` runner (arm64) | Slint testing backend with the software renderer; no window | `cargo run -p xtask -- check`: formatting, generated files, token lint, quickstart guards, the complete headless test suite and clippy with warnings denied; the downloadable starters compile against the kit | `desktop` job of the Check workflow, also run before each release |
| Linux, GitHub-hosted `ubuntu-24.04` runner (x86_64) | Slint testing backend with the software renderer; no window | The kit and the whole workspace build with the Linux development libraries installed; the headless test suite runs; clippy with warnings denied covers the Linux configuration paths | `linux` job of the Check workflow |
| Windows, GitHub-hosted `windows-2025` runner (x86_64) | None: compile only | `cargo check` compiles the kit and its test crate; the build helper's path tests run. No component test runs and no window opens | `windows` job of the Check workflow |
| macOS arm64, maintainer workstation (Darwin 27, Apple M4 Pro, scale factors 1 and 2) | winit backend with the Skia renderer; native window | Component screenshots rendered natively in light and dark themes and at 200% text; the button's pointer click and Tab/Space paths checked in fresh native windows, in French and English | Captions on each component page; `docs/content/captures.json` records the environment and the qualification scope |

Not qualified on any target: pointer and keyboard interaction in a real winit window on Windows and Linux (X11 and Wayland), IME composition on any OS, screen readers (VoiceOver, Narrator, NVDA, Orca), renderers other than Skia on the capture machine, touch, mobile and web. A headless test proves the component contract on the testing backend; a capture proves the rendering of a declared state in the capture environment. Neither is a native, IME or screen-reader qualification. Such a qualification is a manual pass on the actual OS, backend, renderer and assistive technology, reported with those names; the changelog says which features still need one. When you qualify a target for your application, record the same names.

Code and bundled assets are [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE); preserve dependency notices.

Editor previews show static declarations and do not run the Rust services. See [static previews and the Slint 1.18.1 live-preview limitation](../editor/#static-previews-and-rust-services) before enabling `SLINT_LIVE_PREVIEW`.
