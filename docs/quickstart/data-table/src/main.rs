// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{KeyedModel, event, ui};
slint::include_modules!();
#[derive(Clone, Debug, PartialEq, Eq, Hash)] struct PersonName(&'static str);
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?; slint_kit::install(&app);
    let people = [("Elliot", "Design", "Active"), ("Darlene", "Engineering", "Active"), ("Angela", "Research", "Active"), ("Tyrell", "Engineering", "Active"), ("Dom", "Design", "Active"), ("Leon", "Research", "Invited"), ("Trenton", "Engineering", "Active"), ("Romero", "Design", "Invited")];
    let rows = KeyedModel::new(people.map(|(name, team, status)| (PersonName(name), ui::TableRow {cells: ModelRc::new(VecModel::from(vec![name.into(), team.into(), status.into()])), ..Default::default()}))).unwrap();
    rows.bind(&app, App::set_rows, App::set_generation, App::set_active_index);
    app.on_choose(rows.activation(event(&app, |app, key: PersonName| app.set_selected_label(format!("Selected: {}", key.0).into()))));
    app.on_sort(rows.sorting_by(|left, right| left.0.cmp(right.0), event(&app, |app, direction| app.set_name_sort(direction))));
    app.on_move(rows.navigation());
    app.run()
}
