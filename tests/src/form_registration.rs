// SPDX-License-Identifier: GPL-3.0-or-later
use slint_kit::{FormBinding, FormSubmitError};
use std::{cell::Cell, rc::Rc};

mod registration_example {
    pub mod schema {
        use crate::{FormBehaviorFixture as App, RegistrationField};
        slint_kit::form_schema!(Registration, App, RegistrationField, {
            name: required(slint_kit::TextValue, slint_kit::FieldSpec::text(), "Check name"),
            email: required(slint_kit::EmailAddress, slint_kit::FieldSpec::email(), "Check email"),
            birth: required(slint_kit::CivilDate, slint_kit::FieldSpec::date(), "Check birth"),
            consent: checked,
        });
    }
}
#[test]
fn four_field_schema_returns_typed_values_and_routes_server_errors_to_the_stable_field() {
    use crate::{FormBehaviorFixture, RegistrationField};
    use registration_example::schema::Registration;
    use slint::ComponentHandle;
    i_slint_backend_testing::init_no_event_loop();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    let form = Registration::bind(&app);
    let submission = form.submit().unwrap();
    assert_eq!(submission.payload.name.as_str(), "Ada");
    assert_eq!(submission.payload.email.as_str(), "hello@example.com");
    assert_eq!(submission.payload.birth.iso_string(), "2026-10-19");
    assert!(submission.payload.consent);
    assert!(form.finish(
        submission.ticket,
        vec![slint_kit::FieldError {
            field: RegistrationField::Email,
            error: "Already registered".into()
        }]
    ));
    assert_eq!(app.get_email_error(), "Already registered");
    assert!(app.get_name_error().is_empty());
    assert!(!app.get_valid());
    let email = app.get_email_text();
    form.binding().field_edited(&RegistrationField::Name);
    assert_eq!(
        app.get_email_error(),
        "Already registered",
        "unrelated drafts retain server errors"
    );
    app.show().unwrap();
    app.invoke_focus_email();
    app.invoke_select_email();
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyPressed {
            text: "changed@example.com".into(),
        });
    app.window()
        .dispatch_event(slint::platform::WindowEvent::KeyReleased {
            text: "changed@example.com".into(),
        });
    slint::platform::update_timers_and_animations();
    assert!(
        app.get_email_error().is_empty(),
        "native edits clear only their field error"
    );
    assert_eq!(app.get_email_text(), "changed@example.com");
    // Slint keeps selected-text deletion and replacement insertion as two
    // native undo entries, just as the standalone editor does.
    app.invoke_undo_email();
    assert_eq!(app.get_email_text(), "");
    app.invoke_undo_email();
    assert_eq!(
        app.get_email_text(),
        email,
        "error projection preserves native undo"
    );
    let next = form.submit().unwrap();
    form.binding().reset();
    assert!(!form.finish(
        next.ticket,
        vec![slint_kit::FieldError {
            field: RegistrationField::Email,
            error: "Stale".into()
        }]
    ));
    assert!(app.get_email_error().is_empty());
    app.show().unwrap();
}
#[test]
fn typed_parse_failure_is_rendered_without_submitting_and_can_be_corrected() {
    use crate::{FormBehaviorFixture, RegistrationField};
    use registration_example::schema::Registration;
    use slint::ComponentHandle;
    i_slint_backend_testing::init_no_event_loop();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    let form = Registration::bind(&app);
    app.show().unwrap();
    let result = form.binding().submit_typed::<()>(|_| {
        Err(slint_kit::FormErrors::one(
            RegistrationField::Birth,
            "Must be an adult",
        ))
    });
    assert!(
        matches!(result, Err(FormSubmitError::InvalidValue {first: RegistrationField::Birth, remaining}) if remaining.is_empty())
    );
    assert_eq!(app.get_birth_error(), "Must be an adult");
    assert!(
        app.get_birth_focused(),
        "domain parsing focuses its first invalid field"
    );
    assert!(!app.get_submitting());
    form.binding().field_edited(&RegistrationField::Birth);
    assert!(form.submit().is_ok());
}
mod catalogue_english {
    include!(concat!(env!("OUT_DIR"), "/form-en.rs"));
}
mod catalogue_french {
    include!(concat!(env!("OUT_DIR"), "/form-fr.rs"));
}

#[test]
fn fresh_form_windows_keep_the_requested_width_without_forced_resize() {
    use slint::ComponentHandle;
    i_slint_backend_testing::init_no_event_loop();
    // The first show uses WindowProperties.layout_constraints().preferred,
    // including its effective min/max bounds. set_size would bypass them.
    macro_rules! check_initial_width {
        ($window:ty, $width:expr) => {{
            let app = <$window>::new().unwrap();
            slint_kit::install(&app);
            app.show().unwrap();
            assert_eq!(app.window().size().width, $width);
            app.hide().unwrap();
        }};
    }
    check_initial_width!(crate::FormBehaviorFixture, 560);
    check_initial_width!(catalogue_english::Example, 520);
    check_initial_width!(catalogue_french::Example, 520);
}

