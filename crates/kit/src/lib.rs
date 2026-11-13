// SPDX-License-Identifier: GPL-3.0-or-later
//! # Slint Kit
//!
//! Slint Kit is a Rust UI component library for Slint, combining 127 native
//! components with shared interaction logic and typed domain values. It supplies
//! validated forms, exact money and decimals, stable-key choices, virtualized
//! data views and complete light/dark themes. The catalogue includes ready-to-use
//! controls and composition primitives.
//!
//! Fields retain incomplete native drafts while `FieldSpec` returns checked
//! values to Rust. `Choices` and `KeyedModel` preserve domain identity through
//! filtering and sorting and reject stale gestures. Your application supplies
//! rules, data and business commands; Slint supplies rendering and native editing
//! (caret, selection, undo, IME).
//!
//! [Why Slint Kit: behavior, responsibilities and limits](https://slint-kit.dev/guides/why-slint-kit/)
//! · [Documentation en français](https://slint-kit.dev/fr/guides/why-slint-kit/)
//!
//! Build desktop Rust applications for macOS, Windows and Linux, or mobile
//! applications for Android and iOS through Slint's mobile backends. See
//! [platform setup and limits](https://slint-kit.dev/guides/project-setup/#desktop-and-mobile-targets).
//!
//! **[Browse all components](https://slint-kit.dev/components/)** ·
//! [Getting started](https://slint-kit.dev/guides/getting-started/) ·
//! [Forms guide](https://slint-kit.dev/guides/forms/)
//!
//! ## Setup
//!
//! Add `slint-kit` and `slint` to `[dependencies]` and `slint-kit-build` to
//! `[build-dependencies]`, then compile the UI in `build.rs`:
//!
//! ```ignore
//! fn main() {
//!     slint_kit_build::compile("ui/app.slint").expect("compile application UI");
//! }
//! ```
//!
//! Import components from `@kit` in Slint:
//!
//! ```slint
//! import { KitButton, KitTextField } from "@kit";
//! ```
//!
//! Call [`install`] once per window before showing it. It installs validation
//! and the calculation services; component state stays local to each instance.
//!
//! ## Rust building blocks
//!
//! - `FieldSpec<T, E>`: parse input once into a typed value, with HTML-style
//!   constraints and draft/commit/cancel handling.
//! - `Choices<K>`: options and selections keyed by your own stable keys.
//! - `KeyedModel<K, Row>`: models with stable-key sorting and active-row editing.
//! - `Store<T>`: a typed value shared by Rust and the windows that observe it,
//!   projected into a Slint `global`, with a `Send` handle for other threads.
//!
//! Enable the `native` feature for asynchronous file and directory dialogs, or
//! `native-winit` for the winit file-drop bridge. File and directory dialogs
//! return `SelectionOutcome::Failed(HostError::Unsupported)` on Android and iOS;
//! native editing and accessibility
//! require verification on the target device.
//!
//! Slint Kit is before 1.0; breaking changes remain possible. Its build helper
//! uses Slint's experimental module compilation. Tables currently display text
//! cells, and modal surfaces operate inside their host window with host-managed
//! background interaction. Public CI runs full checks on macOS and build/helper
//! checks on Windows; other targets need their own runtime qualification.
//!
//! Custom skins compose the same public primitives. See the
//! [themes guide](https://slint-kit.dev/guides/themes/).
pub use kit_core::*;
pub use kit_slint::*;

#[cfg(feature = "native")]
pub use kit_native as native;

#[cfg(feature = "native")]
mod native_bindings;
#[cfg(feature = "native")]
pub use native_bindings::*;

#[cfg(all(test, feature = "native"))]
mod native_test_ui {
    slint::slint! {
        export component PickerTestWindow inherits Window {
            in-out property <string> error;
        }
    }
}
