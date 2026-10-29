// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{PointerEventButton, WindowEvent},
};

fn fixture() -> ScrollBarFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = ScrollBarFixture::new().unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window
}
/// The standalone bar, placed at x 440 with a 200px track.
fn bar(window: &ScrollBarFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitScrollBar")
        .find_first()
        .unwrap()
}
fn thumb(bar: &ElementHandle) -> ElementHandle {
    bar.query_descendants()
        .match_type_name("Rectangle")
        .find_first()
        .expect("thumb")
}
fn press(window: &ScrollBarFixture, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}
fn release(window: &ScrollBarFixture, x: f32, y: f32) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, y),
            button: PointerEventButton::Left,
        });
}

#[test]
fn thumb_length_and_offset_project_the_page_and_the_value() {
    let window = fixture();
    let recipe = window.invoke_recipe(window.get_light());
    let bar = bar(&window);
    assert_eq!(bar.size().width, recipe.thickness);
    let track = 200. - 2. * recipe.inset;
    let thumb = thumb(&bar);
    // A 200px page over 800px of content shows a quarter of the track.
    assert!((thumb.size().height - track / 4.).abs() < 0.01);
    assert!((thumb.absolute_position().y - recipe.inset).abs() < 0.01);
    assert_eq!(thumb.size().width, recipe.thumb_thickness);
    let right = thumb.absolute_position().x + thumb.size().width;
    assert!((right - (440. + recipe.thickness - recipe.inset)).abs() < 0.01);
    window.set_value(-300.);
    let travel = track - track / 4.;
    assert!((thumb.absolute_position().y - (recipe.inset + travel / 2.)).abs() < 0.01);
    window.set_value(-600.);
    assert!((thumb.absolute_position().y - (recipe.inset + travel)).abs() < 0.01);
    // A tiny page keeps a usable thumb.
    window.set_page_size(1.);
    // Thumb geometry animates over the recipe's transition.
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(500));
    slint::platform::update_timers_and_animations();
    assert!(
        (thumb.size().height - recipe.min_thumb_length).abs() < 0.01,
        "{}",
        thumb.size().height
    );
}

#[test]
fn dragging_the_thumb_scrolls_proportionally_and_clamps() {
    let window = fixture();
    let recipe = window.invoke_recipe(window.get_light());
    let track = 200. - 2. * recipe.inset;
    let travel = track * 3. / 4.;
    press(&window, 445., recipe.inset + 10.);
    assert!(window.get_bar_active(), "a pressed bar is active");
    let thumb_width = thumb(&bar(&window)).size().width;
    assert_eq!(thumb_width, recipe.active_thumb_thickness);
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(445., recipe.inset + 10. + travel / 2.),
    });
    assert!(
        (window.get_value() + 300.).abs() < 0.5,
        "{}",
        window.get_value()
    );
    assert!(window.get_scrolls() > 0);
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(445., 390.),
    });
    assert_eq!(window.get_value(), -600.);
    release(&window, 445., 390.);
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(445., recipe.inset + 10.),
    });
    assert_eq!(
        window.get_value(),
        -600.,
        "a released thumb no longer drags"
    );
}

#[test]
fn a_track_press_moves_one_page_toward_the_pointer() {
    let window = fixture();
    press(&window, 445., 180.);
    release(&window, 445., 180.);
    assert_eq!(window.get_value(), -200.);
    assert_eq!(window.get_scrolls(), 1);
    press(&window, 445., 180.);
    release(&window, 445., 180.);
    press(&window, 445., 180.);
    release(&window, 445., 180.);
    assert_eq!(window.get_value(), -600., "paging stops at the end");
    press(&window, 445., 5.);
    release(&window, 445., 5.);
    assert_eq!(window.get_value(), -400.);
}

#[test]
fn the_wheel_over_the_bar_scrolls_within_bounds() {
    let window = fixture();
    let position = LogicalPosition::new(445., 100.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: 0.,
            delta_y: -120.,
        });
    assert_eq!(window.get_value(), -120.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: 0.,
            delta_y: 500.,
        });
    assert_eq!(window.get_value(), 0.);
}

#[test]
fn policy_and_enabled_decide_visibility_and_input() {
    let window = fixture();
    assert!(window.get_bar_visible());
    assert!(window.get_always_on_visible());
    assert!(!window.get_always_off_visible());
    window.set_maximum(0.);
    assert!(
        !window.get_bar_visible(),
        "as-needed hides with nothing to scroll"
    );
    assert!(window.get_always_on_visible());
    window.set_maximum(600.);
    window.set_enabled(false);
    press(&window, 445., 180.);
    release(&window, 445., 180.);
    assert_eq!(window.get_value(), 0., "a disabled bar ignores the pointer");
    assert!(!window.get_bar_active());
}

#[test]
fn the_bar_is_neither_focusable_nor_announced() {
    let window = fixture();
    for bar in ElementQuery::from_root(&window)
        .match_inherits("KitScrollBar")
        .find_all()
    {
        // Like the std-widgets bar: a pointer affordance outside the
        // accessibility tree; keyboard scrolling belongs to the viewport.
        assert_eq!(
            bar.accessible_role(),
            Some(i_slint_backend_testing::AccessibleRole::None)
        );
        assert!(
            bar.query_descendants()
                .match_type_name("FocusScope")
                .find_first()
                .is_none(),
            "a scroll bar never takes keyboard focus"
        );
    }
}

#[test]
fn scroll_area_uses_a_themed_bar_that_scrolls_its_content() {
    let window = fixture();
    let area = ElementQuery::from_root(&window)
        .match_inherits("KitScrollArea")
        .find_first()
        .unwrap();
    // The horizontal bar is always off; the vertical one runs along the viewport.
    let bars = area
        .query_descendants()
        .match_inherits("KitScrollBar")
        .find_all();
    let vertical = bars
        .iter()
        .find(|bar| bar.size().height > 100.)
        .expect("vertical bar");
    let recipe = window.invoke_recipe(window.get_light());
    assert_eq!(vertical.size().width, recipe.thickness);
    let right = vertical.absolute_position().x + vertical.size().width;
    assert!(
        (right - 200.).abs() < 0.01,
        "the bar overlays the trailing edge"
    );
    assert!(window.get_area_content() > window.get_area_viewport());
    let thumb = thumb(vertical);
    let y = thumb.absolute_position().y + thumb.size().height / 2.;
    let x = thumb.absolute_position().x + thumb.size().width / 2.;
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, y),
    });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: LogicalPosition::new(x, y),
        button: PointerEventButton::Left,
    });
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(x, y + 40.),
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(x, y + 40.),
            button: PointerEventButton::Left,
        });
    assert!(
        window.get_area_offset() < 0.,
        "dragging the bar scrolls the area"
    );
}
