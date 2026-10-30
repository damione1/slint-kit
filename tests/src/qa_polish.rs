// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};

fn fixture() -> QaPolishFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = QaPolishFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, app);
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    app
}
fn click(app: &QaPolishFixture, element: &ElementHandle) {
    let origin = element.absolute_position();
    let size = element.size();
    let position = LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.);
    for event in [
        WindowEvent::PointerMoved { position },
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        app.window().dispatch_event(event);
    }
    slint::platform::update_timers_and_animations();
}
fn find(app: &QaPolishFixture, kind: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(kind)
        .find_first()
        .unwrap()
}
#[test]
fn checkbox_indicator_and_wrapped_label_share_the_vertical_center() {
    let app = fixture();
    for mut theme in [app.get_theme(), app.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let checkbox = find(&app, "KitCheckbox");
            let label = checkbox
                .query_descendants()
                .match_id("KitCheckbox::label")
                .find_first()
                .unwrap();
            let indicator = checkbox
                .query_descendants()
                .match_id("KitCheckbox::indicator")
                .find_first()
                .unwrap();
            let center = |e: &ElementHandle| e.absolute_position().y + e.size().height / 2.;
            assert!(
                (center(&label) - center(&indicator)).abs() < 0.5,
                "scale={scale}"
            );
        }
    }
}
#[test]
fn pointer_toggle_does_not_leave_a_ring_on_the_old_keyboard_option() {
    let app = fixture();
    app.invoke_focus_toggle();
    let group = find(&app, "KitToggleGroup");
    assert!(
        group
            .query_descendants()
            .match_inherits("FocusOutline")
            .find_first()
            .is_some()
    );
    let push = ElementHandle::find_by_accessible_label(&app, "Push")
        .next()
        .unwrap();
    click(&app, &push);
    assert_eq!(app.get_last_toggled(), 23);
    assert!(app.get_toggle_focused());
    assert!(
        group
            .query_descendants()
            .match_inherits("FocusOutline")
            .find_first()
            .is_none()
    );
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::RightArrow.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::RightArrow.into(),
    });
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_active_token(), 23);
    assert!(
        group
            .query_descendants()
            .match_inherits("FocusOutline")
            .find_first()
            .is_some()
    );
}
#[test]
fn avatar_action_supports_pointer_keyboard_and_disabled_states() {
    let app = fixture();
    let action = ElementHandle::find_by_accessible_label(&app, "Open profile")
        .find(|e| e.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::Button))
        .unwrap();
    click(&app, &action);
    assert_eq!(app.get_avatar_activations(), 1);
    app.invoke_focus_avatar();
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    assert_eq!(app.get_avatar_activations(), 2);
    app.set_disabled(true);
    click(&app, &action);
    assert_eq!(app.get_avatar_activations(), 2);
}
#[test]
fn pointer_focus_does_not_publish_the_keyboard_tooltip() {
    let app = fixture();
    let trigger = ElementHandle::find_by_accessible_label(&app, "Create project")
        .next()
        .unwrap();
    click(&app, &trigger);
    assert!(!app.get_tip_visible());
    app.invoke_focus_avatar();
    app.invoke_focus_tip();
    slint::platform::update_timers_and_animations();
    assert!(app.get_tip_visible());
}

#[test]
fn color_popup_actions_fit_inside_the_popup_at_both_text_scales() {
    let app = fixture();
    for scale in [1., 2.] {
        let mut theme = app.get_theme();
        theme.text_scale = scale;
        app.set_theme(theme);
        app.invoke_open_color();
        slint::platform::update_timers_and_animations();
        let bounds = ElementQuery::from_root(&app)
            .match_id("KitColorField::popup-keys")
            .find_first()
            .unwrap();
        let origin = bounds.absolute_position();
        let size = bounds.size();
        for label in ["Cancel", "Apply color"] {
            let button = ElementHandle::find_by_accessible_label(&app, label)
                .next()
                .unwrap();
            let position = button.absolute_position();
            let button_size = button.size();
            assert!(position.x >= origin.x, "{label} left edge, scale={scale}");
            assert!(
                position.x + button_size.width <= origin.x + size.width,
                "{label} right edge, scale={scale}"
            );
            assert!(
                position.y + button_size.height <= origin.y + size.height,
                "{label} bottom edge, scale={scale}"
            );
        }
    }
}

#[test]
fn skeleton_paints_a_visible_placeholder_in_light_and_dark_themes() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = QaPolishFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, app);
    app.show().unwrap();
    for theme in [app.get_theme(), app.get_dark()] {
        app.set_theme(theme.clone());
        let image = app.window().take_snapshot().unwrap();
        let index = (150 * usize::try_from(image.width()).unwrap() + 40) * 4;
        let pixel = &image.as_bytes()[index..index + 3];
        assert_eq!(
            pixel,
            [
                theme.border_subtle.red(),
                theme.border_subtle.green(),
                theme.border_subtle.blue()
            ]
        );
        assert_ne!(
            pixel,
            [
                theme.canvas.red(),
                theme.canvas.green(),
                theme.canvas.blue()
            ]
        );
    }
}

#[test]
fn long_error_summary_actions_wrap_inside_their_padded_surface() {
    let app = fixture();
    for scale in [1., 2.] {
        let mut theme = app.get_theme();
        theme.text_scale = scale;
        app.set_theme(theme.clone());
        let summary = find(&app, "KitFormErrorSummary");
        let action = summary
            .query_descendants()
            .match_inherits("KitPressable")
            .find_first()
            .unwrap();
        let message = summary
            .query_descendants()
            .match_id("KitFormErrorSummary::message")
            .find_first()
            .unwrap();
        let inset = theme.spacing.unit * 2.;
        assert!((message.absolute_position().x - action.absolute_position().x - inset).abs() < 0.5);
        assert!(message.size().width <= action.size().width - 2. * inset + 0.5);
        assert!(message.size().height >= theme.typography.line_height_control * scale * 2.);
    }
}
