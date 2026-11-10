// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

#[test]
fn avatar_groups_keep_keyed_identity_fence_replaced_gestures_and_expose_overflow() {
    use slint::{
        LogicalPosition, Model,
        platform::{PointerEventButton, WindowEvent},
    };
    use std::{cell::RefCell, rc::Rc};
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct ProfileId(u32);
    let row = |name: &str, initials: &str, disabled| ui::AvatarEntry {
        label: name.into(),
        initials: initials.into(),
        disabled,
        tone: Tone::Accent,
        ..Default::default()
    };
    i_slint_backend_testing::init_no_event_loop();
    let window = AvatarGroupFixture::new().unwrap();
    let members = slint_kit::KeyedModel::new([
        (ProfileId(101), row("Alice", "AL", false)),
        (ProfileId(203), row("Bob", "BO", true)),
        (ProfileId(307), row("Eve", "EV", false)),
    ])
    .unwrap();
    window.set_entries(members.model());
    let accepted = Rc::new(RefCell::new(Vec::new()));
    window.on_member_activated({
        let members = members.clone();
        let accepted = accepted.clone();
        move |token, generation| {
            if let Some(id) = members.accept(token, generation) {
                accepted.borrow_mut().push(id);
            }
        }
    });
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    let named = |label: &str| {
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(&window, label)
            .next()
            .unwrap()
    };
    assert_eq!(window.get_overflow_count(), 1);
    named("Alice").invoke_accessible_default_action();
    named("Bob").invoke_accessible_default_action();
    assert_eq!(*accepted.borrow(), [ProfileId(101)]);
    named("1 more avatar").invoke_accessible_default_action();
    assert_eq!(window.get_overflow_actions(), 1);
    let alice = named("Alice");
    let point = LogicalPosition::new(
        alice.absolute_position().x + alice.size().width / 2.,
        alice.absolute_position().y + alice.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: point });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position: point,
        button: PointerEventButton::Left,
    });
    let stale = members.model().row_data(0).unwrap();
    members
        .replace([
            (ProfileId(203), row("Bob", "BO", true)),
            (ProfileId(101), row("Alice", "AL", false)),
            (ProfileId(307), row("Eve", "EV", false)),
        ])
        .unwrap();
    slint::platform::update_timers_and_animations();
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: point,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        *accepted.borrow(),
        [ProfileId(101)],
        "replacement cancels the held member gesture"
    );
    assert!(members.accept(stale.token, stale.generation).is_none());
    named("Alice").invoke_accessible_default_action();
    assert_eq!(*accepted.borrow(), [ProfileId(101), ProfileId(101)]);
    window.set_disabled(true);
    slint::platform::update_timers_and_animations();
    named("Alice").invoke_accessible_default_action();
    named("1 more avatar").invoke_accessible_default_action();
    assert_eq!(accepted.borrow().len(), 2);
    assert_eq!(window.get_overflow_actions(), 1);
}

