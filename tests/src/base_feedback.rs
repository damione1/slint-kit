// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle, LogicalPosition,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

fn fixture() -> BaseFeedbackFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseFeedbackFixture::new().unwrap();
    window.show().unwrap();
    window
}
fn element(window: &impl ComponentHandle, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .expect("accessible element")
}
fn settle(milliseconds: u64) {
    i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(milliseconds));
    slint::platform::update_timers_and_animations();
}
fn key(window: &impl ComponentHandle, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn in_bounds(element: &ElementHandle, width: f32, height: f32) {
    let p = element.absolute_position();
    let s = element.size();
    assert!(
        p.x >= 0. && p.y >= 0. && p.x + s.width <= width && p.y + s.height <= height,
        "out of bounds: {p:?} {s:?}"
    );
}

#[test]
fn keyboard_tooltip_escapes_clipping_and_escape_preserves_the_trigger_focus() {
    let window = fixture();
    window.invoke_focus_trigger();
    settle(1);
    assert!(window.get_tooltip_visible());
    let tip = element(&window, "Keyboard help");
    in_bounds(&tip, 640., 480.);
    assert!(
        tip.absolute_position().x < 600.,
        "the root layer escapes the 30px-wide clipped ancestor"
    );
    let mut theme = window.get_theme();
    theme.text_scale = 2.;
    window.set_theme(theme);
    window.set_viewport_width(360.);
    window.set_viewport_height(280.);
    settle(1);
    in_bounds(&element(&window, "Keyboard help"), 360., 280.);
    key(&window, Key::Escape);
    settle(1);
    assert!(!window.get_tooltip_visible());
    assert!(window.get_target_focused());
    window.invoke_focus_trigger();
    settle(1);
    assert!(
        !window.get_tooltip_visible(),
        "the same focus cannot immediately reopen a dismissed tip"
    );
    window.invoke_blur_trigger();
    settle(1);
    window.invoke_focus_trigger();
    settle(1);
    assert!(window.get_tooltip_visible());
}

#[test]
fn a_tooltip_is_as_wide_as_its_text_up_to_the_recipe_reading_width() {
    let window = fixture();
    let theme = window.get_theme();
    let recipe = window.invoke_tooltip(theme.clone());
    assert_eq!(recipe.padding_inline, theme.spacing.unit * 3.);
    assert_eq!(recipe.padding_block, theme.spacing.unit * 1.5);
    assert_eq!(recipe.max_width, theme.spacing.unit * 80.);
    assert_eq!(recipe.fill, theme.text_primary);
    assert_eq!(recipe.foreground, theme.surface);
    window.invoke_focus_trigger();
    settle(1);
    // A short tip wraps nothing and stays far narrower than the cap.
    let short = element(&window, "Keyboard help").size().width;
    assert!(
        short > 0. && short < recipe.max_width / 2.,
        "short tip text box {short}"
    );
    let long = "Create a new project in this workspace, then invite the people who will review it.";
    window.set_help(long.into());
    settle(1);
    let width = element(&window, long).size().width;
    assert!(
        (width - (recipe.max_width - 2. * recipe.padding_inline)).abs() < 0.5,
        "long tip text box {width}"
    );
    // The cap follows text-scale.
    let mut scaled = theme.clone();
    scaled.text_scale = 2.;
    assert_eq!(
        window.invoke_tooltip(scaled).max_width,
        recipe.max_width * 2.
    );
}

#[test]
fn tooltip_projection_is_window_local_and_overlay_invalidation_blocks_reopening() {
    let first = fixture();
    let second = BaseFeedbackFixture::new().unwrap();
    second.show().unwrap();
    first.invoke_focus_trigger();
    settle(1);
    assert!(first.get_tooltip_visible());
    assert!(!second.get_tooltip_visible());
    second.invoke_focus_trigger();
    settle(1);
    assert!(first.get_tooltip_visible());
    assert!(second.get_tooltip_visible());
    first.set_layer_enabled(false);
    settle(1);
    assert!(!first.get_tooltip_visible());
    assert!(second.get_tooltip_visible());
    first.set_viewport_width(480.);
    first.set_layer_enabled(true);
    settle(1);
    assert!(!first.get_tooltip_visible());
    first.invoke_blur_trigger();
    settle(1);
    first.invoke_focus_trigger();
    settle(1);
    assert!(first.get_tooltip_visible());
}

#[test]
fn toast_hover_and_keyboard_focus_pause_independently_and_forward_stable_tokens() {
    let window = fixture();
    let sources = Rc::new(RefCell::new((false, false)));
    let observed = Rc::clone(&sources);
    window.on_toast_interaction(move |token, source, engaged| {
        assert_eq!(token, 42);
        let mut observed = observed.borrow_mut();
        match source {
            ToastInteraction::Pointer => observed.0 = engaged,
            ToastInteraction::Focus => observed.1 = engaged,
        }
    });
    let actions = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&actions);
    window.on_toast_action(move |token| observed.borrow_mut().push(token));
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(30., 110.),
    });
    settle(1);
    assert_eq!(*sources.borrow(), (true, false));
    element(&window, "Undo").mock_single_click(PointerEventButton::Left);
    settle(1);
    assert_eq!(*sources.borrow(), (true, true));
    assert_eq!(&*actions.borrow(), &[42]);
    window.window().dispatch_event(WindowEvent::PointerMoved {
        position: LogicalPosition::new(620., 20.),
    });
    settle(1);
    assert_eq!(
        *sources.borrow(),
        (false, true),
        "leaving the pointer must not resume a focused toast"
    );
    window.invoke_blur_trigger();
    settle(1);
    assert_eq!(*sources.borrow(), (false, false));
    let dismissed = Rc::new(Cell::new(0));
    let observed = Rc::clone(&dismissed);
    window.on_toast_dismiss(move |token| {
        assert_eq!(token, 42);
        observed.set(observed.get() + 1);
    });
    element(&window, "Dismiss Saved").invoke_accessible_default_action();
    assert_eq!(dismissed.get(), 1);
}

