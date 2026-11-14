// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SliderMarksFixture, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{PointerEventButton, WindowEvent},
};

fn fixture() -> SliderMarksFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = SliderMarksFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.window().take_snapshot().unwrap();
    app
}
fn element(app: &SliderMarksFixture, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_id(format!("SliderMarksFixture::{name}"))
        .find_first()
        .unwrap()
}
fn centre(element: &ElementHandle, fraction: f32) -> LogicalPosition {
    let origin = element.absolute_position();
    let size = element.size();
    LogicalPosition::new(
        origin.x + size.width * fraction,
        origin.y + size.height * 0.5,
    )
}
fn click(app: &SliderMarksFixture, at: LogicalPosition) {
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
fn wheel(app: &SliderMarksFixture, at: LogicalPosition, delta_x: f32, delta_y: f32) {
    app.window()
        .dispatch_event(WindowEvent::PointerMoved { position: at });
    app.window().dispatch_event(WindowEvent::PointerScrolled {
        position: at,
        delta_x,
        delta_y,
    });
}
#[test]
fn labelled_marks_reserve_room_and_unmarked_sliders_keep_their_height() {
    let app = fixture();
    let plain = app.get_plain_height();
    let marked = app.get_marked_height();
    assert!(
        marked > plain,
        "marked {marked:?} must exceed plain {plain:?}"
    );
}

#[test]
fn only_marks_inside_the_range_draw_their_labels() {
    let app = fixture();
    let labels: Vec<String> = ElementQuery::from_root(&app)
        .match_descendants()
        .find_all()
        .iter()
        .flat_map(|handle| {
            handle
                .accessible_label()
                .map(|label| label.to_string())
                .into_iter()
        })
        .collect();
    for shown in ["Min", "Odd", "Half", "Max", "Low", "High", "Bottom", "Top"] {
        assert!(
            labels.iter().any(|label| label == shown),
            "{shown} missing in {labels:?}"
        );
    }
    assert!(!labels.iter().any(|label| label == "Out"), "{labels:?}");
}

#[test]
fn the_label_on_the_current_value_describes_the_slider_using_the_snapped_lattice() {
    let app = fixture();
    let marked = element(&app, "marked");
    // 47 snaps onto 50 on a step of 10 and comes first in the model.
    assert_eq!(marked.accessible_description().as_deref(), Some("Odd"));
    app.set_marked_value("100".into());
    assert_eq!(marked.accessible_description().as_deref(), Some("Max"));
    app.set_marked_value("30".into());
    assert_eq!(marked.accessible_description().as_deref(), Some(""));
}

#[test]
fn wheel_steps_only_an_opted_in_focused_slider_and_up_raises() {
    let app = fixture();
    let plain = element(&app, "plain");
    click(&app, centre(&plain, 0.5));
    let before = app.get_plain_value();
    wheel(&app, centre(&plain, 0.5), 0., 1.);
    assert_eq!(app.get_plain_value(), before, "wheel is off by default");

    let wheel_slider = element(&app, "wheel");
    // Not focused yet: the wheel is left to the surrounding scroll view.
    app.set_wheel_value("50".into());
    wheel(&app, centre(&wheel_slider, 0.5), 0., 1.);
    assert_eq!(app.get_wheel_value(), "50");
    click(&app, centre(&wheel_slider, 0.5));
    app.set_wheel_value("50".into());
    wheel(&app, centre(&wheel_slider, 0.5), 0., 1.);
    assert_eq!(app.get_wheel_value(), "51");
    wheel(&app, centre(&wheel_slider, 0.5), 0., -1.);
    wheel(&app, centre(&wheel_slider, 0.5), 0., -1.);
    assert_eq!(app.get_wheel_value(), "49");
    wheel(&app, centre(&wheel_slider, 0.5), 1., 0.);
    assert_eq!(
        app.get_wheel_value(),
        "50",
        "sideways towards the far end raises"
    );
}

#[test]
fn wheel_follows_flow_and_stops_at_the_bounds() {
    let app = fixture();
    let mirrored = element(&app, "rtl-wheel");
    click(&app, centre(&mirrored, 0.5));
    app.set_rtl_wheel_value("50".into());
    wheel(&app, centre(&mirrored, 0.5), 1., 0.);
    assert_eq!(app.get_rtl_wheel_value(), "49");
    wheel(&app, centre(&mirrored, 0.5), -1., 0.);
    assert_eq!(app.get_rtl_wheel_value(), "50");
    app.set_rtl_wheel_value("100".into());
    wheel(&app, centre(&mirrored, 0.5), 0., 1.);
    assert_eq!(app.get_rtl_wheel_value(), "100");
}

#[test]
fn range_wheel_moves_the_focused_thumb_and_keeps_the_pair_ordered() {
    let app = fixture();
    let range = element(&app, "range-wheel");
    // A press near the right end focuses the upper thumb.
    click(&app, centre(&range, 0.85));
    app.set_range_wheel_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "80".into(),
    });
    wheel(&app, centre(&range, 0.5), 0., 1.);
    let value = app.get_range_wheel_value();
    assert_eq!((value.lower.as_str(), value.upper.as_str()), ("20", "81"));
    app.set_range_wheel_value(ui::DecimalRangeView {
        lower: "20".into(),
        upper: "20".into(),
    });
    wheel(&app, centre(&range, 0.5), 0., -1.);
    let value = app.get_range_wheel_value();
    assert!(
        value.lower.parse::<f32>().unwrap() <= value.upper.parse::<f32>().unwrap(),
        "{value:?}"
    );
}

#[test]
fn vertical_wheel_raises_upwards() {
    let app = fixture();
    let vertical = element(&app, "vertical-wheel");
    let origin = vertical.absolute_position();
    let size = vertical.size();
    let at = LogicalPosition::new(origin.x + size.width * 0.3, origin.y + size.height * 0.5);
    click(&app, at);
    app.set_vertical_wheel_value("50".into());
    wheel(&app, at, 0., 1.);
    assert_eq!(app.get_vertical_wheel_value(), "51");
    wheel(&app, at, 1., 0.);
    assert_eq!(
        app.get_vertical_wheel_value(),
        "51",
        "a vertical slider ignores sideways wheels"
    );
}
