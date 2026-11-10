// SPDX-License-Identifier: GPL-3.0-or-later
//! Drives the exact controller shipped by the documentation consumer.
use super::*;
use crate::ui;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent, WindowEventDispatchResult},
};
#[path = "../../docs/dialog-consumer/src/controller.rs"]
mod example;
const TOOLBAR: i32 = 1;
const CARD: i32 = 2;
const NAME: i32 = 3;
const CANCEL: i32 = 4;
const ARCHIVE: i32 = 5;
const KEEP: i32 = 6;
const CONFIRM: i32 = 7;
struct Harness {
    window: DialogApp,
    host: example::Host,
}
fn harness() -> Harness {
    i_slint_backend_testing::init_no_event_loop();
    let (window, host) = example::create_app_with_host(ui::DialogLanguage::English).unwrap();
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    Harness { window, host }
}
impl Harness {
    fn focused(&self) -> Option<i32> {
        self.host
            .focused()
            .and_then(|target| self.host.token(target))
    }
    fn element(&self, label: &str) -> ElementHandle {
        let requested = label.to_owned();
        ElementQuery::from_root(&self.window)
            .match_predicate(move |element| {
                element.accessible_label().as_deref() == Some(requested.as_str())
            })
            .find_first()
            .unwrap_or_else(|| panic!("missing element {label}"))
    }
    /// Semantic activation, as an assistive technology performs it.
    fn activate(&self, label: &str) {
        self.element(label).invoke_accessible_default_action();
        slint::platform::update_timers_and_animations();
    }
    fn press(&self, key: Key) -> bool {
        let result = self
            .window
            .window()
            .dispatch_event_with_result(WindowEvent::KeyPressed { text: key.into() })
            .unwrap();
        self.window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
        slint::platform::update_timers_and_animations();
        matches!(result, WindowEventDispatchResult::Accepted)
    }
    fn click(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let window = self.window.window();
        window.dispatch_event(WindowEvent::PointerMoved { position });
        window.dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        window.dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        slint::platform::update_timers_and_animations();
    }
}

#[test]
fn each_opener_gets_focus_back_and_tab_cycles_inside_the_layer() {
    let harness = harness();
    let window = &harness.window;
    for (label, opener) in [
        ("Archive from the project card", CARD),
        ("Archive from the toolbar", TOOLBAR),
    ] {
        harness.activate(label);
        assert!(window.get_archive().open && window.get_archive().topmost);
        assert_eq!(
            harness.focused(),
            Some(NAME),
            "the first control of the layer takes focus"
        );
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(CANCEL));
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(ARCHIVE));
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(NAME), "Tab wraps inside the layer");
        assert!(harness.press(Key::Backtab));
        assert_eq!(
            harness.focused(),
            Some(ARCHIVE),
            "Shift+Tab wraps backwards"
        );
        assert!(harness.press(Key::Escape));
        assert!(!window.get_archive().open);
        assert_eq!(
            harness.focused(),
            Some(opener),
            "focus returns to the exact opener"
        );
    }
}

#[test]
fn nested_layer_owns_escape_back_and_tab_then_returns_focus_level_by_level() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the toolbar");
    harness.activate("Archive…");
    assert!(window.get_confirm().topmost);
    assert!(window.get_archive().open && !window.get_archive().topmost);
    assert_eq!(harness.focused(), Some(KEEP));
    assert!(harness.press(Key::Tab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Tab));
    assert_eq!(
        harness.focused(),
        Some(KEEP),
        "Tab stays in the nested layer"
    );
    assert!(harness.press(Key::Backtab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Escape));
    assert!(!window.get_confirm().open);
    assert!(window.get_archive().open && window.get_archive().topmost);
    assert_eq!(
        harness.focused(),
        Some(ARCHIVE),
        "the nested layer returns focus to its own opener"
    );
    harness.activate("Archive…");
    assert_eq!(harness.focused(), Some(KEEP));
    assert!(harness.press(Key::Back));
    assert_eq!(harness.focused(), Some(ARCHIVE));
    assert!(harness.press(Key::Back));
    assert!(!window.get_archive().open);
    assert_eq!(harness.focused(), Some(TOOLBAR));
    assert!(!window.get_archive().open && !window.get_confirm().open);
}

