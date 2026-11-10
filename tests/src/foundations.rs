// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;

#[test]
fn stat_renders_every_tone_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = StatFixture::new().unwrap();
    window.show().unwrap();
    for (mode, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            for (tone, expected) in [
                (Tone::Neutral, theme.text_primary),
                (Tone::Accent, theme.accent),
                (Tone::Success, theme.success),
                (Tone::Warning, theme.warning),
                (Tone::Danger, theme.danger),
                (Tone::Information, theme.information),
            ] {
                window.set_tone(tone);
                slint::platform::update_timers_and_animations();
                let snapshot = window.window().take_snapshot().unwrap();
                assert!(
                    snapshot.as_slice().iter().any(|p| p.r == expected.red()
                        && p.g == expected.green()
                        && p.b == expected.blue()),
                    "{mode} {scale} {tone:?}"
                );
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("stat-{mode}-{scale}-{tone:?}.ppm"));
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
        }
    }
}

#[test]
fn busy_button_animates_unless_motion_is_reduced_and_describes_the_work() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BusyButtonFixture::new().unwrap();
    window.show().unwrap();
    for base in [window.get_theme(), window.get_dark_theme()] {
        for scale in [1., 2.] {
            for reduced_motion in [false, true] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                theme.reduced_motion = reduced_motion;
                window.set_theme(theme);
                slint::platform::update_timers_and_animations();
                let action = i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                    &window, "Save",
                )
                .next()
                .unwrap();
                assert_eq!(
                    action.accessible_description().as_deref(),
                    Some("Saving changes")
                );
                let before = window.window().take_snapshot().unwrap();
                i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(200));
                slint::platform::update_timers_and_animations();
                let after = window.window().take_snapshot().unwrap();
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    for (phase, snapshot) in [("before", &before), ("after", &after)] {
                        let path = std::path::PathBuf::from(&directory).join(format!(
                            "busy-{}-{scale}-{reduced_motion}-{phase}.ppm",
                            if base.canvas == window.get_dark_theme().canvas {
                                "dark"
                            } else {
                                "light"
                            }
                        ));
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
                assert_eq!(
                    before.as_slice() == after.as_slice(),
                    reduced_motion,
                    "motion preference {reduced_motion} at {scale}"
                );
            }
        }
    }
}

#[test]
fn file_and_step_defaults_follow_the_window_strings_once() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = FileLocalizationFixture::new().unwrap();
    let strings = window.global::<ui::KitStrings>();
    strings.set_file_choose_one("Choisir un fichier…".into());
    strings.set_folder_choose("Choisir un dossier…".into());
    strings.set_folder_clear("Effacer le dossier".into());
    strings.set_file_remove("Supprimer".into());
    strings.set_file_remove_template("{name} — {action}".into());
    strings.set_file_list("Fichiers sélectionnés".into());
    strings.set_file_unsupported("Sélection indisponible".into());
    strings.set_selection_cancelled("Sélection annulée".into());
    strings.set_step_current("étape actuelle".into());
    strings.set_step_complete("terminé".into());
    strings.set_step_error("erreur".into());
    strings.set_step_pending("en attente".into());
    window.show().unwrap();
    for (mode, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            slint::platform::update_timers_and_animations();
            for label in [
                "Choisir un fichier… Pièces jointes",
                "Choisir un dossier…",
                "Effacer le dossier",
                "notes.txt — Supprimer",
                "Fichiers sélectionnés",
                "Sélection indisponible",
                "Sélection annulée",
                "Préparer, étape actuelle",
                "Envoyer, terminé",
                "Vérifier, erreur",
                "Terminer, en attente",
            ] {
                assert!(
                    i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                        &window, label
                    )
                    .next()
                    .is_some(),
                    "missing localized {label}"
                );
            }
            let snapshot = window.window().take_snapshot().unwrap();
            if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                use std::io::Write;
                std::fs::create_dir_all(&directory).unwrap();
                let path = std::path::PathBuf::from(directory)
                    .join(format!("localized-files-steps-{mode}-{scale}.ppm"));
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
    }
}

