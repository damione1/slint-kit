use crate::{ColorWheelFixture, FieldAccess};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::Cell, rc::Rc};

#[test]
fn wheel_pointer_keyboard_and_rebind_share_a_closed_edit_gate() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ColorWheelFixture::new().unwrap();
    slint_kit::install(&window);
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    window.on_chosen(move |_| output.set(output.get() + 1));
    let commits = Rc::new(Cell::new(0));
    let output = commits.clone();
    window.on_committed(move || output.set(output.get() + 1));
    window.show().unwrap();
    let position = LogicalPosition::new(130., 130.);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    let color = window.get_value();
    assert_eq!(
        (color.red, color.green, color.blue, color.alpha),
        (255, 255, 255, 255)
    );
    assert_eq!(count.get(), 1);
    window.set_generation(1);
    slint::platform::update_timers_and_animations();
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(180., 130.),
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(count.get(), 1);
    assert_eq!(commits.get(), 0);
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::UpArrow.into(),
    });
    assert_eq!(count.get(), 2);
    window.set_access(FieldAccess::ReadOnly);
    slint::platform::update_timers_and_animations();
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::UpArrow.into(),
    });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert_eq!(count.get(), 2);
}

#[test]
fn achromatic_hue_survives_local_drafts_but_not_rebind_or_cancel() {
    i_slint_backend_testing::init_no_event_loop();
    for gray in [255, 96] {
        let window = ColorWheelFixture::new().unwrap();
        slint_kit::install(&window);
        window.set_value(crate::SrgbColorView {
            red: gray,
            green: gray,
            blue: gray,
            alpha: 255,
        });
        window.show().unwrap();
        let position = LogicalPosition::new(130., 130.);
        window.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        window
            .window()
            .dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        window.set_value(crate::SrgbColorView {
            red: gray,
            green: gray,
            blue: gray,
            alpha: 255,
        });
        for _ in 0..120 {
            window.window().dispatch_event(WindowEvent::KeyPressed {
                text: Key::RightArrow.into(),
            });
            window.window().dispatch_event(WindowEvent::KeyReleased {
                text: Key::RightArrow.into(),
            });
        }
        assert!((window.get_hue() - 120.).abs() < 0.01);
        let before = window.get_value();
        assert_eq!((before.red, before.green, before.blue), (gray, gray, gray));
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::UpArrow.into(),
        });
        let color = window.get_value();
        assert!(color.green > color.red && color.green > color.blue);
        // Return to white through the controlled draft callback: hue persists.
        window.set_value(crate::SrgbColorView {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        });
        slint::platform::update_timers_and_animations();
        assert!((window.get_hue() - 120.).abs() < 1.0);
        window.set_generation(1);
        slint::platform::update_timers_and_animations();
        assert_eq!(window.get_hue(), 0.);
        window.invoke_choose_hue(120.);
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Escape.into(),
        });
        assert_eq!(window.get_hue(), 0.);
    }
}

#[test]
fn explicit_hue_choice_establishes_an_unknown_opaque_color() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ColorWheelFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_known(false);
    window.show().unwrap();
    window.invoke_choose_hue(120.);
    let color = window.get_value();
    assert_eq!(
        (color.red, color.green, color.blue, color.alpha),
        (0, 255, 0, 255)
    );
    assert!(window.get_known());
}
