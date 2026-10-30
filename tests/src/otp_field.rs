// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use i_slint_backend_testing::ElementHandle;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};

fn fixture() -> OtpFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = OtpFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, app);
    app.show().unwrap();
    app.invoke_focus_code();
    app
}
fn key(app: &OtpFixture, text: &str) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}
fn shortcut(app: &OtpFixture, text: &str) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(app, text);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
}
#[test]
fn otp_patterns_validate_numeric_and_alphanumeric_codes_on_submit() {
    let app = fixture();
    key(&app, "A1B2C3");
    assert!(!app.get_validity().valid);
    assert!(
        !app.get_validation_visible(),
        "draft errors wait for validation"
    );
    key(&app, &slint::SharedString::from(Key::Return));
    assert_eq!(app.get_commits(), 0);
    assert!(app.get_validation_visible());
    app.set_pattern("[A-Za-z0-9]*".into());
    assert!(app.get_validity().valid);
    key(&app, &slint::SharedString::from(Key::Return));
    assert_eq!(app.get_commits(), 1);
    app.invoke_select_code();
    key(&app, "AB");
    assert!(app.get_validity().too_short);
    assert!(!app.invoke_report_code());
    key(&app, "12cd");
    assert!(app.get_validity().valid);
}
#[test]
fn otp_read_only_mask_keeps_value_private_and_plain_editor_keeps_native_history() {
    let app = fixture();
    app.set_pattern("[A-Za-z0-9]*".into());
    key(&app, "A1B2C3");
    app.set_access(FieldAccess::ReadOnly);
    app.set_masked(true);
    assert_eq!(app.get_text(), "A1B2C3");
    let editor = ElementHandle::find_by_accessible_label(&app, "Verification code")
        .find(|element| {
            element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
        })
        .unwrap();
    assert_eq!(editor.accessible_value().as_deref().unwrap_or(""), "");
    app.invoke_select_code();
    key(&app, "9");
    shortcut(&app, "z");
    assert_eq!(app.get_text(), "A1B2C3", "masked view remains read-only");
    app.set_masked(false);
    app.set_access(FieldAccess::Editable);
    app.invoke_select_code();
    key(&app, "9");
    assert_eq!(app.get_text(), "9");
    shortcut(&app, "z");
    shortcut(&app, "z");
    assert_eq!(app.get_text(), "A1B2C3");
    app.invoke_select_code();
    shortcut(&app, "c");
    key(&app, "12");
    app.invoke_select_code();
    shortcut(&app, "v");
    assert_eq!(app.get_text(), "A1B2C3");
}
#[test]
fn otp_overflow_is_validation_only_until_upstream_supports_before_input() {
    let app = fixture();
    key(&app, "1234567");
    assert_eq!(
        app.get_text(),
        "1234567",
        "public native editing must not be rewritten"
    );
    assert!(app.get_validity().too_long);
    assert!(!app.get_validation_visible());
    assert!(!app.invoke_report_code());
    app.invoke_select_code();
    key(&app, "123456");
    assert!(app.get_validity().valid);
}

#[test]
fn otp_grouped_display_keeps_the_raw_value_and_native_edit_history() {
    let app = fixture();
    app.set_slots(7);
    app.set_pattern("[A-Z]{2}[0-9]{5}".into());
    key(&app, "FG76657");
    app.set_display_text("FG‑76657".into());
    app.invoke_blur_code();
    let editor = ElementHandle::find_by_accessible_label(&app, "Verification code")
        .find(|element| {
            element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
        })
        .unwrap();
    assert_eq!(editor.accessible_value().as_deref(), Some("FG‑76657"));
    app.invoke_focus_code();
    assert_eq!(editor.accessible_value().as_deref(), Some("FG76657"));
    assert_eq!(app.get_text(), "FG76657");
    assert!(app.get_validity().valid);
    app.invoke_select_code();
    key(&app, "AB12345");
    assert_eq!(app.get_text(), "AB12345");
    shortcut(&app, "z");
    shortcut(&app, "z");
    assert_eq!(app.get_text(), "FG76657");
}
