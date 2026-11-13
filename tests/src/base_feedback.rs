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
    for (value, expected, numeric) in [(-0.5, "0", 0.), (0.37, "37", 37.), (1.5, "100", 100.)] {
        window.set_progress(value);
        for label in ["Upload progress", "Review progress"] {
            let progress = element(&window, label);
            assert_eq!(
                progress.accessible_role(),
                Some(i_slint_backend_testing::AccessibleRole::ProgressIndicator)
            );
            assert_eq!(progress.accessible_value().as_deref(), Some(expected));
            // The native accessibility bridge parses this string as a number.
            // A formatted suffix would lose the platform's numeric value.
            assert_eq!(
                progress.accessible_value().unwrap().parse::<f64>().unwrap(),
                numeric
            );
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
    for reduced_motion in [false, true] {
        let mut theme = window.get_theme();
        theme.reduced_motion = reduced_motion;
        let transition = if reduced_motion {
            0
        } else {
            u64::try_from(theme.motion.slow).unwrap()
        };
        window.set_theme(theme);
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
            // Resolve the animated binding, then check its completed geometry.
            // Reduced motion must reach the same geometry without a transition.
            let _ = fill.size();
            settle(transition + 1);
            assert!((fill.absolute_position().x - track.absolute_position().x).abs() <= 1.);
            assert!((fill.absolute_position().y - track.absolute_position().y).abs() <= 1.);
            assert!((fill.size().width - track.size().width * value.clamp(0., 1.)).abs() <= 1.);
            assert!((fill.size().height - 8.).abs() <= 1.);
        }
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
fn a_toast_draws_a_severity_mark_beside_its_dismiss_icon() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ToastHostFixture::new().unwrap();
    window.show().unwrap();
    let toast = ElementQuery::from_root(&window)
        .match_inherits("KitToast")
        .find_first()
        .unwrap();
    let icons = toast
        .query_descendants()
        .match_inherits("KitIcon")
        .find_all();
    assert_eq!(icons.len(), 2, "severity mark and dismiss glyph");
}

#[test]
fn escape_dismisses_the_focused_toast_and_the_host_takes_focus_for_a_hotkey() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ToastHostFixture::new().unwrap();
    window.show().unwrap();
    let dismissed = Rc::new(RefCell::new(Vec::new()));
    let observed = Rc::clone(&dismissed);
    window.on_toast_dismiss(move |token| observed.borrow_mut().push(token));
    key(&window, Key::Escape);
    settle(1);
    assert!(dismissed.borrow().is_empty(), "no toast has focus yet");
    window.invoke_focus_toasts();
    settle(1);
    key(&window, Key::Escape);
    settle(1);
    assert_eq!(&*dismissed.borrow(), &[1]);
}

#[test]
fn escape_leaves_a_toast_without_dismissal_open() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ToastHostFixture::new().unwrap();
    window.show().unwrap();
    window.set_entries(slint::ModelRc::new(slint::VecModel::from(vec![
        ToastEntry {
            token: 7,
            title: "Syncing".into(),
            message: "".into(),
            action_label: "Details".into(),
            severity: AlertSeverity::Information,
            dismissible: false,
            ..Default::default()
        },
    ])));
    let dismissed = Rc::new(Cell::new(0));
    let observed = Rc::clone(&dismissed);
    window.on_toast_dismiss(move |_| observed.set(observed.get() + 1));
    window.invoke_focus_toasts();
    settle(1);
    key(&window, Key::Escape);
    settle(1);
    assert_eq!(dismissed.get(), 0);
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

