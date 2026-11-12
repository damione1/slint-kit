// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{AlphaPolicy, SrgbColor};
use std::{cell::Cell, rc::Rc};
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let draft = Rc::new(Cell::new(SrgbColor {
        red: 124,
        green: 108,
        blue: 240,
        alpha: 255,
    }));
    app.set_draft_color(draft.get().into());
    app.set_status(format!("Draft: {}", draft.get().hex(AlphaPolicy::Opaque)).into());
    let (weak, editing) = (app.as_weak(), draft.clone());
    app.on_color_chosen(move |view| {
        if let Ok(color) = SrgbColor::try_from(view) {
            editing.set(color);
            if let Some(app) = weak.upgrade() {
                app.set_draft_color(color.into());
                app.set_status(format!("Draft: {}", color.hex(AlphaPolicy::Opaque)).into());
            }
        }
    });
    let weak = app.as_weak();
    app.on_submit_color(move || {
        let color = draft.get();
        if let Some(app) = weak.upgrade() {
            app.set_status(format!("Applied: {}", color.hex(AlphaPolicy::Opaque)).into());
        }
    });
    app.run()
}
