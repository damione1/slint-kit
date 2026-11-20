// SPDX-License-Identifier: GPL-3.0-or-later
use crate::DialogAdaptiveRowFixture;
use slint::ComponentHandle;

#[test]
fn a_row_choosing_its_orientation_from_its_own_width_composes_in_a_dialog_body() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DialogAdaptiveRowFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    let width = window.get_row_width();
    assert!(width > 300., "the row fills the dialog body, got {width}");
    window.set_threshold(width - 40.);
    slint::platform::update_timers_and_animations();
    assert!(
        !window.get_stacked(),
        "a wide body keeps the row horizontal"
    );
    window.set_threshold(width + 40.);
    slint::platform::update_timers_and_animations();
    assert!(window.get_stacked(), "a narrow body stacks the row");
    assert!(
        window.get_control_x().abs() < 1.,
        "the stacked control starts at the row edge"
    );
}
