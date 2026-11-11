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
    kit_slint::install(&app);
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
    // Programmatic drafts remain subject to validation even when user insertion is filtered.
    app.set_text("A1B2C3".into());
    assert!(!app.get_validity().valid);
    assert!(
        !app.get_validation_visible(),
        "draft errors wait for validation"
    );
    key(&app, &slint::SharedString::from(Key::Return));
    assert_eq!(app.get_commits(), 0);
    assert!(app.get_validation_visible());
    app.set_alphabet(ui::OtpAlphabet::AsciiAlphanumeric);
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
    app.set_alphabet(ui::OtpAlphabet::AsciiAlphanumeric);
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
    app.set_filter_kind(ui::TextFilterKind::Unfiltered);
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
    // Explicitly exercise the existing fallback for edits that bypass pre-insertion filtering.
    app.set_filter_kind(ui::TextFilterKind::Unfiltered);
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
    // Explicitly exercise the existing fallback for edits that bypass pre-insertion filtering.
    app.set_filter_kind(ui::TextFilterKind::Unfiltered);
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
    kit_slint::install(&app);
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
    app.set_alphabet(ui::OtpAlphabet::AsciiAlphanumeric);
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
    kit_slint::install(&app);
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

#[test]
fn masked_field_derives_placeholder_bounds_pattern_and_reports_invalid_mask_syntax() {
    let app = fixture();
    assert_eq!(app.get_masked_placeholder(), "__-____");
    assert_eq!(app.get_masked_min_length(), 7);
    assert_eq!(app.get_masked_max_length(), 7);
    assert_eq!(
        app.get_masked_pattern(),
        "[A-Za-z][A-Za-z]\\u{2D}[0-9][0-9][0-9][0-9]"
    );
    assert_eq!(app.get_masked_format_error(), "");

    app.set_mask("---".into());
    assert!(!app.get_masked_format_error().is_empty());
    assert_eq!(app.get_masked_min_length(), -1);
    assert_eq!(app.get_masked_max_length(), -1);
    assert_eq!(app.get_masked_placeholder(), "");
    assert_eq!(app.get_masked_pattern(), "");

    app.global::<KitStrings>()
        .set_mask_format_message("Custom mask error".into());
    assert_eq!(app.get_masked_format_error(), "Custom mask error");

    app.invoke_focus_masked();
    key(&app, "freeform-99");
    assert_eq!(app.get_masked_text(), "");
    assert!(app.get_masked_validity().custom_error);
    assert!(!app.invoke_report_masked());
}

#[test]
fn masked_field_waits_for_constraint_bridge_before_accepting_input() {
    i_slint_backend_testing::init_no_event_loop();
    let app = OtpFixture::new().unwrap();
    app.show().unwrap();
    app.invoke_focus_masked();
    key(&app, "AB-1234");
    assert_eq!(app.get_masked_text(), "");
    app.set_clipboard_text("AB-1234".into());
    app.invoke_prepare_paste();
    app.invoke_paste_masked();
    assert_eq!(
        app.get_masked_text(),
        "",
        "public paste also waits for the bridge"
    );
    app.set_masked_text("AB-12".into());
    app.invoke_select_masked();
    app.invoke_cut_masked();
    assert_eq!(app.get_masked_text(), "AB-12");
    app.set_masked_text("".into());
    kit_slint::install(&app);
    app.invoke_focus_masked();
    assert!(app.get_masked_focused());
    key(&app, "AB-1234");
    assert_eq!(app.get_masked_text(), "AB-1234");
}

