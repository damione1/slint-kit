// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, event, ui::ChipRow};
mod ui { slint::include_modules!(); }
#[derive(Clone, PartialEq, Eq, Hash)] struct TagName(String);
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app);
    let tags = KeyedModel::new([(TagName("Design".into()), ChipRow {label: "Design".into(), removable: true, ..Default::default()})]).unwrap();
    app.set_chips(tags.model());
    let add = tags.clone();
    app.on_create(event(&app, move |app, draft: slint::SharedString| { let label = draft.trim(); if label.is_empty() || label.chars().count() > 40 { app.set_error("Use 1 to 40 characters".into()); return; } if add.insert(TagName(label.into()), ChipRow {label: label.into(), removable: true, ..Default::default()}).is_err() { app.set_error("Tag already exists".into()); return; } app.set_draft("".into()); app.set_error("".into()); }));
    app.on_remove(move |token| { tags.remove(token); });
    app.run()
}
