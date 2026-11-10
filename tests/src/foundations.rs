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

#[test]
fn link_buttons_keep_native_activation_and_an_underlined_inline_recipe() {
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    i_slint_backend_testing::init_no_event_loop();
    let window = ButtonPaletteFixture::new().unwrap();
    window.show().unwrap();
    let link = ElementHandle::find_by_accessible_label(&window, "Learn more")
        .next()
        .unwrap();
    let rule = ElementQuery::from_root(&window)
        .match_id("KitButton::link-rule")
        .find_first()
        .unwrap();
    assert!(rule.size().width > 0. && rule.size().height > 0.);
    assert!(
        link.size().height < 30.,
        "link uses a text line instead of button density"
    );
    link.mock_single_click(PointerEventButton::Left);
    link.invoke_accessible_default_action();
    window.invoke_focus_link();
    for key in [Key::Return, Key::Space] {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }
    assert_eq!(window.get_link_actions(), 4);
    window.set_link_disabled(true);
    slint::platform::update_timers_and_animations();
    link.mock_single_click(PointerEventButton::Left);
    link.invoke_accessible_default_action();
    assert_eq!(window.get_link_actions(), 4);
}

#[test]
fn public_recipes_and_content_bounds_reach_rendered_geometry() {
    use i_slint_backend_testing::ElementQuery;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = RecipeOverrideFixture::new().unwrap();
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
            let mut icon = window
                .global::<ui::KitThemes>()
                .invoke_icon_recipe(theme.clone(), ui::ControlSize::Medium);
            icon.size = 31. * scale;
            window.set_icon_recipe(icon);
            let mut scroll = window
                .global::<ui::KitThemes>()
                .invoke_scroll_bar_recipe(theme.clone());
            scroll.thickness = 27.;
            window.set_scroll_recipe(scroll);
            let mut surface = window
                .global::<ui::KitThemes>()
                .invoke_surface_recipe(theme.clone(), SurfaceLevel::Flat);
            surface.fill = slint::Color::from_rgb_u8(11, 100, 50).into();
            surface.radius = 17.;
            window.set_surface_recipe(surface);
            window.set_card_level(SurfaceLevel::Overlay);
            slint::platform::update_timers_and_animations();
            let find = |kind: &str| {
                ElementQuery::from_root(&window)
                    .match_inherits(kind)
                    .find_first()
                    .unwrap()
            };
            assert_eq!(find("KitIcon").size().width, 31. * scale);
            assert_eq!(find("KitScrollBar").size().width, 27.);
            let content = ElementQuery::from_root(&window)
                .match_id("RecipeOverrideFixture::bounded-body")
                .find_first()
                .unwrap();
            assert_eq!(content.size().width, 140.);
            assert_eq!(window.get_card_recipe().radius, theme.shape.radius_overlay);
            assert_eq!(window.get_card_recipe().elevation, theme.elevation.overlay);
            let snapshot = window.window().take_snapshot().unwrap();
            assert!(
                snapshot
                    .as_slice()
                    .iter()
                    .any(|p| (p.r, p.g, p.b) == (11, 100, 50))
            );
            if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                use std::io::Write;
                std::fs::create_dir_all(&directory).unwrap();
                let path = std::path::PathBuf::from(directory)
                    .join(format!("recipes-{index}-{scale}.ppm"));
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
fn avatars_honor_explicit_small_sizes_scale_initials_and_name_fallbacks() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = AvatarSizingFixture::new().unwrap();
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
            let named = |label: &str| {
                i_slint_backend_testing::ElementHandle::find_by_accessible_label(&window, label)
                    .next()
                    .unwrap()
            };
            assert_eq!(named("EA").size().width, 24.);
            assert_eq!(named("Large initials").size().width, 96.);
            assert_eq!(
                named("Unknown profile").size().width,
                theme.geometry.avatar_compact * scale
            );
            let snapshot = window.window().take_snapshot().unwrap();
            let status = i_slint_backend_testing::ElementQuery::from_root(&window)
                .match_id("AvatarSizingFixture::large-status")
                .find_first()
                .unwrap();
            let decorated = named("Large initials");
            assert_eq!(
                status.absolute_position().x + status.size().width,
                decorated.absolute_position().x + decorated.size().width
            );
            assert_eq!(
                status.absolute_position().y + status.size().height,
                decorated.absolute_position().y + decorated.size().height
            );
            // Keep the status outside the circular ink samples used below.
            let status_pixel =
                snapshot.as_slice()[usize::try_from(118 * snapshot.width() + 173).unwrap()];
            assert_eq!(
                (status_pixel.r, status_pixel.g, status_pixel.b),
                (
                    theme.success.red(),
                    theme.success.green(),
                    theme.success.blue()
                ),
                "status children paint outside the avatar image clip"
            );

            assert!(
                snapshot.as_slice().iter().any(|p| (p.r, p.g, p.b)
                    == (
                        theme.success_subtle.red(),
                        theme.success_subtle.green(),
                        theme.success_subtle.blue()
                    )),
                "success avatar paints its semantic tone"
            );
            let ink_height = |x: u32, y: u32, width: u32, height: u32| {
                let mut first = height;
                let mut last = 0;
                for row in 0..height {
                    for column in 0..width {
                        let pixel = snapshot.as_slice()
                            [usize::try_from((y + row) * snapshot.width() + x + column).unwrap()];
                        let dx = 2 * i64::from(column) + 1 - i64::from(width);
                        let dy = 2 * i64::from(row) + 1 - i64::from(height);
                        let radius = i64::from(width) - 6;
                        if dx * dx + dy * dy >= radius * radius {
                            continue;
                        }
                        let delta = [
                            i64::from(theme.accent.red()) - i64::from(theme.accent_subtle.red()),
                            i64::from(theme.accent.green())
                                - i64::from(theme.accent_subtle.green()),
                            i64::from(theme.accent.blue()) - i64::from(theme.accent_subtle.blue()),
                        ];
                        let observed = [
                            i64::from(pixel.r) - i64::from(theme.accent_subtle.red()),
                            i64::from(pixel.g) - i64::from(theme.accent_subtle.green()),
                            i64::from(pixel.b) - i64::from(theme.accent_subtle.blue()),
                        ];
                        let projection: i64 = observed.iter().zip(delta).map(|(a, b)| a * b).sum();
                        let full: i64 = delta.iter().map(|v| v * v).sum();
                        if projection * 10 > full {
                            first = first.min(row);
                            last = last.max(row);
                        }
                    }
                }
                assert!(first < height);
                last - first + 1
            };
            assert!(ink_height(80, 24, 96, 96) >= ink_height(24, 24, 24, 24) * 3);
            let fallback = i_slint_backend_testing::ElementQuery::from_root(&window)
                .match_inherits("KitIcon")
                .find_first()
                .unwrap();
            assert!(
                fallback.size().width > 0.,
                "an empty avatar paints its theme fallback glyph"
            );
            window.set_disabled(true);
            slint::platform::update_timers_and_animations();
            let disabled = window.window().take_snapshot().unwrap();
            assert_ne!(
                snapshot.as_slice(),
                disabled.as_slice(),
                "the fallback glyph shares the avatar disabled presentation"
            );
            window.set_disabled(false);
            slint::platform::update_timers_and_animations();

            if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                use std::io::Write;
                std::fs::create_dir_all(&directory).unwrap();
                let path = std::path::PathBuf::from(&directory)
                    .join(format!("avatars-{index}-{scale}.ppm"));
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
                let disabled_path = std::path::PathBuf::from(&directory)
                    .join(format!("avatars-disabled-{index}-{scale}.ppm"));
                let mut disabled_file = std::fs::File::create(disabled_path).unwrap();
                write!(
                    disabled_file,
                    "P6\n{} {}\n255\n",
                    disabled.width(),
                    disabled.height()
                )
                .unwrap();
                for pixel in disabled.as_slice() {
                    disabled_file
                        .write_all(&[pixel.r, pixel.g, pixel.b])
                        .unwrap();
                }
            }
        }
    }
}

