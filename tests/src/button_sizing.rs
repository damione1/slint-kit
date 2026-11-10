// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::ComponentHandle;

#[test]
fn disabled_and_busy_semantic_actions_do_not_begin_a_press() {
    use std::{cell::Cell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonSizingFixture::new().unwrap();
    let starts = Rc::new(Cell::new(0));
    let observed = starts.clone();
    window.on_activation_started(move || observed.set(observed.get() + 1));
    window.show().unwrap();
    let button = ElementQuery::from_root(&window)
        .match_inherits("KitButton")
        .find_first()
        .unwrap();
    window.set_disabled(true);
    button.invoke_accessible_default_action();
    assert_eq!(
        starts.get(),
        0,
        "disabled semantic action cannot begin a press"
    );
    window.set_disabled(false);
    window.set_busy(true);
    button.invoke_accessible_default_action();
    assert_eq!(starts.get(), 0, "busy semantic action cannot begin a press");
    window.set_busy(false);
    button.invoke_accessible_default_action();
    assert_eq!(starts.get(), 1, "an available action still begins once");
}

#[test]
fn native_tab_reveals_the_entire_button_focus_outline_at_either_scroll_edge() {
    use slint::platform::{Key, WindowEvent};
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonScrollFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for from_below in [false, true] {
            window.invoke_focus_outside();
            if from_below {
                window.invoke_reset_below();
            } else {
                window.invoke_reset_above();
            }
            window.window().dispatch_event(WindowEvent::KeyPressed {
                text: Key::Tab.into(),
            });
            window.window().dispatch_event(WindowEvent::KeyReleased {
                text: Key::Tab.into(),
            });
            assert!(window.get_target_focused());
            assert!(
                window.get_outline_top() >= window.get_viewport_top() - 0.1,
                "top outline clipped at scale {scale}, from_below={from_below}"
            );
            assert!(
                window.get_outline_bottom() <= window.get_viewport_bottom() + 0.1,
                "bottom outline clipped at scale {scale}, from_below={from_below}"
            );
        }
    }
}

#[test]
fn filled_button_accepts_its_bounded_viewport_with_a_long_label_and_double_text() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonSizingFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let button = ElementQuery::from_root(&window)
            .match_inherits("KitButton")
            .find_first()
            .unwrap();
        assert!(
            (button.size().width - 160.).abs() < 1.,
            "fill width must follow its 160px viewport, scale {scale}: {:?}",
            button.size()
        );
        assert!(button.size().height >= window.get_theme().spacing.comfortable.height);
        let label = button
            .query_descendants()
            .match_type_name("Text")
            .find_first()
            .expect("rendered button label");
        assert!(label.absolute_position().x >= button.absolute_position().x);
        assert!(
            label.absolute_position().x + label.size().width
                <= button.absolute_position().x + button.size().width,
            "the visible label must fit too, scale {scale}"
        );
    }
    window.set_mode(ControlWidth::Intrinsic);
    let button = ElementQuery::from_root(&window)
        .match_inherits("KitButton")
        .find_first()
        .unwrap();
    assert!(
        button.size().width > 160.,
        "intrinsic mode retains the natural label width"
    );
}

#[test]
fn wrapping_button_grows_for_its_text_without_changing_native_activation_guards() {
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    use std::{cell::Cell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let app = ButtonWrapFixture::new().unwrap();
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    app.on_activated(move || observed.set(observed.get() + 1));
    app.show().unwrap();
    let buttons = ElementQuery::from_root(&app)
        .match_inherits("KitButton")
        .find_all();
    assert_eq!(buttons.len(), 3);
    let wrapped = &buttons[1];
    assert!(wrapped.size().height > buttons[0].size().height);
    assert!(wrapped.size().height + 0.1 >= app.get_content_height() + 2. * app.get_padding_block());
    assert!(
        buttons[2].size().height > buttons[0].size().height,
        "an explicit line break grows an intrinsic button"
    );
    let initial = wrapped.size();
    app.invoke_take_focus();
    assert!(app.get_focused());
    for key in [Key::Return, Key::Space] {
        app.window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        app.window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    assert_eq!(count.get(), 2);
    let position = wrapped.absolute_position();
    let point = slint::LogicalPosition::new(
        position.x + initial.width / 2.,
        position.y + initial.height - 1.,
    );
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position: point,
        button: PointerEventButton::Left,
    });
    assert_eq!(
        count.get(),
        3,
        "the extra multiline height belongs to the pointer target"
    );
    app.set_busy(true);
    assert_eq!(wrapped.size(), initial, "busy preserves the label geometry");
    wrapped.invoke_accessible_default_action();
    assert_eq!(count.get(), 3);
    app.set_busy(false);
    app.set_disabled(true);
    assert_eq!(wrapped.size(), initial);
    wrapped.invoke_accessible_default_action();
    assert_eq!(count.get(), 3);
    assert_eq!(
        wrapped.accessible_label().as_deref(),
        Some("Save all workspace changes before leaving")
    );
}

#[test]
fn multiline_button_fits_bounded_widths_in_both_themes_and_at_double_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = ButtonWrapFixture::new().unwrap();
    app.show().unwrap();
    for (dark, base) in [(false, app.get_theme()), (true, app.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            theme.reduced_motion = true;
            app.set_theme(theme);
            let mut narrow_height = None;
            for width in [180., 280., 450.] {
                app.set_available_width(width);
                let buttons = ElementQuery::from_root(&app)
                    .match_inherits("KitButton")
                    .find_all();
                let wrapped = &buttons[1];
                assert!((wrapped.size().width - width).abs() < 1.);
                assert!(
                    wrapped.size().height + 0.1
                        >= app.get_content_height() + 2. * app.get_padding_block()
                );
                if let Some(height) = narrow_height {
                    assert!(wrapped.size().height <= height + 0.1);
                }
                narrow_height = Some(wrapped.size().height);
                let snapshot = app.window().take_snapshot().unwrap();
                crate::choice_presentation::save_capture(
                    &snapshot,
                    &format!("button-wrap-{dark}-{scale}-{width}"),
                );
            }
        }
    }
}
