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
        "the bar occupies the trailing gutter"
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

fn two_axis_frame(window: &ScrollBarFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("ScrollFrame")
        .match_predicate(|item| item.accessible_label().as_deref() == Some("Two axis viewport"))
        .find_first()
        .unwrap()
}

#[test]
fn stable_gutters_protect_both_content_edges_and_do_not_depend_on_bar_visibility() {
    let window = fixture();
    for mut theme in [window.get_light(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            let thickness = window.invoke_recipe(theme.clone()).thickness;
            window.set_wide_content(false);
            window.set_tall_content(false);
            slint::platform::update_timers_and_animations();
            assert!((window.get_frame_visible_width() - (240. - thickness)).abs() < 0.01);
            assert!((window.get_frame_visible_height() - (160. - thickness)).abs() < 0.01);
            assert!(
                two_axis_frame(&window)
                    .query_descendants()
                    .match_inherits("KitScrollBar")
                    .find_all()
                    .is_empty(),
                "as-needed bars are hidden for viewport-bound content"
            );
            window.set_wide_content(true);
            window.set_tall_content(true);
            slint::platform::update_timers_and_animations();
            let frame = two_axis_frame(&window);
            let bars = frame
                .query_descendants()
                .match_inherits("KitScrollBar")
                .find_all();
            assert_eq!(bars.len(), 2);
            let vertical = bars
                .iter()
                .find(|bar| bar.size().height > bar.size().width)
                .unwrap();
            let horizontal = bars
                .iter()
                .find(|bar| bar.size().width > bar.size().height)
                .unwrap();
            let origin = frame.absolute_position();
            assert!(
                (vertical.absolute_position().x - origin.x - window.get_frame_visible_width())
                    .abs()
                    < 0.01
            );
            assert!(
                (horizontal.absolute_position().y - origin.y - window.get_frame_visible_height())
                    .abs()
                    < 0.01
            );
            assert!((vertical.size().height - window.get_frame_visible_height()).abs() < 0.01);
            assert!((horizontal.size().width - window.get_frame_visible_width()).abs() < 0.01);
            let before = window.get_edge_clicks();
            // Native focus may reveal its outline on the first press. Re-query
            // the current button geometry before testing each visible face.
            for index in 0..2 {
                let action = ElementQuery::from_root(&window)
                    .match_inherits("KitButton")
                    .match_predicate(|item| {
                        item.accessible_label().as_deref() == Some("Edge action")
                    })
                    .find_first()
                    .unwrap();
                let position = action.absolute_position();
                let size = action.size();
                assert!(
                    position.x + size.width <= vertical.absolute_position().x + 0.01,
                    "scale={scale}, button={position:?}/{size:?}, vertical bar={:?}",
                    vertical.absolute_position()
                );
                assert!(
                    position.y + size.height <= horizontal.absolute_position().y + 0.01,
                    "scale={scale}, button={position:?}/{size:?}, horizontal bar={:?}",
                    horizontal.absolute_position()
                );
                let (x, y) = if index == 0 {
                    (position.x + size.width - 1., position.y + size.height / 2.)
                } else {
                    (position.x + size.width / 2., position.y + size.height - 1.)
                };
                let previous_offset = (window.get_frame_offset_x(), window.get_frame_offset_y());
                press(&window, x, y);
                release(&window, x, y);
                assert_eq!(
                    window.get_edge_clicks(),
                    before + index + 1,
                    "scale={scale}, edge={index}, point=({x},{y}), scroll=({},{}), button={position:?}/{size:?}",
                    window.get_frame_offset_x(),
                    window.get_frame_offset_y()
                );
                if index == 1 {
                    assert_eq!(
                        (window.get_frame_offset_x(), window.get_frame_offset_y()),
                        previous_offset,
                        "an already focused button's edge must not scroll either gutter"
                    );
                }
            }
            assert_eq!(window.get_edge_clicks(), before + 2);
            window.set_wide_content(false);
            slint::platform::update_timers_and_animations();
            assert_eq!(
                two_axis_frame(&window)
                    .query_descendants()
                    .match_inherits("KitScrollBar")
                    .find_all()
                    .len(),
                1,
                "vertical overflow must not manufacture a horizontal scrollbar"
            );
        }
    }
}

#[test]
fn scrollbar_policies_and_explicit_overlay_mode_control_the_viewport_footprint() {
    let window = fixture();
    let thickness = window.invoke_recipe(window.get_light()).thickness;
    window.set_vertical_off(true);
    window.set_horizontal_off(true);
    assert_eq!(window.get_frame_visible_width(), 240.);
    assert_eq!(window.get_frame_visible_height(), 160.);
    window.set_vertical_off(false);
    assert!((window.get_frame_visible_width() - (240. - thickness)).abs() < 0.01);
    assert_eq!(window.get_frame_visible_height(), 160.);
    window.set_horizontal_off(false);
    window.set_horizontal_always_on(true);
    assert!((window.get_frame_visible_height() - (160. - thickness)).abs() < 0.01);
    window.set_overlay_scrollbars(true);
    assert_eq!(window.get_frame_visible_width(), 240.);
    assert_eq!(window.get_frame_visible_height(), 160.);
}

