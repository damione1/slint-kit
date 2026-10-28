// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;

mod ui { slint::include_modules!(); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let weak = app.as_weak();
    app.on_navigate(move |step| { if let Some(app) = weak.upgrade() {
        let page = app.get_page();
        let next = match step {
            ui::NavigationStep::Next | ui::NavigationStep::PageNext => (page + 1).min(3),
            ui::NavigationStep::Previous | ui::NavigationStep::PagePrevious => (page - 1).max(0),
            ui::NavigationStep::First => 0,
            ui::NavigationStep::Last => 3,
        };
        app.set_page(next);
    } });
    app.run()
}
