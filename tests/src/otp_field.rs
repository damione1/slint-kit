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
