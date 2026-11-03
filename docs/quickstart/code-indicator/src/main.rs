// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use std::{cell::Cell, rc::Rc};
mod ui { slint::include_modules!(); }

const CODE_LENGTH: i32 = 6;

fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    app.set_code_length(CODE_LENGTH);
    // The host owns the count; the indicator only draws it.
    let entered = Rc::new(Cell::new(0));
    let (weak, count) = (app.as_weak(), entered.clone());
    app.on_digit(move || {
        if let Some(app) = weak.upgrade() { count.set((count.get() + 1).min(CODE_LENGTH)); app.set_entered(count.get()); }
    });
    let (weak, count) = (app.as_weak(), entered);
    app.on_delete_requested(move || {
        if let Some(app) = weak.upgrade() { count.set((count.get() - 1).max(0)); app.set_entered(count.get()); }
    });
    app.run()
}
