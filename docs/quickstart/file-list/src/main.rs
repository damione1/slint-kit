// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, event2, ui};
slint::include_modules!();
#[derive(Clone, PartialEq, Eq, Hash)] struct ResourceId(&'static str);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let files = KeyedModel::new([(ResourceId("brief"), ui::FileViewEntry {name: "Brief.pdf".into(), detail: "Project document".into(), removable: true, ..Default::default()}), (ResourceId("photo"), ui::FileViewEntry {name: "Reference.png".into(), removable: true, ..Default::default()})]).unwrap();
    app.set_entries(files.model());
    app.on_remove(event2(&app, move |app, token, generation| { if let Some(key) = files.remove_fenced(token, generation) { app.set_status(format!("Removed: {}", key.0).into()); } }));
    app.run()
}
