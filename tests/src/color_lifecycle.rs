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
    app.set_field_value(value(color(170, 187, 204, 221)));
    app.set_picker_value(value(color(18, 52, 86, 120)));
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
    app.set_picker_value(value(color(9, 8, 7, 6)));
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
    app.set_field_value(value(color(68, 85, 102, 255)));
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
    app.set_picker_value(value(color(20, 30, 40, 50)));
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
    app.set_picker_value(value(color(12, 30, 40, 50)));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_red_text(), "12");
    assert_eq!(app.get_hex_text(), "#0C1E2832");
}

#[test]
fn typed_hex_and_every_channel_commit_on_tab_without_duplicate_enter_or_unchanged_blur_commits() {
    let app = fixture();
    let field = replace(&app, "Standalone color", "#aabbccdd");
    assert_eq!(app.get_field_color(), color(1, 2, 3, 255));
    key(&app, Key::Tab);
    assert_eq!(app.get_field_color(), color(170, 187, 204, 221));
    assert_eq!(app.get_field_text(), "#AABBCCDD");
    assert_eq!(app.get_field_chosen(), 1);
    assert!(field.is_valid());
    assert_eq!(
        app.get_field_requests(),
        0,
        "blur is not an explicit Enter request"
    );
    replace(&app, "Picker color", "#11223344");
    key(&app, Key::Tab);
    assert_eq!(app.get_picker_color(), color(17, 34, 51, 68));
    let mut expected = app.get_picker_color();
    for (channel, label, value) in [
        (ui::ColorChannel::Red, "Red", 120),
        (ui::ColorChannel::Green, "Green", 121),
        (ui::ColorChannel::Blue, "Blue", 122),
        (ui::ColorChannel::Alpha, "Alpha", 123),
    ] {
        let chosen = app.get_picker_chosen();
        let editor = replace(&app, label, &value.to_string());
        key(&app, Key::Tab);
        match channel {
            ui::ColorChannel::Red => expected.red = value,
            ui::ColorChannel::Green => expected.green = value,
            ui::ColorChannel::Blue => expected.blue = value,
            ui::ColorChannel::Alpha => expected.alpha = value,
        }
        assert_eq!(app.get_picker_color(), expected);
        assert_eq!(app.get_picker_chosen(), chosen + 1);
        assert!(editor.is_valid());
    }
    assert_eq!(app.get_channel_requests(), 0);
    replace(&app, "Red", "124");
    let chosen = app.get_picker_chosen();
    key(&app, Key::Return);
    assert_eq!(app.get_picker_chosen(), chosen + 1);
    assert_eq!(app.get_channel_requests(), 1);
    key(&app, Key::Tab);
    assert_eq!(app.get_picker_chosen(), chosen + 1);
    replace(&app, "Standalone color", "#01020304");
    key(&app, Key::Return);
    let chosen = app.get_field_chosen();
    assert_eq!(app.get_field_requests(), 1);
    blur(&app);
    assert_eq!(app.get_field_chosen(), chosen);
    native(&app, "Standalone color").mock_single_click(PointerEventButton::Left);
    blur(&app);
    assert_eq!(app.get_field_chosen(), chosen);
}

#[test]
fn invalid_or_incomplete_color_blur_restores_accepted_text_without_a_color_proposal() {
    let app = fixture();
    for text in ["#12", "not-a-color", ""] {
        replace(&app, "Standalone color", text);
        key(&app, Key::Tab);
        assert_eq!(app.get_field_color(), color(1, 2, 3, 255));
        assert_eq!(app.get_field_text(), "#010203FF");
        assert_eq!(app.get_field_chosen(), 0);
    }
    for text in ["", "256", "-1", "12.5", "1e"] {
        replace(&app, "Red", text);
        key(&app, Key::Tab);
        assert_eq!(app.get_picker_color(), color(4, 5, 6, 255));
        assert_eq!(app.get_red_text(), "4");
        assert_eq!(app.get_picker_chosen(), 0);
    }
}

#[test]
fn controlled_blur_proposes_once_and_readonly_or_disabled_blur_cannot_publish_a_draft() {
    let app = fixture();
    app.set_ownership(ui::Ownership::Controlled);
    replace(&app, "Standalone color", "#11223344");
    key(&app, Key::Tab);
    assert_eq!(app.get_field_chosen(), 1);
    assert_eq!(app.get_field_proposal(), color(17, 34, 51, 68));
    assert_eq!(app.get_field_color(), color(1, 2, 3, 255));
    assert_eq!(app.get_field_text(), "#010203FF");
    app.set_field_value(value(color(17, 34, 51, 68)));
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_field_text(), "#11223344");
    for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
        app.set_access(ui::FieldAccess::Editable);
        replace(&app, "Standalone color", "#ABCDEF12");
        app.set_access(access);
        blur(&app);
        assert_eq!(app.get_field_chosen(), 1);
        assert_eq!(app.get_field_color(), color(17, 34, 51, 68));
        assert_eq!(app.get_field_text(), "#11223344");
    }
}