#[test]
fn gutter_bars_keep_native_wheel_and_thumb_drag_scrolling_on_both_axes() {
    let window = fixture();
    let position = LogicalPosition::new(240., 240.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position,
            delta_x: -40.,
            delta_y: -30.,
        });
    assert!(window.get_frame_offset_x() < 0.);
    assert!(window.get_frame_offset_y() < 0.);
    let frame = two_axis_frame(&window);
    for vertical in [true, false] {
        let bar = frame
            .query_descendants()
            .match_inherits("KitScrollBar")
            .find_all()
            .into_iter()
            .find(|bar| (bar.size().height > bar.size().width) == vertical)
            .unwrap();
        let thumb = thumb(&bar);
        let origin = thumb.absolute_position();
        let size = thumb.size();
        let x = origin.x + size.width / 2.;
        let y = origin.y + size.height / 2.;
        let before = if vertical {
            window.get_frame_offset_y()
        } else {
            window.get_frame_offset_x()
        };
        press(&window, x, y);
        let moved = LogicalPosition::new(
            x + if vertical { 0. } else { 20. },
            y + if vertical { 20. } else { 0. },
        );
        window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position: moved });
        release(&window, moved.x, moved.y);
        let after = if vertical {
            window.get_frame_offset_y()
        } else {
            window.get_frame_offset_x()
        };
        assert!(after < before, "a gutter thumb still controls its axis");
    }
}

fn priority_fixture(policy: ScrollPanPolicy) -> PanPriorityFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = PanPriorityFixture::new().unwrap();
    window.set_pan_policy(policy);
    window.show().unwrap();
    window
}
fn priority_drag(window: &PanPriorityFixture, from: (f32, f32), to: (f32, f32)) {
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: LogicalPosition::new(from.0, from.1),
        button: PointerEventButton::Left,
    });
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(110));
    slint::platform::update_timers_and_animations();
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(to.0, to.1),
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(to.0, to.1),
            button: PointerEventButton::Left,
        });
}
#[test]
fn children_first_preserves_both_control_directions_and_single_commit() {
    let window = priority_fixture(ScrollPanPolicy::ChildrenFirst);
    for (from, to) in [((120., 40.), (120., 170.)), ((120., 170.), (120., 40.))] {
        let before = window.get_commits();
        priority_drag(&window, from, to);
        assert_eq!(window.get_commits(), before + 1);
        assert_eq!(window.get_offset(), 0.);
        assert!(!window.get_adjusting());
    }
}
#[test]
fn children_first_background_pan_crossing_control_keeps_original_scroll_owner() {
    let window = priority_fixture(ScrollPanPolicy::ChildrenFirst);
    priority_drag(&window, (20., 200.), (120., 80.));
    assert!(window.get_offset() < -50.);
    assert_eq!(window.get_commits(), 0);
    assert!(!window.get_adjusting());
    window.invoke_reveal_end();
    assert!(window.get_offset() <= -500.);
    window.invoke_reset_scroll();
    assert_eq!(window.get_offset(), 0.);
}
#[test]
fn native_policy_keeps_existing_flickable_arbitration() {
    let window = priority_fixture(ScrollPanPolicy::Native);
    priority_drag(&window, (120., 170.), (120., 40.));
    assert!(window.get_offset() < -50.);
    assert_eq!(window.get_commits(), 0);
    assert!(!window.get_adjusting());
}
#[test]
fn children_first_disabling_control_cancels_without_commit_and_unclaimed_pan_still_works() {
    let window = priority_fixture(ScrollPanPolicy::ChildrenFirst);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: LogicalPosition::new(120., 40.),
        button: PointerEventButton::Left,
    });
    assert!(window.get_adjusting());
    window.set_control_enabled(false);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: LogicalPosition::new(120., 170.),
            button: PointerEventButton::Left,
        });
    assert_eq!(window.get_commits(), 0);
    assert!(!window.get_adjusting());
    priority_drag(&window, (120., 170.), (120., 40.));
    assert!(window.get_offset() < -50.);
}

#[test]
fn children_first_keeps_wheel_and_scrollbar_input_available() {
    let window = priority_fixture(ScrollPanPolicy::ChildrenFirst);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: LogicalPosition::new(20., 100.),
            delta_x: 0.,
            delta_y: -100.,
        });
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(250));
    slint::platform::update_timers_and_animations();
    assert!(window.get_offset() <= -90.);
    window.invoke_reset_scroll();
    let vertical = ElementQuery::from_root(&window)
        .match_inherits("KitScrollBar")
        .find_all()
        .into_iter()
        .find(|bar| bar.size().height > 200.)
        .unwrap();
    let knob = thumb(&vertical);
    let p = knob.absolute_position();
    let x = p.x + knob.size().width / 2.;
    let y = p.y + knob.size().height / 2.;
    priority_drag(&window, (x, y), (x, y + 50.));
    assert!(window.get_offset() < -50.);
    assert_eq!(window.get_commits(), 0);
}
