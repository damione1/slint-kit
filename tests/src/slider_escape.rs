// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SliderEscapeFixture, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture() -> SliderEscapeFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = SliderEscapeFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn element(app: &SliderEscapeFixture, id: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_id(format!("SliderEscapeFixture::{id}"))
        .find_first()
        .unwrap()
}
fn key(app: &SliderEscapeFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn point(element: &ElementHandle, fraction: f32) -> LogicalPosition {
    let origin = element.absolute_position();
    let size = element.size();
    LogicalPosition::new(
        origin.x + size.width * fraction,
        origin.y + size.height / 2.,
    )
}
fn press(app: &SliderEscapeFixture, at: LogicalPosition) {
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position: at });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position: at,
        button: PointerEventButton::Left,
    });
}
fn release(app: &SliderEscapeFixture, at: LogicalPosition) {
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position: at,
        button: PointerEventButton::Left,
    });
}

#[test]
fn completed_slider_gestures_do_not_let_escape_restore_old_values_or_swallow_parent_escape() {
    let app = fixture();
    for (id, fraction) in [("single", 0.5), ("range", 0.35), ("range", 0.75)] {
        let at = point(&element(&app, id), fraction);
        press(&app, at);
        release(&app, at);
        key(&app, Key::RightArrow);
        let value = app.get_value();
        let range = app.get_range_value();
        let chosen = app.get_chosen_count();
        let escaped = app.get_escaped();
        key(&app, Key::Escape);
        assert_eq!(app.get_value(), value);
        assert_eq!(app.get_range_value(), range);
        assert_eq!(app.get_chosen_count(), chosen);
        assert_eq!(app.get_cancelled_count(), 0);
        assert_eq!(app.get_escaped(), escaped + 1);
        app.set_value("63".into());
        app.set_range_value(ui::DecimalRangeView {
            lower: "31".into(),
            upper: "74".into(),
        });
        key(&app, Key::Escape);
        assert_eq!(app.get_value(), "63");
        assert_eq!(
            app.get_range_value(),
            ui::DecimalRangeView {
                lower: "31".into(),
                upper: "74".into()
            }
        );
        assert_eq!(app.get_escaped(), escaped + 2);
        assert_eq!(app.get_cancelled_count(), 0);
    }
}

#[test]
fn escape_cancels_only_the_current_slider_gesture_and_readonly_values_stay_unchanged() {
    let app = fixture();
    for (id, fraction) in [("single", 0.6), ("range", 0.3), ("range", 0.7)] {
        let value = app.get_value();
        let range = app.get_range_value();
        let chosen = app.get_chosen_count();
        let cancelled = app.get_cancelled_count();
        let at = point(&element(&app, id), fraction);
        press(&app, at);
        assert!(app.get_value() != value || app.get_range_value() != range);
        key(&app, Key::Escape);
        release(&app, at);
        assert_eq!(app.get_value(), value);
        assert_eq!(app.get_range_value(), range);
        assert_eq!(app.get_cancelled_count(), cancelled + 1);
        assert_eq!(app.get_chosen_count(), chosen);
        assert_eq!(app.get_escaped(), 0);
    }
    app.set_access(ui::FieldAccess::ReadOnly);
    let value = app.get_value();
    let range = app.get_range_value();
    let cancelled = app.get_cancelled_count();
    key(&app, Key::RightArrow);
    key(&app, Key::Escape);
    assert_eq!(app.get_value(), value);
    assert_eq!(app.get_range_value(), range);
    assert_eq!(app.get_cancelled_count(), cancelled);
    assert_eq!(app.get_escaped(), 1);
    app.set_access(ui::FieldAccess::Disabled);
    press(&app, point(&element(&app, "single"), 0.8));
    release(&app, point(&element(&app, "single"), 0.8));
    assert_eq!(app.get_value(), value);
    assert_eq!(app.get_cancelled_count(), cancelled);
}

#[test]
fn color_channel_escape_cancels_an_active_gesture_but_keeps_later_keyboard_and_host_values() {
    let app = fixture();
    let red = element(&app, "picker")
        .query_descendants()
        .match_inherits("KitSlider")
        .find_first()
        .unwrap();
    let at = point(&red, 0.7);
    let before = app.get_accepted_color();
    press(&app, at);
    assert_ne!(app.get_accepted_color(), before);
    key(&app, Key::Escape);
    release(&app, at);
    assert_eq!(app.get_accepted_color(), before);
    assert_eq!(app.get_cancelled_count(), 1);
    press(&app, at);
    release(&app, at);
    key(&app, Key::RightArrow);
    let after = app.get_accepted_color();
    key(&app, Key::Escape);
    assert_eq!(app.get_accepted_color(), after);
    assert_eq!(app.get_cancelled_count(), 1);
    assert_eq!(app.get_escaped(), 1);
    let published = ui::SrgbColorView {
        red: 12,
        green: 34,
        blue: 56,
        alpha: 255,
    };
    app.set_accepted_value(crate::ui::ColorValue {
        present: true,
        color: published.clone(),
    });
    key(&app, Key::Escape);
    assert_eq!(app.get_accepted_color(), published);
    assert_eq!(app.get_escaped(), 2);
    for (name, mut theme) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let frame = app.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &frame,
                &format!("slider-escape-{name}-{scale}"),
            );
        }
    }
}
