// SPDX-License-Identifier: GPL-3.0-or-later
//! Scenarios that need a running event loop. The testing backend registers its
//! event-loop proxy once per process, so they share one test and one thread.
use crate::{store, task};

#[test]
fn event_loop_scenarios() {
    i_slint_backend_testing::init_integration_test_with_system_time();
    store::another_thread_updates_the_store_through_the_event_loop_and_dropped_stores_are_skipped();
    task::the_result_reaches_the_window_without_a_weak_reference_in_application_code();
    task::a_response_arriving_after_a_newer_request_is_discarded();
    task::the_older_response_is_discarded_even_when_it_arrives_first();
    task::load_reports_loading_then_the_outcome_and_a_stale_load_reports_nothing();
    task::cancel_discards_the_request_in_flight();
    task::a_destroyed_window_is_never_applied_to();
}
