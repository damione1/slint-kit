// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
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
    slint::platform::update_timers_and_animations();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    slint::platform::update_timers_and_animations();
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
    slint::platform::update_timers_and_animations();
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
fn redo(app: &OtpFixture) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(app, "z");
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
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
fn code(app: &OtpFixture) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, "Verification code")
        .find(|element| {
            element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
        })
        .unwrap()
}

fn select_from_start(app: &OtpFixture, skip: usize, count: usize) {
    // Collapse a native selection rather than Home, which scrolls on macOS.
    app.invoke_select_code();
    key(app, &slint::SharedString::from(Key::LeftArrow));
    for _ in 0..skip {
        key(app, &slint::SharedString::from(Key::RightArrow));
    }
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    for _ in 0..count {
        key(app, &slint::SharedString::from(Key::RightArrow));
    }
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
}

#[test]
fn otp_drops_overtyping_from_native_text_and_deletion_never_reveals_hidden_digits() {
    let app = fixture();
    for digit in ["1", "2", "3", "4", "5", "6", "7", "8"] {
        key(&app, digit);
    }
    assert_eq!(app.get_text(), "123456");
    assert_eq!(code(&app).accessible_value().as_deref(), Some("123456"));
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "12345");
    key(&app, "9");
    assert_eq!(app.get_text(), "123459");
    assert!(app.get_validity().valid);
}

#[test]
fn otp_overflow_in_the_middle_preserves_existing_suffix_and_places_the_caret() {
    let app = fixture();
    key(&app, "123456");
    select_from_start(&app, 3, 0);
    key(&app, "7");
    assert_eq!(
        app.get_text(),
        "123456",
        "refuse inserted surplus, not the existing last digit"
    );
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(
        app.get_text(),
        "12456",
        "caret remains before the preserved suffix"
    );
    app.invoke_select_code();
    key(&app, "123456");
    select_from_start(&app, 1, 2);
    key(&app, "789");
    assert_eq!(
        app.get_text(),
        "123456",
        "overlong replacement is rejected atomically"
    );
    select_from_start(&app, 1, 2);
    key(&app, "78");
    assert_eq!(
        app.get_text(),
        "178456",
        "a fitting replacement remains native"
    );
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "17456");
    key(&app, &slint::SharedString::from(Key::Delete));
    assert_eq!(app.get_text(), "1756");
}

#[test]
fn otp_paste_rejects_overlong_edits_atomically_and_accepts_fitting_replacements() {
    let app = fixture();
    app.set_clipboard_text("12345678".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_text(), "", "overlong paste is refused as a whole");
    app.set_clipboard_text("123456".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_text(), "123456");
    assert_eq!(code(&app).accessible_value().as_deref(), Some("123456"));
    select_from_start(&app, 2, 2);
    app.set_clipboard_text("374".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(
        app.get_text(),
        "123456",
        "ambiguous matching text cannot remove an existing suffix"
    );
    app.invoke_select_code();
    app.set_clipboard_text("765432".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_text(), "765432");
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "76543");
}

#[test]
fn otp_limit_uses_effective_maximum_utf16_units_and_never_splits_a_scalar() {
    let app = fixture();
    app.set_pattern(".*".into());
    app.set_slots(4);
    key(&app, "A😀B");
    key(&app, "z");
    assert_eq!(app.get_text(), "A😀B");
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(
        app.get_text(),
        "A😀",
        "caret uses UTF-8 byte offsets after a UTF-16 cap"
    );
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "A");
    app.invoke_select_code();
    app.set_maximum(3);
    key(&app, "😀x");
    key(&app, "y");
    assert_eq!(
        app.get_text(),
        "😀x",
        "explicit maximum overrides the slot-derived default"
    );
    app.invoke_select_code();
    key(&app, &slint::SharedString::from(Key::Backspace));
    app.set_maximum(1);
    key(&app, "😀");
    assert_eq!(app.get_text(), "");
}

#[test]
fn otp_history_stays_native_until_overflow_requires_a_public_text_rewrite() {
    let app = fixture();
    key(&app, "123456");
    app.invoke_select_code();
    key(&app, "12");
    shortcut(&app, "z");
    shortcut(&app, "z");
    assert_eq!(
        app.get_text(),
        "123456",
        "in-limit replacement preserves native history"
    );
    app.invoke_select_code();
    key(&app, &slint::SharedString::from(Key::RightArrow));
    key(&app, "78");
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_text(), "123456");
    shortcut(&app, "z");
    assert_eq!(
        app.get_text(),
        "123456",
        "Slint 1.18.1 clears undo after strict overflow normalization"
    );
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "12345");
    shortcut(&app, "z");
    assert_eq!(
        app.get_text(),
        "123456",
        "ordinary edits build a new native history"
    );
}