#[test]
fn masked_field_filters_keystrokes_by_slot_and_preserves_native_caret_selection_and_undo() {
    let app = fixture();
    app.invoke_focus_masked();
    key(&app, "A");
    key(&app, "1");
    assert_eq!(app.get_masked_text(), "A");
    key(&app, "B");
    key(&app, "3");
    assert_eq!(app.get_masked_text(), "AB");
    key(&app, "-");
    key(&app, "X");
    assert_eq!(app.get_masked_text(), "AB-");
    key(&app, "1234");
    key(&app, "5");
    assert_eq!(app.get_masked_text(), "AB-1234");
    assert!(app.get_masked_validity().valid);

    app.invoke_focus_witness();
    assert!(
        app.get_witness_focused(),
        "the native history witness owns focus"
    );
    key(&app, "AB-1234");
    assert_eq!(app.get_masked_text(), app.get_witness_text());
    assert_eq!(app.get_masked_cursor(), app.get_witness_cursor());
    assert_eq!(app.get_masked_anchor(), app.get_witness_anchor());

    app.invoke_focus_masked();
    app.invoke_set_masked_selection(3, 7);
    key(&app, "X");
    assert_eq!(app.get_masked_text(), "AB-1234");
    assert_eq!(app.get_masked_anchor(), 3);
    assert_eq!(app.get_masked_cursor(), 7);
    key(&app, "9876");

    app.invoke_focus_witness();
    app.invoke_set_witness_selection(3, 7);
    key(&app, "9876");
    assert_eq!(app.get_masked_text(), "AB-9876");
    assert_eq!(app.get_masked_text(), app.get_witness_text());
    assert_eq!(app.get_masked_cursor(), app.get_witness_cursor());

    app.invoke_focus_masked();
    shortcut(&app, "z");
    app.invoke_focus_witness();
    shortcut(&app, "z");
    assert_eq!(app.get_masked_text(), app.get_witness_text());

    app.invoke_focus_masked();
    redo(&app);
    app.invoke_focus_witness();
    redo(&app);
    assert_eq!(app.get_masked_text(), "AB-9876");
    assert_eq!(app.get_masked_text(), app.get_witness_text());
}

#[test]
fn masked_field_guards_native_paste_shortcuts_and_programmatic_paste() {
    let app = fixture();
    app.invoke_focus_masked();

    app.set_clipboard_text("12-ABCD".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_masked_text(), "");
    app.invoke_paste_masked();
    assert_eq!(app.get_masked_text(), "");

    app.set_clipboard_text("AB-".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_masked_text(), "AB-");

    app.set_clipboard_text("1234".into());
    app.invoke_prepare_paste();
    app.invoke_paste_masked();
    assert_eq!(app.get_masked_text(), "AB-1234");

    app.invoke_set_masked_selection(3, 7);
    app.set_clipboard_text("ABCD".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_masked_text(), "AB-1234");

    app.invoke_set_masked_selection(3, 7);
    app.set_clipboard_text("56789".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_masked_text(), "AB-1234");

    app.invoke_set_masked_selection(3, 7);
    app.set_clipboard_text("5678".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_masked_text(), "AB-5678");
}

#[test]
fn masked_field_preserves_native_deletion_undo_and_invalid_host_values() {
    let app = fixture();
    app.invoke_focus_masked();
    key(&app, "AB-12");
    app.invoke_focus_witness();
    assert!(
        app.get_witness_focused(),
        "the native deletion witness owns focus"
    );
    key(&app, "AB-12");
    app.invoke_focus_masked();
    app.invoke_set_masked_selection(0, 2);
    key(&app, &slint::SharedString::from(Key::Backspace));
    app.invoke_focus_witness();
    app.invoke_set_witness_selection(0, 2);
    key(&app, &slint::SharedString::from(Key::Backspace));
    assert_eq!(app.get_masked_text(), "-12");
    assert_eq!(app.get_masked_text(), app.get_witness_text());
    assert_eq!(app.get_masked_cursor(), app.get_witness_cursor());
    assert_eq!(app.get_masked_anchor(), app.get_witness_anchor());
    assert!(app.get_masked_validity().pattern_mismatch);
    app.invoke_undo_masked();
    app.invoke_focus_witness();
    shortcut(&app, "z");
    assert_eq!(app.get_masked_text(), app.get_witness_text());
    assert_eq!(app.get_masked_cursor(), app.get_witness_cursor());
    app.invoke_redo_masked();
    app.invoke_focus_witness();
    redo(&app);
    assert_eq!(app.get_masked_text(), "-12");
    assert_eq!(app.get_masked_text(), app.get_witness_text());

    app.invoke_focus_masked();
    app.invoke_set_masked_selection(0, 0);
    key(&app, "A");
    assert_eq!(
        app.get_masked_text(),
        "A-12",
        "repair can proceed one character at a time"
    );
    key(&app, "B");
    assert_eq!(app.get_masked_text(), "AB-12");
    app.invoke_focus_witness();
    app.invoke_set_witness_selection(0, 0);
    key(&app, "A");
    key(&app, "B");
    app.invoke_focus_masked();
    app.invoke_set_masked_selection(0, 2);
    app.invoke_cut_masked();
    app.invoke_focus_witness();
    app.invoke_set_witness_selection(0, 2);
    shortcut(&app, "x");
    assert_eq!(
        app.get_masked_text(),
        "-12",
        "native cut remains a real deletion"
    );
    assert_eq!(app.get_masked_text(), app.get_witness_text());
    app.invoke_undo_masked();
    app.invoke_focus_witness();
    shortcut(&app, "z");
    assert_eq!(
        app.get_masked_text(),
        app.get_witness_text(),
        "cut undo has the same history as the native witness"
    );

    app.set_masked_text("CD-987654".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_masked_text(),
        "CD-987654",
        "host text is never cropped to the mask"
    );
    assert!(app.get_masked_validity().pattern_mismatch);
    app.set_masked_text("CD_9876".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_masked_text(), "CD_9876");
    assert!(!app.get_masked_validity().valid);
    app.set_masked_text("CD-9876".into());
    app.set_mask("AA-##".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_masked_text(),
        "CD-9876",
        "changing the mask validates the existing host text without truncation"
    );
    assert!(!app.get_masked_validity().valid);
    app.invoke_commit_masked();
    assert_eq!(app.get_masked_commits(), 0);
}

