// SPDX-License-Identifier: GPL-3.0-or-later
//! Committed absence stays distinct from a malformed date. These software
//! backend checks do not qualify a platform IME or a screen reader.
use crate::{
    CivilDateView, CommitPolicy, FieldAccess, OptionalTemporalFixture, Ownership,
    TemporalNativeKeyboardFixture,
};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn init_backend() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
}
fn fixture() -> OptionalTemporalFixture {
    init_backend();
    let app = OptionalTemporalFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    frame(&app);
    app
}

#[test]
fn installing_services_preserves_an_explicit_host_draft_and_seeds_the_accepted_baseline() {
    init_backend();
    let app = OptionalTemporalFixture::new().unwrap();
    app.set_picker_text("2026-10-05".into());
    slint_kit::install(&app);
    app.show().unwrap();
    frame(&app);
    assert_eq!(app.get_picker_text(), "2026-10-05");
    assert_eq!(app.get_picked().day, 19);
    app.invoke_focus_picker();
    erase(&app, 10);
    key(&app, Key::Tab);
    assert_eq!(app.get_picked(), CivilDateView::default());
    assert_eq!(app.get_picker_clear_count(), 1);
    assert_eq!(app.get_picker_choice_count(), 0);
}
fn frame(app: &OptionalTemporalFixture) -> slint::SharedPixelBuffer<slint::Rgba8Pixel> {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap()
}
fn key(app: &impl ComponentHandle, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn move_to_end(app: &impl ComponentHandle) {
    // Slint follows native Apple Home/End scrolling semantics. Its native
    // end-of-line shortcut is Control+RightArrow on Apple, End elsewhere.
    if cfg!(target_vendor = "apple") {
        app.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        key(app, Key::RightArrow);
        app.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
    } else {
        key(app, Key::End);
    }
}
fn erase(app: &impl ComponentHandle, characters: usize) {
    move_to_end(app);
    for _ in 0..characters {
        key(app, Key::Backspace);
    }
}
fn action(app: &OptionalTemporalFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap()
}

#[test]
fn clearing_the_initial_date_accepts_absence_on_blur_without_a_zero_date_event() {
    let app = fixture();
    // Compare the same initial text and keys with a bare native editor before
    // exercising the kit. No kit callback filters this reference's events.
    let native = TemporalNativeKeyboardFixture::new().unwrap();
    native.set_text("2026-10-19".into());
    native.show().unwrap();
    native.invoke_focus_start();
    key(&native, Key::End);
    key(&native, Key::Backspace);
    if cfg!(target_vendor = "apple") {
        assert_eq!(native.get_text(), "2026-10-19");
    } else {
        assert_eq!(native.get_text(), "2026-10-1");
    }
    native.set_text("2026-10-19".into());
    native.invoke_focus_start();
    erase(&native, 10);
    assert_eq!(native.get_text(), "");
    native.hide().unwrap();
    assert_eq!(app.get_picker_text(), "2026-10-19");
    app.invoke_focus_picker();
    erase(&app, 10);
    assert_eq!(app.get_picker_text(), "");
    assert_eq!(
        app.get_picked().year,
        2026,
        "the native draft is not accepted early"
    );
    assert_eq!(app.get_picker_clear_count(), 0);
    key(&app, Key::Tab);
    assert_eq!(app.get_picked(), CivilDateView::default());
    assert_eq!(app.get_picker_clear_count(), 1);
    assert_eq!(
        app.get_picker_choice_count(),
        0,
        "absence has its own typed signal"
    );
    app.invoke_focus_picker();
    key(&app, Key::Return);
    assert_eq!(
        app.get_picker_clear_count(),
        1,
        "unchanged absence does not repeat"
    );
}

#[test]
fn a_controlled_empty_date_waits_for_acknowledgement_and_required_or_custom_errors_refuse_it() {
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    app.invoke_focus_picker();
    erase(&app, 10);
    key(&app, Key::Return);
    assert_eq!(app.get_picker_clear_count(), 1);
    assert_eq!(
        app.get_picked().day,
        19,
        "the host still owns the accepted date"
    );
    app.set_picked(CivilDateView::default());
    key(&app, Key::Return);
    assert_eq!(app.get_picker_clear_count(), 1);
    for required in [true, false] {
        app.set_picked(CivilDateView {
            year: 2026,
            month: 10,
            day: 19,
        });
        app.set_required(required);
        app.set_custom_validity(if required { "" } else { "Keep the appointment" }.into());
        app.invoke_focus_picker();
        erase(&app, 10);
        key(&app, Key::Return);
        assert!(!app.get_picker_validity().valid);
        assert_eq!(app.get_picked().day, 19);
        assert_eq!(app.get_picker_clear_count(), 1);
    }
}

#[test]
fn the_clear_affordance_edits_the_date_natively_and_respects_access_and_visibility() {
    let app = fixture();
    let editor_count = || {
        ElementQuery::from_root(&app)
            .match_inherits("KitDatePicker")
            .find_first()
            .unwrap()
            .query_descendants()
            .match_type_name("TextInput")
            .find_all()
            .len()
    };
    assert_eq!(editor_count(), 1);
    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        app.set_access(access);
        frame(&app);
        action(&app, "Clear optional date").invoke_accessible_default_action();
        app.invoke_clear_picker();
        assert_eq!(app.get_picker_text(), "2026-10-19");
        action(&app, "Clear optional appointment").invoke_accessible_default_action();
        app.invoke_clear_local();
        assert_eq!(app.get_local_date(), "2026-10-19");
        assert_eq!(app.get_local_time(), "09:30");
        assert_eq!(app.get_local_clear_count(), 0);
    }
    app.set_access(FieldAccess::Editable);
    app.invoke_focus_picker();
    action(&app, "Clear optional date").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_picker_text(), "");
    assert_eq!(editor_count(), 1, "the native editor stays mounted");
    key(&app, Key::Tab);
    assert_eq!(app.get_picker_clear_count(), 1);
    app.set_clearable(false);
    assert!(
        ElementHandle::find_by_accessible_label(&app, "Clear optional date")
            .next()
            .is_none()
    );
}

