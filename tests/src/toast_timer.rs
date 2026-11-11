// SPDX-License-Identifier: GPL-3.0-or-later
use slint::Model;
use slint_kit::{
    ToastBinding, ToastLifetime,
    ui::{ToastEntry, ToastInteraction},
};
use std::{num::NonZeroUsize, time::Duration};

fn entry(title: &str) -> ToastEntry {
    ToastEntry {
        title: title.into(),
        ..Default::default()
    }
}
fn next_timer() -> Option<Duration> {
    slint::platform::duration_until_next_timer_update()
}

#[test]
fn toast_timer_stops_for_empty_persistent_and_independently_paused_queues() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(3).unwrap());
    assert_eq!(
        next_timer(),
        None,
        "constructing an empty queue must not arm a poll"
    );
    binding
        .push(1, entry("Persistent"), ToastLifetime::Persistent)
        .unwrap();
    assert_eq!(next_timer(), None);
    binding
        .push(
            2,
            entry("Timed"),
            ToastLifetime::Timed(Duration::from_secs(60)),
        )
        .unwrap();
    let timed = binding.model().row_data(1).unwrap().token;
    assert!(next_timer().is_some_and(|delay| delay > Duration::from_secs(30)));
    let pause = binding.interaction_handler();
    pause(timed, ToastInteraction::Pointer, true);
    assert_eq!(next_timer(), None);
    pause(timed, ToastInteraction::Focus, true);
    pause(timed, ToastInteraction::Pointer, false);
    assert_eq!(next_timer(), None, "focus retains its independent pause");
    pause(timed, ToastInteraction::Focus, false);
    assert!(next_timer().is_some_and(|delay| delay > Duration::from_secs(30)));
    assert_eq!(binding.dismiss(timed), Some(2));
    assert_eq!(
        next_timer(),
        None,
        "the remaining persistent toast needs no timer"
    );
}

#[test]
fn toast_timer_rearms_for_an_earlier_deadline_and_stops_after_zero_lifetime_expiry() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(2).unwrap());
    binding
        .push(
            1,
            entry("Later"),
            ToastLifetime::Timed(Duration::from_secs(60)),
        )
        .unwrap();
    binding
        .push(
            2,
            entry("Earlier"),
            ToastLifetime::Timed(Duration::from_secs(10)),
        )
        .unwrap();
    let earlier = binding.model().row_data(1).unwrap().token;
    let delay = next_timer().unwrap();
    assert!(delay > Duration::from_secs(5) && delay <= Duration::from_secs(10));
    assert_eq!(binding.dismiss(earlier), Some(2));
    assert!(next_timer().is_some_and(|delay| delay > Duration::from_secs(30)));
    let later = binding.model().row_data(0).unwrap().token;
    assert_eq!(binding.dismiss(later), Some(1));
    assert_eq!(next_timer(), None);
    binding
        .push(3, entry("Expires"), ToastLifetime::Timed(Duration::ZERO))
        .unwrap();
    assert_eq!(next_timer(), Some(Duration::ZERO));
    slint::platform::update_timers_and_animations();
    assert_eq!(binding.model().row_count(), 0);
    assert_eq!(
        next_timer(),
        None,
        "a completed one-shot must not rearm an idle poll"
    );
}

#[test]
fn the_last_binding_drop_stops_the_timer_while_a_projected_model_survives() {
    i_slint_backend_testing::init_no_event_loop();
    let binding = ToastBinding::new(NonZeroUsize::new(1).unwrap());
    binding
        .push(
            (),
            entry("Timed"),
            ToastLifetime::Timed(Duration::from_secs(60)),
        )
        .unwrap();
    let projected = binding.model();
    let clone = binding.clone();
    drop(binding);
    assert!(next_timer().is_some(), "a live clone still owns its timer");
    drop(clone);
    assert_eq!(
        next_timer(),
        None,
        "the timer closure cannot retain its binding"
    );
    assert_eq!(
        projected.row_count(),
        1,
        "retaining the projection does not retain the timer"
    );
}