fn value(color: ui::SrgbColorView) -> ui::ColorValue {
    ui::ColorValue {
        present: true,
        color,
    }
}
fn action(app: &ColorLifecycleFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap_or_else(|| panic!("missing action {label}"))
}
fn settle(app: &ColorLifecycleFixture) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap()
}

#[test]
fn optional_native_clear_accepts_absence_once_and_renders_it_apart_from_transparent_color() {
    let app = fixture();
    app.set_required(false);
    app.set_clearable(true);
    settle(&app);
    action(&app, "Clear standalone color").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert_eq!(app.get_field_text(), "");
    assert!(
        app.get_field_value().present,
        "clear edits the native draft first"
    );
    assert_eq!(app.get_field_cleared(), 0);
    key(&app, Key::Tab);
    assert!(!app.get_field_value().present);
    assert_eq!(app.get_field_cleared(), 1);
    app.invoke_commit_field();
    blur(&app);
    assert_eq!(app.get_field_cleared(), 1);
    action(&app, "Clear picker color").mock_single_click(PointerEventButton::Left);
    settle(&app);
    key(&app, Key::Return);
    key(&app, Key::Tab);
    assert!(!app.get_picker_value().present);
    assert_eq!(app.get_picker_cleared(), 1);
    assert_eq!((app.get_field_chosen(), app.get_picker_chosen()), (0, 0));
    assert!(app.get_field_valid() && app.get_picker_valid());
    assert_eq!(
        (
            app.get_hex_text(),
            app.get_red_text(),
            app.get_green_text(),
            app.get_blue_text(),
            app.get_alpha_text()
        ),
        ("".into(), "".into(), "".into(), "".into(), "".into())
    );
    app.set_empty_label("Aucune couleur".into());
    settle(&app);
    assert_eq!(
        action(&app, "Open color picker for Standalone color")
            .accessible_description()
            .as_deref(),
        Some("Aucune couleur")
    );
    let transparent = color(18, 52, 86, 0);
    let themes = [("light", app.get_theme()), ("dark", app.get_dark_theme())];
    for state in ["empty", "transparent"] {
        if state == "transparent" {
            app.set_field_value(value(transparent.clone()));
            app.set_picker_value(value(transparent.clone()));
            slint::platform::update_timers_and_animations();
            assert!(app.get_field_value().present && app.get_picker_value().present);
            assert_eq!(app.get_field_text(), "#12345600");
            assert_eq!(app.get_hex_text(), "#12345600");
        }
        for (name, mut theme) in themes.clone() {
            for scale in [1., 2.] {
                theme.text_scale = scale;
                app.set_theme(theme.clone());
                app.window()
                    .set_size(slint::LogicalSize::new(900., 1100. * scale));
                let frame = settle(&app);
                crate::choice_presentation::save_capture(
                    &frame,
                    &format!("optional-color-{state}-{name}-{scale}"),
                );
            }
        }
    }
}

#[test]
fn required_custom_invalid_and_blocked_access_never_publish_color_absence() {
    let app = fixture();
    for custom in ["", "A color is locked by the form"] {
        app.set_required(custom.is_empty());
        app.set_custom_validity(custom.into());
        app.invoke_clear_field();
        assert_eq!(app.get_field_text(), "");
        app.invoke_commit_field();
        assert!(!app.get_field_valid());
        assert!(app.get_field_value().present);
        app.invoke_clear_picker();
        assert_eq!(app.get_hex_text(), "");
        app.invoke_commit_picker();
        assert!(!app.get_picker_valid());
        assert!(app.get_picker_value().present);
        blur(&app);
    }
    app.set_custom_validity("".into());
    app.set_required(false);
    for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
        app.set_access(access);
        app.invoke_clear_field();
        app.invoke_clear_picker();
        app.invoke_commit_field();
        app.invoke_commit_picker();
        assert_eq!(app.get_field_text(), "#010203FF");
        assert_eq!(app.get_hex_text(), "#040506FF");
        assert!(app.get_field_value().present && app.get_picker_value().present);
    }
    assert_eq!((app.get_field_cleared(), app.get_picker_cleared()), (0, 0));
    assert_eq!((app.get_field_chosen(), app.get_picker_chosen()), (0, 0));
}