#[test]
fn busy_layer_consumes_escape_back_and_backdrop_until_the_host_completes() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the project card");
    harness.activate("Archive…");
    assert!(harness.press(Key::Tab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Return), "Enter starts the work");
    assert!(window.get_confirm().busy);
    assert_eq!(
        harness.focused(),
        Some(CONFIRM),
        "the busy action keeps focus inside the layer"
    );
    assert!(harness.press(Key::Escape), "Escape is consumed");
    assert!(harness.press(Key::Back), "system Back is consumed");
    harness.click(12., 12.);
    harness
        .host
        .close(example::Layer::Archive, slint_kit::DismissReason::Escape);
    assert!(window.get_confirm().busy);
    assert!(window.get_confirm().open && window.get_archive().open);
    assert!(harness.press(Key::Tab));
    assert_eq!(
        harness.focused(),
        Some(CONFIRM),
        "disabled controls are skipped"
    );
    harness.activate("Keep project");
    assert!(window.get_confirm().busy, "a disabled action does nothing");
    // Exercise the real example's Timer::single_shot completion callback.
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(1200));
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_status().as_str(), "Atlas is archived.");
    assert!(!window.get_confirm().open && !window.get_archive().open);
    assert_eq!(
        harness.focused(),
        Some(CARD),
        "focus returns to the first opener"
    );
}

#[test]
fn background_actions_stay_inert_while_a_layer_is_open() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the toolbar");
    assert!(window.get_archive().open && !window.get_confirm().open);
    harness.activate("Archive from the project card");
    assert!(
        window.get_archive().open && !window.get_confirm().open,
        "semantic activation of a covered control does nothing"
    );
    assert_eq!(harness.focused(), Some(NAME));
    assert!(!harness.host.eligible(example::Target::Card));
    assert!(
        !window
            .get_slots()
            .row_data(usize::try_from(CARD).unwrap())
            .unwrap()
            .eligible
    );
    harness.click(12., 12.);
    assert!(!window.get_archive().open, "the backdrop dismisses");
    assert_eq!(harness.focused(), Some(TOOLBAR));
    assert!(
        window
            .get_slots()
            .row_data(usize::try_from(CARD).unwrap())
            .unwrap()
            .eligible
    );
}

#[test]
fn modal_host_skips_disabled_initial_targets_and_does_not_retain_the_window() {
    let harness = harness();
    harness.host.set_enabled(example::Target::Name, false);
    harness.activate("Archive from the toolbar");
    assert_eq!(harness.focused(), Some(CANCEL));
    harness.host.set_enabled(example::Target::Cancel, false);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        harness.focused(),
        Some(ARCHIVE),
        "disabling focused target moves focus to an eligible fallback"
    );
    let weak = harness.window.as_weak();
    let host = harness.host.clone();
    harness.window.hide().unwrap();
    drop(harness);
    assert!(weak.upgrade().is_none(), "the host retains a weak window");
    host.close(
        example::Layer::Archive,
        slint_kit::DismissReason::Programmatic,
    );
}

#[test]
fn dialog_factory_initializes_the_real_window() {
    i_slint_backend_testing::init_no_event_loop();
    let window = example::create_app(ui::DialogLanguage::French).unwrap();
    assert_eq!(window.get_status().as_str(), "Atlas est actif.");
    assert_eq!(window.get_slots().row_count(), 8);
    assert!(!window.get_archive().open && !window.get_confirm().open);
    window.show().unwrap();
    for label in [
        "Archiver depuis la barre d’outils",
        "Archiver…",
        "Archiver maintenant",
    ] {
        ElementHandle::find_by_accessible_label(&window, label)
            .next()
            .unwrap()
            .invoke_accessible_default_action();
        slint::platform::update_timers_and_animations();
    }
    assert_eq!(
        ElementHandle::find_by_accessible_label(&window, "Archivage…")
            .next()
            .unwrap()
            .accessible_description()
            .as_deref(),
        Some("Archivage…")
    );
}