#[test]
fn a_local_pair_clears_atomically_and_a_partial_or_required_pair_never_clears_its_value() {
    let app = fixture();
    app.invoke_focus_local();
    erase(&app, 10);
    key(&app, Key::Tab);
    assert_eq!(
        app.get_local_clear_count(),
        0,
        "moving between editors is not leaving the group"
    );
    app.invoke_focus_after();
    assert_eq!(app.get_local_value().date.day, 19);
    assert!(
        !app.get_local_validity().valid,
        "one empty segment is not absence"
    );
    app.set_local_date("2026-10-19".into());
    app.invoke_focus_local();
    action(&app, "Clear optional appointment").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_local_date(), "");
    assert_eq!(app.get_local_time(), "");
    assert_eq!(app.get_local_value().date.day, 19);
    assert_eq!(app.get_local_choice_count(), 0);
    assert!(
        app.get_local_editor_focused(),
        "the clear action leaves a native editor focused"
    );
    key(&app, Key::Tab);
    assert!(app.get_after_focused());
    assert_eq!(app.get_local_value(), crate::LocalDateTimeView::default());
    assert_eq!(app.get_local_clear_count(), 1);
    assert_eq!(app.get_local_choice_count(), 0);
    app.set_local_value(crate::LocalDateTimeView {
        date: CivilDateView {
            year: 2026,
            month: 10,
            day: 19,
        },
        time: crate::LocalTimeView {
            hour: 9,
            minute: 30,
            ..Default::default()
        },
    });
    app.set_local_date("2026-10-19".into());
    app.set_local_time("09:30".into());
    app.set_required(true);
    app.invoke_clear_local();
    app.invoke_commit_local();
    assert_eq!(app.get_local_value().date.day, 19);
    assert_eq!(app.get_local_clear_count(), 1);
    assert!(!app.get_local_validity().valid);
    app.set_required(false);
    app.set_custom_validity("Keep the appointment".into());
    app.set_local_date("2026-10-19".into());
    app.set_local_time("09:30".into());
    app.invoke_clear_local();
    app.invoke_commit_local();
    assert!(!app.get_local_validity().valid);
    assert_eq!(app.get_local_value().date.day, 19);
    assert_eq!(app.get_local_clear_count(), 1);
}