#[test]
fn controlled_empty_acknowledgment_and_same_color_reintroduction_use_the_atomic_value() {
    let app = fixture();
    app.set_required(false);
    app.set_ownership(ui::Ownership::Controlled);
    let old_field = app.get_field_color();
    let old_picker = app.get_picker_color();
    app.invoke_clear_field();
    key(&app, Key::Return);
    assert!(app.get_field_value().present);
    assert_eq!(app.get_field_cleared(), 1);
    key(&app, Key::Tab);
    assert_eq!(app.get_field_cleared(), 1);
    assert_eq!(app.get_field_text(), "#010203FF");
    app.set_field_value(ui::ColorValue::default());
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_field_text(), "");
    app.set_field_value(value(old_field));
    slint::platform::update_timers_and_animations();
    assert!(app.get_field_value().present);
    assert_eq!(app.get_field_text(), "#010203FF");
    app.invoke_clear_picker();
    key(&app, Key::Return);
    assert!(app.get_picker_value().present);
    assert_eq!(app.get_picker_cleared(), 1);
    key(&app, Key::Tab);
    assert_eq!(app.get_picker_cleared(), 1);
    app.set_picker_value(ui::ColorValue::default());
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_hex_text(), "");
    assert_eq!(app.get_red_text(), "");
    app.set_picker_value(value(old_picker));
    slint::platform::update_timers_and_animations();
    assert!(app.get_picker_value().present);
    assert_eq!(app.get_hex_text(), "#040506FF");
    assert_eq!(app.get_red_text(), "4");
    assert_eq!((app.get_field_chosen(), app.get_picker_chosen()), (0, 0));
}

#[test]
fn host_absence_and_native_clear_preserve_editor_selection_identity_and_undo_history() {
    let app = fixture();
    app.set_required(false);
    app.set_ownership(ui::Ownership::Controlled);
    let reference = crate::ColorNativeEditingReference::new().unwrap();
    slint_kit::install(&reference);
    reference.show().unwrap();
    replace(&reference, "Native reference", "#123456");
    key(&reference, Key::LeftArrow);
    key(&reference, "A");
    reference.set_host_version(1);
    key(&reference, "B");
    let before = reference.get_text();
    menu(&reference, "Undo");
    let after = reference.get_text();
    assert_ne!(before, after);
    reference.hide().unwrap();
    let editor = replace(&app, "Standalone color", "#123456");
    key(&app, Key::LeftArrow);
    key(&app, "A");
    app.set_field_value(ui::ColorValue::default());
    slint::platform::update_timers_and_animations();
    key(&app, "B");
    assert!(editor.is_valid());
    assert_eq!(
        app.get_field_text(),
        before,
        "host absence must not move the native caret"
    );
    menu(&app, "Undo");
    assert_eq!(app.get_field_text(), after);
    // The shared clear uses select-all + native edit-selection, so undo restores
    // the exact draft rather than a color layer's replacement history.
    app.invoke_clear_field();
    assert_eq!(app.get_field_text(), "");
    menu(&app, "Undo");
    assert_eq!(app.get_field_text(), after);
    assert!(editor.is_valid());
    assert_eq!(app.get_field_cleared(), 0);
    blur(&app);
    assert_eq!(app.get_field_text(), "");
}

#[test]
fn empty_picker_channels_and_popup_cancel_never_invent_an_accepted_color() {
    let app = fixture();
    app.set_required(false);
    app.set_field_value(ui::ColorValue::default());
    app.set_picker_value(ui::ColorValue::default());
    settle(&app);
    app.invoke_open_field_picker();
    settle(&app);
    app.invoke_cancel_field_picker();
    settle(&app);
    assert!(!app.get_field_value().present);
    assert_eq!((app.get_field_chosen(), app.get_field_cleared()), (0, 0));
    app.invoke_open_field_picker();
    settle(&app);
    action(&app, "Choose RGB 239, 68, 68").mock_single_click(PointerEventButton::Left);
    action(&app, "Apply color").mock_single_click(PointerEventButton::Left);
    settle(&app);
    assert!(app.get_field_value().present);
    assert_eq!(app.get_field_color(), color(239, 68, 68, 255));
    assert_eq!(app.get_field_chosen(), 1);
    replace(&app, "Red", "27");
    key(&app, Key::Tab);
    assert!(app.get_picker_value().present);
    assert_eq!(app.get_picker_color(), color(27, 0, 0, 255));
    assert_eq!(app.get_picker_chosen(), 1);
    assert_eq!(app.get_picker_cleared(), 0);
}
