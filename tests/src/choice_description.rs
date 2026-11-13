// SPDX-License-Identifier: GPL-3.0-or-later
//! Helper text and label side of the checkbox and the switch.
use crate::ChoiceDescriptionFixture;
use i_slint_backend_testing::ElementQuery;
use slint::ComponentHandle;
use slint_kit::ui::ChoiceLabelPosition;

fn fixture() -> ChoiceDescriptionFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceDescriptionFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

fn description_of(window: &ChoiceDescriptionFixture, label: &str) -> Option<String> {
    let wanted = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(wanted.as_str())
        })
        .find_all()
        .into_iter()
        .filter_map(|element| element.accessible_description())
        .map(|text| text.to_string())
        .find(|text| !text.is_empty())
}

#[test]
fn helper_text_is_announced_and_grows_the_row() {
    let window = fixture();
    assert_eq!(
        description_of(&window, "Notifications").as_deref(),
        Some("Receive a summary every Monday.")
    );
    assert_eq!(
        description_of(&window, "Weekly digest").as_deref(),
        Some("Partially selected. Receive a summary every Monday.")
    );
    assert!(window.get_check_height() > window.get_plain_height());
    window.set_description("".into());
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_check_height(), window.get_plain_height());
}

#[test]
fn label_position_moves_the_indicator_to_the_trailing_edge() {
    let window = fixture();
    let leading_x = window.get_toggle_x();
    window.set_position(ChoiceLabelPosition::Start);
    slint::platform::update_timers_and_animations();
    let toggle = ElementQuery::from_root(&window)
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Notifications"))
        .find_first()
        .unwrap();
    let track = toggle
        .query_descendants()
        .match_type_name("Rectangle")
        .match_predicate(|rectangle| {
            rectangle.size().width > rectangle.size().height * 1.5 && rectangle.size().width < 100.
        })
        .find_all()
        .into_iter()
        .min_by(|a, b| a.size().height.total_cmp(&b.size().height))
        .unwrap();
    let trailing = track.absolute_position().x + track.size().width;
    let row_end = leading_x + window.get_toggle_width();
    assert!(
        (row_end - trailing).abs() < 20.,
        "track ends at {trailing}, row at {row_end}"
    );
    assert!(track.absolute_position().x > leading_x + window.get_toggle_width() / 2.);
}
