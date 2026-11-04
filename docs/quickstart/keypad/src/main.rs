// SPDX-License-Identifier: GPL-3.0-or-later
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}

const CODE_LENGTH: usize = 4;
// Example data: a real host checks the code where it is stored, never in the view.
const LOCKER_CODE: [Digit; CODE_LENGTH] = [Digit(2), Digit(4), Digit(6), Digit(8)];

/// One decimal digit, parsed once where the keypad's integer enters Rust.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Digit(u8);
impl Digit {
    fn parse(value: i32) -> Option<Self> {
        u8::try_from(value)
            .ok()
            .filter(|digit| *digit <= 9)
            .map(Self)
    }
}

fn show(app: &ui::App, code: &[Digit], status: &str) {
    app.set_entered(i32::try_from(code.len()).expect("a short code"));
    app.set_can_confirm(code.len() == CODE_LENGTH);
    app.set_status(status.into());
}

fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    slint_kit::init_form_validation!(ui, app);
    app.set_code_length(i32::try_from(CODE_LENGTH).expect("a short code"));
    // The host keeps the entered code; the keypad only requests changes.
    let code: Rc<RefCell<Vec<Digit>>> = Rc::default();
    show(&app, &code.borrow(), "");
    let (weak, entered) = (app.as_weak(), code.clone());
    app.on_digit(move |value| {
        let Some(app) = weak.upgrade() else { return };
        let mut code = entered.borrow_mut();
        if let Some(digit) = Digit::parse(value)
            && code.len() < CODE_LENGTH
        {
            code.push(digit);
        }
        show(&app, &code, "");
    });
    let (weak, entered) = (app.as_weak(), code.clone());
    app.on_delete_requested(move || {
        let Some(app) = weak.upgrade() else { return };
        let mut code = entered.borrow_mut();
        code.pop();
        show(&app, &code, "");
    });
    let (weak, entered) = (app.as_weak(), code);
    app.on_confirm(move || {
        let Some(app) = weak.upgrade() else { return };
        let mut code = entered.borrow_mut();
        // Guard direct host calls too, independently of the disabled UI action.
        if code.len() != CODE_LENGTH {
            return;
        }
        let status = if code.as_slice() == LOCKER_CODE {
            "Locker unlocked"
        } else {
            "Wrong code, try again"
        };
        code.clear();
        show(&app, &code, status);
    });
    app.run()
}
