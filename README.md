<div align="center">

<!-- Match GitHub's selected theme even when it differs from the system theme. -->
<img src="assets/branding/slint-kit-stacked.svg#gh-light-mode-only" alt="Slint Kit" width="240">
<img src="assets/branding/slint-kit-stacked-dark.svg#gh-dark-mode-only" alt="Slint Kit" width="240">

<h1>Slint Kit</h1>

<p><strong>Typed behavior. Native components. Your visual language.</strong></p>

<p>A Rust UI component library for Slint applications, with shared interaction logic and typed domain values.<br>Validated forms, exact money, stable-key data views and customizable light/dark themes.</p>

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

<p><strong>127 native components</strong> · typed forms and validation · virtualized tables and lists · dates, times and exact decimals · light/dark themes</p>

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

**Slint Kit supplies reusable behavior for Rust business applications built with Slint.** Fields handle native drafts and validation; selection models retain stable domain keys; exact values stay typed in Rust. The catalogue includes complete controls and composition primitives for buttons, forms, dialogs, calendars, virtualized data, display, navigation and layout. Slint provides the renderer, window backends and native editor. The built-in visual system includes light/dark modes and Lucide icons. Inter is an optional font import; local styles and custom theme values remain supported.

[Why Slint Kit](https://slint-kit.dev/guides/why-slint-kit/) · [Website](https://slint-kit.dev/) · [Documentation](https://slint-kit.dev/guides/getting-started/) · [Components](https://slint-kit.dev/components/) · [docs.rs](https://docs.rs/slint-kit)

## Desktop and mobile

Build Rust interfaces for **macOS, Windows and Linux**, and reuse Slint Kit components in **Android and iOS** applications through Slint's mobile backends. Android uses `backend-android-activity-06`; iOS uses winit and Skia, with Xcode for packaging. See [mobile project setup](https://slint-kit.dev/guides/project-setup/#desktop-and-mobile-targets) for prerequisites and platform limits.

## Why Slint Kit

Slint Kit combines themed components with a shared Rust behavior layer. Your application supplies domain rules, data loading and business commands:

- **Typed forms.** `FieldSpec<T, E>` parses input once into a typed value, with HTML-style constraints (required, email, URL, ranges) and draft/commit/cancel handling, while native caret, selection, undo and IME stay with Slint.
- **Stable identity, not row indexes.** `Choices<K>` and `KeyedModel<K, Row>` keep your business keys in Rust, so selection, sorting and editing survive model changes.
- **Large collections.** Virtualized tables and lists render only what is visible.
- **Exact values.** Money and decimals use exact arithmetic; floats are only a rendering projection.
- **One theme, every state.** Hover, pressed, focus, disabled, read-only, invalid and selected states are designed in light and dark, and verified at 200% text size.

Try the behavior in a complete project: [reject an invalid email](https://slint-kit.dev/components/email-field/), [sort a table while retaining selection](https://slint-kit.dev/guides/table-example/), or [resolve an ambiguous local time](https://slint-kit.dev/components/zoned-date-time-field/). Each component reference includes an API, a runnable demo and a downloadable project.

## Adoption and scope

The current series is before 1.0 and breaking changes remain possible. The build helper uses Slint’s experimental module compilation; use matching runtime/build releases and the documented Slint version. `KitDataTable` displays text, badge and check cells and edits field cells in place, and modal surfaces operate inside their host window with application-managed background interaction. The shipped visual system is Tailwind-inspired, with light/dark modes and custom theme values. See [responsibilities and adoption limits](https://slint-kit.dev/guides/why-slint-kit/).

## Engineering guarantees

Every change passes `cargo run -p xtask -- check` in CI:

- No `unsafe` in the kit's hand-written Rust; the domain core and adapters use `#![forbid(unsafe_code)]`.
- A CI matrix with named checks: the full check on macOS, build checks on Windows, and a newly added Linux headless/clippy job whose first successful result is pending. [Announced and qualified targets](https://slint-kit.dev/guides/getting-started/) names the OS, backend and renderer of each proof; a headless test or a screenshot is not a native, IME or screen-reader qualification.
- `cargo fmt` and `clippy` with warnings denied; wildcard match arms on domain enums are rejected.
- Behavior tests cover keyboard, pointer, focus, identity and geometry, alongside domain logic. [Public CI](https://github.com/damione1/slint-kit/actions/workflows/check.yml) runs the full suite on macOS and build/helper checks on Windows; this does not establish Linux or mobile runtime qualification. IME and screen readers require checks on the target device.
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

## En français

Slint Kit associe **127 composants UI Rust et Slint** à une couche de comportements partagés : validation de formulaires, valeurs exactes, clés stables et thèmes personnalisables. Votre application fournit ses règles métier et ses données ; le kit fournit les mécanismes d’interaction réutilisables. Consultez [Pourquoi Slint Kit](https://slint-kit.dev/fr/guides/why-slint-kit/), les [formulaires typés](https://slint-kit.dev/fr/guides/forms/) et l’[installation](https://slint-kit.dev/fr/guides/getting-started/).

## Contribute

Slint Kit is heading to 1.0. Until then, pull requests are paused so the foundation can stabilize quickly; contributions will open after 1.0.

[Issues](https://github.com/damione1/slint-kit/issues) and [discussions](https://github.com/damione1/slint-kit/discussions) are welcome now for bug reports, questions and feedback.

## License

**Forever free for open-source use.** The base Slint Kit components remain free for open-source projects under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

**Commercial license — coming.** A commercial license for proprietary applications is planned, on request. [Contact the author](https://github.com/damione1/slint-kit/discussions) to discuss your needs and availability.

**Slint Kit Pro — coming.** A separate set of advanced components is in preparation. [Get in touch](https://github.com/damione1/slint-kit/discussions) for information.

Les composants du kit de base restent **gratuits pour toujours pour l’usage open source**, sous GPL-3.0-or-later. La licence commerciale est à venir, sur demande en [contactant l’auteur](https://github.com/damione1/slint-kit/discussions). Slint Kit Pro et ses composants avancés sont également à venir.
