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

An announced target is one the kit is built for. A qualified target is one with a named proof: a check that runs on a named OS, backend and renderer, and whose result you can read. The table lists every proof the kit publishes. The continuous-integration proofs run on every push to `main` and on every pull request; their current result is the latest run of the [Check workflow](https://github.com/damione1/slint-kit/actions/workflows/check.yml), not a claim made here. Each result below is a recorded past run, not the current state.

| Target | Backend and renderer | What the proof checks | Where |
| --- | --- | --- | --- |
| macOS, GitHub-hosted `macos-26-arm64` image (past run; CI no longer runs on macOS) | Slint testing backend with the software renderer; no window | The complete `xtask check` (including 314 component and 42 showcase tests), downloadable starters and complete documentation applications passed at `6f553989` on 2026-11-10 | [Successful `desktop` job](https://github.com/damione1/slint-kit/actions/runs/42776551241/job/129696271661) |
| Linux, GitHub-hosted `ubuntu-24.04` runner (x86_64) | Slint testing backend with the software renderer; no window | The Check workflow now runs the complete `xtask check` here. Last recorded result: headless component tests, the remaining workspace tests and clippy with warnings denied passed at `59ece023` on 2026-11-10 | [Successful `linux` job](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933) |
| Windows, GitHub-hosted `windows-2025` runner (x86_64) | None: compile only | `cargo check` compiles the kit and its test crate; the build helper's path tests run. No component test runs and no window opens | `windows` job of the Check workflow |
| macOS arm64, maintainer workstation (Darwin 27, Apple M4 Pro, scale factors 1 and 2) | winit backend with the Skia renderer; native window | Component screenshots rendered natively in light and dark themes and at 200% text; the button's pointer click and Tab/Space paths checked in fresh native windows, in French and English | Captions on each component page; `docs/content/captures.json` records the environment and the qualification scope |

Not qualified on any target (the procedures to do so are in the next section): pointer and keyboard interaction in a real winit window on Windows and Linux (X11 and Wayland), IME composition on any OS, screen readers (VoiceOver, Narrator, NVDA, Orca), renderers other than Skia on the capture machine, touch, mobile and web. A headless test proves the component contract on the testing backend; a capture proves the rendering of a declared state in the capture environment. Neither is a native, IME or screen-reader qualification. Such a qualification is a manual pass on the actual OS, backend, renderer and assistive technology, reported with those names; the changelog says which features still need one. When you qualify a target for your application, record the same names.

### Qualify IME and screen readers

IME composition and screen readers each have their own manual procedure. No CI job, headless test or capture replaces them. Run one on the real OS, backend, renderer and device, then publish the result with those names and the kit version or commit.

**IME composition.** Use a text control (`KitTextField`, `KitTextArea`, `KitMaskedField`) in a real winit window, with the platform's own input method: for example Japanese or Chinese on macOS and Windows, and IBus or Fcitx on Linux under X11 and Wayland.

1. Focus the control and start a composition. Check that the preedit text shows and that the candidate window opens next to the caret.
2. Commit one candidate. Check that the field holds the committed text only once and fires its accepted callback once.
3. Press Enter and Escape during composition. Check that they act on the composition and that neither submits the form nor cancels the draft.
4. Compose in a multiline control, in a masked control and in a control with `maxlength`. Record which constraints apply to the committed text.

**Screen readers.** Test VoiceOver on macOS, Narrator and NVDA on Windows, and Orca on Linux. Use the same screen for each reader.

1. Tab through the screen. Check that each control announces its name, role, state (checked, expanded, invalid, read-only, disabled) and description.
2. Edit a field. Check that the new value, the validation message and the progress state are announced once.
3. Open and close a dialog, a popup and a menu. Check that focus moves in and returns to the opener.
4. Use the reader's own actions (for example VoiceOver `VO+Space`) to press buttons and set slider values.

**Record.** For each pass, write the target (OS and version, backend, renderer, device or reader and version), the components tested, the date, the commit, and each step as passed, failed or not tested. Link an issue for every failure. A step you did not run stays "not tested"; do not infer it from another target.

Code and bundled assets are [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE); preserve dependency notices.

Editor previews show static declarations and do not run the Rust services. See [static previews and the Slint 1.18.1 live-preview limitation](../editor/#static-previews-and-rust-services) before enabling `SLINT_LIVE_PREVIEW`.