fn save_dialog_snapshot(window: &DialogApp, name: &str) {
    let snapshot = window.window().take_snapshot().unwrap();
    if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
        use std::io::Write;
        std::fs::create_dir_all(&directory).unwrap();
        let path = std::path::PathBuf::from(directory).join(format!("{name}.ppm"));
        let mut file = std::fs::File::create(path).unwrap();
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

#[test]
fn modal_composition_renders_in_both_themes_and_at_double_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    for (language, locale, panel_name, keep_name, confirm_name) in [
        (
            ui::DialogLanguage::English,
            "en",
            "Confirm archiving",
            "Keep project",
            "Archive now",
        ),
        (
            ui::DialogLanguage::French,
            "fr",
            "Confirmer l’archivage",
            "Conserver le projet",
            "Archiver maintenant",
        ),
    ] {
        let (window, host) = example::create_app_with_host(language).unwrap();
        window.show().unwrap();
        let themes = [
            window.get_theme(),
            window.global::<ui::KitThemes>().get_dark(),
        ];
        for (index, base) in themes.into_iter().enumerate() {
            for scale in [1., 2.] {
                for width in [560., 960.] {
                    window
                        .window()
                        .set_size(slint::LogicalSize::new(width, 800.));
                    let mut theme = base.clone();
                    theme.text_scale = scale;
                    window.set_theme(theme);
                    slint::platform::update_timers_and_animations();
                    let toolbar_names = match language {
                        ui::DialogLanguage::English => {
                            ["Archive from the toolbar", "Archive from the project card"]
                        }
                        ui::DialogLanguage::French => [
                            "Archiver depuis la barre d’outils",
                            "Archiver depuis la fiche du projet",
                        ],
                    };
                    for label in toolbar_names {
                        let action = ElementHandle::find_by_accessible_label(&window, label)
                            .next()
                            .unwrap();
                        let point = action.absolute_position();
                        let bounds = action.size();
                        assert!(
                            point.x >= 0.
                                && point.x + bounds.width <= width + 1.
                                && point.y >= 0.
                                && point.y + bounds.height <= 801.,
                            "opener stays inside viewport: {locale}/{index}/{scale}/{width}"
                        );
                    }
                    save_dialog_snapshot(
                        &window,
                        &format!("dialog-{locale}-{index}-{scale}-{width}-closed"),
                    );
                    host.open(example::Layer::Archive, example::Target::Toolbar);
                    host.open(example::Layer::Confirm, example::Target::Archive);
                    slint::platform::update_timers_and_animations();
                    let panel = ElementHandle::find_by_accessible_label(&window, panel_name)
                        .next()
                        .unwrap();
                    let keep = ElementHandle::find_by_accessible_label(&window, keep_name)
                        .next()
                        .unwrap();
                    let confirm = ElementHandle::find_by_accessible_label(&window, confirm_name)
                        .next()
                        .unwrap();
                    let origin = panel.absolute_position();
                    let size = panel.size();
                    for button in [&keep, &confirm] {
                        let point = button.absolute_position();
                        let bounds = button.size();
                        assert!(
                            point.x >= origin.x
                                && point.y >= origin.y
                                && point.x + bounds.width <= origin.x + size.width + 1.
                                && point.y + bounds.height <= origin.y + size.height + 1.,
                            "footer action stays inside panel: {locale}/{index}/{scale}/{width}"
                        );
                    }
                    let a = keep.absolute_position();
                    let b = confirm.absolute_position();
                    let sa = keep.size();
                    let sb = confirm.size();
                    assert!(
                        a.x + sa.width <= b.x + 1.
                            || b.x + sb.width <= a.x + 1.
                            || a.y + sa.height <= b.y + 1.
                            || b.y + sb.height <= a.y + 1.,
                        "footer actions do not overlap: {locale}/{index}/{scale}/{width}"
                    );
                    assert!(window.get_confirm().topmost);
                    save_dialog_snapshot(
                        &window,
                        &format!("dialog-{locale}-{index}-{scale}-{width}"),
                    );
                    host.close(
                        example::Layer::Archive,
                        slint_kit::DismissReason::Programmatic,
                    );
                    slint::platform::update_timers_and_animations();
                }
            }
        }
    }
}

