// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use std::cell::Cell;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let count = Cell::new(0_u32);
    let weak = app.as_weak();
    app.on_action_requested(move |action| {
        if let Some(app) = weak.upgrade() {
            let next = count.get().checked_add(1).expect("action count capacity");
            count.set(next);
            let label = match action {
                DemoAction::SaveProject => "Save project",
            };
            app.set_status(format!("{label} handled by Rust ({next})").into());
        }
    });
    app.run()
}
