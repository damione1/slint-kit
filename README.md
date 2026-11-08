<div align="center">

<!-- Match GitHub's selected theme even when it differs from the system theme. -->
<img src="assets/branding/slint-kit-stacked.svg#gh-light-mode-only" alt="Slint Kit" width="240">
<img src="assets/branding/slint-kit-stacked-dark.svg#gh-dark-mode-only" alt="Slint Kit" width="240">

<h1>Slint Kit</h1>

<p><strong>Native components. Your visual language.</strong></p>

<p>A Rust GUI component library for desktop and mobile apps built with Slint.<br>Typed forms, native text editing and customizable light/dark themes.</p>

<p>
  <a href="https://github.com/damione1/slint-kit/actions/workflows/check.yml"><img alt="Kit checks on main" src="https://github.com/damione1/slint-kit/actions/workflows/check.yml/badge.svg?branch=main"></a>
  <a href="https://crates.io/crates/slint-kit"><img alt="crates.io version" src="https://img.shields.io/crates/v/slint-kit.svg"></a>
  <a href="LICENSE"><img alt="License: GPL v3 or later" src="https://img.shields.io/badge/License-GPLv3%2B-blue.svg"></a>
  <img alt="Rust 1.92 or later" src="https://img.shields.io/badge/Rust-1.92%2B-B7410E?logo=rust&amp;logoColor=white">
  <a href="https://slint.dev"><img alt="Slint 1.18.1" src="https://img.shields.io/badge/Slint-1.18.1-818CF8"></a>
  <img alt="Native UI" src="https://img.shields.io/badge/UI-native-0D9488">
  <img alt="No unsafe code in the kit's hand-written Rust" src="https://img.shields.io/badge/kit_unsafe-none-success">
  <a href="https://docs.rs/slint-kit"><img alt="docs.rs" src="https://img.shields.io/docsrs/slint-kit"></a>
</p>

<p><strong>125+ native components</strong> · typed forms and validation · virtualized tables and lists · dates, times and exact decimals · light/dark themes</p>

</div>

<table>
  <tr>
    <td align="center" width="33%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://slint-kit.dev/assets/components/data-table-dark@2x.png">
        <img alt="KitDataTable: virtualized, sortable data table" src="https://slint-kit.dev/assets/components/data-table@2x.png">
      </picture>
      <br><sub><a href="https://slint-kit.dev/components/data-table/">Virtualized data table</a></sub>
    </td>
    <td align="center" width="33%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://slint-kit.dev/assets/components/calendar-dark@2x.png">
        <img alt="KitCalendar: keyboard-navigable calendar" src="https://slint-kit.dev/assets/components/calendar@2x.png">
      </picture>
      <br><sub><a href="https://slint-kit.dev/components/calendar/">Calendar</a></sub>
    </td>
    <td align="center" width="33%">
      <picture>
        <source media="(prefers-color-scheme: dark)" srcset="https://slint-kit.dev/assets/components/dialog-surface-dark@2x.png">
        <img alt="KitDialogSurface: modal dialog" src="https://slint-kit.dev/assets/components/dialog-surface@2x.png">
      </picture>
      <br><sub><a href="https://slint-kit.dev/components/dialog-surface/">Dialog</a></sub>
    </td>
  </tr>
</table>

<p align="center"><a href="https://slint-kit.dev/components/"><strong>Browse all components, in light and dark →</strong></a></p>

Buttons, fields, selections, dialogs, calendars, virtualized data views and composable display, navigation and layout primitives. Rust owns validated values and identity; Slint owns their presentation and native interaction. The built-in visual system includes light/dark modes and Lucide icons. Inter is an optional font import; local styles and custom theme values remain supported.

