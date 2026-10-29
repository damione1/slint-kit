// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Public model projection and weak-window binding helpers.
pub const UI_DIRECTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/ui");
#[doc(hidden)]
pub use kit_core as __core;
#[cfg(feature = "adapter")]
mod select;
#[cfg(feature = "adapter")]
pub use select::*;
#[cfg(feature = "adapter")]
mod macros;

#[cfg(feature = "adapter")]
mod field;
#[cfg(feature = "adapter")]
pub use field::*;

#[cfg(feature = "adapter")]
mod html;
#[cfg(feature = "adapter")]
pub use html::*;

#[cfg(all(feature = "adapter", target_arch = "wasm32"))]
mod web_phone;
#[cfg(all(feature = "adapter", target_arch = "wasm32"))]
#[doc(hidden)]
pub use web_phone::*;
