// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    SwitchPreferredHeight, WrapOverrides, WrapSwitchFirstFrame, WrapSwitchGeometry, WrapSwitchList,
    WrappedChoices,
};
use i_slint_backend_testing::{
    AccessibleRole, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition};
use slint_kit::ui::{ChoiceLabelPosition, ControlWidth};
use std::{cell::RefCell, rc::Rc};

fn platform() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}

fn first_frame(wrap: bool, caption: &str) {
    platform();
    let window = WrapSwitchFirstFrame::new().unwrap();
    slint_kit::install(&window);
    window.set_wrap(wrap);
    window.set_caption(caption.into());
    window.show().unwrap();
    let frame = window.window().take_snapshot().unwrap();
    assert!(frame.width() > 0 && frame.height() > 0);
    let switch = ElementQuery::from_root(&window)
        .match_inherits("KitSwitch")
        .find_first()
        .unwrap();
    assert!(switch.size().width > 0. && switch.size().height > 0.);
}

#[test]
fn wrap_empty() {
    first_frame(true, "");
}
#[test]
fn wrap_label() {
    first_frame(true, "Masquer");
}
#[test]
fn inline_empty() {
    first_frame(false, "");
}
#[test]
fn inline_label() {
    first_frame(false, "Masquer");
}

#[test]
fn caller_preferred_height_is_honored() {
    platform();
    let window = SwitchPreferredHeight::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    assert!((window.get_actual() - window.get_requested()).abs() <= 1.);
}

#[test]
fn wrapped_caption_uses_the_space_left_by_the_track() {
    platform();
    let window = WrapSwitchGeometry::new().unwrap();
    slint_kit::install(&window);
    let light = window.get_theme();
    let dark = window.get_dark();
    window.show().unwrap();
    for palette in [light, dark] {
        for scale in [1., 2.] {
            let mut theme = palette.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for side in [ChoiceLabelPosition::Start, ChoiceLabelPosition::End] {
                window.set_side(side);
                for policy in [ControlWidth::Intrinsic, ControlWidth::Fill] {
                    window.set_policy(policy);
                    for width in [360., 162.] {
                        window.set_available(width);
                        for help in ["", "Tap to see all of the new items in the list."] {
                            window.set_help(help.into());
                            slint::platform::update_timers_and_animations();
                            window.window().take_snapshot().unwrap();
                            let measured = window.get_control_height();
                            let expected = window.get_expected_height();
                            assert!(
                                (measured - expected).abs() <= 1.,
                                "{scale} {side:?} {policy:?} {width} {help:?}: {measured} vs {expected}"
                            );
                            assert!(window.get_control_width() <= width + 1.);
                            if policy == ControlWidth::Intrinsic {
                                assert_eq!(window.get_new_line(), width == 162.);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn controlled_switch_keeps_focus_and_waits_for_the_host() {
    platform();
    let window = WrapSwitchGeometry::new().unwrap();
    slint_kit::install(&window);
    let requests = Rc::new(RefCell::new(Vec::new()));
    let received = requests.clone();
    window.on_proposed(move |next| received.borrow_mut().push(next));
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    let switch = ElementQuery::from_root(&window)
        .match_predicate(|e| e.accessible_role() == Some(AccessibleRole::Switch))
        .find_first()
        .unwrap();
    assert_eq!(switch.accessible_label().as_deref(), Some("Tap to see all"));
    assert_eq!(switch.accessible_checkable(), Some(true));
    window.invoke_focus_switch();
    window.set_available(162.);
    window.window().take_snapshot().unwrap();
    assert!(window.get_focused());
    window.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    window.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    assert_eq!(*requests.borrow(), [true]);
    assert_eq!(switch.accessible_checked(), Some(false));
    window.set_accepted(true);
    assert_eq!(switch.accessible_checked(), Some(true));
    let p = switch.absolute_position();
    let s = switch.size();
    let position = LogicalPosition::new(p.x + s.width / 2., p.y + s.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(*requests.borrow(), [true, false]);
    assert_eq!(switch.accessible_checked(), Some(true));
    window.set_accepted(false);
    assert_eq!(switch.accessible_checked(), Some(false));
}

#[test]
fn virtual_rows_keep_the_controlled_value_with_the_option() {
    platform();
    let window = WrapSwitchList::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    exercise_list(&window);
}

fn exercise_list(window: &WrapSwitchList) {
    use i_slint_backend_testing::{AccessibleRole, ElementQuery};
    use slint::{Model, ModelRc, VecModel};
    use slint_kit::ui::ChoiceRow;
    use std::cell::Cell;
    use std::rc::Rc;

    let proposal = Rc::new(Cell::new(None));
    let received = proposal.clone();
    window.on_proposed(move |key, next| received.set(Some((key, next))));
    let frame = || {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
    };
    let switches = || {
        ElementQuery::from_root(window)
            .match_predicate(|e| e.accessible_role() == Some(AccessibleRole::Switch))
            .find_all()
    };
    frame();
    assert!(switches().is_empty());
    for count in [1, 100, 1000] {
        let mut rows: Vec<_> = (1..=count)
            .map(|key| ChoiceRow {
                token: key,
                label: format!("Option {key}").into(),
                selected: false,
                disabled: false,
            })
            .collect();
        let model = Rc::new(VecModel::from(rows.clone()));
        window.set_rows(ModelRc::from(model.clone()));
        window.invoke_reveal(0);
        frame();
        assert!(switches().len() < 20);
        for _ in 0..2 {
            window.invoke_reveal(count - 1);
            frame();
        }
        let find = |label: &str| {
            switches()
                .into_iter()
                .find(|e| e.accessible_label().as_deref() == Some(label))
                .unwrap()
        };
        let last = format!("Option {count}");
        let target = find(&last);
        assert_eq!(target.accessible_checked(), Some(false));
        target.invoke_accessible_default_action();
        assert_eq!(proposal.take(), Some((count, true)));
        assert_eq!(target.accessible_checked(), Some(false));
        rows.last_mut().unwrap().selected = true;
        model.set_row_data(rows.len() - 1, rows.last().unwrap().clone());
        frame();
        assert_eq!(find(&last).accessible_checked(), Some(true));
        rows.reverse();
        window.set_rows(ModelRc::new(VecModel::from(rows)));
        window.invoke_reveal(0);
        window.set_viewport_width(280.);
        frame();
        let first = find(&last);
        assert_eq!(first.accessible_checked(), Some(true));
        first.invoke_accessible_default_action();
        assert_eq!(proposal.take(), Some((count, false)));
        assert!(switches().len() < 20);
        window.set_viewport_width(360.);
        window.set_rows(ModelRc::default());
        frame();
        assert!(switches().is_empty());
    }
}

#[test]
fn checkbox_and_horizontal_radio_measure_in_the_same_wrap() {
    platform();
    let window = WrappedChoices::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    for role in [AccessibleRole::Checkbox, AccessibleRole::RadioButton] {
        let control = ElementQuery::from_root(&window)
            .match_predicate(move |e| e.accessible_role() == Some(role))
            .find_first()
            .unwrap();
        assert!(control.size().width > 0. && control.size().height > 0.);
    }
}

#[test]
fn caller_bounds_and_padding_survive_flex_measurement() {
    platform();
    let window = WrapOverrides::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    window.window().take_snapshot().unwrap();
    assert_eq!(window.get_control_width(), 140.);
    assert_eq!(window.get_control_height(), 48.);
    assert_eq!(window.get_reduced_height(), 16.);
}
