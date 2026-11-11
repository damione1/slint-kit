// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{ChoiceValidityFixture, ValidationChoice};
use i_slint_backend_testing::{ElementQuery, TestingBackend, TestingBackendOptions};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_kit::ui::{ChipRow, FieldAccess, OptionRow, Ownership, SelectionRef, ValidationTiming};

const CONTROLS: [ValidationChoice; 9] = [
    ValidationChoice::Checkbox,
    ValidationChoice::Switch,
    ValidationChoice::Radio,
    ValidationChoice::Segmented,
    ValidationChoice::Toggle,
    ValidationChoice::Multi,
    ValidationChoice::Tags,
    ValidationChoice::Combo,
    ValidationChoice::Select,
];
fn fixture() -> ChoiceValidityFixture {
    let app = ChoiceValidityFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    outside(&app);
    app
}
fn focus(app: &ChoiceValidityFixture, control: ValidationChoice) {
    app.invoke_focus_choice(control);
    slint::platform::update_timers_and_animations();
}
fn outside(app: &ChoiceValidityFixture) {
    app.invoke_outside();
    slint::platform::update_timers_and_animations();
}
fn key(app: &ChoiceValidityFixture, key: Key) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    slint::platform::update_timers_and_animations();
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}
fn visible(app: &ChoiceValidityFixture, index: usize) -> bool {
    app.get_errors().row_data(index).unwrap()
}
fn edit(app: &ChoiceValidityFixture, control: ValidationChoice) {
    focus(app, control);
    match control {
        ValidationChoice::Checkbox | ValidationChoice::Switch => key(app, Key::Space),
        ValidationChoice::Radio | ValidationChoice::Segmented | ValidationChoice::Toggle => {
            key(app, Key::Home);
            key(app, Key::Space);
        }
        ValidationChoice::Multi | ValidationChoice::Select => {
            key(app, Key::DownArrow);
            key(app, Key::Return);
        }
        ValidationChoice::Tags | ValidationChoice::Combo => {
            app.window()
                .dispatch_event(WindowEvent::KeyPressed { text: "x".into() });
            app.window()
                .dispatch_event(WindowEvent::KeyReleased { text: "x".into() });
            slint::platform::update_timers_and_animations();
        }
    }
}
#[test]
fn all_nine_choices_share_pristine_blur_report_and_reset_timing() {
    i_slint_backend_testing::init_no_event_loop();
    for (index, control) in CONTROLS.into_iter().enumerate() {
        let app = fixture();
        assert!(!visible(&app, index), "{control:?}: pristine");
        assert!(!app.get_messages().row_data(index).unwrap().is_empty());
        assert!(!app.invoke_check_choice(control));
        assert!(!visible(&app, index), "checking is non-reporting");
        focus(&app, control);
        outside(&app);
        assert!(
            app.get_text_error(),
            "text fields expose the same constraint visibility alias"
        );
        assert!(visible(&app, index), "{control:?}: real focus departure");
        app.invoke_reset_choice(control);
        assert!(!visible(&app, index));
        app.set_timing(ValidationTiming::OnSubmit);
        focus(&app, control);
        outside(&app);
        assert!(!visible(&app, index), "{control:?}: submit suppresses blur");
        assert!(!app.invoke_report_choice(control));
        assert!(visible(&app, index));
        assert!(app.get_sentinel_focused(), "report preserves focus");
        app.set_access(FieldAccess::Disabled);
        assert!(!visible(&app, index));
        assert!(app.invoke_report_choice(control));
    }
}
#[test]
fn admitted_local_and_controlled_edits_show_errors_on_input_and_reset_clears_them() {
    i_slint_backend_testing::init_no_event_loop();
    for ownership in [Ownership::Local, Ownership::Controlled] {
        for (index, control) in CONTROLS.into_iter().enumerate() {
            let app = fixture();
            app.set_ownership(ownership);
            app.set_timing(ValidationTiming::OnInput);
            app.set_custom_error("Rejected".into());
            assert!(!visible(&app, index));
            edit(&app, control);
            assert!(
                visible(&app, index),
                "{ownership:?}/{control:?}: admitted edit"
            );
            app.invoke_reset_choice(control);
            assert!(!visible(&app, index));
            app.set_access(FieldAccess::ReadOnly);
            edit(&app, control);
            app.set_access(FieldAccess::Editable);
            assert!(!visible(&app, index), "read-only cannot dirty validation");
            app.set_access(FieldAccess::Disabled);
            edit(&app, control);
            app.set_access(FieldAccess::Editable);
            assert!(!visible(&app, index), "disabled cannot dirty validation");
        }
    }
}
#[test]
fn chips_keep_native_pointer_and_tab_focus_inside_the_tag_group() {
    i_slint_backend_testing::init_no_event_loop();
    let app = fixture();
    app.set_custom_error("Rejected".into());
    app.set_tag_chips(ModelRc::new(VecModel::from(vec![ChipRow {
        token: 42,
        label: "Existing tag".into(),
        removable: true,
    }])));
    focus(&app, ValidationChoice::Tags);
    // Shift+Tab reaches the preceding removable chip through native traversal.
    key(&app, Key::Backtab);
    assert!(app.get_tags_focused(), "chip focus belongs to tag group");
    assert!(!visible(&app, 6), "editor-to-chip is not a blur");
    key(&app, Key::Tab);
    assert!(app.get_tags_focused());
    assert!(!visible(&app, 6));
    let chip = ElementQuery::from_root(&app)
        .match_predicate(|element| {
            element.accessible_label().as_deref() == Some("Remove Existing tag")
        })
        .find_first()
        .unwrap();
    chip.mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert!(app.get_tags_focused(), "removal returns to native editor");
    assert!(!visible(&app, 6));
    outside(&app);
    assert!(visible(&app, 6));
}
#[test]
fn multi_chip_focus_and_removal_stay_inside_the_group() {
    i_slint_backend_testing::init_no_event_loop();
    let app = fixture();
    app.set_custom_error("Rejected".into());
    app.set_multi_selected(ModelRc::new(VecModel::from(vec![SelectionRef {
        present: true,
        token: 1,
    }])));
    focus(&app, ValidationChoice::Multi);
    key(&app, Key::Backtab);
    assert!(app.get_multi_focused(), "native Tab enters the chip");
    assert!(!visible(&app, 5));
    key(&app, Key::Return);
    assert!(app.get_multi_focused(), "removal returns to trigger");
    assert_eq!(app.get_multi_selected().row_count(), 0);
    assert!(!visible(&app, 5));
    outside(&app);
    assert!(visible(&app, 5));
}
#[test]
fn blocked_choice_actions_and_host_model_changes_do_not_dirty_on_input() {
    i_slint_backend_testing::init_no_event_loop();
    for (index, control) in CONTROLS.into_iter().enumerate().filter(|(_, control)| {
        !matches!(
            control,
            ValidationChoice::Checkbox | ValidationChoice::Switch | ValidationChoice::Tags
        )
    }) {
        let app = fixture();
        app.set_timing(ValidationTiming::OnInput);
        app.set_custom_error("Rejected".into());
        app.set_options(ModelRc::new(VecModel::from(vec![OptionRow {
            token: 8,
            generation: 2,
            label: "Blocked".into(),
            description: "".into(),
            disabled: true,
        }])));
        assert!(
            !visible(&app, index),
            "programmatic source replacement is not edit"
        );
        if control == ValidationChoice::Combo {
            // Inline typing is an admitted edit even when every suggestion is disabled.
            // Here exercise only the blocked selection action.
            focus(&app, control);
            key(&app, Key::DownArrow);
            key(&app, Key::Return);
        } else {
            edit(&app, control);
        }
        assert!(
            !visible(&app, index),
            "{control:?}: blocked activation is not edit"
        );
        assert_eq!(app.get_proposals(), 0);
    }
}
#[test]
fn popup_focus_is_internal_until_native_focus_leaves_the_selector() {
    i_slint_backend_testing::init_no_event_loop();
    for (index, control) in [
        (5, ValidationChoice::Multi),
        (7, ValidationChoice::Combo),
        (8, ValidationChoice::Select),
    ] {
        let app = fixture();
        focus(&app, control);
        key(&app, Key::DownArrow);
        assert!(!visible(&app, index), "opening is not blur");
        let focused = match control {
            ValidationChoice::Multi => app.get_multi_focused(),
            ValidationChoice::Combo => app.get_combo_focused(),
            ValidationChoice::Select => app.get_select_focused(),
            ValidationChoice::Checkbox
            | ValidationChoice::Switch
            | ValidationChoice::Radio
            | ValidationChoice::Segmented
            | ValidationChoice::Toggle
            | ValidationChoice::Tags => unreachable!(),
        };
        assert!(
            focused,
            "{control:?}: real trigger/list/search focus remains in group (multi={}, combo={}, select={})",
            app.get_multi_open(),
            app.get_combo_open(),
            app.get_select_open()
        );
        key(&app, Key::Escape);
        assert!(!visible(&app, index), "closing restores internal focus");
        outside(&app);
        assert!(visible(&app, index));
    }
}
#[test]
fn coordinated_form_hosts_the_choice_blur_error_and_keeps_server_message_precedence() {
    use crate::FormBehaviorFixture;
    use slint_kit::ui::CheckState;

    i_slint_backend_testing::init_no_event_loop();
    let app = FormBehaviorFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_consent_state(CheckState::Unchecked);
    app.show().unwrap();
    slint::platform::update_timers_and_animations();
    let message = app.get_consent_field().message.to_string();
    assert!(!message.is_empty());
    let shown = |text: &str| {
        let text = text.to_owned();
        ElementQuery::from_root(&app)
            .match_predicate(move |element| {
                element.accessible_label().as_deref() == Some(text.as_str())
            })
            .find_first()
            .is_some()
    };
    assert!(
        !shown(&message),
        "a pristine constraint has no visible host message"
    );
    app.invoke_focus_consent();
    slint::platform::update_timers_and_animations();
    app.invoke_focus_email();
    slint::platform::update_timers_and_animations();
    assert!(shown(&message), "real choice blur reveals its host message");
    app.set_consent_error("Consent rejected by server".into());
    slint::platform::update_timers_and_animations();
    assert!(shown("Consent rejected by server"));
    assert!(
        !shown(&message),
        "the host renders one message with explicit precedence"
    );
}

#[test]
fn host_owned_choice_errors_render_in_light_dark_and_double_text() {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = fixture();
    app.set_custom_error("Choose an available value".into());
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            for control in CONTROLS {
                assert!(!app.invoke_report_choice(control));
            }
            assert!(app.get_errors().iter().all(|shown| shown));
            let frame = app.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &frame,
                &format!("choice-validity-{name}-{scale}"),
            );
        }
    }
}
