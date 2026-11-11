// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CodeFeedbackFixture;
use i_slint_backend_testing::{
    AccessibleLiveness, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::ui::{FieldAccess, ValidationPresentation};
use std::cell::OnceCell;

thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn fixture() -> CodeFeedbackFixture {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap();
        });
    });
    let window = CodeFeedbackFixture::new().unwrap();
    slint_kit::install(&window);
    window
}
fn settle(window: &CodeFeedbackFixture) {
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
}
fn indicators(window: &CodeFeedbackFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("KitCodeIndicator")
        .find_all()
}
fn key(window: &CodeFeedbackFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(window);
}
fn button(window: &CodeFeedbackFixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label.as_str()))
        .find_first()
        .unwrap()
}
fn assert_announcement(window: &CodeFeedbackFixture, expected: &str) {
    let entries = indicators(window);
    assert_eq!(entries.len(), 3);
    for item in entries {
        assert_eq!(item.accessible_label().as_deref(), Some(expected));
        assert_eq!(
            item.accessible_live_region(),
            Some(AccessibleLiveness::Polite)
        );
    }
}
#[test]
fn keypad_pointer_and_keyboard_edits_publish_localized_polite_progress_without_the_code() {
    let window = fixture();
    window.show().unwrap();
    settle(&window);
    assert_announcement(&window, "0 of 4 digits entered");
    button(&window, "7").mock_single_click(PointerEventButton::Left);
    settle(&window);
    assert_eq!(window.get_filled(), 1);
    assert_announcement(&window, "1 of 4 digits entered");
    window.invoke_focus_keypad();
    key(&window, "9");
    assert_eq!(window.get_filled(), 2);
    assert_announcement(&window, "2 of 4 digits entered");
    key(&window, Key::Backspace);
    assert_eq!(window.get_filled(), 1);
    button(&window, "Delete").mock_single_click(PointerEventButton::Left);
    settle(&window);
    assert_announcement(&window, "0 of 4 digits entered");
    window.set_progress_format("{filled} chiffres sur 4 saisis".into());
    window.invoke_focus_keypad();
    for _ in 0..5 {
        key(&window, "3");
    }
    assert_eq!(window.get_filled(), 4);
    key(&window, Key::Return);
    assert_eq!(window.get_validation(), ValidationPresentation::Invalid);
    assert_announcement(
        &window,
        "4 chiffres sur 4 saisis. Code incorrect. Try again.",
    );
    window.set_message("Code incorrect. Réessayez.".into());
    settle(&window);
    assert_announcement(
        &window,
        "4 chiffres sur 4 saisis. Code incorrect. Réessayez.",
    );
    key(&window, Key::Backspace);
    assert_eq!(window.get_validation(), ValidationPresentation::Pristine);
    assert_announcement(&window, "3 chiffres sur 4 saisis");
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_access(access);
        window.invoke_focus_keypad();
        key(&window, "5");
        key(&window, Key::Backspace);
        assert_eq!(window.get_filled(), 3);
        assert_announcement(&window, "3 chiffres sur 4 saisis");
    }
}
fn capture(window: &CodeFeedbackFixture, mode: &str, scale: f32, state: &str, ink: slint::Color) {
    let image = window.window().take_snapshot().unwrap();
    let size = window
        .window()
        .size()
        .to_logical(window.window().scale_factor());
    let entries = indicators(window);
    let heights: Vec<_> = entries
        .iter()
        .map(|entry| {
            let origin = entry.absolute_position();
            let bounds = entry.size();
            assert!(bounds.width > 0. && bounds.height > 0.);
            assert!(
                origin.x >= 0.
                    && origin.y >= 0.
                    && origin.x + bounds.width <= size.width + 1.
                    && origin.y + bounds.height <= size.height + 1.
            );
            bounds.height
        })
        .collect();
    assert_eq!(heights.len(), 3);
    assert!(heights[0] < heights[1] && heights[1] < heights[2]);
    assert!(
        image
            .as_slice()
            .iter()
            .filter(|p| p.r == ink.red() && p.g == ink.green() && p.b == ink.blue())
            .count()
            > 20
    );
    if let Some(dir) = std::env::var_os("KIT_CODE_FEEDBACK_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&dir).unwrap();
        let mut file = std::fs::File::create(
            std::path::PathBuf::from(dir).join(format!("code-feedback-{mode}-{scale}-{state}.ppm")),
        )
        .unwrap();
        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
        for pixel in image.as_slice() {
            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }
}
#[test]
fn validation_marks_and_three_intrinsic_sizes_render_at_both_theme_and_text_scales() {
    for (mode, dark) in [("light", false), ("dark", true)] {
        for scale in [1., 2.] {
            let window = fixture();
            let themes = window.global::<slint_kit::ui::KitThemes>();
            let mut theme = if dark {
                themes.get_dark()
            } else {
                themes.get_light()
            };
            theme.text_scale = scale;
            theme.reduced_motion = true;
            window.set_theme(theme.clone());
            window.set_filled(2);
            window.set_progress_format("{filled} chiffres sur 4 saisis".into());
            window.show().unwrap();
            settle(&window);
            for (state, validation, message, ink) in [
                (
                    "invalid",
                    ValidationPresentation::Invalid,
                    "Code incorrect. Réessayez.",
                    theme.danger,
                ),
                (
                    "valid",
                    ValidationPresentation::Valid,
                    "Code accepté.",
                    theme.success,
                ),
            ] {
                window.set_validation(validation);
                window.set_message(message.into());
                settle(&window);
                assert_announcement(&window, &format!("2 chiffres sur 4 saisis. {message}"));
                capture(&window, mode, scale, state, ink);
            }
        }
    }
}
