// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{Ownership, SelectOption, SelectionIntent};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Destination { Overview, Activity, Settings }
impl Destination {
    fn label(self) -> &'static str {
        match self { Self::Overview => "Overview", Self::Activity => "Activity", Self::Settings => "Settings" }
    }
}

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    let selection = slint_kit::bind_select!(ui, app, vec![
        SelectOption::new(Destination::Overview, "Overview"),
        SelectOption::new(Destination::Activity, "Activity"),
        SelectOption::new(Destination::Settings, "Settings"),
    ], Some(Destination::Overview), Ownership::Local);
    let weak = app.as_weak();
    selection.on_intent(move |intent| {
        if let Some(app) = weak.upgrade() {
            app.set_selected_label(match intent {
                SelectionIntent::Commit(key) => key.label(),
                SelectionIntent::Clear => "No selection",
            }.into());
        }
    });
    selection.open();
    fn publish(app: &ui::App, expanded: Option<Destination>) {
        use slint::Model;
        let rows = app.get_rows().iter().map(|item| {
            let key = match item.token { 0 => Some(Destination::Overview), 1 => Some(Destination::Activity), 2 => Some(Destination::Settings), _ => None };
            ui::DisclosureRow { item: ui::OptionRow { description: "Details for this project section.".into(), ..item }, expanded: key.is_some() && key == expanded }
        }).collect::<Vec<_>>();
        app.set_disclosures(slint::ModelRc::new(slint::VecModel::from(rows)));
        app.set_selected_label(expanded.map_or("All sections collapsed", Destination::label).into());
    }
    publish(&app, None);
    let weak = app.as_weak();
    app.on_expand(move |token, generation, expanded, _mode| {
        if let Some(app) = weak.upgrade() {
            if generation != app.get_select_state().generation { return; }
            let key = match token { 0 => Some(Destination::Overview), 1 => Some(Destination::Activity), 2 => Some(Destination::Settings), _ => None };
            if key.is_some() { publish(&app, if expanded { key } else { None }); }
        }
    });
    app.run()
}
