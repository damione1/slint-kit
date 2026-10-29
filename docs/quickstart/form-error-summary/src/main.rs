// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use std::cell::Cell;
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    let sequence = Cell::new(0_i32);
    let weak = app.as_weak();
    app.on_request_focus(move |mut target: FocusRequest| {
        if let Some(app) = weak.upgrade() {
            let request = sequence
                .get()
                .checked_add(1)
                .expect("focus request capacity");
            sequence.set(request);
            target.request = request;
            app.set_focus_request(target);
        }
    });
    app.run()
}
