// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use slint_kit::{action, event};
use std::{cell::RefCell, rc::Rc};
mod ui { slint::include_modules!(); }
#[derive(Clone, Copy, PartialEq, Eq)] struct Digit(u8);
fn show(app: &ui::App, code: &[Digit], status: &str) { app.set_entered(i32::try_from(code.len()).expect("four digits")); app.set_can_confirm(code.len() == 4); app.set_status(status.into()); }
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?; slint_kit::install(&app); app.set_code_length(4);
    let code: Rc<RefCell<Vec<Digit>>> = Rc::default(); let entered = code.clone();
    app.on_digit(event(&app, move |app, value| { let mut code = entered.borrow_mut(); if let Ok(digit) = u8::try_from(value) && digit <= 9 && code.len() < 4 { code.push(Digit(digit)); } show(&app, &code, ""); }));
    let entered = code.clone(); app.on_delete_requested(action(&app, move |app| { let mut code = entered.borrow_mut(); code.pop(); show(&app, &code, ""); }));
    app.on_confirm(action(&app, move |app| { let mut code = code.borrow_mut(); if code.len() != 4 { return; } let status = if code.as_slice() == [Digit(2), Digit(4), Digit(6), Digit(8)] { "Locker unlocked" } else { "Wrong code, try again" }; code.clear(); show(&app, &code, status); }));
    app.run()
}
