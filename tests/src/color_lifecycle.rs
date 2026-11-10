// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ColorLifecycleFixture, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
fn fixture() -> ColorLifecycleFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = ColorLifecycleFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn key(app: &impl ComponentHandle, value: impl Into<slint::SharedString>) {
    let text = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn native(app: &impl ComponentHandle, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_type_name("TextInput")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn menu(app: &impl ComponentHandle, label: &str) {
    key(app, Key::Menu);
    let label = label.to_owned();
    ElementQuery::from_root(app)
        .match_inherits("KitOption")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
}
fn replace(app: &impl ComponentHandle, label: &str, text: &str) -> ElementHandle {
    let editor = native(app, label);
    editor.mock_single_click(PointerEventButton::Left);
    // Select all is disabled for an empty editor; clicking it would leave the
    // menu open and route the following keys to that menu instead of the input.
    if !editor
        .accessible_value()
        .expect("native text value")
        .is_empty()
    {
        menu(app, "Select all");
    }
    if text.is_empty() {
        key(app, Key::Backspace);
    } else {
        key(app, text);
    }
    assert_eq!(
        editor.accessible_value().as_deref(),
        Some(text),
        "native replacement must actually reach the editor"
    );
    editor
}
fn color(red: i32, green: i32, blue: i32, alpha: i32) -> ui::SrgbColorView {
    ui::SrgbColorView {
        red,
        green,
        blue,
        alpha,
    }
}
fn blur(app: &ColorLifecycleFixture) {
    app.invoke_blur();
    slint::platform::update_timers_and_animations();
}
#[test]
fn host_colors_and_alpha_policy_refresh_idle_text_after_native_edits_and_publications() {
    let app = fixture();
    replace(&app, "Standalone color", "#11223344");
    key(&app, Key::Return);
    blur(&app);
    replace(&app, "Red", "42");
    key(&app, Key::Return);
    blur(&app);
    app.set_field_color(color(170, 187, 204, 221));
    app.set_picker_color(color(18, 52, 86, 120));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_field_text(), "#AABBCCDD");
    assert_eq!(app.get_hex_text(), "#12345678");
    assert_eq!(
        (
            app.get_red_text(),
            app.get_green_text(),
            app.get_blue_text(),
            app.get_alpha_text()
        ),
        ("18".into(), "52".into(), "86".into(), "120".into())
    );
    app.set_alpha_mode(ui::ColorAlphaMode::Opaque);
    app.set_picker_color(color(9, 8, 7, 6));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_field_text(), "#AABBCC");
    assert_eq!(app.get_hex_text(), "#090807");
    app.set_alpha_mode(ui::ColorAlphaMode::Editable);
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_hex_text(), "#09080706");
    assert_eq!(app.get_alpha_text(), "6");
    for (name, mut theme) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let frame = app.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &frame,
                &format!("color-drafts-{name}-{scale}"),
            );
        }
    }
}
#[test]
fn controlled_host_updates_preserve_focused_hex_caret_history_and_native_identity_until_blur() {
    let app = fixture();
    app.set_ownership(ui::Ownership::Controlled);
    let reference = crate::ColorNativeEditingReference::new().unwrap();
    slint_kit::install(&reference);
    reference.show().unwrap();
    replace(&reference, "Native reference", "#123456");
    for _ in 0..6 {
        key(&reference, Key::LeftArrow);
    }
    key(&reference, "A");
    key(&reference, Key::LeftArrow);
    reference.set_host_version(1);
    slint::platform::update_timers_and_animations();
    key(&reference, "B");
    let native_before_undo = reference.get_text();
    menu(&reference, "Undo");
    let native_undo_one = reference.get_text();
    menu(&reference, "Undo");
    let native_undo_two = reference.get_text();
    assert_ne!(
        native_before_undo, native_undo_one,
        "native Undo must actually change text"
    );
    assert_ne!(
        native_undo_one, native_undo_two,
        "the second native Undo must change text"
    );
    reference.hide().unwrap();
    let editor = replace(&app, "Standalone color", "#123456");
    for _ in 0..6 {
        key(&app, Key::LeftArrow);
    }
    key(&app, "A");
    assert_eq!(app.get_field_text(), "#A123456");
    key(&app, Key::LeftArrow);
    app.set_field_color(color(68, 85, 102, 255));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_field_text(), "#A123456");
    assert!(editor.is_valid());
    key(&app, "B");
    assert_eq!(app.get_field_text(), "#BA123456");
    menu(&app, "Undo");
    assert_eq!(app.get_field_text(), native_undo_one);
    menu(&app, "Undo");
    assert_eq!(app.get_field_text(), native_undo_two);
    assert_eq!(app.get_field_chosen(), 0);
    blur(&app);
    assert_eq!(app.get_field_text(), "#445566FF");
    assert_eq!(app.get_field_color(), color(68, 85, 102, 255));
}
#[test]
fn controlled_picker_acknowledgement_preserves_the_focused_channel_and_updates_other_idle_views() {
    let app = fixture();
    app.set_ownership(ui::Ownership::Controlled);
    let reference = crate::ColorNativeEditingReference::new().unwrap();
    reference.set_numeric(true);
    slint_kit::install(&reference);
    reference.show().unwrap();
    replace(&reference, "Native reference", "12");
    key(&reference, "3");
    key(&reference, Key::LeftArrow);
    reference.set_host_version(1);
    slint::platform::update_timers_and_animations();
    key(&reference, "4");
    let native_before_undo = reference.get_text();
    menu(&reference, "Undo");
    let native_undo_one = reference.get_text();
    menu(&reference, "Undo");
    let native_undo_two = reference.get_text();
    assert_ne!(
        native_before_undo, native_undo_one,
        "native Undo must actually change text"
    );
    assert_ne!(
        native_undo_one, native_undo_two,
        "the second native Undo must change text"
    );
    reference.hide().unwrap();
    let editor = replace(&app, "Red", "12");
    key(&app, "3");
    key(&app, Key::LeftArrow);
    app.set_picker_color(color(20, 30, 40, 50));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_red_text(), "123");
    assert_eq!(app.get_green_text(), "30");
    assert_eq!(app.get_hex_text(), "#141E2832");
    assert!(editor.is_valid());
    key(&app, "4");
    assert_eq!(app.get_red_text(), "1243");
    menu(&app, "Undo");
    assert_eq!(app.get_red_text(), native_undo_one);
    menu(&app, "Undo");
    assert_eq!(app.get_red_text(), native_undo_two);
    replace(&app, "Red", "12");
    key(&app, Key::Return);
    assert_eq!(app.get_picker_proposal(), color(12, 30, 40, 50));
    assert_eq!(app.get_picker_chosen(), 1);
    assert_eq!(app.get_picker_color(), color(20, 30, 40, 50));
    blur(&app);
    assert_eq!(app.get_red_text(), "20");
    app.set_picker_color(color(12, 30, 40, 50));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_red_text(), "12");
    assert_eq!(app.get_hex_text(), "#0C1E2832");
}