#[test]
fn disabling_an_action_clears_observed_focus_without_restoring_it_on_enable() {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = DisabledActionFocusFixture::new().unwrap();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let observed = observations.clone();
    let redirect = Rc::new(Cell::new(false));
    let redirect_on_blur = redirect.clone();
    let weak = window.as_weak();
    window.on_focus_observed(move |custom, focused| {
        observed.borrow_mut().push((custom, focused));
        if !focused && redirect_on_blur.get() {
            weak.upgrade().unwrap().invoke_focus_fallback();
        }
    });
    window.show().unwrap();
    for (mode, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            theme.reduced_motion = true;
            window.set_theme(theme);
            for custom in [false, true] {
                window.set_busy(false);
                window.set_disabled(false);
                window.invoke_focus_action(custom);
                slint::platform::update_timers_and_animations();
                assert!(if custom {
                    window.get_custom_focused()
                } else {
                    window.get_standard_focused()
                });
                observations.borrow_mut().clear();
                for (state, disabled) in
                    [("focused", false), ("disabled", true), ("reenabled", false)]
                {
                    window.set_disabled(disabled);
                    slint::platform::update_timers_and_animations();
                    let focused = if custom {
                        window.get_custom_focused()
                    } else {
                        window.get_standard_focused()
                    };
                    let visible = if custom {
                        window.get_custom_focus_visible()
                    } else {
                        window.get_standard_focus_visible()
                    };
                    assert_eq!(focused, state == "focused", "{custom} {state}");
                    assert_eq!(visible, state == "focused", "{custom} {state}");
                    let snapshot = window.window().take_snapshot().unwrap();
                    if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                        use std::io::Write;
                        std::fs::create_dir_all(&directory).unwrap();
                        let path = std::path::PathBuf::from(directory)
                            .join(format!("action-focus-{mode}-{scale}-{custom}-{state}.ppm"));
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
                assert_eq!(
                    &*observations.borrow(),
                    &[(custom, false)],
                    "one blur notification on disable"
                );
                window.invoke_focus_action(custom);
                window.set_busy(true);
                slint::platform::update_timers_and_animations();
                assert!(
                    if custom {
                        window.get_custom_focused()
                    } else {
                        window.get_standard_focused()
                    },
                    "busy retains focus"
                );
            }
        }
    }
    window.set_busy(false);
    window.set_disabled(false);
    window.invoke_focus_action(false);
    redirect.set(true);
    window.set_disabled(true);
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_fallback_focused(),
        "host redirect after blur wins"
    );
    window.set_disabled(false);
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_fallback_focused(),
        "reenable clears only the routing scope"
    );
    redirect.set(false);
    window.invoke_focus_action(false);
    slint::platform::update_timers_and_animations();
    window.set_disabled(true);
    window.invoke_focus_fallback();
    slint::platform::update_timers_and_animations();
    assert!(
        window.get_fallback_focused(),
        "host redirect before deferred disable notification wins"
    );
    window.set_disabled(false);
    slint::platform::update_timers_and_animations();
    assert!(window.get_fallback_focused());
}

#[test]
fn drop_zone_browse_supports_pointer_keyboard_and_accessibility_and_honors_access() {
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    use std::{cell::Cell, rc::Rc};
    i_slint_backend_testing::init_no_event_loop();
    let window = DropZoneFixture::new().unwrap();
    let count = Rc::new(Cell::new(0));
    let result = count.clone();
    window.on_browse(move || result.set(result.get() + 1));
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    let zone = i_slint_backend_testing::ElementHandle::find_by_accessible_label(
        &window,
        "Choose or drop documents",
    )
    .next()
    .unwrap();
    assert_eq!(
        zone.accessible_description().as_deref(),
        Some("PDF and text files")
    );
    zone.mock_single_click(PointerEventButton::Left);
    assert_eq!(count.get(), 1);
    window.invoke_focus_zone();
    for key in [Key::Return, Key::Space] {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    assert_eq!(count.get(), 3);
    zone.invoke_accessible_default_action();
    assert_eq!(count.get(), 4);
    let children = Rc::new(Cell::new(0));
    let child_count = children.clone();
    window.on_child_activated(move || child_count.set(child_count.get() + 1));
    window.set_composed_action(true);
    slint::platform::update_timers_and_animations();
    i_slint_backend_testing::ElementHandle::find_by_accessible_label(&window, "Additional action")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        children.get(),
        1,
        "composed child retains its own activation"
    );
    assert_eq!(count.get(), 4, "child activation does not browse");
    window.set_composed_action(false);
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        window.set_access(access);
        slint::platform::update_timers_and_animations();
        zone.mock_single_click(PointerEventButton::Left);
        zone.invoke_accessible_default_action();
        assert_eq!(
            count.get(),
            4,
            "non-editable zone ignores browse activation"
        );
    }
}

#[test]
fn status_button_palettes_render_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = ButtonPaletteFixture::new().unwrap();
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
            window.set_theme(theme.clone());
            slint::platform::update_timers_and_animations();
            let snapshot = window.window().take_snapshot().unwrap();
            for expected in [
                theme.success,
                theme.warning,
                theme.information,
                theme.danger,
            ] {
                assert!(snapshot.as_slice().iter().any(|p| p.r == expected.red()
                    && p.g == expected.green()
                    && p.b == expected.blue()));
            }
            if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                use std::io::Write;
                std::fs::create_dir_all(&directory).unwrap();
                let path = std::path::PathBuf::from(directory)
                    .join(format!("palette-{index}-{scale}.ppm"));
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
    }
}
