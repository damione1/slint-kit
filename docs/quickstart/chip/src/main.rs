// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{KeyedModel, ui};
slint::include_modules!();
#[derive(Clone, PartialEq, Eq, Hash)] struct TagId(&'static str);
fn tags() -> impl Iterator<Item=(TagId, ui::ChipRow)> { ["Design", "Research", "Accessibility", "Rust", "Mobile", "Documentation"].into_iter().map(|label| (TagId(label), ui::ChipRow {label: label.into(), removable: true, ..Default::default()})) }
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let tags = KeyedModel::new(tags()).unwrap();
    app.set_chips(tags.model());
    let removal = tags.clone();
    app.on_remove(move |token| { removal.remove(token); });
    app.on_reset(move || { tags.replace(self::tags()).unwrap(); });
    app.run()
}
