// SPDX-License-Identifier: GPL-3.0-or-later
use crate::TextFieldBehaviorFixture;
use slint_kit::{
    FormBinding, FormFieldSnapshot, FormIntent, FormRegistrationError, FormSubmitError,
    HtmlValidity,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    Name,
    Email,
}
#[test]
fn form_reports_all_fields_focuses_first_and_guards_submission_lifetimes() {
    i_slint_backend_testing::init_no_event_loop();
    let app = TextFieldBehaviorFixture::new().unwrap();
    let invalid = Rc::new(Cell::new(true));
    let focused = Rc::new(RefCell::new(Vec::new()));
    let reports = Rc::new(Cell::new(0));
    let resets = Rc::new(Cell::new(0));
    let projections = Rc::new(RefCell::new(Vec::new()));
    let published = projections.clone();
    let binding = FormBinding::new(&app, move |_, view| {
        published
            .borrow_mut()
            .push((view.valid, view.dirty, view.submitting))
    });
    for key in [Field::Name, Field::Email] {
        let invalid_snapshot = invalid.clone();
        let invalid_report = invalid.clone();
        let reports = reports.clone();
        let focused = focused.clone();
        let resets = resets.clone();
        binding
            .register(
                key,
                move |_| FormFieldSnapshot {
                    will_validate: true,
                    validity: HtmlValidity {
                        value_missing: invalid_snapshot.get(),
                        ..Default::default()
                    },
                    dirty: true,
                    message: "Required".into(),
                    target: Default::default(),
                },
                move |_| {
                    reports.set(reports.get() + 1);
                    !invalid_report.get()
                },
                move |_| focused.borrow_mut().push(key),
                move |_| resets.set(resets.get() + 1),
            )
            .unwrap();
    }
    assert!(!binding.report_validity());
    assert_eq!(reports.get(), 2);
    assert_eq!(*focused.borrow(), [Field::Name]);
    assert_eq!(
        binding.submit("payload"),
        Err(FormSubmitError::InvalidFields {
            first: Field::Name,
            remaining: vec![Field::Email]
        })
    );
    invalid.set(false);
    let ticket = match binding.submit("payload").unwrap() {
        FormIntent::Submit { ticket, payload } => {
            assert_eq!(payload, "payload");
            ticket
        }
        FormIntent::Reset => panic!("submit intent"),
    };
    assert!(matches!(
        binding.submit("second"),
        Err(FormSubmitError::Busy)
    ));
    binding.reset();
    assert_eq!(resets.get(), 2);
    assert!(
        !binding.finish(ticket, vec![]),
        "reset invalidates an in-flight response"
    );
    assert_eq!(
        binding.register(Field::Name, |_| unreachable!(), |_| true, |_| {}, |_| {}),
        Err(FormRegistrationError::DuplicateKey)
    );
    assert!(projections.borrow().iter().any(|projection| projection.2));
    drop(app);
    assert!(matches!(
        binding.submit("closed"),
        Err(FormSubmitError::WindowClosed)
    ));
}
#[test]
fn composed_form_implicitly_submits_only_after_reporting_and_blocks_busy_actions() {
    use crate::FormBehaviorFixture;
    use slint::{
        ComponentHandle,
        platform::{Key, WindowEvent},
    };
    i_slint_backend_testing::init_no_event_loop();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    let binding = FormBinding::new(&app, |app, view| {
        app.set_valid(view.valid);
        app.set_dirty(view.dirty);
        app.set_submitting(view.submitting);
        app.set_errors(view.errors);
    });
    binding
        .register(
            Field::Email,
            |app| FormFieldSnapshot {
                will_validate: app.get_email_will_validate(),
                validity: app.get_email_validity().into(),
                dirty: app.get_email_dirty(),
                message: app.get_email_message(),
                target: Default::default(),
            },
            FormBehaviorFixture::invoke_report_email,
            |app| {
                app.invoke_focus_email();
            },
            FormBehaviorFixture::invoke_reset_email,
        )
        .unwrap();
    let validate = binding.clone();
    app.on_validate_form(move || validate.report_validity());
    let accepted = Rc::new(Cell::new(0));
    let observed = accepted.clone();
    app.on_field_committed(move |_| observed.set(observed.get() + 1));
    let submitted = Rc::new(Cell::new(0));
    let observed = submitted.clone();
    let accepted_on_submit = accepted.clone();
    app.on_send_form(move || {
        assert!(
            accepted_on_submit.get() > 0,
            "native acceptance precedes the form action"
        );
        observed.set(observed.get() + 1);
    });
    app.show().unwrap();
    app.invoke_focus_email();
    let enter = || {
        slint::platform::update_timers_and_animations();
        app.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Return.into(),
        });
        app.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Return.into(),
        });
        slint::platform::update_timers_and_animations();
    };
    app.set_email_text("".into());
    enter();
    assert_eq!(submitted.get(), 0);
    assert!(!app.get_valid());
    app.set_email_text("person@example.com".into());
    enter();
    assert_eq!(submitted.get(), 1);
    app.set_submitting(true);
    enter();
    assert_eq!(submitted.get(), 1, "busy form blocks implicit resubmission");
}
#[test]
fn refused_reporting_cannot_produce_an_empty_invalid_field_result() {
    i_slint_backend_testing::init_no_event_loop();
    let app = TextFieldBehaviorFixture::new().unwrap();
    let binding = FormBinding::new(&app, |_, _| {});
    binding
        .register(
            Field::Name,
            |_| FormFieldSnapshot {
                will_validate: true,
                validity: HtmlValidity::default(),
                dirty: false,
                message: "Validation is not ready".into(),
                target: Default::default(),
            },
            |_| false,
            |_| {},
            |_| {},
        )
        .unwrap();
    assert_eq!(
        binding.submit(()),
        Err(FormSubmitError::InvalidFields {
            first: Field::Name,
            remaining: vec![]
        })
    );
}

