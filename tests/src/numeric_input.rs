// SPDX-License-Identifier: GPL-3.0-or-later
use super::NumericInputFixture;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

fn type_text(window: &NumericInputFixture, text: &str) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}

#[test]
fn numeric_editors_filter_letters_without_changing_the_plain_text_editor() {
    i_slint_backend_testing::init_no_event_loop();
    let window = NumericInputFixture::new().unwrap();
    window.show().unwrap();
    for kind in 0..7 {
        window.set_kind(kind);
        window.set_value("12".into());
        window.invoke_focus_editor();
        type_text(&window, "Ada");
        assert_eq!(window.get_value(), "12", "numeric kind {kind}");
        window.set_value("".into());
        type_text(&window, "NaN");
        assert_eq!(
            window.get_value(),
            "",
            "non-finite text insertion kind {kind}"
        );
        for character in ["-", "1", "2", ".", "5"] {
            type_text(&window, character);
        }
        assert_eq!(
            window.get_value(),
            "-12.5",
            "signed decimal draft kind {kind}"
        );
    }
    window.set_kind(7);
    window.set_value("".into());
    window.invoke_focus_editor();
    type_text(&window, "Ada");
    assert_eq!(window.get_value(), "Ada");
}

#[test]
fn decimal_filter_preserves_native_selection_replacement() {
    i_slint_backend_testing::init_no_event_loop();
    let window = NumericInputFixture::new().unwrap();
    window.show().unwrap();
    window.set_kind(1);
    window.set_value("123.45".into());
    window.invoke_focus_editor();
    window.invoke_select_value();
    type_text(&window, "abc");
    assert_eq!(
        window.get_value(),
        "123.45",
        "rejected replacement preserves selection and value"
    );
    type_text(&window, "67.89");
    assert_eq!(window.get_value(), "67.89");
    shortcut(&window, "z");
    shortcut(&window, "z");
    assert_eq!(
        window.get_value(),
        "123.45",
        "replacement remains in native undo history"
    );
}

fn shortcut(window: &NumericInputFixture, key: &str) {
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    type_text(window, key);
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}

#[test]
fn numeric_paste_rejects_letters_but_accepts_an_exact_signed_decimal() {
    i_slint_backend_testing::init_no_event_loop();
    let window = NumericInputFixture::new().unwrap();
    window.show().unwrap();
    for kind in 0..7 {
        for (clipboard, expected) in [
            ("12Ada", "123.45"),
            ("NaN", "123.45"),
            ("-INFINITY", "123.45"),
            ("-67.89", "-67.89"),
        ] {
            window.set_kind(7);
            window.set_value(clipboard.into());
            window.invoke_focus_editor();
            shortcut(&window, "a");
            shortcut(&window, "c");
            window.set_kind(kind);
            window.set_value("123.45".into());
            window.invoke_focus_editor();
            shortcut(&window, "a");
            shortcut(&window, "v");
            assert_eq!(window.get_value(), expected);
        }
    }
}
