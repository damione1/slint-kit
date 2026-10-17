// SPDX-License-Identifier: GPL-3.0-or-later
//! Public facade for the independent Slint Kit.
//!
//! Compile Slint sources using `kit_build::compile` in the application's build
//! script, then import controls from `@kit/button.slint`, `@kit/select.slint`,
//! and `@kit/dialog.slint`. Custom skins compose the same public primitives.
pub use kit_core::*;
pub use kit_slint::*;

#[cfg(feature = "native")]
pub use kit_native as native;