[Website](https://slint-kit.dev/) · [Documentation](https://slint-kit.dev/guides/getting-started/) · [Components](https://slint-kit.dev/components/) · [docs.rs](https://docs.rs/slint-kit)

## Desktop and mobile

Build Rust interfaces for **macOS, Windows and Linux**, and reuse Slint Kit components in **Android and iOS** applications through Slint's mobile backends. Android uses `backend-android-activity-06`; iOS uses winit and Skia, with Xcode for packaging. See [mobile project setup](https://slint-kit.dev/guides/project-setup/#desktop-and-mobile-targets) for prerequisites and platform limits.

## Why Slint Kit

Slint ships a solid set of standard widgets. Slint Kit adds what business applications need on top of them:

- **Typed forms.** `FieldSpec<T, E>` parses input once into a typed value, with HTML-style constraints (required, email, URL, ranges) and draft/commit/cancel handling, while native caret, selection, undo and IME stay with Slint.
- **Stable identity, not row indexes.** `Choices<K>` and `KeyedModel<K, Row>` keep your business keys in Rust, so selection, sorting and editing survive model changes.
- **Large collections.** Virtualized tables and lists render only what is visible.
- **Exact values.** Money and decimals use exact arithmetic; floats are only a rendering projection.
- **One theme, every state.** Hover, pressed, focus, disabled, read-only, invalid and selected states are designed in light and dark, and verified at 200% text size.

## Engineering guarantees

Every change passes `cargo run -p xtask -- check` in CI:

- No `unsafe` in the kit's hand-written Rust; the domain core and adapters use `#![forbid(unsafe_code)]`.
- `cargo fmt` and `clippy` with warnings denied; wildcard match arms on domain enums are rejected.
- Behavior tests cover keyboard, pointer, focus, identity and geometry, alongside domain logic. See the [CI results](https://github.com/damione1/slint-kit/actions/workflows/check.yml).
- Generated files (shared contracts, Lucide icons) are verified against their generators.
- A token lint rejects hard-coded colors, sizes and durations in components: everything comes from the theme.
- Strict types: closed sets are enums, identifiers are newtypes, invalid states are unrepresentable. See the [typed controllers guide](https://slint-kit.dev/guides/controllers/) for application examples.

## Try it

Install the toolkit from [crates.io](https://crates.io/crates/slint-kit) with Rust 1.92 or later and the native dependencies required by Slint:

```toml
[dependencies]
slint-kit = "0.7"
slint = "~1.18.1"

[build-dependencies]
slint-kit-build = "0.7"
```

Compile your Slint UI with `slint_kit_build::compile("ui/app.slint")` in `build.rs`. Import components from `@kit`. See the [installation guide](https://slint-kit.dev/guides/getting-started/) or [consumption skill](.agents/skills/slint-kit/SKILL.md) for a complete application.

For `@kit` completion and diagnostics in VS Code or Zed, configure the language server separately with the project-relative paths in [Editor setup](https://slint-kit.dev/guides/editor/).


## Build the showcase

The showcase is a working application that demonstrates the public components. Build it from the source repository with Git and **Rust 1.92+**.

Prepare the desktop dependencies for your OS: Xcode Command Line Tools on macOS, an MSVC Rust toolchain and Visual Studio Build Tools on Windows, or the required development libraries on Linux. See the [Slint 1.18.1 build guide](https://github.com/slint-ui/slint/blob/v1.18.1/docs/building.md#prerequisites) for platform setup.

```sh
git clone https://github.com/damione1/slint-kit.git
cd slint-kit
cargo build --release --locked -p kit-showcase
cargo run --release --locked -p kit-showcase
```

## Contribute

Slint Kit is heading to 1.0. Until then, pull requests are paused so the foundation can stabilize quickly; contributions will open after 1.0.

[Issues](https://github.com/damione1/slint-kit/issues) and [discussions](https://github.com/damione1/slint-kit/discussions) are welcome now for bug reports, questions and feedback.

## License

Slint Kit is open source under the [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

**Commercial license.** From 1.0, a commercial license will be available for proprietary applications that can't use the GPL. **Slint Kit Pro**, a set of advanced components, is also in preparation. Interested? [Open a discussion](https://github.com/damione1/slint-kit/discussions) or get in touch through [slint-kit.dev](https://slint-kit.dev/).
