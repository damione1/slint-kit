// SPDX-License-Identifier: GPL-3.0-or-later
slint::include_modules!();
fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::install(&app);
    let count = std::cell::Cell::new(0_u32);
    app.on_action_requested(slint_kit::event(&app, move |app, action| {
        let next = count.get().checked_add(1).expect("action count capacity");
        count.set(next);
        app.set_status(format!("{action:?} handled by Rust ({next})").into());
    }));
    app.run()
}
