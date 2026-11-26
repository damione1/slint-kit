// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ContentSizingFixture;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

fn fixture() -> ContentSizingFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ContentSizingFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

#[test]
fn compact_suffix_stays_before_the_step_buttons() {
    let window = fixture();
    let suffix = ElementQuery::from_root(&window)
        .match_predicate(|element| element.accessible_label().as_deref() == Some(" min"))
        .find_first()
        .expect("visible suffix");
    let decrement = ElementHandle::find_by_accessible_label(&window, "Decrease value")
        .next()
        .expect("step button");
    let suffix_end = suffix.absolute_position().x + suffix.size().width;
    assert!(
        suffix_end <= decrement.absolute_position().x,
        "suffix ends at {suffix_end}, buttons start at {}",
        decrement.absolute_position().x
    );
}

#[test]
fn a_label_only_row_centres_its_caption_on_the_control() {
    let window = fixture();
    let caption = ElementQuery::from_root(&window)
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Delay"))
        .find_first()
        .expect("row label");
    let control = ElementHandle::find_by_accessible_label(&window, "Decrease value")
        .next()
        .expect("step button");
    let caption_centre = caption.absolute_position().y + caption.size().height / 2.;
    let control_centre = control.absolute_position().y + control.size().height / 2.;
    assert!(
        (caption_centre - control_centre).abs() <= 1.,
        "label centre {caption_centre}, control centre {control_centre}"
    );
}

#[test]
fn a_hugging_bubble_keeps_an_unwrapped_measure_on_one_line() {
    let window = fixture();
    let flow = ElementHandle::find_by_accessible_label(&window, "Words")
        .next()
        .expect("wrapping flow");
    assert!(
        flow.size().height < 1.5 * window.get_flow_line_height(),
        "the last word wrapped: {}",
        flow.size().height
    );
}

#[test]
fn inline_alert_actions_follow_the_copy() {
    let window = fixture();
    let message = ElementQuery::from_root(&window)
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Capture paused."))
        .find_first()
        .expect("alert message");
    let action = ElementHandle::find_by_accessible_label(&window, "Resume")
        .next()
        .expect("alert action");
    let gap = action.absolute_position().x - message.absolute_position().x - message.size().width;
    assert!(
        (0. ..=24.).contains(&gap),
        "action is {gap}px after the copy"
    );
    assert!(
        message.size().height < 1.5 * window.get_flow_line_height(),
        "the copy wrapped: {}",
        message.size().height
    );
}