#[test]
fn timeline_markers_share_their_connectors_axis_at_each_text_scale() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDisplayQualityFixture::new().unwrap();
    window.show().unwrap();
    for mut theme in [window.get_theme(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            let markers = ElementQuery::from_root(&window)
                .match_id("KitTimeline::marker")
                .find_all();
            let connectors = ElementQuery::from_root(&window)
                .match_id("KitTimeline::connector")
                .find_all();
            assert_eq!(markers.len(), 2);
            assert_eq!(connectors.len(), 1, "no dangling rule after the last event");
            let first = &markers[0];
            let connector = &connectors[0];
            let center_x =
                |element: &ElementHandle| element.absolute_position().x + element.size().width / 2.;
            assert!((center_x(first) - center_x(connector)).abs() < 0.1);
            assert!((center_x(first) - center_x(&markers[1])).abs() < 0.1);
            assert_eq!(first.size().width, theme.spacing.unit * 2. * scale);
            assert!(
                connector.absolute_position().y > first.absolute_position().y + first.size().height
            );
            assert!(
                connector.absolute_position().y + connector.size().height
                    < markers[1].absolute_position().y
            );
            // A one-line title uses the same metrics as the marker's first-line
            // measurement, so their vertical centers agree in both themes.
            let title = element(&window, "Project created");
            assert!(
                (first.absolute_position().y + first.size().height / 2.
                    - title.absolute_position().y
                    - title.size().height / 2.)
                    .abs()
                    < 0.5
            );
        }
    }
}

#[test]
fn stopping_a_skeleton_restores_a_solid_placeholder_after_either_pulse_phase() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDisplayQualityFixture::new().unwrap();
    window.show().unwrap();
    let skeleton = ElementQuery::from_root(&window)
        .match_inherits("KitSkeleton")
        .find_first()
        .unwrap();
    for elapsed in [650, 1300] {
        window.set_running(true);
        settle(elapsed);
        let _ = skeleton.computed_opacity();
        settle(500);
        window.set_running(false);
        let _ = skeleton.computed_opacity();
        settle(500);
        assert_eq!(skeleton.computed_opacity(), 1.);
        settle(1400);
        assert_eq!(skeleton.computed_opacity(), 1.);
    }
}

#[test]
fn toast_dismissal_draws_the_themes_close_glyph_inside_its_named_button() {
    let window = fixture();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let dismiss = element(&window, "Dismiss Saved");
        let icon = dismiss
            .query_descendants()
            .match_inherits("KitIcon")
            .find_first()
            .unwrap();
        let path = icon
            .query_descendants()
            .match_type_name("Path")
            .find_first()
            .unwrap();
        // An empty default glyph has no view box, hence zero stroke width.
        // The theme's real close glyph has the fitted, stroked bounds used by
        // all icon controls and remains centered when text is enlarged.
        assert!(path.size().width > icon.size().width);
        assert!(
            (icon.absolute_position().x + icon.size().width / 2.
                - dismiss.absolute_position().x
                - dismiss.size().width / 2.)
                .abs()
                < 0.5
        );
        assert!(
            (icon.absolute_position().y + icon.size().height / 2.
                - dismiss.absolute_position().y
                - dismiss.size().height / 2.)
                .abs()
                < 0.5
        );
    }
}

#[test]
fn labelled_divider_rules_share_the_labels_vertical_center() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDisplayQualityFixture::new().unwrap();
    window.show().unwrap();
    for mut theme in [window.get_theme(), window.get_dark()] {
        for scale in [1., 2.] {
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            let label = element(&window, "Activity");
            let center_y =
                |item: &ElementHandle| item.absolute_position().y + item.size().height / 2.;
            for id in ["KitDivider::leading-rule", "KitDivider::trailing-rule"] {
                let rule = ElementQuery::from_root(&window)
                    .match_id(id)
                    .find_first()
                    .unwrap();
                assert!(
                    (center_y(&rule) - center_y(&label)).abs() < 0.5,
                    "{id} scale={scale}"
                );
                assert_eq!(rule.size().height, theme.shape.border_width);
            }
        }
    }
}