#[test]
fn otp_overflow_correction_keeps_undo_redo_safe_across_following_edits() {
    let app = fixture();
    key(&app, "12");
    shortcut(&app, "z");
    assert_eq!(app.get_text(), "");
    redo(&app);
    assert_eq!(app.get_text(), "12", "the test exercises native redo");
    app.invoke_select_code();
    key(&app, "123456");
    app.invoke_select_code();
    key(&app, &slint::SharedString::from(Key::RightArrow));
    key(&app, "78");
    assert_eq!(app.get_text(), "123456");
    for _ in 0..3 {
        redo(&app);
        assert_eq!(
            app.get_text(),
            "123456",
            "redo cannot resurrect dropped input"
        );
        shortcut(&app, "z");
        assert_eq!(
            app.get_text(),
            "123456",
            "overflow correction resets old history"
        );
    }
    app.invoke_select_code();
    key(&app, &slint::SharedString::from(Key::RightArrow));
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "12345");
    key(&app, "9");
    assert_eq!(app.get_text(), "123459");
    shortcut(&app, "z");
    assert_eq!(app.get_text(), "12345");
    redo(&app);
    assert_eq!(app.get_text(), "123459");
}

#[test]
fn otp_repeated_overflow_and_selection_histories_never_reintroduce_hidden_text() {
    let app = fixture();
    app.set_text("123456".into());
    app.invoke_select_code();
    key(&app, &slint::SharedString::from(Key::RightArrow));
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_text(), "12345");
    select_from_start(&app, 1, 0);
    key(&app, "9");
    assert_eq!(app.get_text(), "192345");
    key(&app, "78");
    assert_eq!(app.get_text(), "192345");
    for _ in 0..8 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 6);
    }
    for _ in 0..32 {
        app.invoke_select_code();
        key(&app, "123456");
        select_from_start(&app, 1, 2);
        key(&app, "789");
        for _ in 0..4 {
            shortcut(&app, "z");
            assert!(app.get_text().encode_utf16().count() <= 6);
        }
        for _ in 0..4 {
            redo(&app);
            assert!(app.get_text().encode_utf16().count() <= 6);
        }
        app.invoke_select_code();
        key(&app, "12");
        app.set_clipboard_text("3456789".into());
        app.invoke_prepare_paste();
        shortcut(&app, "v");
        for _ in 0..4 {
            shortcut(&app, "z");
            redo(&app);
            assert!(app.get_text().encode_utf16().count() <= 6);
            assert_eq!(
                code(&app).accessible_value().as_deref(),
                Some(app.get_text().as_str())
            );
        }
    }
}

#[test]
fn otp_host_assignment_reduced_maximum_and_reset_notify_only_bounded_text() {
    let app = fixture();
    app.set_text("12345678".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_text(), "123456");
    app.set_maximum(4);
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_text(), "1234");
    app.set_default_value("87654321".into());
    app.invoke_reset_field();
    assert_eq!(app.get_text(), "8765");
    assert_eq!(app.get_last_edited(), "8765");
    assert_eq!(code(&app).accessible_value().as_deref(), Some("8765"));
}

#[test]
fn otp_waits_for_the_constraint_bridge_before_accepting_native_input() {
    i_slint_backend_testing::init_no_event_loop();
    let app = OtpFixture::new().unwrap();
    app.show().unwrap();
    app.invoke_focus_code();
    key(&app, "12345678");
    assert_eq!(app.get_text(), "");
    kit_slint::init_form_validation!(ui, app);
    key(&app, "123456");
    key(&app, "78");
    assert_eq!(app.get_text(), "123456");
}

#[test]
fn otp_reentrant_host_text_assignment_is_bounded_before_semantic_notification() {
    let app = fixture();
    key(&app, "12");
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_edit_observed(move |_| {
        if !once.replace(true) {
            weak.upgrade().unwrap().set_text("12345678".into());
        }
    });
    key(&app, "3");
    assert_eq!(app.get_text(), "123456");
    assert_eq!(app.get_semantic_value(), "123456");
    for _ in 0..3 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 6);
    }
}

#[test]
fn otp_reentrant_lower_bound_cancels_native_overflow_before_semantic_notification() {
    let app = fixture();
    key(&app, "123");
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_edit_observed(move |_| {
        if !once.replace(true) {
            weak.upgrade().unwrap().set_maximum(3);
        }
    });
    key(&app, "4");
    assert_eq!(app.get_text(), "123");
    assert_eq!(app.get_semantic_value(), "123");
    for _ in 0..3 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 3);
    }
}