#[test]
fn modal_host_rejects_invalid_topology_at_construction() {
    use slint_kit::{
        DialogFocusSpec, DialogHost, DialogHostError, DialogHostPorts, DialogLayerSpec,
    };
    i_slint_backend_testing::init_no_event_loop();
    let window = DialogApp::new().unwrap();
    let layer = DialogLayerSpec {
        key: 1u8,
        parent: None,
        initial_focus: 10u8,
    };
    let target = DialogFocusSpec {
        key: 10u8,
        layer: Some(1u8),
        token: 0,
        enabled: true,
    };
    for (layers, targets, expected) in [
        (
            vec![layer.clone(), layer.clone()],
            vec![target.clone()],
            DialogHostError::DuplicateLayer,
        ),
        (
            vec![DialogLayerSpec {
                parent: Some(2),
                ..layer.clone()
            }],
            vec![target.clone()],
            DialogHostError::MissingParent,
        ),
        (
            vec![layer.clone()],
            vec![target.clone(), target.clone()],
            DialogHostError::DuplicateTarget,
        ),
        (
            vec![layer.clone()],
            vec![
                target.clone(),
                DialogFocusSpec {
                    key: 11,
                    ..target.clone()
                },
            ],
            DialogHostError::DuplicateToken,
        ),
        (
            vec![layer.clone()],
            vec![DialogFocusSpec {
                layer: Some(2),
                ..target.clone()
            }],
            DialogHostError::MissingLayer,
        ),
        (
            vec![layer],
            vec![DialogFocusSpec {
                layer: None,
                ..target
            }],
            DialogHostError::InvalidInitialFocus,
        ),
    ] {
        let result = DialogHost::new(
            &window,
            layers,
            targets,
            DialogHostPorts {
                slots: ui::DialogApp::set_slots,
                state: |_, _, _| {},
                focus: |_, _| {},
            },
        );
        assert!(
            matches!(result, Err(error) if error == expected),
            "{expected:?}"
        );
    }
}

#[test]
fn modal_layers_without_eligible_controls_keep_tab_escape_and_restore_the_trigger() {
    let harness = harness();
    for target in [
        example::Target::Name,
        example::Target::Cancel,
        example::Target::Archive,
    ] {
        harness.host.set_enabled(target, false);
    }
    harness.activate("Archive from the toolbar");
    assert!(harness.window.get_archive().topmost);
    assert_eq!(harness.focused(), None);
    assert!(harness.press(Key::Tab));
    harness
        .window
        .window()
        .dispatch_event(WindowEvent::KeyPressed {
            text: Key::Shift.into(),
        });
    assert!(harness.press(Key::Tab));
    harness
        .window
        .window()
        .dispatch_event(WindowEvent::KeyReleased {
            text: Key::Shift.into(),
        });
    assert!(harness.press(Key::Escape));
    assert!(!harness.window.get_archive().open);
    assert_eq!(harness.focused(), Some(TOOLBAR));

    harness.host.set_enabled(example::Target::Name, true);
    harness.activate("Archive from the toolbar");
    assert_eq!(harness.focused(), Some(NAME));
    harness.host.set_enabled(example::Target::Name, false);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        harness.focused(),
        None,
        "last disabled control yields layer focus"
    );
    assert!(harness.press(Key::Tab));
    harness.host.set_enabled(example::Target::Cancel, true);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        harness.focused(),
        Some(CANCEL),
        "newly eligible target replaces layer fallback"
    );
    assert!(harness.press(Key::Escape));
    assert_eq!(harness.focused(), Some(TOOLBAR));

    harness.host.set_enabled(example::Target::Archive, true);
    harness.host.set_enabled(example::Target::Keep, false);
    harness.host.set_enabled(example::Target::Confirm, false);
    harness.activate("Archive from the toolbar");
    harness.activate("Archive…");
    assert!(harness.window.get_confirm().topmost);
    assert_eq!(harness.focused(), None);
    harness.window.invoke_confirm_requested();
    slint::platform::update_timers_and_animations();
    assert!(
        !harness.window.get_confirm().busy,
        "direct Rust action rejects a disabled target"
    );
    assert!(harness.press(Key::Tab));
    assert!(harness.press(Key::Escape));
    assert!(!harness.window.get_confirm().open);
    assert_eq!(harness.focused(), Some(ARCHIVE));
    assert!(harness.press(Key::Escape));
    assert_eq!(harness.focused(), Some(TOOLBAR));
}