#[test]
fn assertive_alert_dismisses_once_and_removes_its_action_from_navigation() {
    let window = fixture();
    let alert = element(&window, "Review needed");
    assert_eq!(
        alert.accessible_live_region(),
        Some(i_slint_backend_testing::AccessibleLiveness::Assertive)
    );
    let dismissed = Rc::new(Cell::new(0));
    let observed = Rc::clone(&dismissed);
    window.on_alert_dismissed(move || observed.set(observed.get() + 1));
    element(&window, "Dismiss Review needed").mock_single_click(PointerEventButton::Left);
    key(&window, Key::Return);
    key(&window, Key::Space);
    settle(1);
    assert_eq!(dismissed.get(), 1);
    assert!(!window.get_alert_visible());
    assert!(
        ElementQuery::from_root(&window)
            .match_predicate(
                |element| element.accessible_label().as_deref() == Some("Dismiss Review needed")
            )
            .find_first()
            .is_none()
    );
}

#[test]
fn progress_accessibility_is_bounded_and_reduced_motion_freezes_loading_marks() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseProgressFixture::new().unwrap();
    let mut theme = window.get_theme();
    theme.reduced_motion = true;
    window.set_theme(theme.clone());
    window.show().unwrap();
    for (value, expected) in [(-0.5, "0%"), (0.37, "37%"), (1.5, "100%")] {
        window.set_progress(value);
        for label in ["Upload progress", "Review progress"] {
            let progress = element(&window, label);
            assert_eq!(
                progress.accessible_role(),
                Some(i_slint_backend_testing::AccessibleRole::ProgressIndicator)
            );
            assert_eq!(progress.accessible_value().as_deref(), Some(expected));
            assert_eq!(progress.accessible_value_minimum(), Some(0.));
            assert_eq!(progress.accessible_value_maximum(), Some(100.));
        }
    }
    let spinner = ElementQuery::from_root(&window)
        .match_inherits("KitSpinner")
        .find_first()
        .unwrap();
    let dots = spinner
        .query_descendants()
        .match_type_name("Rectangle")
        .find_all();
    assert_eq!(dots.len(), 8);
    let skeleton = ElementQuery::from_root(&window)
        .match_inherits("KitSkeleton")
        .find_first()
        .unwrap();
    for elapsed in [1, 650, 1700] {
        settle(elapsed);
        assert!(dots.iter().all(|dot| dot.computed_opacity() == 1.));
        assert_eq!(skeleton.computed_opacity(), 1.);
    }
    theme.reduced_motion = false;
    window.set_theme(theme.clone());
    settle(1);
    let before: Vec<_> = dots.iter().map(ElementHandle::computed_opacity).collect();
    settle(101);
    let after: Vec<_> = dots.iter().map(ElementHandle::computed_opacity).collect();
    assert_ne!(before, after);
    theme.reduced_motion = true;
    window.set_theme(theme);
    settle(1500);
    assert!(dots.iter().all(|dot| dot.computed_opacity() == 1.));
    assert_eq!(skeleton.computed_opacity(), 1.);
}

