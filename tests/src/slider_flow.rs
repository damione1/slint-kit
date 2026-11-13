// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SliderFlowFixture, ui};
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture() -> SliderFlowFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = SliderFlowFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn element(app: &SliderFlowFixture, name: &str) -> (LogicalPosition, slint::LogicalSize) {
    let found = ElementQuery::from_root(app)
        .match_id(format!("SliderFlowFixture::{name}"))
        .find_first()
        .unwrap();
    (found.absolute_position(), found.size())
}
fn key(app: &SliderFlowFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn click(app: &SliderFlowFixture, at: LogicalPosition) {
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position: at });
    app.window().dispatch_event(WindowEvent::PointerPressed {
        position: at,
        button: PointerEventButton::Left,
    });
    app.window().dispatch_event(WindowEvent::PointerReleased {
        position: at,
        button: PointerEventButton::Left,
    });
}
fn focus(app: &SliderFlowFixture, name: &str, fraction: f32) {
    let (origin, size) = element(app, name);
    // A press near the middle focuses the control without moving far.
    click(
        app,
        LogicalPosition::new(
            origin.x + size.width * fraction,
            origin.y + size.height / 2.,
        ),
    );
}
#[test]
fn right_to_left_flips_arrows_and_pointer_while_left_to_right_is_unchanged() {
    let app = fixture();
    focus(&app, "ltr", 0.5);
    app.set_ltr_value("50".into());
    key(&app, Key::RightArrow);
    assert_eq!(app.get_ltr_value(), "51");
    focus(&app, "rtl", 0.5);
    app.set_rtl_value("50".into());
    key(&app, Key::RightArrow);
    assert_eq!(app.get_rtl_value(), "49");
    key(&app, Key::LeftArrow);
    key(&app, Key::LeftArrow);
    assert_eq!(app.get_rtl_value(), "51");
    key(&app, Key::UpArrow);
    assert_eq!(app.get_rtl_value(), "52");
    key(&app, Key::End);
    assert_eq!(app.get_rtl_value(), "100");
    // The minimum sits at the right edge, so a press near it picks a low value.
    focus(&app, "rtl", 0.95);
    let low: f32 = app.get_rtl_value().parse().unwrap();
    assert!(low < 20., "a press at the right edge selected {low}");
    focus(&app, "ltr", 0.95);
    let high: f32 = app.get_ltr_value().parse().unwrap();
    assert!(high > 80., "a press at the right edge selected {high}");
}
#[test]
fn page_steps_set_the_page_key_distance() {
    let app = fixture();
    focus(&app, "paged", 0.5);
    app.set_paged_value("50".into());
    key(&app, Key::PageUp);
    assert_eq!(app.get_paged_value(), "75");
    key(&app, Key::PageDown);
    key(&app, Key::PageDown);
    assert_eq!(app.get_paged_value(), "25");
}
#[test]
fn minimum_steps_keep_the_range_thumbs_apart_for_keys_and_pointer() {
    let app = fixture();
    app.set_spaced_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "25".into(),
    });
    // The pair violates the distance, so the invalid value is not accepted
    // until a gesture repairs it. A press near the left edge drags lower.
    focus(&app, "spaced", 0.05);
    let range = app.get_spaced_value();
    let (lower, upper): (i32, i32) = (range.lower.parse().unwrap(), range.upper.parse().unwrap());
    assert!(upper - lower >= 10, "{range:?}");
    app.set_spaced_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "30".into(),
    });
    focus(&app, "spaced", 0.2);
    key(&app, Key::RightArrow);
    let range = app.get_spaced_value();
    let (lower, upper): (i32, i32) = (range.lower.parse().unwrap(), range.upper.parse().unwrap());
    assert!(upper - lower >= 10, "{range:?}");
}
#[test]
fn range_flow_and_vertical_direction_place_thumbs_and_keys() {
    let app = fixture();
    // Right to left: the lower thumb keeps the minimum, drawn near the right.
    app.set_mirrored_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "80".into(),
    });
    let (origin, size) = element(&app, "mirrored");
    click(
        &app,
        LogicalPosition::new(origin.x + size.width * 0.9, origin.y + size.height / 2.),
    );
    // The press near the right edge moves the lower thumb towards the minimum.
    let pressed: i32 = app.get_mirrored_value().lower.parse().unwrap();
    assert!(pressed < 20, "a press at the right edge selected {pressed}");
    // Left raises the focused thumb in a right-to-left range.
    key(&app, Key::LeftArrow);
    let range = app.get_mirrored_value();
    assert_eq!(range.lower.parse::<i32>().unwrap(), pressed + 1);
    assert_eq!(range.upper, "80");
    // Vertical: the minimum is at the bottom.
    app.set_vertical_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "80".into(),
    });
    let (origin, size) = element(&app, "upright");
    click(
        &app,
        LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height * 0.95),
    );
    let range = app.get_vertical_value();
    let lower: i32 = range.lower.parse().unwrap();
    assert!(lower < 20, "a press at the bottom selected {range:?}");
}
