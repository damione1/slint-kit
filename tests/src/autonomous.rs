// SPDX-License-Identifier: GPL-3.0-or-later
use crate::AutonomousFixture;
use slint::{
    ComponentHandle, Model,
    platform::{Key, WindowEvent},
};
use slint_kit::{Choices, FieldSpec, FieldSpecKey, ui};
use std::{cell::RefCell, rc::Rc};
fn key(window: &AutonomousFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn setup() -> AutonomousFixture {
    let window = AutonomousFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [FieldSpec::email().named(FieldSpecKey::new("email").unwrap())],
    )
    .unwrap();
    window.show().unwrap();
    window
}
#[test]
fn field_commits_are_typed_guarded_and_independent_and_cancel_restores() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    window.on_email_committed(move |commit| {
        output
            .borrow_mut()
            .push(FieldSpec::email().accept(commit).unwrap().unwrap())
    });
    window.set_email_text("arbitrary text".into());
    window.invoke_commit_email();
    assert!(received.borrow().is_empty());
    window.set_email_text("after@example.org".into());
    window.invoke_commit_email();
    assert_eq!(received.borrow().len(), 1);
    assert_eq!(window.get_email_state().accepted.text, "after@example.org");
    assert_eq!(
        window.get_second_state().accepted.text,
        "second@example.com"
    );
    window.set_email_text("unfinished@".into());
    window.invoke_cancel_email();
    assert_eq!(window.get_email_text(), "after@example.org");
    window.set_access(ui::FieldAccess::ReadOnly);
    window.set_email_text("blocked@example.org".into());
    window.invoke_commit_email();
    assert_eq!(received.borrow().len(), 1);
    window.set_access(ui::FieldAccess::Editable);
    window.invoke_reset_email();
    assert_eq!(window.get_email_state().accepted.text, "reset@example.com");
    assert_eq!(window.get_email_text(), "reset@example.com");
    let another = setup();
    assert_eq!(
        another.get_email_state().accepted.text,
        "before@example.com"
    );
}
#[test]
fn numeric_keyboard_uses_exact_steps_bounds_and_controlled_ownership() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    window.invoke_focus_slider();
    key(&window, Key::RightArrow);
    assert_eq!(
        window
            .get_slider_value()
            .parse::<slint_kit::DecimalValue>()
            .unwrap(),
        "0.3".parse().unwrap()
    );
    key(&window, Key::End);
    assert_eq!(
        window
            .get_slider_value()
            .parse::<slint_kit::DecimalValue>()
            .unwrap(),
        "1".parse().unwrap()
    );
    key(&window, Key::RightArrow);
    assert_eq!(
        window
            .get_slider_value()
            .parse::<slint_kit::DecimalValue>()
            .unwrap(),
        "1".parse().unwrap()
    );
    window.set_ownership(ui::Ownership::Controlled);
    key(&window, Key::LeftArrow);
    assert_eq!(
        window
            .get_slider_value()
            .parse::<slint_kit::DecimalValue>()
            .unwrap(),
        "1".parse().unwrap()
    );
    window.set_access(ui::FieldAccess::Disabled);
    key(&window, Key::Home);
    assert_eq!(
        window
            .get_slider_value()
            .parse::<slint_kit::DecimalValue>()
            .unwrap(),
        "1".parse().unwrap()
    );
    window.set_access(ui::FieldAccess::Editable);
    window.invoke_focus_range();
    key(&window, Key::End);
    let range =
        slint_kit::OrderedRange::<slint_kit::DecimalValue>::try_from(window.get_range_value())
            .unwrap();
    assert_eq!(range.start(), range.end());
}
#[test]
fn multiple_selection_limits_and_source_keys_survive_reordering() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    enum Team {
        Design,
        Code,
        Support,
    }
    let choices = Choices::new([
        (Team::Design, "Design"),
        (Team::Code, "Code"),
        (Team::Support, "Support"),
    ])
    .unwrap();
    assert_eq!(
        choices
            .selections(&[Team::Design, Team::Support])
            .unwrap()
            .row_count(),
        2
    );
    assert!(choices.selections(&[Team::Design, Team::Design]).is_none());
    window.set_options(choices.model());
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    let source = choices.clone();
    window.on_selection_changed(move |row, added| {
        output
            .borrow_mut()
            .push((source.accept(row).unwrap(), added))
    });
    window.invoke_focus_toggles();
    key(&window, Key::Return);
    key(&window, Key::RightArrow);
    key(&window, Key::Return);
    assert_eq!(window.get_selected().row_count(), 2);
    key(&window, Key::RightArrow);
    key(&window, Key::Return);
    assert_eq!(received.borrow().len(), 2);
    choices
        .replace(vec![
            slint_kit::SelectOption::new(Team::Support, "Support"),
            slint_kit::SelectOption::new(Team::Code, "Code"),
            slint_kit::SelectOption::new(Team::Design, "Design"),
        ])
        .unwrap();
    key(&window, Key::End);
    key(&window, Key::Return);
    assert_eq!(received.borrow().last(), Some(&(Team::Design, false)));
    window.set_access(ui::FieldAccess::ReadOnly);
    key(&window, Key::Return);
    assert_eq!(window.get_selected().row_count(), 1);
}
#[test]
fn invalid_color_preserves_accepted_value_and_byte_conversion_is_checked() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    let initial = window.get_color_value();
    window.invoke_commit_color("#12".into());
    assert_eq!(window.get_color_value(), initial);
    window.invoke_commit_color("#123456".into());
    let color = slint_kit::SrgbColor::try_from(window.get_color_value()).unwrap();
    assert_eq!(color.red, 18);
    assert_eq!(color.green, 52);
    assert_eq!(color.blue, 86);
    assert!(
        slint_kit::SrgbColor::try_from(ui::SrgbColorView {
            red: 256,
            green: 0,
            blue: 0,
            alpha: 255
        })
        .is_err()
    );
}
#[test]
fn autonomous_controls_render_light_dark_and_double_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = setup();
    let choices = Choices::new([(1, "Design"), (2, "Code"), (3, "Support")]).unwrap();
    window.set_options(choices.model());
    for (name, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            slint::platform::update_timers_and_animations();
            let snapshot = window.window().take_snapshot().unwrap();
            assert!(snapshot.width() > 0);
            crate::choice_presentation::save_capture(
                &snapshot,
                &format!("autonomous-{name}-{scale}"),
            );
            let calendar = crate::CalendarServiceFixture::new().unwrap();
            slint_kit::install(&calendar);
            calendar.set_theme(window.get_theme());
            calendar.show().unwrap();
            crate::choice_presentation::save_capture(
                &calendar.window().take_snapshot().unwrap(),
                &format!("calendar-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn temporal_services_validate_dst_occurrences_and_range_interiors() {
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    let service = window.global::<ui::KitTimeService>();
    let draft = ui::ZonedDateTimeDraft {
        date: "2026-11-01".into(),
        time: "01:30".into(),
        zone: "America/Toronto".into(),
    };
    let resolution = service.invoke_resolve(
        draft.clone(),
        ui::TimePrecision::Minute,
        ui::ClockFormat::TwentyFourHour,
    );
    assert_eq!(resolution.state, ui::ZonedTimePresentation::Ambiguous);
    let commit = ui::ZonedDateTimeCommit {
        local: resolution.local.clone(),
        zone: draft.zone.clone(),
        acceptance: ui::ZonedAcceptance::Unique,
    };
    assert!(slint_kit::ResolvedDateTime::try_from(commit.clone()).is_err());
    let earlier = slint_kit::ResolvedDateTime::try_from(ui::ZonedDateTimeCommit {
        acceptance: ui::ZonedAcceptance::Earlier,
        ..commit.clone()
    })
    .unwrap();
    let later = slint_kit::ResolvedDateTime::try_from(ui::ZonedDateTimeCommit {
        acceptance: ui::ZonedAcceptance::Later,
        ..commit
    })
    .unwrap();
    assert_ne!(earlier.instant, later.instant);
    let gap = service.invoke_resolve(
        ui::ZonedDateTimeDraft {
            date: "2026-03-08".into(),
            time: "02:30".into(),
            zone: draft.zone,
        },
        ui::TimePrecision::Minute,
        ui::ClockFormat::TwentyFourHour,
    );
    assert_eq!(gap.state, ui::ZonedTimePresentation::Nonexistent);
    assert_eq!(
        service.invoke_adjust(
            "23:59".into(),
            ui::TimePrecision::Minute,
            ui::ClockFormat::TwentyFourHour,
            ui::TimeSegment::Minute,
            ui::TimeAdjustment::Increment
        ),
        "00:00"
    );
    let date = |day| ui::CivilDateView {
        year: 2026,
        month: 10,
        day,
    };
    let range = ui::DateRangeView {
        start: date(12),
        end: date(16),
    };
    let view = ui::CalendarView {
        selected: date(12),
        focused: date(12),
        unavailable_ranges: slint::ModelRc::new(slint::VecModel::from(vec![ui::DateRangeView {
            start: date(13),
            end: date(15),
        }])),
        ..Default::default()
    };
    assert!(
        !window
            .global::<ui::KitCalendarService>()
            .invoke_range_valid(view, range, "".into(), "".into())
    );
    let service = window.global::<ui::KitCalendarService>();
    let view = ui::CalendarView {
        focused: date(19),
        ..Default::default()
    };
    for (end, accepted) in [(20, false), (21, true), (23, true)] {
        assert_eq!(
            service.invoke_range_valid(
                view.clone(),
                ui::DateRangeView {
                    start: date(19),
                    end: date(end)
                },
                "2".into(),
                "14".into()
            ),
            accepted
        );
    }
    assert!(!service.invoke_range_valid(
        view.clone(),
        ui::DateRangeView {
            start: date(1),
            end: date(16)
        },
        "2".into(),
        "14".into()
    ));
    let draft = ui::DateRangeDraft {
        phase: ui::DateRangePhase::ChoosingEnd,
        start: date(19),
        ..Default::default()
    };
    let rejected = service.invoke_range_choose(
        view.clone(),
        draft.clone(),
        date(20),
        "2".into(),
        "14".into(),
    );
    assert!(rejected.rejected);
    assert_eq!(rejected.draft, draft);
    assert!(
        !service
            .invoke_range_choose(view.clone(), draft, date(21), "2".into(), "14".into())
            .rejected
    );
    for (minimum, maximum) in [("-1", "14"), ("15", "14"), ("two", "14")] {
        assert!(!service.invoke_range_valid(
            view.clone(),
            ui::DateRangeView {
                start: date(19),
                end: date(23)
            },
            minimum.into(),
            maximum.into()
        ));
    }
}

#[test]
fn slider_pointer_escape_restores_the_pre_gesture_decimal() {
    use i_slint_backend_testing::ElementQuery;
    use slint::{LogicalPosition, platform::PointerEventButton};
    i_slint_backend_testing::init_no_event_loop();
    let window = setup();
    let slider = ElementQuery::from_root(&window)
        .match_inherits("KitSlider")
        .find_first()
        .unwrap();
    let origin = slider.absolute_position();
    let size = slider.size();
    let position = LogicalPosition::new(origin.x + size.width * 0.8, origin.y + size.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert_ne!(window.get_slider_value(), "0.2");
    key(&window, Key::Escape);
    assert_eq!(window.get_slider_value(), "0.2");
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(window.get_slider_value(), "0.2");
}

#[test]
fn a_changed_zone_draft_invalidates_an_in_progress_occurrence_click() {
    use i_slint_backend_testing::ElementQuery;
    use slint::{LogicalPosition, platform::PointerEventButton};
    i_slint_backend_testing::init_no_event_loop();
    let window = crate::ZonedServiceFixture::new().unwrap();
    slint_kit::install(&window);
    window.show().unwrap();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    window.on_chosen(move |value| {
        output
            .borrow_mut()
            .push(slint_kit::ResolvedDateTime::try_from(value).unwrap())
    });
    window.invoke_resolve();
    assert_eq!(
        window.get_resolution(),
        ui::ZonedTimePresentation::Ambiguous
    );
    let button = ElementQuery::from_root(&window)
        .match_predicate(|element| {
            element.accessible_label().as_deref() == Some("Earlier occurrence")
        })
        .find_first()
        .unwrap();
    let origin = button.absolute_position();
    let size = button.size();
    let position = LogicalPosition::new(origin.x + size.width / 2., origin.y + size.height / 2.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window.set_date_text("2026-11-02".into());
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert!(received.borrow().is_empty());
    assert_eq!(
        window.get_resolution(),
        ui::ZonedTimePresentation::Unresolved
    );
    window.invoke_resolve();
    assert_eq!(window.get_resolution(), ui::ZonedTimePresentation::Unique);
    assert_eq!(received.borrow().len(), 1);
}

#[test]
fn field_submission_rejects_an_absent_projection_with_hidden_text() {
    let commit = ui::FieldCommit {
        value: ui::FieldValue {
            present: false,
            text: "name@example.org".into(),
        },
        constraints: ui::KitInputConstraints {
            kind: ui::HtmlInputKind::Email,
            min_length: -1,
            max_length: -1,
            ..Default::default()
        },
        ..Default::default()
    };
    assert!(FieldSpec::email().accept(commit).is_err());
}

#[test]
fn localized_field_codecs_keep_decimal_comma_and_twelve_hour_grammar_through_commit_and_cancel() {
    use slint_kit::{
        ClockFormat, DecimalRules, DecimalScale, DecimalValue, LocalTime, NumberLocale, SignPolicy,
        TimePrecision,
    };
    i_slint_backend_testing::init_no_event_loop();
    let window = crate::LocalizedFieldFixture::new().unwrap();
    slint_kit::install(&window);
    let decimal = FieldSpec::decimal(DecimalRules {
        locale: NumberLocale::FrCa,
        scale: DecimalScale::new(2).unwrap(),
        sign: SignPolicy::Signed,
        bounds: None,
    });
    let clock = FieldSpec::time(TimePrecision::Minute, ClockFormat::TwelveHour);
    slint_kit::install_field_specs(
        &window,
        [
            decimal.named(FieldSpecKey::new("decimal-fr").unwrap()),
            clock.named(FieldSpecKey::new("time12").unwrap()),
        ],
    )
    .unwrap();
    let number = Rc::new(RefCell::new(None));
    let output = number.clone();
    window
        .on_decimal_committed(move |commit| *output.borrow_mut() = decimal.accept(commit).unwrap());
    let time = Rc::new(RefCell::new(None));
    let output = time.clone();
    window.on_time_committed(move |commit| *output.borrow_mut() = clock.accept(commit).unwrap());
    window.show().unwrap();
    window.invoke_commit_decimal();
    assert_eq!(
        *number.borrow(),
        Some("1234.50".parse::<DecimalValue>().unwrap())
    );
    assert_eq!(window.get_decimal_state().accepted.text, "1234,50");
    window.set_decimal_text("-".into());
    window.invoke_cancel_decimal();
    assert_eq!(window.get_decimal_text(), "1234,50");
    window.invoke_commit_time();
    assert_eq!(*time.borrow(), Some(LocalTime::new(13, 30, 0).unwrap()));
    assert_eq!(window.get_time_state().accepted.text, "01:30 PM");
    window.set_time_text("01:".into());
    window.invoke_cancel_time();
    assert_eq!(window.get_time_text(), "01:30 PM");
}

#[test]
fn domain_validation_obeys_commit_policy_and_never_blocks_cancel() {
    use slint_kit::{DraftValue, TextValue};
    i_slint_backend_testing::init_no_event_loop();
    let window = AutonomousFixture::new().unwrap();
    slint_kit::install(&window);
    let spec = FieldSpec::new(
        |text| DraftValue::Parsed(TextValue::new(text)),
        |value| value.as_str().into(),
        |value| {
            if value.as_str() == "forbidden" {
                Err("Reserved project name")
            } else {
                Ok(())
            }
        },
        |error| (*error).into(),
    );
    slint_kit::install_field_specs(&window, [spec.named(FieldSpecKey::new("project").unwrap())])
        .unwrap();
    let service = window.global::<ui::KitFieldService>();
    let state = ui::FieldState {
        accepted: ui::FieldValue {
            present: true,
            text: "accepted".into(),
        },
        draft: "accepted".into(),
        ..Default::default()
    };
    let constraints = ui::KitInputConstraints {
        min_length: -1,
        max_length: -1,
        ..Default::default()
    };
    let edited = service.invoke_reduce(
        state,
        ui::FieldAction::Edit,
        "forbidden".into(),
        constraints.clone(),
        ui::CommitPolicy::OnSubmit,
        ui::Ownership::Local,
        "project".into(),
    );
    assert!(edited.error.is_empty());
    assert!(edited.dirty);
    let rejected = service.invoke_reduce(
        edited,
        ui::FieldAction::Submit,
        "forbidden".into(),
        constraints.clone(),
        ui::CommitPolicy::OnSubmit,
        ui::Ownership::Local,
        "project".into(),
    );
    assert_eq!(rejected.error, "Reserved project name");
    assert_eq!(rejected.accepted.text, "accepted");
    let cancelled = service.invoke_reduce(
        rejected,
        ui::FieldAction::Cancel,
        "forbidden".into(),
        constraints,
        ui::CommitPolicy::OnSubmit,
        ui::Ownership::Local,
        "project".into(),
    );
    assert!(cancelled.error.is_empty());
    assert_eq!(cancelled.draft, "accepted");
    assert!(!cancelled.dirty);
}

#[test]
fn narrow_toggle_labels_stay_inside_their_buttons_in_both_themes_and_scales() {
    use i_slint_backend_testing::ElementQuery;
    use slint::platform::PointerEventButton;
    i_slint_backend_testing::init_no_event_loop();
    for dark in [false, true] {
        for scale in [1., 2.] {
            let window = crate::NarrowToggleFixture::new().unwrap();
            slint_kit::install(&window);
            let mut theme = if dark {
                window.get_dark_theme()
            } else {
                window.get_theme()
            };
            theme.text_scale = scale;
            window.set_theme(theme);
            window.show().unwrap();
            let options = ElementQuery::from_root(&window)
                .match_inherits("ChoiceOption")
                .find_all();
            assert_eq!(options.len(), 8);
            for option in &options {
                let label = option
                    .query_descendants()
                    .match_id("KitToggleGroup::option-label")
                    .find_first()
                    .unwrap();
                let origin = option.absolute_position();
                let bounds = option.size();
                let text = label.absolute_position();
                let size = label.size();
                assert!(text.x > origin.x, "label starts inside the padded button");
                assert!(
                    text.x + size.width < origin.x + bounds.width + 0.1,
                    "label fits button at scale {scale}"
                );
                assert!(
                    text.y >= origin.y && text.y + size.height <= origin.y + bounds.height + 0.1
                );
                assert!(size.width > 0. && size.height > 0.);
            }
            assert_eq!(
                options[5].accessible_label().as_deref(),
                Some("Weekly digest")
            );
            for name in ["Email", "Calendar"] {
                // Membership publishes a new row projection. Resolve the live
                // delegate after each change instead of retaining a removed one.
                let option = ElementQuery::from_root(&window)
                    .match_inherits("ChoiceOption")
                    .match_predicate(move |element| {
                        element.accessible_label().as_deref() == Some(name)
                    })
                    .find_first()
                    .unwrap();
                let origin = option.absolute_position();
                let size = option.size();
                let position = slint::LogicalPosition::new(
                    origin.x + size.width / 2.,
                    origin.y + size.height / 2.,
                );
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
            }
            assert_eq!(
                window
                    .get_selected()
                    .iter()
                    .map(|row| row.token)
                    .collect::<Vec<_>>(),
                vec![1, 8]
            );
            window.invoke_focus_toggles();
            for key in [Key::Home, Key::Space, Key::End, Key::Space] {
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
            }
            assert_eq!(window.get_selected().row_count(), 0);
            window.hide().unwrap();
        }
    }
}