#[test]
fn revealing_an_oversized_item_settles_at_its_leading_edge_without_oscillating() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseLayoutFixture::new().unwrap();
    window.show().unwrap();
    let review = element(&window, "Review");
    window.set_viewport_height(80.);
    let scroll = ElementQuery::from_root(&window)
        .match_inherits("KitScrollArea")
        .find_first()
        .unwrap();
    let card = ElementQuery::from_root(&window)
        .match_inherits("KitCard")
        .find_first()
        .unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for width in [320., 800.] {
            window.set_viewport_width(width);
            settle(1);
            let top = card.absolute_position().y
                - scroll.absolute_position().y
                - window.get_scroll_offset();
            let bottom = top + card.size().height;
            assert!(bottom - top > window.get_scroll_viewport_height());
            window.invoke_reveal_range(top, bottom);
            settle(1);
            let offset = window.get_scroll_offset();
            assert!(
                (offset + top).abs() < 1.,
                "oversized target keeps its leading edge visible"
            );
            for _ in 0..8 {
                window.invoke_reveal_range(top, bottom);
                settle(1);
                assert!(
                    (window.get_scroll_offset() - offset).abs() < 0.1,
                    "repeated focus geometry must not alternate scroll edges"
                );
            }
            let target = review.absolute_position().y
                - scroll.absolute_position().y
                - window.get_scroll_offset();
            let end = target + review.size().height;
            assert!(end - target <= window.get_scroll_viewport_height());
            window.invoke_reveal_range(target, end);
            settle(1);
            let offset = window.get_scroll_offset();
            assert!(target + offset >= -0.1);
            assert!(end + offset <= window.get_scroll_viewport_height() + 0.1);
            window.invoke_reveal_range(target, end);
            settle(1);
            assert!((window.get_scroll_offset() - offset).abs() < 0.1);
            // Returning to a large card must preserve an existing scroll
            // position when its leading edge is already on screen.
            window.invoke_reveal_range(0., 1.);
            settle(1);
            let inset_top = top + 16.;
            assert!(bottom - inset_top > window.get_scroll_viewport_height());
            let offset = window.get_scroll_offset();
            assert!(inset_top + offset >= 0.);
            assert!(inset_top + offset < window.get_scroll_viewport_height());
            window.invoke_reveal_range(inset_top, bottom);
            settle(1);
            assert!(
                (window.get_scroll_offset() - offset).abs() < 0.1,
                "an already visible leading edge must preserve the saved scroll position"
            );
        }
    }
}

#[test]
fn card_and_layout_reflow_at_double_text_without_recreating_the_native_editor() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseLayoutFixture::new().unwrap();
    window.show().unwrap();
    let card = ElementQuery::from_root(&window)
        .match_inherits("KitCard")
        .find_first()
        .unwrap();
    let review = element(&window, "Review");
    let publish = element(&window, "Publish");
    let original_height = card.size().height;
    assert!(window.get_content_width() <= 720.);
    assert!((review.absolute_position().y - publish.absolute_position().y).abs() < 1.);
    window.invoke_focus_editor();
    key_text(&window, "x");
    let before_resize = window.get_draft().to_string();
    assert!(before_resize.contains('x'));
    let mut theme = window.get_theme();
    theme.text_scale = 2.;
    window.set_theme(theme);
    window.set_viewport_width(280.);
    settle(1);
    assert!(window.get_editor_focused());
    window.invoke_reveal_editor();
    settle(1);
    key_text(&window, "y");
    let draft = window.get_draft();
    assert_eq!(draft.replace('y', ""), before_resize);
    assert_eq!(
        draft.chars().filter(|character| *character == 'y').count(),
        1
    );
    assert!(card.size().width <= 280.);
    assert!(card.size().height > original_height);
    assert!(publish.absolute_position().y > review.absolute_position().y);
    assert!(window.get_scroll_content_height() > window.get_scroll_viewport_height());
}

