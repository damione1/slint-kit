// SPDX-License-Identifier: GPL-3.0-or-later
//! The kit's mount and reveal test for tabs. The body is the one of the kit
//! test crate; only its helpers are restated, because that module also covers
//! collections this crate does not build.
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, VecModel,
    platform::{Key, WindowEvent},
};
use std::time::Duration;

fn init() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}

fn key(window: &impl ComponentHandle, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}

fn frame(window: &impl ComponentHandle, ms: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(ms));
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
}

fn settle_scroll(window: &impl ComponentHandle) {
    for _ in 0..80 {
        slint::platform::update_timers_and_animations();
        for element in ElementQuery::from_root(window).find_all() {
            let _ = element.size();
        }
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(16));
    }
    slint::platform::update_timers_and_animations();
}

#[test]
fn controlled_tab_is_visible_on_mount_and_remount_without_activation_or_focus() {
    use crate::{ChoiceFlow, TabsMountFixture};
    use i_slint_backend_testing::{AccessibleRole, Orientation};

    init();
    for (orientation, flow, token) in [
        (Orientation::Horizontal, ChoiceFlow::LeftToRight, 8),
        (Orientation::Horizontal, ChoiceFlow::RightToLeft, 1),
        (Orientation::Vertical, ChoiceFlow::RightToLeft, 8),
    ] {
        let window = TabsMountFixture::new().unwrap();
        slint_kit::install(&window);
        window.set_flow(flow);
        window.set_orientation(orientation);
        let options = std::rc::Rc::new(VecModel::from(
            (1..=8)
                .map(|token| slint_kit::ui::OptionRow {
                    token,
                    generation: 1,
                    label: format!("Destination {token}").into(),
                    ..Default::default()
                })
                .collect::<Vec<_>>(),
        ));
        window.set_options(options.clone().into());
        window.set_selected(slint_kit::ui::SelectionRef {
            present: true,
            token,
        });
        window.show().unwrap();
        window.invoke_focus_outside();
        for mount in 1..=2 {
            window.set_mounted(true);
            settle_scroll(&window);
            let visible_tab = |token| {
                let viewport = ElementQuery::from_root(&window)
                    .match_type_name("Flickable")
                    .find_first()
                    .unwrap();
                let tab = viewport
                    .query_descendants()
                    .match_accessible_role(AccessibleRole::Tab)
                    .match_accessible_label(format!("Destination {token}"))
                    .find_first()
                    .unwrap_or_else(|| {
                        panic!("tab {token} clipped on mount {mount} in {orientation:?}/{flow:?}")
                    });
                let origin = viewport.absolute_position();
                let pos = tab.absolute_position();
                assert!(pos.x >= origin.x - 0.1 && pos.y >= origin.y - 0.1);
                assert!(pos.x + tab.size().width <= origin.x + viewport.size().width + 0.1);
                assert!(pos.y + tab.size().height <= origin.y + viewport.size().height + 0.1);
                tab
            };
            assert_eq!(visible_tab(token).accessible_item_selected(), Some(true));
            // The same token moves when its neighbours are relabelled, and
            // has less space when the caller narrows the strip.
            use slint::Model;
            let mut neighbour = options.row_data(3).unwrap();
            neighbour.label = format!("A longer neighbouring destination on mount {mount}").into();
            options.set_row_data(3, neighbour);
            settle_scroll(&window);
            assert_eq!(visible_tab(token).accessible_item_selected(), Some(true));
            window.set_strip_width(190.);
            window.set_strip_height(90.);
            settle_scroll(&window);
            assert_eq!(visible_tab(token).accessible_item_selected(), Some(true));
            assert!(window.get_outside_focused());
            assert_eq!(window.get_choices(), 0);
            // Manual navigation may reveal a different tab without choosing it.
            key(&window, Key::Tab);
            let (key_to_other_end, other_token) = if token == 8 {
                (Key::Home, 1)
            } else {
                (Key::End, 8)
            };
            key(&window, key_to_other_end);
            settle_scroll(&window);
            assert_eq!(
                visible_tab(other_token).accessible_item_selected(),
                Some(false)
            );
            assert_eq!(window.get_selected().token, token);
            assert_eq!(window.get_choices(), 0);
            window.invoke_focus_outside();
            window.set_mounted(false);
            frame(&window, 0);
            assert!(
                ElementQuery::from_root(&window)
                    .match_inherits("KitTabs")
                    .find_first()
                    .is_none()
            );
        }
    }
}