#[test]
fn masked_field_respects_access_modes_and_commits_typed_spec_only_when_complete() {
    let app = fixture();
    let key_id = kit_slint::FieldSpecKey::new("reference").unwrap();
    let mask = slint_kit::MaskPattern::compile("AA-####").unwrap();
    kit_slint::install_field_specs(
        &app,
        [kit_slint::FieldSpec::masked(mask).named(key_id.clone())],
    )
    .unwrap();
    app.set_masked_spec_key(key_id.as_str().into());
    app.invoke_focus_masked();

    key(&app, "AB-12");
    assert!(!app.get_masked_validity().valid);
    assert!(app.get_masked_validity().too_short);
    assert!(!app.invoke_report_masked());
    app.invoke_commit_masked();
    assert_eq!(app.get_masked_commits(), 0);
    assert_eq!(app.get_masked_accepted_text(), "");

    key(&app, "34");
    assert_eq!(app.get_masked_text(), "AB-1234");
    assert!(app.get_masked_validity().valid);
    app.invoke_commit_masked();
    assert_eq!(app.get_masked_commits(), 1);
    assert_eq!(app.get_masked_accepted_text(), "AB-1234");

    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        app.set_masked_access(access);
        app.invoke_select_masked();
        key(&app, "Z");
        app.set_clipboard_text("XY-9999".into());
        app.invoke_prepare_paste();
        app.invoke_paste_masked();
        app.invoke_cut_masked();
        app.invoke_commit_masked();
        assert_eq!(app.get_masked_text(), "AB-1234");
        assert_eq!(app.get_masked_commits(), 1);
    }

    app.set_masked_access(FieldAccess::Editable);
    for base in [app.get_theme(), app.get_dark()] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            let masked_editor = ElementHandle::find_by_accessible_label(&app, "Project reference")
                .find(|element| {
                    element.accessible_role()
                        == Some(i_slint_backend_testing::AccessibleRole::TextInput)
                })
                .unwrap();
            assert_eq!(masked_editor.accessible_value().as_deref(), Some("AB-1234"));
            assert!(masked_editor.size().width > 0. && masked_editor.size().height > 0.);
        }
    }
}

#[test]
fn masked_field_renders_the_unmodified_invalid_host_draft_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = OtpFixture::new().unwrap();
    kit_slint::install(&app);
    app.window().set_size(slint::LogicalSize::new(900., 600.));
    app.set_masked_text("AB-123456".into());
    app.show().unwrap();
    let light = app.get_theme();
    for (dark, base) in [(false, light), (true, app.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.invoke_focus_masked();
            assert!(!app.invoke_report_masked());
            slint::platform::update_timers_and_animations();
            let snapshot = app.window().take_snapshot().unwrap();
            assert_eq!(app.get_masked_text(), "AB-123456");
            assert!(app.get_masked_validity().pattern_mismatch);
            assert!(app.get_masked_validation_visible());
            crate::choice_presentation::save_capture(
                &snapshot,
                &format!("mask-invalid-native-{dark}-{scale}"),
            );
        }
    }
}

