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
fn a_top_aligned_row_keeps_its_caption_level_with_the_control() {
    let window = fixture();
    let caption = ElementQuery::from_root(&window)
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Prompt"))
        .find_first()
        .expect("row label");
    let control = ElementHandle::find_by_accessible_label(&window, "Prompt editor")
        .next()
        .expect("tall control");
    let offset = caption.absolute_position().y - control.absolute_position().y;
    assert!(
        offset.abs() <= 1.,
        "label is {offset}px below the control top"
    );
}

#[test]
fn compact_stepper_and_text_area_spacing_options_move_only_their_edge() {
    let window = fixture();
    let element = |label: &'static str| {
        ElementHandle::find_by_accessible_label(&window, label)
            .next()
            .expect(label)
    };
    let gap = |editor: ElementHandle, button: ElementHandle| {
        button.absolute_position().x - editor.absolute_position().x - editor.size().width
    };
    let tight = gap(element("Tight"), element("Tight less"));
    assert!(tight.abs() <= 0.5, "first button gap is {tight}px");
    let added = element("Lowered").size().height - element("Even").size().height;
    assert!((added - 14.).abs() <= 0.5, "top padding added {added}px");
    assert_eq!(
        element("Thin").size().height,
        element("Thick").size().height
    );
}

#[test]
fn a_locked_select_with_full_ink_stays_disabled() {
    let window = fixture();
    let select = ElementHandle::find_by_accessible_label(&window, "Locked model")
        .find(|element| element.accessible_enabled().is_some())
        .expect("locked select trigger");
    assert_eq!(select.accessible_enabled(), Some(false));
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