#[test]
fn local_pair_clear_obeys_submit_and_change_policies_and_the_host_owned_value() {
    let app = fixture();
    app.set_policy(CommitPolicy::OnSubmit);
    app.set_ownership(Ownership::Controlled);
    app.invoke_clear_picker();
    app.invoke_focus_after();
    assert_eq!(app.get_picker_clear_count(), 0);
    app.invoke_commit_picker();
    assert_eq!(app.get_picker_clear_count(), 1);
    assert_eq!(app.get_picked().day, 19);
    app.set_picked(CivilDateView::default());
    app.invoke_commit_picker();
    assert_eq!(app.get_picker_clear_count(), 1);
    app.invoke_clear_local();
    app.invoke_focus_after();
    assert_eq!(app.get_local_clear_count(), 0);
    app.invoke_commit_local();
    assert_eq!(app.get_local_clear_count(), 1);
    assert_eq!(app.get_local_value().date.day, 19);
    app.set_local_value(Default::default());
    app.invoke_commit_local();
    assert_eq!(app.get_local_clear_count(), 1);
    app.set_local_value(crate::LocalDateTimeView {
        date: CivilDateView {
            year: 2026,
            month: 10,
            day: 19,
        },
        time: crate::LocalTimeView {
            hour: 9,
            minute: 30,
            ..Default::default()
        },
    });
    app.set_local_date("2026-10-19".into());
    app.set_local_time("09:30".into());
    app.set_ownership(Ownership::Local);
    app.set_policy(CommitPolicy::OnChangeWhenValid);
    app.invoke_clear_local();
    assert_eq!(app.get_local_value(), crate::LocalDateTimeView::default());
    assert_eq!(app.get_local_clear_count(), 2);
    assert_eq!(
        app.get_local_choice_count(),
        0,
        "no intermediate nonempty pair is accepted while clearing"
    );
    assert_eq!(
        app.get_local_request_count(),
        2,
        "valid-change acceptance never emits an explicit submit request"
    );
}

#[test]
fn temporal_clear_keeps_native_undo_and_rejects_a_restored_partial_pair() {
    let app = fixture();
    app.set_policy(CommitPolicy::OnSubmit);
    let undo = || {
        app.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        app.window()
            .dispatch_event(WindowEvent::KeyPressed { text: "z".into() });
        app.window()
            .dispatch_event(WindowEvent::KeyReleased { text: "z".into() });
        app.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
    };
    app.invoke_clear_picker();
    assert_eq!(app.get_picker_text(), "");
    undo();
    assert_eq!(app.get_picker_text(), "2026-10-19");
    assert_eq!(app.get_picker_clear_count(), 0);
    assert_eq!(app.get_picker_choice_count(), 0);
    app.invoke_clear_local();
    assert_eq!(app.get_local_date(), "");
    assert_eq!(app.get_local_time(), "");
    assert_eq!(app.get_local_edit_count(), 2);
    assert_eq!(app.get_local_clear_count(), 0);
    undo();
    assert_eq!(app.get_local_time(), "09:30");
    assert_eq!(app.get_local_date(), "");
    app.invoke_commit_local();
    assert!(!app.get_local_validity().valid);
    assert_eq!(app.get_local_value().date.day, 19);
    assert_eq!(app.get_local_clear_count(), 0);
    app.invoke_focus_local();
    undo();
    assert_eq!(app.get_local_date(), "2026-10-19");
    app.invoke_commit_local();
    assert!(app.get_local_validity().valid);
    assert_eq!(app.get_local_choice_count(), 0);
    app.invoke_clear_local();
    app.invoke_commit_local();
    assert_eq!(app.get_local_value(), crate::LocalDateTimeView::default());
    assert_eq!(app.get_local_clear_count(), 1);
    assert_eq!(app.get_local_choice_count(), 0);
}