#[test]
fn otp_insertion_filter_preserves_selection_and_can_be_explicitly_relaxed() {
    let app = fixture();
    key(&app, "A1");
    assert_eq!(app.get_text(), "");
    assert_eq!(app.get_edits(), 0);
    key(&app, "12");
    app.invoke_select_code();
    key(&app, "x");
    assert_eq!(app.get_text(), "12");
    key(&app, "34");
    assert_eq!(
        app.get_text(),
        "34",
        "rejected insertion keeps the native selection"
    );
    app.set_alphabet(ui::OtpAlphabet::AsciiAlphanumeric);
    app.set_pattern("[A-Za-z0-9]*".into());
    app.invoke_select_code();
    key(&app, "AB12cd");
    assert_eq!(app.get_text(), "AB12cd");
    app.set_filter_kind(ui::TextFilterKind::Unfiltered);
    app.set_pattern("[0-9]*".into());
    app.invoke_select_code();
    key(&app, "ABCDEF");
    assert_eq!(app.get_text(), "ABCDEF");
    assert!(
        !app.get_validity().valid,
        "validation remains active when filtering is off"
    );
    app.set_slots(2);
    app.set_pattern("\\p{Letter}*".into());
    app.invoke_select_code();
    key(&app, "éA");
    assert_eq!(app.get_text(), "éA");
    assert!(
        app.get_validity().valid,
        "custom Unicode patterns keep native text editing"
    );
}
#[test]
fn otp_completion_observes_final_native_edits_once_after_host_rewrites() {
    let app = fixture();
    key(&app, "12345");
    assert_eq!(app.get_completions(), 0);
    key(&app, "6");
    assert_eq!(app.get_completions(), 1);
    assert_eq!(app.get_completed_code(), "123456");
    app.invoke_select_code();
    key(&app, "123456");
    assert_eq!(
        app.get_completions(),
        1,
        "unchanged complete codes are deduplicated"
    );
    key(&app, &slint::SharedString::from(Key::LeftArrow));
    assert_eq!(app.get_completions(), 1);
    app.set_text("654321".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_completions(),
        1,
        "model updates do not simulate native completion"
    );
    let weak = app.as_weak();
    app.on_edit_observed(move |value| {
        if value == "222222" {
            weak.upgrade().unwrap().set_text("12".into());
        }
    });
    app.invoke_select_code();
    key(&app, "222222");
    assert_eq!(app.get_text(), "12");
    assert_eq!(
        app.get_completions(),
        1,
        "an incomplete host rewrite cannot complete"
    );
    let weak = app.as_weak();
    app.on_value_observed(move |value| {
        if value == "123456" {
            weak.upgrade().unwrap().set_text("765432".into());
        }
    });
    app.invoke_select_code();
    key(&app, "123456");
    assert_eq!(app.get_text(), "765432");
    assert_eq!(app.get_completed_code(), "765432");
    assert_eq!(app.get_completions(), 2);
    let weak = app.as_weak();
    app.on_completion_observed(move |_| weak.upgrade().unwrap().set_text("".into()));
    app.invoke_select_code();
    key(&app, "111111");
    assert_eq!(app.get_text(), "");
    assert_eq!(app.get_completions(), 3);
    key(&app, "999999");
    assert_eq!(app.get_text(), "");
    assert_eq!(
        app.get_completions(),
        4,
        "clearing in an observer rearms the next completion"
    );
}
#[test]
fn text_filters_cache_typed_rules_and_keep_invalid_sources_distinct_from_disabled_filters() {
    let engine = kit_slint::HtmlValidationEngine::default();
    let first = engine
        .text_filter(kit_core::TextFilterKind::Pattern, "[0-9]*")
        .unwrap();
    let again = engine
        .text_filter(kit_core::TextFilterKind::Pattern, "[0-9]*")
        .unwrap();
    assert!(
        std::rc::Rc::ptr_eq(&first, &again),
        "one window reuses its compiled rule"
    );
    assert_eq!(
        engine
            .text_filter(kit_core::TextFilterKind::Pattern, "[")
            .unwrap_err(),
        kit_core::ParseIssue::InvalidSyntax
    );
    assert!(
        engine
            .text_filter(kit_core::TextFilterKind::Unfiltered, "[")
            .unwrap()
            .allows_draft("anything")
    );
    assert!(first.allows_draft("123"));
    assert!(!first.allows_draft("12A"));
}

