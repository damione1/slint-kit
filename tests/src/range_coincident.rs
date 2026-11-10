// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SliderEscapeFixture, ui};
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
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
fn point(app: &SliderEscapeFixture, fraction: f32) -> LogicalPosition {
    let range = ElementQuery::from_root(app)
        .match_id("SliderEscapeFixture::range")
        .find_first()
        .unwrap();
    let origin = range.absolute_position();
    let size = range.size();
    LogicalPosition::new(
        origin.x + (size.width * fraction).clamp(1., size.width - 1.),
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
fn key(app: &SliderEscapeFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn coincident(app: &SliderEscapeFixture, value: &str) {
    app.set_range_value(ui::DecimalRangeView {
        lower: value.into(),
        upper: value.into(),
    });
}
#[test]
fn coincident_range_ends_separate_from_the_shared_thumb_and_middle_presses_follow_the_press_side() {
    let app = fixture();
    // The minimum starts with lower active; that case leaves upper active
    // before the maximum. Neither prior thumb may block separation.
    for (shared, start, finish, lower_moves) in [
        ("0", 0., 0.4, false),
        ("100", 1., 0.6, true),
        ("50", 0.3, 0.25, true),
        ("50", 0.7, 0.75, false),
    ] {
        coincident(&app, shared);
        let chosen = app.get_chosen_count();
        press(&app, point(&app, start));
        let finish = point(&app, finish);
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position: finish });
        let range = app.get_range_value();
        if lower_moves {
            assert_ne!(range.lower, shared);
            assert_eq!(range.upper, shared);
        } else {
            assert_eq!(range.lower, shared);
            assert_ne!(range.upper, shared);
        }
        release(&app, finish);
        assert_eq!(app.get_chosen_count(), chosen + 1);
        assert_eq!(app.get_cancelled_count(), 0);
        key(&app, Key::RightArrow);
        let after_key = app.get_range_value();
        if lower_moves {
            assert_eq!(after_key.upper, range.upper);
            assert_ne!(after_key.lower, range.lower);
        } else {
            assert_eq!(after_key.lower, range.lower);
            assert_ne!(after_key.upper, range.upper);
        }
    }
    for (name, mut theme) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            app.set_theme(theme.clone());
            let frame = app.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &frame,
                &format!("coincident-range-{name}-{scale}"),
            );
        }
    }
}
#[test]
fn separating_coincident_ends_can_be_cancelled_without_committing_or_bypassing_access_guards() {
    let app = fixture();
    for (shared, start, finish) in [("0", 0., 0.4), ("100", 1., 0.6)] {
        coincident(&app, shared);
        let before = app.get_range_value();
        let chosen = app.get_chosen_count();
        let cancelled = app.get_cancelled_count();
        press(&app, point(&app, start));
        let finish = point(&app, finish);
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position: finish });
        assert_ne!(app.get_range_value(), before);
        key(&app, Key::Escape);
        release(&app, finish);
        assert_eq!(app.get_range_value(), before);
        assert_eq!(app.get_chosen_count(), chosen);
        assert_eq!(app.get_cancelled_count(), cancelled + 1);
    }
    for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
        app.set_access(access);
        coincident(&app, "0");
        let before = app.get_range_value();
        let chosen = app.get_chosen_count();
        press(&app, point(&app, 0.7));
        release(&app, point(&app, 0.7));
        assert_eq!(app.get_range_value(), before);
        assert_eq!(app.get_chosen_count(), chosen);
    }
}