#[test]
fn schema_synchronous_callback_guards_rust_submission_and_resets_registered_fields() {
    use crate::FormBehaviorFixture;
    use registration_example::schema::Registration;
    use slint::{ComponentHandle, Model};
    i_slint_backend_testing::init_no_event_loop();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    Registration::on_submit(&app, move |app, value| {
        observed.set(observed.get() + 1);
        app.set_result(value.email.as_str().into());
    });
    app.show().unwrap();
    app.set_name_text("".into());
    app.invoke_send_form();
    assert_eq!(
        calls.get(),
        0,
        "the Rust callback cannot bypass constraints"
    );
    assert!(app.get_errors().row_count() > 0);
    app.invoke_reset_form();
    assert_eq!(app.get_name_text(), "Ada");
    assert_eq!(app.get_errors().row_count(), 0);
    app.invoke_send_form();
    assert_eq!(calls.get(), 1);
    assert_eq!(app.get_result(), "hello@example.com");
    assert!(
        !app.get_submitting(),
        "a synchronous response finishes its ticket"
    );
}

#[test]
fn mixed_seventeen_control_form_reports_before_focus_and_skips_unfocusable_invalid_files() {
    use crate::{MixedFormField, MixedFormFixture};
    use slint::platform::{Key, WindowEvent};
    use slint::{ComponentHandle, Model};
    i_slint_backend_testing::init_no_event_loop();
    let app = MixedFormFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_file_access(slint_kit::ui::FieldAccess::ReadOnly);
    let form = FormBinding::new(&app, |_, _| {});
    let fields = [
        MixedFormField::Checkbox,
        MixedFormField::Switch,
        MixedFormField::Radio,
        MixedFormField::Segmented,
        MixedFormField::Toggle,
        MixedFormField::Multi,
        MixedFormField::Tags,
        MixedFormField::Combo,
        MixedFormField::Select,
        MixedFormField::Files,
        MixedFormField::Directory,
        MixedFormField::DateTime,
        MixedFormField::Zoned,
        MixedFormField::Stepper,
        MixedFormField::Date,
        MixedFormField::Time,
        MixedFormField::Color,
    ];
    // Register the invalid read-only file first, while preserving its real form identity.
    for (index, field) in std::iter::once((9, MixedFormField::Files)).chain(
        fields
            .into_iter()
            .enumerate()
            .filter(|(_, key)| *key != MixedFormField::Files),
    ) {
        form.register_view(
            field,
            move |app: &MixedFormFixture| app.get_fields().row_data(index).unwrap(),
            move |app| app.invoke_report_field(field),
            move |app| app.invoke_focus_field(field),
            |_| {},
            |_, _| {},
        )
        .unwrap();
    }
    form.set_focus_fallback(MixedFormFixture::invoke_focus_form);
    let validate = form.clone();
    app.on_validate(move || validate.report_validity());
    let submissions = Rc::new(Cell::new(0));
    let observed = submissions.clone();
    app.on_submitted(move || observed.set(observed.get() + 1));
    app.show().unwrap();
    app.invoke_focus_checkbox();
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Space.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Space.into(),
    });
    assert!(
        app.get_checkbox_checked(),
        "Space retains native choice activation"
    );
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Return.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Return.into(),
    });
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_invalid_count(),
        18,
        "all seventeen fields report once, including both zoned parts"
    );
    assert!(
        app.get_checkbox_focused(),
        "the first focusable invalid field follows the read-only file"
    );
    assert_eq!(submissions.get(), 0);
    assert!(
        app.get_checkbox_checked(),
        "form Return reports without toggling the checkbox"
    );
    app.set_access(slint_kit::ui::FieldAccess::Disabled);
    assert!(!form.report_validity());
    assert!(
        app.get_form_focused(),
        "the form receives focus when its only invalid field cannot"
    );
    assert_eq!(app.get_invalid_count(), 19);
    app.set_file_access(slint_kit::ui::FieldAccess::Disabled);
    assert!(form.report_validity());
    assert_eq!(app.get_invalid_count(), 19);
}

#[test]
fn four_field_registration_errors_and_actions_render_in_both_themes_at_double_text() {
    use crate::{FormBehaviorFixture, RegistrationField};
    use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
    use registration_example::schema::Registration;
    use slint::{ComponentHandle, Model};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    let form = Registration::bind(&app);
    app.set_name_text("".into());
    app.set_email_text("bad".into());
    app.set_birth_text("no".into());
    app.set_consent_state(slint_kit::ui::CheckState::Unchecked);
    app.show().unwrap();
    app.window().set_size(slint::LogicalSize::new(1000., 1800.));
    for (name, base) in [
        ("light", app.get_theme()),
        ("dark", app.global::<slint_kit::ui::KitThemes>().get_dark()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            assert!(!form.binding().report_validity());
            assert_eq!(app.get_errors().row_count(), 4);
            let first = app.get_name_field();
            assert!(form.binding().focus_target(&first.target));
            let frame = app.window().take_snapshot().unwrap();
            let action = ElementQuery::from_root(&app)
                .match_predicate(|e| e.accessible_label().as_deref() == Some("Register"))
                .find_first()
                .unwrap();
            assert!(action.absolute_position().y + action.size().height <= 1800.);
            crate::choice_presentation::save_capture(
                &frame,
                &format!("registration-form-{name}-{scale}"),
            );
        }
    }
    form.binding().field_edited(&RegistrationField::Name);
}