fn form_key(app: &crate::FormKeyboardFixture, key: slint::platform::Key) {
    use slint::ComponentHandle;
    slint::platform::update_timers_and_animations();
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: key.into() });
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn keyboard_binding(
    app: &crate::FormKeyboardFixture,
) -> FormBinding<crate::FormKeyboardFixture, Field> {
    use crate::FormKeyboardFixture;
    let binding = FormBinding::new(app, |app: &FormKeyboardFixture, view| {
        app.set_submitting(view.submitting);
        app.set_errors(view.errors);
    });
    for key in [Field::Name, Field::Email] {
        binding
            .register(
                key,
                move |app| match key {
                    Field::Name => FormFieldSnapshot {
                        will_validate: app.get_first_will_validate(),
                        validity: app.get_first_validity().into(),
                        dirty: app.get_first_dirty(),
                        message: app.get_first_message(),
                        target: slint_kit::ui::FocusRequest {
                            present: true,
                            token: 1,
                            generation: 1,
                            request: 1,
                        },
                    },
                    Field::Email => FormFieldSnapshot {
                        will_validate: app.get_second_will_validate(),
                        validity: app.get_second_validity().into(),
                        dirty: app.get_second_dirty(),
                        message: app.get_second_message(),
                        target: slint_kit::ui::FocusRequest {
                            present: true,
                            token: 2,
                            generation: 1,
                            request: 1,
                        },
                    },
                },
                move |app| match key {
                    Field::Name => app.invoke_report_first(),
                    Field::Email => app.invoke_report_second(),
                },
                move |app| match key {
                    Field::Name => app.invoke_focus_first(),
                    Field::Email => app.invoke_focus_second(),
                },
                move |app| match key {
                    Field::Name => app.invoke_reset_first(),
                    Field::Email => app.invoke_reset_second(),
                },
            )
            .unwrap();
    }
    let validate = binding.clone();
    app.on_validate(move || validate.report_validity());
    let reset = binding.clone();
    app.on_reset_requested(move || reset.reset());
    binding
}
#[test]
fn implicit_form_reporting_is_ordered_once_and_reset_invalidates_submission() {
    use slint::platform::Key;
    use slint::{ComponentHandle, Model};
    i_slint_backend_testing::init_no_event_loop();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    let binding = keyboard_binding(&app);
    let first_invalid = Rc::new(Cell::new(0));
    let first = first_invalid.clone();
    app.on_first_invalid(move || first.set(first.get() + 1));
    let second_invalid = Rc::new(Cell::new(0));
    let second = second_invalid.clone();
    app.on_second_invalid(move || second.set(second.get() + 1));
    let tickets = Rc::new(RefCell::new(Vec::new()));
    let submitted = tickets.clone();
    let submit = binding.clone();
    app.on_submitted(move || match submit.submit(()) {
        Ok(FormIntent::Submit {
            ticket,
            payload: (),
        }) => submitted.borrow_mut().push(ticket),
        Ok(FormIntent::Reset) => panic!("submit intent"),
        Err(error) => panic!("unexpected guard: {error:?}"),
    });
    app.show().unwrap();
    app.invoke_focus_first();
    form_key(&app, Key::Return);
    assert_eq!(
        (first_invalid.get(), second_invalid.get()),
        (1, 1),
        "each invalid control reports once after native Enter"
    );
    assert_eq!(app.get_errors().row_count(), 2);
    assert!(
        app.get_first_focus(),
        "only the first invalid registration receives focus"
    );
    assert!(tickets.borrow().is_empty());
    app.set_first_text("First value".into());
    app.set_second_text("Second value".into());
    app.invoke_focus_first();
    form_key(&app, Key::Return);
    assert_eq!(tickets.borrow().len(), 1);
    assert!(app.get_submitting());
    form_key(&app, Key::Return);
    assert_eq!(tickets.borrow().len(), 1, "busy form cannot resubmit");
    app.invoke_reset();
    assert!(!app.get_submitting());
    assert_eq!(app.get_errors().row_count(), 0);
    assert!(app.get_first_text().is_empty() && app.get_second_text().is_empty());
    assert!(
        !binding.finish(tickets.borrow()[0], vec![]),
        "reset fences the response"
    );
}
#[test]
fn implicit_form_submission_preserves_auxiliary_multiline_and_host_shortcuts() {
    use slint::{ComponentHandle, platform::Key};
    i_slint_backend_testing::init_no_event_loop();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    app.on_validate(|| true);
    app.set_first_text("First".into());
    app.set_second_text("Second".into());
    let submits = Rc::new(Cell::new(0));
    let count = submits.clone();
    app.on_submitted(move || count.set(count.get() + 1));
    let commits = Rc::new(Cell::new(0));
    let count = commits.clone();
    app.on_first_committed(move |_| count.set(count.get() + 1));
    app.show().unwrap();
    app.set_auxiliary(true);
    app.invoke_focus_second();
    form_key(&app, Key::Return);
    assert_eq!(submits.get(), 0, "auxiliary single-line editor owns Enter");
    app.set_multiline(true);
    app.invoke_focus_first();
    form_key(&app, Key::Return);
    assert_eq!(submits.get(), 0);
    assert!(
        app.get_first_text().contains('\n'),
        "multiline Enter stays native"
    );
    app.set_multiline(false);
    app.set_first_text("First".into());
    app.set_consume_return(true);
    app.invoke_focus_first();
    form_key(&app, Key::Return);
    assert_eq!(submits.get(), 0);
    assert_eq!(commits.get(), 0, "host shortcut precedes native acceptance");
    app.set_consume_return(false);
    app.set_implicit_submit(false);
    form_key(&app, Key::Return);
    assert_eq!(submits.get(), 0);
    assert_eq!(
        commits.get(),
        1,
        "opting out preserves native field acceptance"
    );
    app.set_implicit_submit(true);
    app.set_access(slint_kit::ui::FieldAccess::ReadOnly);
    form_key(&app, Key::Return);
    assert_eq!(
        submits.get(),
        1,
        "read-only single-line field still permits form Enter"
    );
    assert_eq!(commits.get(), 1);
}
#[test]
fn explicit_validation_opt_out_retains_busy_and_window_guards() {
    i_slint_backend_testing::init_no_event_loop();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    let binding = keyboard_binding(&app);
    assert!(
        !app.invoke_submit(),
        "form defaults to enforcing validation"
    );
    app.set_form_no_validate(true);
    let submits = Rc::new(Cell::new(0));
    let count = submits.clone();
    app.on_submitted(move || count.set(count.get() + 1));
    assert!(app.invoke_submit());
    assert_eq!(submits.get(), 1);
    assert!(matches!(
        binding.submit(()),
        Err(FormSubmitError::InvalidFields { .. })
    ));
    let ticket = match binding.submit_without_validation(()).unwrap() {
        FormIntent::Submit {
            ticket,
            payload: (),
        } => ticket,
        FormIntent::Reset => panic!("submit intent"),
    };
    assert!(matches!(
        binding.submit_without_validation(()),
        Err(FormSubmitError::Busy)
    ));
    assert!(!app.invoke_submit());
    assert!(binding.finish(ticket, vec![]));
    drop(app);
    assert!(matches!(
        binding.submit_without_validation(()),
        Err(FormSubmitError::WindowClosed)
    ));
}

