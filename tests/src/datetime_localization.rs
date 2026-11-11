// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{ComponentHandle, platform::PointerEventButton};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[test]
fn localized_ambiguity_labels_keep_choices_explicit_and_typed() {
    i_slint_backend_testing::init_no_event_loop();
    let window = DateTimeLocalizationFixture::new().unwrap();
    let choices = Rc::new(RefCell::new(Vec::new()));
    let output = choices.clone();
    let starts = Rc::new(Cell::new(0));
    let observed = starts.clone();
    window.on_started(move || observed.set(observed.get() + 1));
    let observed = starts.clone();
    window.on_chosen(move |value| {
        assert!(observed.get() > output.borrow().len());
        output.borrow_mut().push(value);
    });
    window.show().unwrap();
    assert!(choices.borrow().is_empty());
    for (label, choice) in [
        ("Première occurrence", AmbiguousTimeChoice::Earlier),
        ("Seconde occurrence", AmbiguousTimeChoice::Later),
    ] {
        window.set_date_text("2026-11-02".into());
        slint::platform::update_timers_and_animations();
        window.set_date_text("2026-11-01".into());
        slint::platform::update_timers_and_animations();
        ElementQuery::from_root(&window)
            .match_inherits("KitButton")
            .match_predicate(move |element| element.accessible_label().as_deref() == Some(label))
            .find_first()
            .expect("localized ambiguity action")
            .mock_single_click(PointerEventButton::Left);
        assert_eq!(choices.borrow().last(), Some(&choice));
        assert_eq!(
            window.get_occurrence_selection(),
            match choice {
                AmbiguousTimeChoice::Earlier => ui::ZonedOccurrenceSelection::Earlier,
                AmbiguousTimeChoice::Later => ui::ZonedOccurrenceSelection::Later,
            }
        );
    }
    for label in ["Date civile", "Heure locale", "Fuseau horaire"] {
        assert!(ElementQuery::from_root(&window).match_predicate(move|element|element.accessible_label().as_deref()==Some(label)).find_first().is_some(),"{label}");
    }
    window.set_date_text("2026-11-02".into());
    slint::platform::update_timers_and_animations();
    window.set_access(FieldAccess::ReadOnly);
    let before = choices.borrow().len();
    ElementQuery::from_root(&window)
        .match_inherits("KitButton")
        .match_predicate(|element| {
            element.accessible_label().as_deref() == Some("Seconde occurrence")
        })
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(choices.borrow().len(), before);
}

#[test]
fn choosing_an_overlap_occurrence_clears_validity_and_accepts_only_once() {
    use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let reference = ZonedServiceFixture::new().unwrap();
    let themes = [
        reference.global::<ui::KitThemes>().get_light(),
        reference.global::<ui::KitThemes>().get_dark(),
    ];
    for (mode, base) in themes.into_iter().enumerate() {
        for scale in [1., 2.] {
            for (name, expected) in [
                ("Earlier occurrence", ui::ZonedOccurrenceSelection::Earlier),
                ("Later occurrence", ui::ZonedOccurrenceSelection::Later),
            ] {
                let window = ZonedServiceFixture::new().unwrap();
                slint_kit::install(&window);
                let mut theme = base.clone();
                theme.text_scale = scale;
                window.set_theme(theme);
                let accepted = Rc::new(RefCell::new(Vec::new()));
                let output = accepted.clone();
                window.on_chosen(move |commit| {
                    output
                        .borrow_mut()
                        .push(slint_kit::ResolvedDateTime::try_from(commit).unwrap())
                });
                window.show().unwrap();
                slint::platform::update_timers_and_animations();
                window.invoke_resolve();
                slint::platform::update_timers_and_animations();
                assert_eq!(
                    window.get_resolution(),
                    ui::ZonedTimePresentation::Ambiguous
                );
                assert!(!window.invoke_check_validity());
                let save = |state: &str| {
                    if let Some(dir) = std::env::var_os("KIT_ZONED_OCCURRENCE_RENDER_DIR") {
                        use std::io::Write;
                        std::fs::create_dir_all(&dir).unwrap();
                        let image = window.window().take_snapshot().unwrap();
                        let path = std::path::PathBuf::from(dir).join(format!(
                            "zoned-overlap-{mode}-{scale}-{expected:?}-{state}.ppm"
                        ));
                        let mut file = std::fs::File::create(path).unwrap();
                        write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                        for p in image.as_slice() {
                            file.write_all(&[p.r, p.g, p.b]).unwrap();
                        }
                    }
                };
                if expected == ui::ZonedOccurrenceSelection::Earlier {
                    save("unselected");
                }
                let action = ElementHandle::find_by_accessible_label(&window, name)
                    .next()
                    .unwrap();
                action.invoke_accessible_default_action();
                slint::platform::update_timers_and_animations();
                assert_eq!(window.get_occurrence_selection(), expected);
                assert_eq!(action.accessible_checkable(), Some(true));
                assert_eq!(action.accessible_checked(), Some(true));
                assert!(
                    window.invoke_check_validity(),
                    "an explicit DST occurrence is a valid local value"
                );
                assert_eq!(accepted.borrow().len(), 1);
                let expected_instant = match expected {
                    ui::ZonedOccurrenceSelection::Unselected => {
                        unreachable!("choice cases are selected")
                    }
                    ui::ZonedOccurrenceSelection::Earlier => 1793511000,
                    ui::ZonedOccurrenceSelection::Later => 1793514600,
                };
                assert_eq!(
                    accepted.borrow()[0].instant.unix_seconds(),
                    expected_instant
                );
                let message = if expected == ui::ZonedOccurrenceSelection::Earlier {
                    "Earlier occurrence accepted."
                } else {
                    "Later occurrence accepted."
                };
                assert!(
                    ElementHandle::find_by_accessible_label(&window, message)
                        .next()
                        .is_some()
                );
                save("accepted");
                action.invoke_accessible_default_action();
                let other = if expected == ui::ZonedOccurrenceSelection::Earlier {
                    "Later occurrence"
                } else {
                    "Earlier occurrence"
                };
                ElementHandle::find_by_accessible_label(&window, other)
                    .next()
                    .unwrap()
                    .mock_single_click(PointerEventButton::Left);
                assert_eq!(
                    accepted.borrow().len(),
                    1,
                    "both UI entry paths remain blocked after acceptance"
                );
                window.set_date_text("2026-11-02".into());
                slint::platform::update_timers_and_animations();
                assert_eq!(
                    window.get_occurrence_selection(),
                    ui::ZonedOccurrenceSelection::Unselected
                );
                assert_eq!(
                    window.get_resolution(),
                    ui::ZonedTimePresentation::Unresolved
                );
            }
        }
    }
}

