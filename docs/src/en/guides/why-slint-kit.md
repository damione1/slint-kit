---
layout: base.njk
locale: en
title: "Why Slint Kit"
translationKey: "guides/why-slint-kit/"
permalink: "{{ 'guides/why-slint-kit/' | localeUrl(locale) }}index.html"
---

# What Slint Kit adds to a Rust and Slint application

**Slint Kit supplies UI components and a shared behavior layer for Rust applications built with Slint.** It combines native text editing, typed field acceptance, stable collection identity, exact domain values and a customizable theme. The {{ components.length }} public components include complete controls and smaller composition primitives. Your application keeps ownership of business rules, data and commands.

Choose it when your application needs forms, editable values, searchable choices and data views that share consistent behavior. Start with [installation](/guides/getting-started/), or inspect the [component catalogue](/components/) and runnable examples before adopting it.

## Slint, Slint Kit and your application

[Slint](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/overview/) provides the declarative language, rendering, window backends, native editor and standard widgets. Slint Kit is an independent library built on Slint, distributed through Cargo and imported as `@kit`. Its Rust services and Slint components work together; the browser demos run those same components and Rust code compiled to WebAssembly.

| Need | Slint Kit provides | Your application defines |
| --- | --- | --- |
| Validated forms | Shared HTML-style constraints, native drafts, commit/cancel transitions and `FieldSpec<T, E>` acceptance | Domain parsers and rules, persistence and a guarded submit command |
| Searchable choices | `Choices<K>`, keyboard navigation, disabled options and stable selection tokens | Your key type, available choices and the resulting action |
| Tables and lists | Virtualized views and `KeyedModel<K, Row>` with stable identity and generation checks | Data loading, filtering and sort comparators |
| Exact numbers and dates | `Money`, `DecimalValue`, civil dates and explicit time-zone resolution | Currency, precision, availability and time-zone policies |
| Appearance | Complete light/dark themes, state recipes, sizing and icon roles | Brand values, localized text and themes for composed children |
| Files and directories | Optional asynchronous native bindings with selected, cancelled, failed and unsupported outcomes | File processing and handling the capabilities of the target platform |

## Behavior you can verify in an example

### A draft can be incomplete without becoming business data

Open the [email field](/components/email-field/), replace the address with `not-an-address` and try to apply it. The shared constraint engine rejects the candidate. A valid candidate passes through `FieldSpec::accept` at the Rust boundary, which returns a typed value. Native caret, selection, undo and composition remain in Slint’s editor.

The [forms guide](/guides/forms/) explains intrinsic constraints, domain rules and guarded submission. The [complete typed application](/guides/typed-example/) adds a custom project-name type and keyed workspace choice. A callback and disabled button alone are insufficient to validate an application's submit path.

### A sorted row keeps its identity

In the [sortable table application](/guides/table-example/), select Atlas, then sort Tasks. The selected project remains Atlas even when its row position changes. With `KeyedModel`, stable domain keys survive reordering and source generations reject a gesture that started against an older projection. Your comparator decides the order; your application decides which records to load.

### Formatting and rendering preserve domain values

The [money field](/components/money-field/) connects editing to exact `Money` values and currency rules. A [zoned date-time field](/components/zoned-date-time-field/) rejects a nonexistent local time and requires an explicit choice when a local time occurs twice. These rules are implemented in Rust and shared with component interaction. Renderer floats and formatted labels remain projections.

## Customize presentation while reusing behavior

[KitTheme and the public composition primitives](/guides/themes/) separate appearance from interaction. Colors, spacing, geometry, focus rings and durations come from the theme. Ready-to-use fields include labels and validation messages; smaller parts support custom layouts. Pass a theme explicitly to application-supplied children when composing a themed region.

`slint_kit::install(&window)` installs shared services once per window. Each field or selector retains its own local interaction state. Choose controlled ownership when a host decision must acknowledge a proposed change. The [controllers guide](/guides/controllers/) covers the typed boundaries and collection models.

## Evidence and adoption limits

The [public behavior tests](https://github.com/damione1/slint-kit/tree/main/tests/src) cover keyboard, pointer, focus, identity and geometry. In particular, [field tests](https://github.com/damione1/slint-kit/blob/main/tests/src/autonomous.rs) exercise invalid commits and independent instances, and [model tests](https://github.com/damione1/slint-kit/blob/main/tests/src/keyed_model.rs) exercise reordering and stale gestures. [CI results](https://github.com/damione1/slint-kit/actions/workflows/check.yml) expose the checks. Headless tests and screenshots do not establish IME, screen-reader or physical-device qualification.

The shipped appearance is one Tailwind-inspired visual system in light and dark modes. `KitDataTable` displays text, badge and check cells and edits field cells in place from the keyboard; caller-supplied cell templates are not available because Slint 1.18 does not accept `@children` inside repeated rows. Dialog surfaces are modals inside their host window: the application must disable background interaction and connect focus handling as documented in the [dialog reference](/components/dialog-surface/).

The [macOS headless job passed at `6f553989`](https://github.com/damione1/slint-kit/actions/runs/42776551241/job/129696271661) on the GitHub-hosted `macos-26-arm64` image: the complete check, downloadable starters and complete documentation applications. Published CI also runs build/helper checks on Windows. The [Linux job passed at `59ece023`](https://github.com/damione1/slint-kit/actions/runs/42769690387/job/129674322933) on Ubuntu 24.04 (x86_64): headless component tests using the Slint testing backend and software renderer, the remaining workspace tests and clippy with warnings denied. This qualifies that headless configuration; native Linux interaction, mobile, IME and screen readers still need target verification.

The current series is **{{ project.version }}{% if project.preOne %}, before 1.0{% endif %}**; breaking changes remain possible. Pull requests are paused until 1.0; issues and discussions remain open. Assess the project’s maintenance capacity and your upgrade budget for long-lived applications. Use matching runtime/build packages. The kit accepts Slint 1.18 patch releases from 1.18.1 (`~1.18.1`); downloadable examples lock their resolved dependencies. The build helper uses Slint’s experimental module compilation; see [build compatibility](https://docs.rs/crate/slint-kit-build/latest) and [editor preview limits](/guides/editor/#static-previews-and-rust-services).

**Forever free for open-source use.** The base kit components remain free for open-source projects. Code and bundled assets are [GPL-3.0-or-later](https://github.com/damione1/slint-kit/blob/main/LICENSE), with dependency notices preserved. Slint’s own license options do not relicense this kit. When distributing an application linked to a GPL library, the GPL applies to the combined program; private use within one organization does not by itself require public release. See the GNU FAQ on [GPL libraries](https://www.gnu.org/licenses/gpl-faq.en.html#IfLibraryIsGPL) and [internal use](https://www.gnu.org/licenses/gpl-faq.en.html#InternalDistribution). **A commercial license is coming, on request** for proprietary applications: [contact the author](/contact/) to discuss needs and availability. **Slint Kit Pro**, a separate set of advanced components, is also coming. The [platform guide](/guides/project-setup/#desktop-and-mobile-targets) distinguishes desktop and mobile setup. Optional native file dialogs are desktop services and return an unsupported outcome on mobile.

Slint’s standard widgets remain a reasonable starting point when they cover your needs. Evaluate Slint Kit when you want to reuse the typed form, identity and interaction layer described here. Prototype your critical screen against the documented examples and qualify its behavior on your actual backend and devices.
