// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ActionId(u16);
fn row(text: &str, disabled: bool) -> ui::ButtonGroupEntry {
    ui::ButtonGroupEntry {
        text: text.into(),
        disabled,
        intent: ButtonIntent::Outline,
        ..Default::default()
    }
}
fn named(window: &ButtonGroupFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(window, label)
        .next()
        .unwrap()
}
#[test]
fn grouped_actions_keep_independent_keyboard_focus_and_keyed_gesture_guards() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonGroupFixture::new().unwrap();
    let actions = slint_kit::KeyedModel::new([
        (ActionId(101), row("Back", false)),
        (ActionId(203), row("Next", true)),
    ])
    .unwrap();
    window.set_actions(actions.model());
    window.set_attached(true);
    let accepted = Rc::new(RefCell::new(Vec::new()));
    window.on_action_requested({
        let actions = actions.clone();
        let accepted = accepted.clone();
        move |token, generation| {
            if let Some(key) = actions.accept(token, generation) {
                accepted.borrow_mut().push(key);
            }
        }
    });
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    for key in [Key::Tab, Key::Space, Key::Return] {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    assert_eq!(*accepted.borrow(), [ActionId(101), ActionId(101)]);
    named(&window, "Next").invoke_accessible_default_action();
    named(&window, "Back").mock_single_click(PointerEventButton::Left);
    assert_eq!(accepted.borrow().len(), 3);
    let back = named(&window, "Back");
    let point = LogicalPosition::new(
        back.absolute_position().x + back.size().width / 2.,
        back.absolute_position().y + back.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: point });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    let stale = actions.model().row_data(0).unwrap();
    actions
        .replace([
            (ActionId(203), row("Next", false)),
            (ActionId(101), row("Back", false)),
        ])
        .unwrap();
    slint::platform::update_timers_and_animations();
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(accepted.borrow().len(), 3);
    assert!(actions.accept(stale.token, stale.generation).is_none());
    window.set_disabled(true);
    slint::platform::update_timers_and_animations();
    named(&window, "Next").invoke_accessible_default_action();
    assert_eq!(accepted.borrow().len(), 3);
    window.set_disabled(false);
    window.set_busy_label("Chargement…".into());
    let mut busy = row("Back", false);
    busy.busy = true;
    actions
        .replace([(ActionId(101), busy), (ActionId(203), row("Next", false))])
        .unwrap();
    slint::platform::update_timers_and_animations();
    let back = named(&window, "Back");
    assert_eq!(
        back.accessible_description().as_deref(),
        Some("Chargement…")
    );
    back.invoke_accessible_default_action();
    let busy_row = actions.model().row_data(0).unwrap();
    assert!(
        actions
            .accept(busy_row.token, busy_row.generation)
            .is_none()
    );
    assert_eq!(
        accepted.borrow().len(),
        3,
        "busy remains guarded in UI and Rust"
    );
}
fn snapshot(window: &ButtonGroupFixture, state: &str, theme_index: usize, scale: f32) {
    if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
        use std::io::Write;
        let image = window.window().take_snapshot().unwrap();
        std::fs::create_dir_all(&directory).unwrap();
        let path = std::path::PathBuf::from(directory)
            .join(format!("button-group-{state}-{theme_index}-{scale}.ppm"));
        let mut file = std::fs::File::create(path).unwrap();
        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
        for pixel in image.as_slice() {
            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
        }
    }
}
#[test]
fn groups_join_without_gaps_change_orientation_and_wrap_unattached_actions() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ButtonGroupFixture::new().unwrap();
    window.set_actions(slint::ModelRc::new(slint::VecModel::from(vec![
        row("Previous", false),
        row("Continue", false),
        row("Cancel", false),
    ])));
    window.show().unwrap();
    for (index, base) in [
        window.get_theme(),
        window.global::<ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            window.set_group_width(540.);
            window.set_wrap(false);
            window.set_attached(true);
            for (name, orientation) in [
                ("horizontal", slint::language::Orientation::Horizontal),
                ("vertical", slint::language::Orientation::Vertical),
            ] {
                window.set_orientation(orientation);
                slint::platform::update_timers_and_animations();
                let first = named(&window, "Previous");
                let second = named(&window, "Continue");
                if orientation == slint::language::Orientation::Horizontal {
                    assert!(
                        (first.absolute_position().x + first.size().width
                            - second.absolute_position().x)
                            .abs()
                            < 1.
                    );
                } else {
                    assert!(
                        (first.absolute_position().y + first.size().height
                            - second.absolute_position().y)
                            .abs()
                            < 1.
                    );
                    assert_eq!(first.size().width, second.size().width);
                }
                assert_eq!(
                    ElementQuery::from_root(&window)
                        .match_id("GroupButtonItem::separator")
                        .find_all()
                        .len(),
                    2
                );
                let group = named(&window, "Document commands");
                assert!(group.size().width > 0.);
                let last = named(&window, "Cancel");
                let expected_bottom = last.absolute_position().y + last.size().height;
                assert!(
                    (group.absolute_position().y + group.size().height - expected_bottom).abs()
                        < 1.,
                    "the group outline follows its content height rather than the window"
                );
                snapshot(&window, name, index, scale);
            }
            window.set_attached(false);
            window.set_wrap(true);
            window.set_orientation(slint::language::Orientation::Horizontal);
            window.set_group_width(180.);
            slint::platform::update_timers_and_animations();
            let first = named(&window, "Previous");
            let last = named(&window, "Cancel");
            assert!(
                last.absolute_position().y > first.absolute_position().y,
                "unattached actions wrap when constrained"
            );
            let group = named(&window, "Document commands");
            assert!(
                (group.absolute_position().y + group.size().height
                    - last.absolute_position().y
                    - last.size().height)
                    .abs()
                    < 1.,
                "wrapped groups keep the intrinsic height of all rows"
            );
            snapshot(&window, "wrapped", index, scale);
            window.set_group_width(540.);
            window.set_wrap(false);
            window.set_attached(true);
            named(&window, "Previous").mock_single_click(PointerEventButton::Left);
            window.window().dispatch_event(WindowEvent::KeyPressed {
                text: Key::Tab.into(),
            });
            window.window().dispatch_event(WindowEvent::KeyReleased {
                text: Key::Tab.into(),
            });
            slint::platform::update_timers_and_animations();
            assert_eq!(
                ElementQuery::from_root(&window)
                    .match_inherits("FocusOutline")
                    .find_all()
                    .len(),
                1,
                "the focused-state capture includes the real keyboard focus outline"
            );
            snapshot(&window, "focused", index, scale);
            let mut busy = row("Previous", false);
            busy.busy = true;
            window.set_actions(slint::ModelRc::new(slint::VecModel::from(vec![
                busy,
                row("Continue", false),
                row("Cancel", false),
            ])));
            slint::platform::update_timers_and_animations();
            snapshot(&window, "busy", index, scale);
            window.set_disabled(true);
            slint::platform::update_timers_and_animations();
            snapshot(&window, "disabled", index, scale);
            window.set_disabled(false);
            window.set_actions(slint::ModelRc::new(slint::VecModel::from(vec![
                row("Previous", false),
                row("Continue", false),
                row("Cancel", false),
            ])));
        }
    }
}