#[test]
fn toast_queue_changes_keep_the_focused_delegate_and_release_both_pause_sources() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::Model;
    use slint_kit::{ToastBinding, ToastLifetime};
    use std::num::NonZeroUsize;
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Action {
        Earlier,
        Undo,
        Followup,
        Other,
    }
    fn entry(title: &str, action: &str) -> ToastEntry {
        ToastEntry {
            title: title.into(),
            action_label: action.into(),
            dismissible: true,
            ..Default::default()
        }
    }
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    for dark in [false, true] {
        for scale in [1., 2.] {
            let window = ToastHostFixture::new().unwrap();
            window
                .window()
                .set_size(slint::LogicalSize::new(640., 900.));
            let mut theme = if dark {
                window.global::<KitThemes>().get_dark()
            } else {
                window.get_theme()
            };
            theme.text_scale = scale;
            window.set_theme(theme);
            let binding = ToastBinding::new(NonZeroUsize::new(2).unwrap());
            binding
                .push(
                    Action::Earlier,
                    entry("Earlier", ""),
                    ToastLifetime::Persistent,
                )
                .unwrap();
            binding
                .push(
                    Action::Undo,
                    entry("Saved", "Undo kept"),
                    ToastLifetime::Persistent,
                )
                .unwrap();
            let token = binding.model().row_data(1).unwrap().token;
            window.set_entries(binding.model());
            let pauses = Rc::new(RefCell::new((false, false)));
            let observed = pauses.clone();
            let interaction = binding.interaction_handler();
            window.on_toast_interaction(move |id, source, engaged| {
                if id == token {
                    let mut flags = observed.borrow_mut();
                    match source {
                        ToastInteraction::Pointer => flags.0 = engaged,
                        ToastInteraction::Focus => flags.1 = engaged,
                    }
                }
                interaction(id, source, engaged);
            });
            window.on_toast_dismiss(binding.dismiss_handler());
            let actions = Rc::new(RefCell::new(Vec::new()));
            let recorded = actions.clone();
            window.on_toast_action(move |id| recorded.borrow_mut().push(id));
            window.show().unwrap();
            settle(1);
            window.window().take_snapshot().unwrap();
            let kept = element(&window, "Undo kept");
            let point = kept.absolute_position();
            let size = kept.size();
            window.window().dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(
                    point.x + size.width / 2.,
                    point.y + size.height / 2.,
                ),
            });
            kept.mock_single_click(PointerEventButton::Left);
            settle(1);
            assert_eq!(*pauses.borrow(), (true, true));

            // Capacity evicts an earlier row, so the focused delegate changes
            // index. A zero-lifetime sibling then exercises timer removal.
            binding
                .push(
                    Action::Followup,
                    entry("Followup", ""),
                    ToastLifetime::Timed(Duration::ZERO),
                )
                .unwrap();
            settle(1);
            window.window().take_snapshot().unwrap();
            assert!(kept.is_valid(), "eviction must retain the focused delegate");
            assert_eq!(binding.model().row_data(0).unwrap().token, token);
            assert!(pauses.borrow().1, "the real focused button keeps its pause");
            key(&window, Key::Space);
            assert_eq!(actions.borrow().last(), Some(&token));
            settle(60);
            window.window().take_snapshot().unwrap();
            assert_eq!(binding.model().row_count(), 1, "the sibling expires");
            assert!(kept.is_valid(), "expiry must retain other delegates");

            binding
                .push(Action::Other, entry("Other", ""), ToastLifetime::Persistent)
                .unwrap();
            let other = binding.model().row_data(1).unwrap().token;
            window.window().take_snapshot().unwrap();
            assert!(kept.is_valid(), "append must retain other delegates");
            assert_eq!(binding.dismiss(other), Some(Action::Other));
            settle(1);
            window.window().take_snapshot().unwrap();
            assert!(kept.is_valid(), "dismissal must retain other delegates");
            let count = actions.borrow().len();
            key(&window, Key::Space);
            assert_eq!(actions.borrow().len(), count + 1);
            assert_eq!(actions.borrow().last(), Some(&token));
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("toast-surviving-focus-{dark}-{scale}"),
            );

            // Refresh hit testing at the retained button's current position,
            // then leave it while keyboard focus stays in the toast.
            let point = kept.absolute_position();
            window.window().dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(
                    point.x + size.width / 2.,
                    point.y + size.height / 2.,
                ),
            });
            settle(1);
            assert_eq!(*pauses.borrow(), (true, true));
            window.window().dispatch_event(WindowEvent::PointerMoved {
                position: LogicalPosition::new(630., 890.),
            });
            settle(1);
            assert_eq!(*pauses.borrow(), (false, true));
            window.invoke_blur_toasts();
            settle(1);
            assert_eq!(*pauses.borrow(), (false, false));
            assert_eq!(binding.model().row_data(0).unwrap().token, token);
            window.hide().unwrap();
        }
    }
}

