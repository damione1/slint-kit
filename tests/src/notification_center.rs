// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementHandle;
use slint::{
    ComponentHandle, ModelRc, VecModel,
    platform::{Key, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};

const BASE_TOKEN: i32 = 1000;
const ROW_COUNT: i32 = 60;

fn fixture() -> NotificationCenterFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = NotificationCenterFixture::new().unwrap();
    let rows = (0..ROW_COUNT)
        .map(|index| NotificationRow {
            item: OptionRow {
                token: BASE_TOKEN + index,
                generation: 1,
                label: format!("Notice {index}").into(),
                description: format!("Body {index}").into(),
                ..Default::default()
            },
            read: index % 2 == 1,
        })
        .collect::<Vec<_>>();
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

#[test]
fn unread_state_is_part_of_the_accessible_name() {
    let window = fixture();
    let name = |label: &str| ElementHandle::find_by_accessible_label(&window, label).next();
    assert!(name("Notice 0, Unread").is_some());
    assert!(name("Notice 1").is_some());
    assert!(name("Notice 1, Unread").is_none());
    window.set_unread_label("Non lu".into());
    slint::platform::update_timers_and_animations();
    assert!(name("Notice 0, Non lu").is_some());
    assert_eq!(
        name("Notice 0, Non lu")
            .unwrap()
            .accessible_description()
            .as_deref(),
        Some("Body 0")
    );
}

#[test]
fn focus_request_scrolls_an_unmounted_row_into_view_and_focuses_it() {
    let window = fixture();
    let activated = Rc::new(RefCell::new(Vec::new()));
    window.on_activated({
        let activated = activated.clone();
        move |token, generation| activated.borrow_mut().push((token, generation))
    });
    // Row 50 is far outside the virtual viewport and has no delegate yet.
    assert!(
        ElementHandle::find_by_accessible_label(&window, "Notice 50, Unread")
            .next()
            .is_none()
    );
    let target = 50;
    window.set_focus_token(BASE_TOKEN + target);
    window.set_focus_index(target);
    window.set_focus_request(1);
    for _ in 0..3 {
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(5));
    }
    assert!(
        ElementHandle::find_by_accessible_label(&window, "Notice 50, Unread")
            .next()
            .is_some()
    );
    let row = ElementHandle::find_by_accessible_label(&window, "Notice 50, Unread")
        .next()
        .unwrap();
    let (top, height) = (row.absolute_position().y, row.size().height);
    assert!(
        (0. ..360.).contains(&top),
        "the focused row starts inside the viewport: {top} (height {height})"
    );
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    slint::platform::update_timers_and_animations();
    assert_eq!(*activated.borrow(), vec![(BASE_TOKEN + target, 1)]);
}
