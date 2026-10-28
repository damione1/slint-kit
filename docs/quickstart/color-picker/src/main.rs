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
    app.set_red(color.red.to_string().into());
    app.set_green(color.green.to_string().into());
    app.set_blue(color.blue.to_string().into());
    app.set_alpha("255".into());
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

    fn set(color: &mut SrgbColor, channel: ui::ColorChannel, value: u8) {
        match channel {
            ui::ColorChannel::Red => color.red = value,
            ui::ColorChannel::Green => color.green = value,
            ui::ColorChannel::Blue => color.blue = value,
            ui::ColorChannel::Alpha => {}
        }
    }
    fn get(color: SrgbColor, channel: ui::ColorChannel) -> u8 {
        match channel {
            ui::ColorChannel::Red => color.red,
            ui::ColorChannel::Green => color.green,
            ui::ColorChannel::Blue => color.blue,
            ui::ColorChannel::Alpha => 255,
        }
    }
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_channel(move |_channel, _text| {
        if let Some(app) = weak.upgrade() {
            let channels = app
                .get_red()
                .parse::<u8>()
                .ok()
                .zip(app.get_green().parse::<u8>().ok())
                .zip(app.get_blue().parse::<u8>().ok());
            if let Some(((red, green), blue)) = channels {
                let color = SrgbColor {
                    red,
                    green,
                    blue,
                    alpha: 255,
                };
                *shared.borrow_mut() = (color, color);
                publish(&app, color, true);
                app.set_error("".into());
            } else {
                app.set_error("Use whole sRGB channels from 0 to 255".into());
            }
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_position(move |channel, position| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            set(
                &mut state.1,
                channel,
                (position.clamp(0.0, 1.0) * 255.0).round() as u8,
            );
            publish(&app, state.1, false);
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_navigate(move |channel, step| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            let old = get(state.1, channel) as i32;
            let next = match step {
                ui::NavigationStep::First => 0,
                ui::NavigationStep::Last => 255,
                ui::NavigationStep::Next => old + 1,
                ui::NavigationStep::Previous => old - 1,
                ui::NavigationStep::PageNext => old + 16,
                ui::NavigationStep::PagePrevious => old - 16,
            };
            set(&mut state.1, channel, next.clamp(0, 255) as u8);
            state.0 = state.1;
            publish(&app, state.0, true);
        }
    });
    let shared = state.clone();
    app.on_begin_gesture(move |_| {
        let mut state = shared.borrow_mut();
        state.1 = state.0;
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_finish(move |_| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            state.0 = state.1;
            publish(&app, state.0, true);
        }
    });
    let weak = app.as_weak();
    let shared = state.clone();
    app.on_cancel_channel(move |_| {
        if let Some(app) = weak.upgrade() {
            let mut state = shared.borrow_mut();
            state.1 = state.0;
            publish(&app, state.0, true);
            app.set_error("".into());
        }
    });

    app.run()
}