#[test]
fn otp_reentrant_bounded_replacement_cannot_leave_a_stale_native_undo_entry() {
    let app = fixture();
    key(&app, "123");
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_edit_observed(move |_| {
        if !once.replace(true) {
            weak.upgrade().unwrap().set_text("123".into());
        }
    });
    key(&app, "4");
    assert_eq!(app.get_text(), "123");
    assert_eq!(app.get_semantic_value(), "123");
    for _ in 0..3 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 6);
    }
}

#[test]
fn otp_semantic_callback_can_restore_previous_text_and_reduce_the_bound_safely() {
    let app = fixture();
    key(&app, "123");
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_value_observed(move |_| {
        if !once.replace(true) {
            let app = weak.upgrade().unwrap();
            app.set_text("123".into());
            app.set_maximum(3);
        }
    });
    key(&app, "4");
    assert_eq!(app.get_text(), "123");
    for _ in 0..3 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 3);
    }
}

#[test]
fn otp_both_callbacks_can_change_text_and_bounds_in_one_native_transaction() {
    let app = fixture();
    key(&app, "123");
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_edit_observed(move |_| {
        if !once.replace(true) {
            weak.upgrade().unwrap().set_maximum(3);
        }
    });
    let weak = app.as_weak();
    let once = std::cell::Cell::new(false);
    app.on_value_observed(move |value| {
        if !once.replace(true) {
            assert_eq!(
                value, "123",
                "the first callback's bound applies before this payload"
            );
            let app = weak.upgrade().unwrap();
            app.set_text("12345678".into());
            app.set_maximum(2);
        }
    });
    key(&app, "4");
    assert_eq!(app.get_text(), "12");
    assert_eq!(code(&app).accessible_value().as_deref(), Some("12"));
    for _ in 0..3 {
        shortcut(&app, "z");
        redo(&app);
        assert!(app.get_text().encode_utf16().count() <= 2);
    }
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

#[test]
fn otp_shipped_monospace_font_has_equal_real_advances_for_digits_letters_and_masks() {
    let app = fixture();
    assert_eq!(
        app.get_theme().typography.family_monospace,
        "JetBrains Mono"
    );
    assert_eq!(app.get_dark().typography.family_monospace, "JetBrains Mono");
    let bytes = include_bytes!("../../crates/kit-slint/ui/fonts/JetBrainsMono-Regular.ttf");
    let font = ttf_parser::Face::parse(bytes, 0).expect("valid bundled original font");
    assert!(font.is_monospaced());
    let advance = |character| {
        font.glyph_hor_advance(font.glyph_index(character).expect("glyph present"))
            .expect("advance present")
    };
    for character in ['W', 'i', 'A', 'z', '•', '●'] {
        assert_eq!(
            advance(character),
            advance('0'),
            "{character} must occupy one native code cell"
        );
    }
}

#[test]
fn otp_placeholder_paints_a_complete_digit_in_every_slot_at_200_percent() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = OtpFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, app);
    app.set_placeholder("888888".into());
    app.show().unwrap();
    app.invoke_blur_code();
    for base in [app.get_theme(), app.get_dark()] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let surface = ElementQuery::from_root(&app)
                .match_inherits("KitInputGroup")
                .find_first()
                .unwrap();
            let image = app.window().take_snapshot().unwrap();
            let origin = surface.absolute_position();
            let size = surface.size();
            // Six evenly spaced code slots are the public visual contract.
            // Count actual ink inside each slot, excluding the underline and
            // neighbouring glyphs, without relying on the placeholder Text's
            // own reported width (which stays unchanged when it elides).
            let inset = theme.spacing.medium.padding_inline;
            let cell = (size.width - 2. * inset) / 6.;
            for slot in 0..6 {
                let left = (origin.x + inset + slot as f32 * cell + cell * 0.2).ceil() as u32;
                let right = (origin.x + inset + slot as f32 * cell + cell * 0.8).floor() as u32;
                let top = (origin.y + theme.spacing.medium.padding_block).ceil() as u32;
                let bottom =
                    (origin.y + size.height - theme.spacing.medium.padding_block).floor() as u32;
                let mut ink = 0;
                for y in top..bottom {
                    for x in left..right {
                        let pixel = image.as_slice()[(y * image.width() + x) as usize];
                        if pixel.r.abs_diff(theme.placeholder.red()) <= 16
                            && pixel.g.abs_diff(theme.placeholder.green()) <= 16
                            && pixel.b.abs_diff(theme.placeholder.blue()) <= 16
                        {
                            ink += 1;
                        }
                    }
                }
                assert!(
                    ink >= 8,
                    "slot {slot} at scale {scale} has only {ink} placeholder pixels"
                );
            }
        }
    }
}