#[test]
fn tabbing_to_the_pair_clear_action_keeps_the_group_pending_and_space_clears_it() {
    let app = fixture();
    app.invoke_focus_local();
    move_to_end(&app);
    key(&app, Key::Backspace);
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: "8".into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: "8".into() });
    assert_eq!(app.get_local_date(), "2026-10-18");
    key(&app, Key::Tab);
    key(&app, Key::Tab);
    assert!(
        !app.get_local_editor_focused(),
        "the auxiliary action has native keyboard focus"
    );
    assert_eq!(
        app.get_local_value().date.day,
        19,
        "the focus is still inside the composite"
    );
    assert_eq!(app.get_local_choice_count(), 0);
    key(&app, Key::Space);
    assert_eq!(app.get_local_date(), "");
    assert_eq!(app.get_local_time(), "");
    assert!(
        app.get_local_editor_focused(),
        "the deferred button focus cleanup preserves the newly focused editor"
    );
    key(&app, Key::Tab);
    assert!(app.get_after_focused());
    assert_eq!(app.get_local_value(), crate::LocalDateTimeView::default());
    assert_eq!(app.get_local_clear_count(), 1);
    assert_eq!(app.get_local_choice_count(), 0);
}

#[test]
fn optional_temporal_clear_states_render_in_both_themes_and_at_double_text() {
    let app = fixture();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            theme.reduced_motion = true;
            app.set_theme(theme);
            app.set_access(FieldAccess::Editable);
            app.set_picked(CivilDateView {
                year: 2026,
                month: 10,
                day: 19,
            });
            app.set_local_date("2026-10-19".into());
            app.set_local_time("09:30".into());
            app.set_local_value(crate::LocalDateTimeView {
                date: CivilDateView {
                    year: 2026,
                    month: 10,
                    day: 19,
                },
                time: crate::LocalTimeView {
                    hour: 9,
                    minute: 30,
                    ..Default::default()
                },
            });
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-ready-{name}-{scale}"),
            );
            let clear = action(&app, "Clear optional appointment");
            let position = clear.absolute_position();
            let size = clear.size();
            assert!(size.width > 0. && size.height > 0.);
            let position = slint::LogicalPosition::new(
                position.x + size.width / 2.,
                position.y + size.height / 2.,
            );
            app.window()
                .dispatch_event(WindowEvent::PointerMoved { position });
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-hover-{name}-{scale}"),
            );
            app.window().dispatch_event(WindowEvent::PointerPressed {
                position,
                button: PointerEventButton::Left,
            });
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-pressed-{name}-{scale}"),
            );
            app.window().dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
            app.set_local_date("2026-10-19".into());
            app.set_local_time("09:30".into());
            app.invoke_focus_local();
            key(&app, Key::Tab);
            key(&app, Key::Tab);
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-focused-{name}-{scale}"),
            );
            app.set_access(FieldAccess::ReadOnly);
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-readonly-{name}-{scale}"),
            );
            app.set_access(FieldAccess::Disabled);
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-disabled-{name}-{scale}"),
            );
            app.set_access(FieldAccess::Editable);
            app.set_required(true);
            app.invoke_clear_picker();
            app.invoke_clear_local();
            app.invoke_commit_local();
            app.invoke_focus_after();
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-required-{name}-{scale}"),
            );
            app.set_required(false);
            app.invoke_focus_picker();
            key(&app, Key::Return);
            app.invoke_commit_local();
            assert!(app.get_picker_validity().valid && app.get_local_validity().valid);
            crate::choice_presentation::save_capture(
                &frame(&app),
                &format!("optional-temporal-empty-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn a_dynamically_mounted_date_picker_can_commit_absence_from_an_empty_host_draft() {
    init_backend();
    let app = crate::DynamicOptionalTemporalFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.set_show_date(true);
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    ElementHandle::find_by_accessible_label(&app, "Dynamic optional date")
        .next()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: "2".into() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: "2".into() });
    key(&app, Key::Backspace);
    key(&app, Key::Tab);
    assert_eq!(app.get_cleared_count(), 1);
    assert_eq!(app.get_chosen_count(), 0);
}
