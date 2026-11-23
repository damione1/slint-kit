// SPDX-License-Identifier: GPL-3.0-or-later
use crate::LayoutFillFixture;
use slint::ComponentHandle;

fn fixture() -> LayoutFillFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = LayoutFillFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}

#[test]
fn filling_surface_and_card_hand_their_allocated_height_to_a_scroll_area() {
    let window = fixture();
    assert!((window.get_surface_height() - 150.).abs() < 0.5);
    assert!((window.get_card_height() - 150.).abs() < 0.5);
    assert!(
        window.get_surface_scroll_height() > 100.,
        "{}",
        window.get_surface_scroll_height()
    );
    assert!(
        window.get_card_scroll_height() > 90.,
        "{}",
        window.get_card_scroll_height()
    );
}

#[test]
fn surface_and_card_without_fill_keep_their_natural_height() {
    let window = fixture();
    assert!((window.get_packed_surface_height() - window.get_packed_surface_natural()).abs() < 0.5);
    assert!((window.get_packed_card_height() - window.get_packed_card_natural()).abs() < 0.5);
    assert!(window.get_packed_surface_height() < 60.);
}

#[test]
fn inline_children_with_zero_preferred_and_minimum_width_share_the_row_equally() {
    let window = fixture();
    let (a, b, c) = (
        window.get_a_width(),
        window.get_b_width(),
        window.get_c_width(),
    );
    assert!((a - b).abs() < 0.5 && (b - c).abs() < 0.5, "{a} {b} {c}");
    assert!(a > 90., "{a}");
}
