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
    let persistent = selection.clone();
    app.on_select_activate(move |token, generation| {
        persistent.activate(token, generation);
        persistent.open();
    });
    app.run()
}
