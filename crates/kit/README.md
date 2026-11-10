# Slint Kit

**Slint Kit is a Rust UI component library for Slint, combining 127 native components with shared interaction logic and typed domain values.** It supplies validated forms, exact money and decimals, stable-key choices, virtualized data views and complete light/dark themes. The catalogue includes ready-to-use controls and composition primitives.

Fields retain incomplete native drafts while `FieldSpec` returns checked values to Rust. `Choices` and `KeyedModel` preserve domain identity through filtering and sorting and reject stale gestures. Your application supplies its rules, data and business commands; Slint supplies rendering and native editing (caret, selection, undo, IME).

Build **desktop Rust applications** for macOS, Windows and Linux. Android and iOS remain unqualified Slint integration paths rather than announced kit targets. See [platform setup and limits](https://slint-kit.dev/guides/project-setup/#desktop-and-mobile-targets).

<p>
  <img alt="Virtualized data table" src="https://slint-kit.dev/assets/components/data-table@2x.png" width="32%">
  <img alt="Calendar in dark theme" src="https://slint-kit.dev/assets/components/calendar-dark@2x.png" width="32%">
  <img alt="Dialog" src="https://slint-kit.dev/assets/components/dialog-surface@2x.png" width="32%">
</p>

**[Browse all components, in light and dark →](https://slint-kit.dev/components/)** · [Getting started](https://slint-kit.dev/guides/getting-started/) · [Forms guide](https://slint-kit.dev/guides/forms/) · [Documentation en français](https://slint-kit.dev/fr/)

[Why Slint Kit: behavior, responsibilities and limits](https://slint-kit.dev/guides/why-slint-kit/) · [Pourquoi Slint Kit](https://slint-kit.dev/fr/guides/why-slint-kit/)

## Install

```toml
[dependencies]
slint-kit = "0.7"
slint = { version = "~1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.7"
```

Compile your UI in `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Import public components in Slint:

```slint
import { KitButton, KitTextField } from "@kit";
```

Call `slint_kit::install(&window)` once per window before showing it. It installs validation and the pure calculation services; component state stays local to each instance.

Use `slint_kit` in Rust for strict domain values, `Choices<K>`, `FieldSpec<T,E>` and `KeyedModel<K,Row>`. Enable `native` for asynchronous file/directory bindings, or `native-winit` for the explicit winit file-drop bridge. The manifest above is a desktop example; choose the Slint backend and renderer for your target as described in [Project setup](https://slint-kit.dev/guides/project-setup/). Keep `slint-kit` and `slint-kit-build` on the same release.

## Why Slint Kit

- **Typed forms:** input is parsed once into typed values, with HTML-style constraints and draft/commit/cancel handling.
- **Stable identity:** selection, sorting and editing follow your business keys, never row indexes.
- **Large collections:** virtualized tables and lists render only what is visible.
- **Exact values:** money and decimals use exact arithmetic.
- **Complete themes:** every state is designed in light and dark, and checked at 200% text size.

Try [email validation](https://slint-kit.dev/components/email-field/) or [sorting while retaining selection](https://slint-kit.dev/guides/table-example/) in a runnable project. Each component reference includes a demo, public API and complete download.

## Adoption and scope

Slint Kit is before 1.0; breaking changes remain possible. The build helper uses Slint’s experimental module compilation. Keep runtime/build releases matched. Tables display text, badge and check cells and edit field cells in place; modal surfaces are internal to a host window and require background-interaction gating by the application. The [selection guide](https://slint-kit.dev/guides/why-slint-kit/) explains supported use cases and qualification limits.

## Quality

No `unsafe` in the kit's hand-written Rust. [CI](https://github.com/damione1/slint-kit/actions/workflows/check.yml) runs `cargo fmt`, `clippy` with warnings denied, behavior tests (keyboard, pointer, focus, identity, geometry), generated-file verification and a lint that rejects hard-coded design values. Public CI runs the full suite on macOS and build/helper checks on Windows; the [Linux job passed headless component/workspace tests and clippy on Ubuntu 24.04 at `59ece023`](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933). Native runtime behavior, mobile, IME and screen readers need verification on the target device.

## Requirements

Rust 1.92 or later and Slint 1.18 (patch releases from 1.18.1). Announced desktop platforms: macOS, Windows and Linux; [announced and qualified targets](https://slint-kit.dev/guides/getting-started/) names the OS, backend and renderer of each proof. Android and iOS are not announced kit targets; Slint mobile integration requires separate platform setup and device qualification. Optional `native` file/directory dialogs are desktop services and return `SelectionOutcome::Failed(HostError::Unsupported)` on mobile. [Source, showcase and bilingual documentation](https://github.com/damione1/slint-kit).

## License

**Forever free for open-source use.** The base Slint Kit components remain free for open-source projects under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

**Commercial license — coming, on request.** [Contact the author](https://github.com/damione1/slint-kit/discussions) about proprietary applications and availability. **Slint Kit Pro**, a separate set of advanced components, is also coming.

Les composants de base restent gratuits pour toujours pour l’usage open source, sous GPL-3.0-or-later. La licence commerciale est à venir, sur demande en contactant l’auteur ; Slint Kit Pro est également à venir.