#[test]
fn avatar_group_members_have_native_tab_and_space_activation() {
    use slint::platform::{Key, WindowEvent};
    use std::{cell::Cell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let window = AvatarGroupFixture::new().unwrap();
    window.set_entries(slint::ModelRc::new(slint::VecModel::from(vec![
        ui::AvatarEntry {
            token: 11,
            generation: 7,
            label: "Alice".into(),
            initials: "AL".into(),
            ..Default::default()
        },
    ])));
    let actions = Rc::new(Cell::new(0));
    window.on_member_activated({
        let actions = actions.clone();
        move |token, generation| {
            assert_eq!((token, generation), (11, 7));
            actions.set(actions.get() + 1);
        }
    });
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    for key in [Key::Tab, Key::Space] {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    assert_eq!(actions.get(), 1);
}

#[test]
fn avatar_group_mounts_only_visible_members_and_renders_the_overflow_matrix() {
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::{Key, WindowEvent};
    #[derive(Clone, Copy, Debug)]
    enum State {
        Available,
        Focused,
        Disabled,
    }
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = AvatarGroupFixture::new().unwrap();
    #[derive(Clone, PartialEq, Eq, Hash)]
    struct MemberId(u32);
    let members = slint_kit::KeyedModel::new((0u32..1000).map(|id| {
        (
            MemberId(id),
            ui::AvatarEntry {
                label: format!("Member {}", id + 1).into(),
                initials: if id == 0 { "EA".into() } else { "DA".into() },
                tone: if id == 0 { Tone::Accent } else { Tone::Success },
                ..Default::default()
            },
        )
    }))
    .unwrap();
    window.set_entries(members.model());
    window.show().unwrap();
    for (index, base) in [
        window.get_theme(),
        window.global::<ui::KitThemes>().get_dark(),
    ]
    .into_iter()
    .enumerate()
    {
        for scale in [1., 2.] {
            for flow in [ui::ChoiceFlow::LeftToRight, ui::ChoiceFlow::RightToLeft] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                theme.direction = flow;
                window.set_theme(theme);
                for state in [State::Available, State::Focused, State::Disabled] {
                    window.set_disabled(match state {
                        State::Available | State::Focused => false,
                        State::Disabled => true,
                    });
                    if let State::Focused = state {
                        for event in [
                            WindowEvent::KeyPressed {
                                text: Key::Tab.into(),
                            },
                            WindowEvent::KeyReleased {
                                text: Key::Tab.into(),
                            },
                        ] {
                            window.window().dispatch_event(event);
                        }
                    }
                    slint::platform::update_timers_and_animations();
                    let snapshot = window.window().take_snapshot().unwrap();
                    assert_eq!(window.get_overflow_count(), 998);
                    assert_eq!(
                        ElementQuery::from_root(&window)
                            .match_inherits("KitAvatar")
                            .find_all()
                            .len(),
                        3
                    );
                    let first = ElementHandle::find_by_accessible_label(&window, "Member 1")
                        .next()
                        .unwrap();
                    let overflow =
                        ElementHandle::find_by_accessible_label(&window, "998 more avatars")
                            .next()
                            .unwrap();
                    match flow {
                        ui::ChoiceFlow::LeftToRight => {
                            assert!(first.absolute_position().x < overflow.absolute_position().x)
                        }
                        ui::ChoiceFlow::RightToLeft => {
                            assert!(first.absolute_position().x > overflow.absolute_position().x)
                        }
                    }
                    if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                        use std::io::Write;
                        std::fs::create_dir_all(&directory).unwrap();
                        let path = std::path::PathBuf::from(directory).join(format!(
                            "avatar-group-{index}-{scale}-{flow:?}-{state:?}.ppm"
                        ));
                        let mut file =
                            std::io::BufWriter::new(std::fs::File::create(path).unwrap());
                        write!(
                            file,
                            "P6\n{} {}\n255\n",
                            snapshot.width(),
                            snapshot.height()
                        )
                        .unwrap();
                        for pixel in snapshot.as_slice() {
                            file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn avatar_group_static_and_overflow_actions_have_independent_tab_stops_and_localized_counts() {
    use slint::{
        ModelRc, VecModel,
        platform::{Key, WindowEvent},
    };
    i_slint_backend_testing::init_no_event_loop();
    let window = AvatarGroupFixture::new().unwrap();
    let rows = slint_kit::KeyedModel::new([
        (
            1u8,
            ui::AvatarEntry {
                label: "Alice".into(),
                initials: "AL".into(),
                ..Default::default()
            },
        ),
        (
            2u8,
            ui::AvatarEntry {
                label: "Bob".into(),
                initials: "BO".into(),
                ..Default::default()
            },
        ),
        (
            3u8,
            ui::AvatarEntry {
                label: "Eve".into(),
                initials: "EV".into(),
                ..Default::default()
            },
        ),
    ])
    .unwrap();
    window.set_entries(rows.model());
    window.set_interaction(ui::AvatarInteraction::Static);
    window
        .global::<ui::KitStrings>()
        .set_avatar_more_singular("Un autre membre".into());
    window
        .global::<ui::KitStrings>()
        .set_avatar_more_template("{count} autres membres".into());
    window.show().unwrap();
    let key = |value: Key| {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: value.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: value.into() });
    };
    key(Key::Tab);
    key(Key::Space);
    assert_eq!(
        window.get_overflow_actions(),
        0,
        "static members and overflow have no Tab action"
    );
    assert!(
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(
            &window,
            "Un autre membre"
        )
        .next()
        .is_some()
    );
    window.set_max_visible(0);
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_overflow_count(), 3);
    assert!(
        i_slint_backend_testing::ElementHandle::find_by_accessible_label(
            &window,
            "3 autres membres"
        )
        .next()
        .is_some()
    );
    window.set_overflow_interaction(ui::AvatarInteraction::Action);
    slint::platform::update_timers_and_animations();
    key(Key::Tab);
    key(Key::Space);
    assert_eq!(window.get_overflow_actions(), 1);
    window.set_entries(ModelRc::new(VecModel::from(Vec::<ui::AvatarEntry>::new())));
    slint::platform::update_timers_and_animations();
    let group = i_slint_backend_testing::ElementQuery::from_root(&window)
        .match_inherits("KitAvatarGroup")
        .find_first()
        .unwrap();
    assert_eq!(group.size().width, 0.);
    assert_eq!(group.size().height, 0.);
    assert_eq!(window.get_overflow_count(), 0);
}
