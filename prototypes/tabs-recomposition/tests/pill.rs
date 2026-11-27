// SPDX-License-Identifier: GPL-3.0-or-later
//! The kit's tab suites, unmodified, against the application-built pill tabs.
mod ui {
    pub use slint_kit::ui::*;
    include!(concat!(env!("OUT_DIR"), "/pill.rs"));
}
pub use ui::*;

#[path = "../../../tests/src/tabs_panels.rs"]
mod tabs_panels;

#[path = "../../../tests/src/composite_accessibility.rs"]
mod composite_accessibility;

#[path = "shared/reveal.rs"]
mod reveal;