fn placement_fixture(placement: TooltipPlacement, target_y: f32) -> TooltipPlacementFixture {
    let window = TooltipPlacementFixture::new().unwrap();
    window.set_placement(placement);
    window.set_target_y(target_y);
    window.show().unwrap();
    window.invoke_focus_trigger();
    settle(1);
    assert!(window.get_tooltip_visible());
    window
}

#[test]
fn tooltip_placement_puts_the_tip_on_the_requested_side_of_its_target() {
    i_slint_backend_testing::init_no_event_loop();
    let side = |placement| {
        let window = placement_fixture(placement, 200.);
        let tip = element(&window, "Delete the project");
        let (x, y) = (
            window.get_target_x_position(),
            window.get_target_y_position(),
        );
        let (w, h) = (window.get_target_width(), window.get_target_height());
        let (p, s) = (tip.absolute_position(), tip.size());
        (x, y, w, h, p, s)
    };
    let (_, y, _, h, p, _) = side(TooltipPlacement::Below);
    assert!(
        p.y >= y + h,
        "below: tip {p:?} under target bottom {}",
        y + h
    );
    let (_, y, _, _, p, s) = side(TooltipPlacement::Above);
    assert!(
        p.y + s.height <= y,
        "above: tip {p:?} {s:?} over target top {y}"
    );
    let (x, y, w, h, p, s) = side(TooltipPlacement::Right);
    assert!(
        p.x >= x + w,
        "right: tip {p:?} after target right {}",
        x + w
    );
    assert!(
        (p.y + s.height / 2. - (y + h / 2.)).abs() < 12.,
        "right: tip is centered on the target height"
    );
    let (x, _, _, _, p, s) = side(TooltipPlacement::Left);
    assert!(
        p.x + s.width <= x,
        "left: tip {p:?} {s:?} before target left {x}"
    );
    // Auto sits below, then flips above when the bottom edge has no room.
    let (_, y, _, h, p, _) = side(TooltipPlacement::Auto);
    assert!(p.y >= y + h);
    let window = placement_fixture(TooltipPlacement::Auto, 460.);
    let tip = element(&window, "Delete the project");
    assert!(tip.absolute_position().y + tip.size().height <= window.get_target_y_position());
    in_bounds(&tip, 640., 480.);
}

#[test]
fn a_tooltip_without_an_enabled_layer_never_claims_the_keyboard_tip() {
    let with_layer = fixture();
    let window = TooltipBareFixture::new().unwrap();
    window.show().unwrap();
    window.invoke_focus_trigger();
    settle(1);
    assert!(
        !window.get_tooltip_visible(),
        "without a layer the native hover tip stays available"
    );
    with_layer.invoke_focus_trigger();
    settle(1);
    assert!(with_layer.get_tooltip_visible());
    with_layer.set_layer_enabled(false);
    settle(1);
    with_layer.invoke_blur_trigger();
    settle(1);
    with_layer.invoke_focus_trigger();
    settle(1);
    assert!(!with_layer.get_tooltip_visible());
}

#[test]
fn a_pressable_exposes_the_tooltip_text_as_its_accessible_description() {
    i_slint_backend_testing::init_no_event_loop();
    let window = placement_fixture(TooltipPlacement::Auto, 200.);
    let button = element(&window, "Delete");
    assert_eq!(
        button.accessible_description().as_deref(),
        Some("Delete the project")
    );
}

fn options_fixture() -> FeedbackOptionsFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = FeedbackOptionsFixture::new().unwrap();
    window.show().unwrap();
    window
}