#[test]
fn conditional_alert_measures_its_allocated_width_without_empty_card_space() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseConditionalAlertFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for width in [720., 300., 720.] {
            window.set_viewport_width(width);
            for dismissible in [false, true] {
                window.set_dismissible(dismissible);
                for message in [
                    "Your session is ready.",
                    "",
                    "A longer message that must wrap at the allocated width and remain entirely inside the alert.",
                ] {
                    window.set_message(message.into());
                    window.set_expanded(false);
                    // Searching materializes changed conditionals as a native draw does.
                    assert!(
                        ElementQuery::from_root(&window)
                            .match_inherits("KitAlert")
                            .find_all()
                            .is_empty()
                    );
                    window.set_expanded(true);
                    let card = ElementQuery::from_root(&window)
                        .match_inherits("KitCard")
                        .find_first()
                        .unwrap();
                    let next = element(&window, "Continue");
                    let content_bottom = next.absolute_position().y + next.size().height;
                    let card_bottom = card.absolute_position().y + card.size().height;
                    assert!(
                        (card_bottom - content_bottom - 24.).abs() < 1.,
                        "scale={scale} width={width}: card bottom={card_bottom}, content bottom={content_bottom}"
                    );
                    let alert = element(&window, "Signed in");
                    assert!(
                        alert.absolute_position().y + alert.size().height
                            <= next.absolute_position().y
                    );
                    for text in ElementQuery::from_root(&window)
                        .match_inherits("Text")
                        .find_all()
                    {
                        let position = text.absolute_position();
                        let size = text.size();
                        assert!(position.x >= 0. && position.x + size.width <= width + 1.);
                    }
                }
            }
        }
    }
}
fn key_text(window: &impl ComponentHandle, text: &str) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}

#[test]
fn progress_fill_starts_at_the_track_origin_for_all_bounded_values() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseProgressFixture::new().unwrap();
    window.show().unwrap();
    for value in [-1., 0., 0.25, 0.5, 0.75, 1., 2.] {
        window.set_progress(value);
        settle(1);
        let progress = ElementQuery::from_root(&window)
            .match_inherits("KitProgress")
            .find_first()
            .unwrap();
        let rectangles = progress
            .query_descendants()
            .match_type_name("Rectangle")
            .find_all();
        assert_eq!(rectangles.len(), 2);
        let track = &rectangles[0];
        let fill = &rectangles[1];
        assert!((fill.absolute_position().x - track.absolute_position().x).abs() <= 1.);
        assert!((fill.absolute_position().y - track.absolute_position().y).abs() <= 1.);
        assert!((fill.size().width - track.size().width * value.clamp(0., 1.)).abs() <= 1.);
        assert!((fill.size().height - 8.).abs() <= 1.);
    }
}

#[test]
fn a_toast_host_measures_exactly_its_toasts() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ToastHostFixture::new().unwrap();
    window.show().unwrap();
    let host = ElementQuery::from_root(&window)
        .match_inherits("KitToastHost")
        .find_first()
        .unwrap();
    let toast = ElementQuery::from_root(&window)
        .match_inherits("KitToast")
        .find_first()
        .unwrap();
    let below = element(&window, "Below the host");
    assert!(
        (host.size().height - toast.size().height).abs() < 0.5,
        "host {:?} toast {:?}",
        host.size(),
        toast.size()
    );
    let host_bottom = host.absolute_position().y + host.size().height;
    assert!((below.absolute_position().y - host_bottom).abs() < 0.5);
}

#[test]
fn a_repeated_card_measures_exactly_its_content() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ToastHostFixture::new().unwrap();
    window.show().unwrap();
    let card = ElementQuery::from_root(&window)
        .match_inherits("KitCard")
        .find_first()
        .unwrap();
    let below = element(&window, "Below the cards");
    let card_bottom = card.absolute_position().y + card.size().height;
    assert!(
        (below.absolute_position().y - card_bottom).abs() < 0.5,
        "card {:?} {:?}, below {:?}",
        card.absolute_position(),
        card.size(),
        below.absolute_position()
    );
}
