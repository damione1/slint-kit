// SPDX-License-Identifier: GPL-3.0-or-later
//! # Slint Kit
//!
//! 125+ native UI components for Rust and Slint: typed forms and validation,
//! virtualized tables and lists, date and time pickers, exact decimal fields,
//! dialogs, navigation and layout, with complete light and dark themes.
//!
//! Rust owns validated values and stable identity; Slint owns presentation and
//! native interaction (caret, selection, undo, IME).
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
//!
//! Enable the `native` feature for asynchronous file and directory dialogs, or
//! `native-winit` for the winit file-drop bridge. File and directory dialogs
//! return `SelectionOutcome::Failed(HostError::Unsupported)` on Android and iOS;
//! native editing and accessibility
//! require verification on the target device.
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

// Keep generated Slint implementation in its own test-only module; the native
// adapter retains forbid(unsafe_code), including its handwritten tests.
#[cfg(all(test, feature = "native"))]
mod native_test_fixture {
    slint::slint! { export component FileBindingTestWindow inherits Window {} }
}