#[test]
fn overlap_confirmation_resets_on_resolution_and_draft_changes_and_preserves_host_errors() {
    use i_slint_backend_testing::ElementHandle;
    i_slint_backend_testing::init_no_event_loop();
    let window = ZonedServiceFixture::new().unwrap();
    slint_kit::install(&window);
    let choices = Rc::new(RefCell::new(Vec::new()));
    let output = choices.clone();
    window.on_chosen(move |value| output.borrow_mut().push(value.acceptance));
    window.show().unwrap();
    let choose = |name: &str| {
        ElementHandle::find_by_accessible_label(&window, name)
            .next()
            .unwrap()
            .mock_single_click(PointerEventButton::Left);
        slint::platform::update_timers_and_animations();
    };
    window.invoke_resolve();
    window.set_access(FieldAccess::ReadOnly);
    choose("Earlier occurrence");
    assert_eq!(
        window.get_occurrence_selection(),
        ui::ZonedOccurrenceSelection::Unselected
    );
    assert!(choices.borrow().is_empty());
    window.set_access(FieldAccess::Editable);
    choose("Earlier occurrence");
    assert!(window.invoke_check_validity());
    assert_eq!(*choices.borrow(), [ui::ZonedAcceptance::Earlier]);
    window.invoke_resolve();
    assert_eq!(
        window.get_occurrence_selection(),
        ui::ZonedOccurrenceSelection::Unselected
    );
    assert!(!window.invoke_check_validity());
    choose("Later occurrence");
    assert!(window.invoke_check_validity());
    assert_eq!(
        *choices.borrow(),
        [ui::ZonedAcceptance::Earlier, ui::ZonedAcceptance::Later]
    );
    window.set_custom_validity("Server rejected this appointment".into());
    assert!(
        !window.invoke_check_validity(),
        "acceptance must not erase a host error"
    );
    window.set_custom_validity("".into());
    assert!(window.invoke_check_validity());
    for (date, time, zone) in [
        ("2026-11-02", "01:30", "America/Toronto"),
        ("2026-11-01", "02:30", "America/Toronto"),
        ("2026-11-01", "01:30", "Europe/Paris"),
    ] {
        window.set_date_text(date.into());
        window.set_time_text(time.into());
        window.set_zone_text(zone.into());
        slint::platform::update_timers_and_animations();
        assert_eq!(
            window.get_occurrence_selection(),
            ui::ZonedOccurrenceSelection::Unselected
        );
        assert_eq!(
            window.get_resolution(),
            ui::ZonedTimePresentation::Unresolved
        );
        window.set_date_text("2026-11-01".into());
        window.set_time_text("01:30".into());
        window.set_zone_text("America/Toronto".into());
        slint::platform::update_timers_and_animations();
        window.invoke_resolve();
        choose("Earlier occurrence");
        assert!(window.invoke_check_validity());
    }
}

#[test]
fn host_resolution_inside_the_choice_callback_keeps_the_new_state() {
    use i_slint_backend_testing::ElementHandle;
    i_slint_backend_testing::init_no_event_loop();
    let window = ZonedServiceFixture::new().unwrap();
    slint_kit::install(&window);
    let weak = window.as_weak();
    let choices = Rc::new(RefCell::new(Vec::new()));
    let output = choices.clone();
    window.on_chosen(move |value| {
        output.borrow_mut().push(value.acceptance);
        if value.acceptance == ui::ZonedAcceptance::Earlier {
            let window = weak.upgrade().unwrap();
            window.set_date_text("2026-11-02".into());
            window.invoke_resolve();
        }
    });
    window.show().unwrap();
    window.invoke_resolve();
    ElementHandle::find_by_accessible_label(&window, "Earlier occurrence")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        *choices.borrow(),
        [ui::ZonedAcceptance::Earlier, ui::ZonedAcceptance::Unique]
    );
    assert_eq!(
        window.get_occurrence_selection(),
        ui::ZonedOccurrenceSelection::Unselected
    );
    assert!(window.invoke_check_validity());
}