#[test]
fn indeterminate_progress_reports_no_value_and_runs_a_segment_along_the_track() {
    let window = options_fixture();
    let progress = ElementQuery::from_root(&window)
        .match_inherits("KitProgress")
        .find_first()
        .unwrap();
    assert!(
        progress
            .accessible_value()
            .is_none_or(|value| value.parse::<f64>().is_err())
    );
    let rectangles = progress
        .query_descendants()
        .match_type_name("Rectangle")
        .find_all();
    let (track, fill) = (&rectangles[0], &rectangles[1]);
    settle(1);
    assert!((fill.size().width - track.size().width * 0.4).abs() <= 1.);
    let before = fill.absolute_position().x;
    settle(300);
    assert!((fill.absolute_position().x - before).abs() > 1.);
    // Reduced motion parks the segment at the track origin.
    let mut theme = window.get_theme();
    theme.reduced_motion = true;
    window.set_theme(theme);
    settle(1);
    assert!((fill.absolute_position().x - track.absolute_position().x).abs() <= 1.);
    // The visible reading sits beside the label.
    element(&window, "Syncing");
    window.set_mode(ProgressMode::Determinate);
    settle(1);
    assert_eq!(
        ElementQuery::from_root(&window)
            .match_inherits("KitProgress")
            .find_first()
            .unwrap()
            .accessible_value()
            .as_deref(),
        Some("50")
    );
}

#[test]
fn radial_progress_follows_its_diameter_and_spinner_follows_its_control_size() {
    let window = options_fixture();
    let radial = ElementQuery::from_root(&window)
        .match_inherits("KitRadialProgress")
        .find_first()
        .unwrap();
    assert_eq!(radial.size().width, 120.);
    assert_eq!(radial.size().height, 120.);
    let spinner = || {
        ElementQuery::from_root(&window)
            .match_inherits("KitSpinner")
            .find_first()
            .unwrap()
    };
    let mut sizes = Vec::new();
    for size in [
        ControlSize::Compact,
        ControlSize::Medium,
        ControlSize::Comfortable,
    ] {
        window.set_spinner_size(size);
        settle(1);
        sizes.push(spinner().size().width);
    }
    assert_eq!(sizes[1], 24.);
    assert!(sizes[0] < sizes[1] && sizes[1] < sizes[2]);
}

#[test]
fn an_alert_places_its_actions_under_the_copy_and_grows_only_when_it_has_any() {
    let window = options_fixture();
    settle(1);
    assert!(window.get_actions_alert_height() > window.get_plain_alert_height());
    let retried = Rc::new(Cell::new(0));
    let observed = Rc::clone(&retried);
    window.on_retry(move || observed.set(observed.get() + 1));
    let retry = element(&window, "Retry");
    let alert = ElementQuery::from_root(&window)
        .match_inherits("KitAlert")
        .find_all()
        .pop()
        .unwrap();
    let (alert_position, alert_size) = (alert.absolute_position(), alert.size());
    in_bounds_of(&retry, alert_position, alert_size);
    assert!(retry.absolute_position().y > alert_position.y + alert_size.height / 2.);
    retry.mock_single_click(PointerEventButton::Left);
    settle(1);
    assert_eq!(retried.get(), 1);
    element(&window, "Details");
}

#[test]
fn an_alert_without_a_title_draws_only_its_message_and_labels_itself_with_it() {
    let window = options_fixture();
    window.set_alert_title("".into());
    settle(1);
    assert!(window.get_plain_alert_height() > 0.);
    element(&window, "The server did not answer.");
}

fn in_bounds_of(element: &ElementHandle, origin: LogicalPosition, size: slint::LogicalSize) {
    let (position, extent) = (element.absolute_position(), element.size());
    assert!(position.x >= origin.x - 1. && position.y >= origin.y - 1.);
    assert!(position.x + extent.width <= origin.x + size.width + 1.);
    assert!(position.y + extent.height <= origin.y + size.height + 1.);
}