#[test]
fn otp_rejected_letters_and_overflow_preserve_the_native_selection_and_history() {
    let app = fixture();
    // Start from an authored draft so the native insertion is one distinct undo item.
    app.set_text("12".into());
    key(&app, "3456");
    assert_eq!(app.get_completions(), 1);
    let cursor = app.get_cursor();
    key(&app, "78");
    key(&app, "X");
    assert_eq!(app.get_text(), "123456");
    assert_eq!(app.get_cursor(), cursor);
    assert_eq!(app.get_completions(), 1);
    app.invoke_undo_code();
    assert_eq!(
        app.get_text(),
        "12",
        "rejected edits leave the prior native transaction intact"
    );
    app.invoke_redo_code();
    assert_eq!(app.get_text(), "123456");
    select_from_start(&app, 2, 2);
    let selection = (app.get_anchor(), app.get_cursor());
    key(&app, "x");
    assert_eq!((app.get_anchor(), app.get_cursor()), selection);
    key(&app, "90");
    assert_eq!(app.get_text(), "129056");
    app.invoke_undo_code();
    // Slint models replacing a selection as native deletion and insertion items.
    app.invoke_undo_code();
    assert_eq!(app.get_text(), "123456");
}

#[test]
fn otp_native_raw_clipboard_paste_keeps_undo_while_formatted_paste_is_explicitly_unsupported() {
    let app = fixture();
    // Start from an authored draft so the native insertion is one distinct undo item.
    app.set_text("12".into());
    app.set_clipboard_text("3456".into());
    app.invoke_prepare_paste();
    shortcut(&app, "v");
    assert_eq!(app.get_text(), "123456");
    app.invoke_undo_code();
    assert_eq!(app.get_text(), "12");
    app.invoke_redo_code();
    assert_eq!(app.get_text(), "123456");
    app.invoke_select_code();
    let selected = (app.get_anchor(), app.get_cursor());
    for copied in ["123 456", "123-456", "123456\n", "1234567"] {
        app.set_clipboard_text(copied.into());
        app.invoke_prepare_paste();
        shortcut(&app, "v");
        assert_eq!(app.get_text(), "123456");
        assert_eq!(
            (app.get_anchor(), app.get_cursor()),
            selected,
            "rejecting an unsupported paste preserves selection"
        );
    }
    app.invoke_undo_code();
    assert_eq!(
        app.get_text(),
        "12",
        "rejected formatted pastes do not clear undo"
    );
}

#[test]
fn otp_custom_whole_value_patterns_keep_native_prefixes_editable() {
    let app = fixture();
    app.set_slots(7);
    app.set_pattern("[A-Z]{2}[0-9]{5}".into());
    for character in ["F", "G", "7", "6", "6", "5", "7"] {
        key(&app, character);
    }
    assert_eq!(app.get_text(), "FG76657");
    assert!(app.get_validity().valid);
    assert_eq!(app.get_completions(), 1);
}

#[test]
fn otp_cells_groups_and_native_selection_render_at_both_theme_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = OtpFixture::new().unwrap();
    kit_slint::install(&app);
    app.window().set_size(slint::LogicalSize::new(1000., 600.));
    app.show().unwrap();
    for (name, theme) in [("light", app.get_theme()), ("dark", app.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.set_group_size(3);
            app.set_placeholder("000000".into());
            app.set_text("123456".into());
            app.invoke_focus_code();
            select_from_start(&app, 1, 3);
            slint::platform::update_timers_and_animations();
            let selected = (app.get_anchor(), app.get_cursor());
            let frame = app.window().take_snapshot().unwrap();
            assert_eq!(app.get_text(), "123456");
            assert_eq!(selected, (1, 4));
            assert_eq!(
                ElementQuery::from_root(&app)
                    .match_type_name("TextInput")
                    .match_predicate(
                        |item| item.accessible_label().as_deref() == Some("Verification code")
                    )
                    .find_all()
                    .len(),
                1,
                "all boxes share one real editor"
            );
            crate::choice_presentation::save_capture(
                &frame,
                &format!("otp-cells-selected-{name}-{scale}"),
            );
            app.set_text("12".into());
            assert!(!app.invoke_report_code());
            slint::platform::update_timers_and_animations();
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("otp-cells-invalid-{name}-{scale}"),
            );
            app.set_access(FieldAccess::ReadOnly);
            slint::platform::update_timers_and_animations();
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("otp-cells-readonly-{name}-{scale}"),
            );
            app.set_access(FieldAccess::Disabled);
            slint::platform::update_timers_and_animations();
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("otp-cells-disabled-{name}-{scale}"),
            );
            app.set_access(FieldAccess::Editable);
            app.invoke_reset_code();
        }
    }
}