#[test]
fn avatar_semantic_tones_render_their_palette_in_both_themes_and_text_scales() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = AvatarSizingFixture::new().unwrap();
    window.show().unwrap();
    for (mode, base) in [
        ("light", window.get_theme()),
        ("dark", window.global::<ui::KitThemes>().get_dark()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme.clone());
            for (tone, expected) in [
                (Tone::Neutral, theme.surface),
                (Tone::Accent, theme.accent_subtle),
                (Tone::Success, theme.success_subtle),
                (Tone::Warning, theme.warning_subtle),
                (Tone::Danger, theme.danger_subtle),
                (Tone::Information, theme.information_subtle),
            ] {
                window.set_avatar_tone(tone);
                slint::platform::update_timers_and_animations();
                let avatar = i_slint_backend_testing::ElementHandle::find_by_accessible_label(
                    &window,
                    "Positive profile",
                )
                .next()
                .unwrap();
                let origin = avatar.absolute_position();
                let extent = avatar.size();
                let snapshot = window.window().take_snapshot().unwrap();
                let width = usize::try_from(snapshot.width()).unwrap();
                assert!(
                    snapshot
                        .as_slice()
                        .iter()
                        .enumerate()
                        .any(|(index, pixel)| {
                            let x = f64::from(u32::try_from(index % width).unwrap()) + 0.5;
                            let y = f64::from(u32::try_from(index / width).unwrap()) + 0.5;
                            x > f64::from(origin.x) + 1.
                                && x < f64::from(origin.x + extent.width) - 1.
                                && y > f64::from(origin.y) + 1.
                                && y < f64::from(origin.y + extent.height) - 1.
                                && (pixel.r, pixel.g, pixel.b)
                                    == (expected.red(), expected.green(), expected.blue())
                        }),
                    "{mode}/{scale}/{tone:?} paints its semantic fill inside the avatar"
                );
                if let Some(directory) = std::env::var_os("KIT_FOUNDATION_RENDER_DIR") {
                    use std::io::Write;
                    std::fs::create_dir_all(&directory).unwrap();
                    let path = std::path::PathBuf::from(directory)
                        .join(format!("avatar-tone-{mode}-{scale}-{tone:?}.ppm"));
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
