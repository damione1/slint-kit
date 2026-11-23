// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ChatBubbleContentFixture;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{ComponentHandle, platform::PointerEventButton};

fn fixture() -> ChatBubbleContentFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ChatBubbleContentFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

fn bubbles(window: &ChatBubbleContentFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_id("KitChatBubble::bubble")
        .find_all()
}

#[test]
fn bubble_tone_overrides_the_side_fill() {
    let window = fixture();
    let snapshot = window.window().take_snapshot().unwrap();
    let bubble = &bubbles(&window)[0];
    let (position, size) = (bubble.absolute_position(), bubble.size());
    let probe = snapshot.as_slice()[(position.y + size.height / 2.) as usize
        * snapshot.width() as usize
        + (position.x + 4.) as usize];
    let expected = window.get_subtle();
    assert_eq!(
        [probe.r, probe.g, probe.b],
        [expected.red(), expected.green(), expected.blue()]
    );
    assert_ne!(window.get_subtle(), window.get_accent());
}

#[test]
fn children_are_hosted_inside_the_bubble_and_stay_interactive() {
    let window = fixture();
    let bubble = &bubbles(&window)[1];
    let button = ElementHandle::find_by_accessible_label(&window, "Open")
        .find(|element| element.accessible_role() == Some(AccessibleRole::Button))
        .unwrap();
    let (outer, inner) = (
        (bubble.absolute_position(), bubble.size()),
        (button.absolute_position(), button.size()),
    );
    assert!(inner.0.x >= outer.0.x && inner.0.y >= outer.0.y);
    assert!(inner.0.x + inner.1.width <= outer.0.x + outer.1.width + 0.5);
    assert!(inner.0.y + inner.1.height <= outer.0.y + outer.1.height + 0.5);
    button.mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_opened(), 1);
}

#[test]
fn content_share_limits_the_bubble_column() {
    let window = fixture();
    let all = bubbles(&window);
    let (full, default, sent) = (
        all[2].size().width,
        all[3].size().width,
        all[4].size().width,
    );
    assert!((full - 384.).abs() < 0.5, "{full}");
    assert!(default < full, "{default} {full}");
    assert!(sent < full, "{sent} {full}");
    assert!(all[4].absolute_position().x > all[3].absolute_position().x);
}
