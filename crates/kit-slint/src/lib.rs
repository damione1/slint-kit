// SPDX-License-Identifier: GPL-3.0-or-later
//! Public Slint components and Rust behavior adapters for Slint Kit: native
//! drafts, typed field acceptance, shared interaction services and stable-key
//! collection models, with customizable light/dark themes.
//!
//! Applications normally use the `slint-kit` facade and matching
//! `slint-kit-build` helper to import components from `@kit`.
//! See [components](https://slint-kit.dev/components/) and
//! [behavior and responsibilities](https://slint-kit.dev/guides/why-slint-kit/).
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
#[doc(hidden)]
pub use paste::paste as __form_identifiers;

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

#[cfg(feature = "adapter")]
pub mod ui {
    slint::include_modules!();
}

#[cfg(feature = "adapter")]
mod callback;
#[cfg(feature = "adapter")]
pub use callback::{action, event, event2, event3};

#[cfg(feature = "adapter")]
mod data_service;
#[cfg(feature = "adapter")]
mod selection_service;

#[cfg(feature = "adapter")]
mod calendar;
#[cfg(feature = "adapter")]
pub use calendar::*;

/// Install shared pure services once per window, before showing the UI.
/// Components keep their own state; repeated installation is harmless.
#[cfg(feature = "adapter")]
pub fn install<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitFormValidation<'a>: slint::Global<'a, W>,
    for<'a> ui::KitCalendarService<'a>: slint::Global<'a, W>,
    for<'a> ui::KitFieldService<'a>: slint::Global<'a, W>,
    for<'a> ui::KitSelectionService<'a>: slint::Global<'a, W>,
    for<'a> ui::KitTimeService<'a>: slint::Global<'a, W>,
    for<'a> ui::KitDataService<'a>: slint::Global<'a, W>,
{
    html::install_validation(window);
    calendar::install_calendar(window);
    field_spec::install_fields(window);
    selection_service::install_selection(window);
    data_service::install_data(window);
    datetime::install_time(window);
}

#[cfg(feature = "adapter")]
mod field_spec;
#[cfg(feature = "adapter")]
pub use field_spec::*;

#[cfg(feature = "adapter")]
mod datetime;
#[cfg(feature = "adapter")]
pub use datetime::*;

#[cfg(feature = "adapter")]
mod store;
#[cfg(feature = "adapter")]
pub use store::*;

#[cfg(feature = "adapter")]
mod task;
#[cfg(feature = "adapter")]
pub use task::*;

#[cfg(feature = "adapter")]
mod keyed_model;
#[cfg(feature = "adapter")]
pub use keyed_model::*;

#[cfg(feature = "adapter")]
mod toast;
#[cfg(feature = "adapter")]
pub use toast::*;

#[cfg(feature = "adapter")]
mod table;
#[cfg(feature = "adapter")]
pub use table::*;

#[cfg(feature = "adapter")]
mod numeric_service;

#[cfg(feature = "adapter")]
mod color_service;
#[cfg(feature = "adapter")]
mod popup;

#[cfg(feature = "adapter")]
mod dialog;
#[cfg(feature = "adapter")]
pub use dialog::*;
#[cfg(feature = "adapter")]
mod text_edit;
#[cfg(feature = "adapter")]
pub use form::*;

#[cfg(feature = "adapter")]
mod form;

#[cfg(feature = "adapter")]
mod form_schema;
#[cfg(feature = "adapter")]
pub use form_schema::*;
