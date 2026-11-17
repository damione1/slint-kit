// SPDX-License-Identifier: GPL-3.0-or-later
//! `KitFocusTarget` delegates focus to the real control and reports that control's focus once.
use crate::{FocusRequest, FocusTargetFixture};
use slint::ComponentHandle;
use std::{cell::RefCell, rc::Rc};

fn request(token: i32, generation: i32, request: i32) -> FocusRequest {
    FocusRequest {
        present: true,
        token,
        generation,
        request,
    }
}

#[test]
fn a_matching_request_focuses_the_real_control_in_one_step_and_reports_it_once() {
    i_slint_backend_testing::init_no_event_loop();
    let window = FocusTargetFixture::new().unwrap();
    let reports = Rc::new(RefCell::new(Vec::new()));
    let sink = reports.clone();
    window.on_observed(move |token, generation, focused| {
        sink.borrow_mut().push((token, generation, focused));
    });
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    assert!(!window.get_action_focused());

    window.set_request(request(7, 1, 1));
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_action_focused(),
        "one request puts focus on the real control"
    );
    assert_eq!(window.get_applied(), 1);
    assert_eq!(
        *reports.borrow(),
        [(7, 1, true)],
        "the host sees one gained report, with the registered generation"
    );

    window.invoke_focus_other();
    slint::platform::update_timers_and_animations();
    assert!(!window.get_action_focused());
    assert_eq!(*reports.borrow(), [(7, 1, true), (7, 1, false)]);
}

#[test]
fn a_stale_generation_or_ineligible_target_does_not_take_focus() {
    i_slint_backend_testing::init_no_event_loop();
    let window = FocusTargetFixture::new().unwrap();
    window.show().unwrap();
    window.set_generation(2);
    window.set_request(request(7, 1, 1));
    slint::platform::update_timers_and_animations();
    assert!(
        !window.get_action_focused(),
        "recycled target ignores the old generation"
    );

    window.set_eligible(false);
    window.set_request(request(7, 2, 2));
    slint::platform::update_timers_and_animations();
    assert!(!window.get_action_focused());
    assert_eq!(window.get_applied(), 0);

    window.set_eligible(true);
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_action_focused(),
        "becoming eligible applies the pending request"
    );
    assert_eq!(window.get_applied(), 2);
}
