// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{Ownership, SelectOption, SelectionIntent};
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Score { One, Two, Three }
impl Score {
    fn label(self) -> &'static str {
        match self { Self::One => "1 star", Self::Two => "2 stars", Self::Three => "3 stars" }
    }
}

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let selection = slint_kit::bind_select!(ui, app, vec![
        SelectOption::new(Score::One, "1 star"),
        SelectOption::new(Score::Two, "2 stars"),
        SelectOption::new(Score::Three, "3 stars"),
    ], Some(Score::One), Ownership::Local);
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
    let target = selection.clone();
    let weak = app.as_weak();
    app.on_clear(move || { target.set_selected(None); target.open(); if let Some(app) = weak.upgrade() { app.set_selected_label("No rating".into()); } });
    let persistent = selection.clone();
    app.on_select_activate(move |token, generation| {
        persistent.activate(token, generation);
        persistent.open();
    });
    app.run()
}