#[test]
fn coordinated_form_renders_named_summary_and_first_invalid_focus_in_all_themes() {
    use i_slint_backend_testing::{
        AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
    };
    use slint::{ComponentHandle, Model};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    let _binding = keyboard_binding(&app);
    app.show().unwrap();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            assert!(!app.invoke_submit());
            assert_eq!(app.get_errors().row_count(), 2);
            assert!(app.get_first_focus());
            let form = ElementHandle::find_by_accessible_label(&app, "Example form")
                .find(|e| e.accessible_role() == Some(AccessibleRole::Form))
                .unwrap();
            assert!(form.size().height > 0.);
            let submit = ElementQuery::from_root(&app)
                .match_predicate(|e| e.accessible_label().as_deref() == Some("Submit"))
                .find_first()
                .unwrap();
            assert!(
                submit.absolute_position().y + submit.size().height
                    <= app
                        .window()
                        .size()
                        .to_logical(app.window().scale_factor())
                        .height,
                "form actions remain inside the visible window at {name}/{scale}"
            );
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("coordinated-form-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn form_container_does_not_add_a_blank_tab_stop() {
    use slint::{ComponentHandle, platform::Key};
    i_slint_backend_testing::init_no_event_loop();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_first();
    for _ in 0..5 {
        form_key(&app, Key::Tab);
    }
    assert!(
        app.get_first_focus(),
        "Tab visits only the two fields and three actions before returning to the first field"
    );
}

#[test]
fn disabling_the_focused_editor_cannot_turn_its_rejected_enter_into_a_form_action() {
    use slint::{ComponentHandle, platform::Key};
    i_slint_backend_testing::init_no_event_loop();
    let app = crate::FormKeyboardFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_first_text("First".into());
    app.set_second_text("Second".into());
    app.on_validate(|| true);
    let submissions = Rc::new(Cell::new(0));
    let observed = submissions.clone();
    app.on_submitted(move || observed.set(observed.get() + 1));
    app.show().unwrap();
    app.invoke_focus_first();
    app.set_access(slint_kit::ui::FieldAccess::Disabled);
    form_key(&app, Key::Return);
    assert_eq!(submissions.get(), 0);
}
