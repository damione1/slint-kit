// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    FieldActionsFixture,
    ui::{
        FieldAccess, SecretToggleVisibility, SecretVisibility, TextPurpose, ValidationPresentation,
    },
};
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::{
    ComponentHandle,
    platform::{Key, PointerEventButton, WindowEvent},
};

fn fixture() -> FieldActionsFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = FieldActionsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.invoke_focus_field();
    app
}
fn key(app: &FieldActionsFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn button(app: &FieldActionsFixture, label: &str) -> Option<ElementHandle> {
    let requested = label.to_owned();
    ElementQuery::from_root(app)
        .match_predicate(move |e| {
            e.accessible_role() == Some(AccessibleRole::Button)
                && e.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
}
fn click(app: &FieldActionsFixture, label: &str) {
    button(app, label)
        .unwrap_or_else(|| panic!("missing action {label}"))
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
}
fn editor(app: &FieldActionsFixture) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, "Access token")
        .find(|e| e.accessible_role() == Some(AccessibleRole::TextInput))
        .unwrap()
}
#[test]
fn interactive_adornments_keep_native_editor_focus_selection_and_commit_policy() {
    let app = fixture();
    app.invoke_selection(5, 5);
    key(&app, "X");
    assert_eq!(app.get_text(), "alphaX");
    assert_eq!(app.get_commit_count(), 0);
    click(&app, "Unit selector");
    assert_eq!(app.get_unit_count(), 1);
    assert!(app.get_editor_focus());
    assert_eq!(app.get_commit_count(), 0);
    click(&app, "Copy value");
    assert_eq!(app.get_copied_count(), 1);
    assert_eq!(app.invoke_read_clipboard(), "alphaX");
    assert!(app.get_editor_focus());
    assert_eq!(app.get_commit_count(), 0);
    assert_eq!((app.get_anchor(), app.get_cursor()), (0, 6));
}
#[test]
fn a_unit_selector_can_open_its_popup_inside_the_surface_without_replacing_the_editor() {
    let app = fixture();
    app.invoke_selection(5, 5);
    app.set_show_selector(true);
    slint::platform::update_timers_and_animations();
    ElementQuery::from_root(&app)
        .match_inherits("KitSelectTrigger")
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    ElementQuery::from_root(&app)
        .match_inherits("KitOption")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("EUR"))
        .find_first()
        .expect("unit popup option must mount")
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_unit_count(), 1);
    assert_eq!(app.get_text(), "alpha");
    app.invoke_focus_field();
    key(&app, "X");
    assert_eq!(app.get_text(), "alphaX");
}
#[test]
fn clear_action_is_configurable_and_undoable_and_emits_only_after_a_real_clear() {
    let app = fixture();
    click(&app, "Clear text");
    assert_eq!(app.get_text(), "");
    assert_eq!(app.get_cleared_count(), 1);
    assert!(app.get_editor_focus());
    app.invoke_clear();
    assert_eq!(app.get_cleared_count(), 1);
    app.invoke_undo();
    assert_eq!(app.get_text(), "alpha");
    assert_eq!(app.get_cleared_count(), 1);
    app.set_clearable(false);
    assert!(button(&app, "Clear text").is_none());
    app.set_access(FieldAccess::ReadOnly);
    app.invoke_clear();
    assert_eq!(app.get_text(), "alpha");
    assert_eq!(app.get_cleared_count(), 1);
    app.set_clearable(true);
    click(&app, "Clear text");
    assert_eq!(app.get_text(), "alpha");
    assert_eq!(app.get_cleared_count(), 1);
    app.set_access(FieldAccess::Disabled);
    click(&app, "Clear text");
    assert_eq!(app.get_text(), "alpha");
    assert_eq!(app.get_cleared_count(), 1);
}
#[test]
fn every_single_line_purpose_can_offer_the_same_native_clear_action() {
    let app = fixture();
    for (purpose, text, label) in [
        (TextPurpose::Plain, "note", "Clear text"),
        (TextPurpose::Email, "person@example.com", "Clear text"),
        (TextPurpose::Url, "https://example.com", "Clear text"),
        (TextPurpose::Telephone, "+1 416 555 0100", "Clear text"),
        (TextPurpose::Search, "project", "Clear search"),
        (TextPurpose::Password, "secret", "Clear text"),
    ] {
        app.set_purpose(purpose);
        app.set_text(text.into());
        let count = app.get_cleared_count();
        click(&app, label);
        assert_eq!(app.get_text(), "");
        assert_eq!(app.get_cleared_count(), count + 1);
        app.invoke_undo();
        assert_eq!(app.get_text(), text);
    }
}
#[test]
fn password_reveal_policy_and_observation_preserve_native_selection() {
    let app = fixture();
    app.set_purpose(TextPurpose::Password);
    app.invoke_selection(1, 4);
    let selected = (app.get_anchor(), app.get_cursor());
    click(&app, "Show password");
    assert_eq!(app.get_secret(), SecretVisibility::Revealed);
    assert_eq!(app.get_reveal_count(), 1);
    assert_eq!(app.get_text(), "alpha");
    assert_eq!((app.get_anchor(), app.get_cursor()), selected);
    assert!(app.get_editor_focus());
    click(&app, "Hide password");
    assert_eq!(app.get_secret(), SecretVisibility::Hidden);
    assert_eq!(app.get_reveal_count(), 2);
    app.set_reveal_toggle(SecretToggleVisibility::Hidden);
    assert!(button(&app, "Show password").is_none());
    app.set_secret(SecretVisibility::Revealed);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        app.get_reveal_count(),
        3,
        "host transitions are observable without the toggle"
    );
    assert!(button(&app, "Hide password").is_none());
    app.set_secret(SecretVisibility::Hidden);
    slint::platform::update_timers_and_animations();
    assert_eq!(app.get_reveal_count(), 4);
    app.set_reveal_toggle(SecretToggleVisibility::Shown);
    app.set_access(FieldAccess::ReadOnly);
    click(&app, "Show password");
    assert_eq!(app.get_secret(), SecretVisibility::Revealed);
    assert_eq!(app.get_text(), "alpha");
    app.set_access(FieldAccess::Disabled);
    let changes = app.get_reveal_count();
    click(&app, "Hide password");
    assert_eq!(app.get_reveal_count(), changes);
    assert_eq!(app.get_secret(), SecretVisibility::Revealed);
}
#[test]
fn adornments_keep_native_declaration_order_for_keyboard_navigation() {
    let app = fixture();
    app.set_clearable(false);
    app.set_reveal_toggle(SecretToggleVisibility::Hidden);
    key(&app, Key::Tab);
    assert!(app.get_unit_focus());
    key(&app, Key::Tab);
    assert!(app.get_copy_focus());
    key(&app, Key::Tab);
    assert!(app.get_other_focus());
}
#[test]
fn built_in_actions_remain_keyboard_operable_before_custom_children() {
    let app = fixture();
    app.set_purpose(TextPurpose::Password);
    key(&app, Key::Tab);
    key(&app, " ");
    assert_eq!(app.get_text(), "");
    assert_eq!(app.get_cleared_count(), 1);
    assert!(app.get_editor_focus());
    app.invoke_undo();
    assert_eq!(app.get_text(), "alpha");
    key(&app, Key::Tab);
    key(&app, Key::Tab);
    key(&app, " ");
    assert_eq!(app.get_secret(), SecretVisibility::Revealed);
    assert_eq!(app.get_reveal_count(), 1);
    assert!(app.get_editor_focus());
    assert_eq!(app.get_unit_count(), 0);
}
#[test]
fn busy_is_separate_from_validation_and_keeps_both_accessible_statuses() {
    let app = fixture();
    app.set_busy(true);
    assert!(!app.get_invalid());
    key(&app, "Z");
    assert!(app.get_text().contains('Z'));
    assert!(
        editor(&app)
            .accessible_description()
            .unwrap()
            .contains("Working")
    );
    app.set_validation(ValidationPresentation::Checking);
    let description = editor(&app).accessible_description().unwrap();
    assert!(description.contains("Working"));
    assert!(description.contains("Checking"));
    app.set_busy(false);
    assert!(
        editor(&app)
            .accessible_description()
            .unwrap()
            .contains("Checking")
    );
}
#[test]
fn field_actions_and_busy_invalid_states_render_with_intrinsic_adornment_geometry() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = FieldActionsFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app.set_busy(true);
    app.set_purpose(TextPurpose::Password);
    app.set_validation(ValidationPresentation::Invalid);
    app.invoke_focus_field();
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            let frame = app.window().take_snapshot().unwrap();
            let native = editor(&app);
            let leading = button(&app, "Unit selector").unwrap();
            let trailing = button(&app, "Copy value").unwrap();
            assert!(
                leading.absolute_position().x + leading.size().width
                    <= native.absolute_position().x
            );
            assert!(
                trailing.absolute_position().x
                    >= native.absolute_position().x + native.size().width
            );
            assert!(native.size().width > 0.);
            let surface = ElementQuery::from_root(&app)
                .match_inherits("KitInputGroup")
                .find_first()
                .unwrap();
            for action in [&leading, &trailing] {
                assert!(action.absolute_position().y >= surface.absolute_position().y);
                assert!(
                    action.absolute_position().y + action.size().height
                        <= surface.absolute_position().y + surface.size().height
                );
            }
            crate::choice_presentation::save_capture(
                &frame,
                &format!("field-actions-{name}-{scale}"),
            );
        }
    }
}
