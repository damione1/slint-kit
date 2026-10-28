use slint::ComponentHandle;
use slint_kit::{AlphaPolicy, DraftValue, SrgbColor};
use std::{cell::RefCell, rc::Rc};
mod ui {
    slint::include_modules!();
}
fn publish(app: &ui::App, color: SrgbColor, accepted: bool) {
    app.set_hex(color.hex(AlphaPolicy::Opaque).into());
    app.set_color_value(ui::SrgbColorView {
        red: color.red.into(),
        green: color.green.into(),
        blue: color.blue.into(),
        alpha: 255,
    });
    app.set_swatch(slint::Color::from_rgb_u8(
        color.red,
        color.green,
        color.blue,
    ));
    if accepted {
        app.set_accepted(color.hex(AlphaPolicy::Opaque).into());
    }
}
fn main() -> Result<(), slint::PlatformError> {
    let app = ui::App::new()?;
    let initial = SrgbColor {
        red: 79,
        green: 70,
        blue: 229,
        alpha: 255,
    };
    let state = Rc::new(RefCell::new((initial, initial)));
    publish(&app, initial, true);
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_commit(move |text| {
        if let Some(app) = weak.upgrade() {
            match SrgbColor::parse_hex(&text, AlphaPolicy::Opaque) {
                DraftValue::Parsed(value) => {
                    *shared.borrow_mut() = (value, value);
                    publish(&app, value, true);
                    app.set_error("".into());
                }
                _ => app.set_error("Use #RRGGBB or #RGB hexadecimal".into()),
            }
        }
    });
    let weak = app.as_weak();
    app.on_edit(move || {
        if let Some(app) = weak.upgrade() {
            app.set_error("".into());
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_choose(move |view| {
        if let Some(app) = weak.upgrade() {
            if let (Ok(red), Ok(green), Ok(blue)) = (
                view.red.try_into(),
                view.green.try_into(),
                view.blue.try_into(),
            ) {
                let color = SrgbColor {
                    red,
                    green,
                    blue,
                    alpha: 255,
                };
                *shared.borrow_mut() = (color, color);
                publish(&app, color, true);
                app.set_error("".into());
            }
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_cancel(move || {
        if let Some(app) = weak.upgrade() {
            let color = shared.borrow().0;
            publish(&app, color, true);
            app.set_error("".into());
        }
    });

    app.run()
}
