// SPDX-License-Identifier: GPL-3.0-or-later
use crate::*;
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use std::{cell::RefCell, rc::Rc};
fn fixture() -> TextFieldBehaviorFixture {
    i_slint_backend_testing::init_no_event_loop();
    let app = TextFieldBehaviorFixture::new().unwrap();
    kit_slint::install(&app);
    app.show().unwrap();
    app.invoke_focus_field();
    app
}
fn key(app: &TextFieldBehaviorFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
#[test]
fn change_policy_commits_the_final_native_edit_and_ignores_navigation() {
    let app = fixture();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    app.on_committed(move |commit| observed.borrow_mut().push(commit.value.text.to_string()));
    for text in ["a", "b", "c"] {
        key(&app, text);
    }
    assert_eq!(*commits.borrow(), ["a", "ab", "abc"]);
    key(&app, Key::LeftArrow);
    key(&app, Key::RightArrow);
    assert_eq!(commits.borrow().len(), 3);
}
#[test]
fn untouched_blur_does_not_commit_or_report_and_submit_timing_stays_hidden() {
    let app = fixture();
    app.set_policy(CommitPolicy::OnBlur);
    app.set_required(true);
    let commits = Rc::new(RefCell::new(0));
    let observed = commits.clone();
    app.on_committed(move |_| *observed.borrow_mut() += 1);
    let invalid = Rc::new(RefCell::new(0));
    let observed = invalid.clone();
    app.on_invalid(move || *observed.borrow_mut() += 1);
    app.invoke_blur_field();
    slint::platform::update_timers_and_animations();
    assert_eq!(*commits.borrow(), 0);
    assert_eq!(*invalid.borrow(), 0);
    assert!(!app.get_error_visible());
    app.invoke_focus_field();
    key(&app, "x");
    app.invoke_blur_field();
    slint::platform::update_timers_and_animations();
    assert_eq!(*commits.borrow(), 1);
    app.invoke_focus_field();
    app.invoke_blur_field();
    slint::platform::update_timers_and_animations();
    assert_eq!(*commits.borrow(), 1);
}
#[test]
fn untouched_programmatic_short_text_respects_html_dirty_state() {
    let app = fixture();
    app.set_policy(CommitPolicy::OnBlur);
    app.set_minimum(5);
    app.set_text("a".into());
    app.invoke_blur_field();
    slint::platform::update_timers_and_animations();
    assert!(!app.get_error_visible());
    app.set_text("valid value".into());
    slint::platform::update_timers_and_animations();
    assert!(!app.get_error_visible());
}
#[test]
fn reporting_invalid_fields_does_not_move_focus() {
    let app = fixture();
    app.set_required(true);
    app.invoke_blur_field();
    slint::platform::update_timers_and_animations();
    assert!(!app.invoke_report());
    assert!(!app.get_has_focus());
    assert!(app.get_error_visible());
    assert!(app.invoke_focus_invalid());
    assert!(app.get_has_focus());
}
#[test]
fn overflowing_multiline_editor_preserves_mouse_selection_and_has_scrollbar() {
    use i_slint_backend_testing::{
        ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
    };
    use slint::platform::PointerEventButton;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = TextFieldBehaviorFixture::new().unwrap();
    kit_slint::install(&app);
    app.show().unwrap();
    app.invoke_focus_field();
    app.set_multiline(true);
    app.set_text(
        (0..20)
            .map(|_| "A selectable overflowing line\n")
            .collect::<String>()
            .into(),
    );
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.invoke_focus_field();
            app.invoke_selection(5, 5);
            slint::platform::update_timers_and_animations();
            // Render the actual glyph layout before pointer hit-testing.
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("textarea-before-{name}-{scale}"),
            );
            let editor = ElementHandle::find_by_accessible_label(&app, "Name")
                .find(|element| {
                    element.accessible_role()
                        == Some(i_slint_backend_testing::AccessibleRole::TextInput)
                })
                .unwrap();
            let bounds = editor.absolute_position();
            let start = slint::LogicalPosition::new(bounds.x + 4., bounds.y + 8.);
            let end = slint::LogicalPosition::new(bounds.x + 140., bounds.y + 30.);
            app.window()
                .dispatch_event(WindowEvent::PointerMoved { position: start });
            app.window().dispatch_event(WindowEvent::PointerPressed {
                position: start,
                button: PointerEventButton::Left,
            });
            app.window()
                .dispatch_event(WindowEvent::PointerMoved { position: end });
            app.window().dispatch_event(WindowEvent::PointerReleased {
                position: end,
                button: PointerEventButton::Left,
            });
            assert_ne!(
                app.get_cursor(),
                app.get_anchor(),
                "mouse drag selects a native text range ({name}/{scale})"
            );
            assert_eq!(
                editor.absolute_position().y,
                bounds.y,
                "mouse drag must not pan the text viewport ({name}/{scale})"
            );
            let scrollbar = ElementQuery::from_root(&app)
                .match_inherits("KitScrollBar")
                .find_first()
                .unwrap();
            assert!(scrollbar.size().width > 0.);
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("textarea-drag-{name}-{scale}"),
            );
            assert!(
                scrollbar.absolute_position().x >= bounds.x + editor.size().width,
                "the scrollbar stays in its gutter ({name}/{scale}): bar={:?}/{:?}, editor={bounds:?}/{:?}",
                scrollbar.absolute_position(),
                scrollbar.size(),
                editor.size()
            );
        }
    }
}
#[test]
fn selection_insertion_and_history_forward_to_the_same_native_editor() {
    let app = fixture();
    key(&app, "a😀b");
    app.invoke_selection(1, 5);
    assert_eq!(app.get_anchor(), 1);
    assert_eq!(app.get_cursor(), 5);
    app.invoke_insert("XY".into());
    assert_eq!(app.get_text(), "aXYb");
    app.invoke_undo();
    app.invoke_undo();
    assert_eq!(app.get_text(), "a😀b");
    app.invoke_redo();
    app.invoke_redo();
    assert_eq!(app.get_text(), "aXYb");
}
#[test]
fn native_clear_preserves_undo_and_dispatches_one_final_change() {
    let app = fixture();
    key(&app, "query");
    let changes = Rc::new(RefCell::new(Vec::new()));
    let observed = changes.clone();
    app.on_committed(move |commit| observed.borrow_mut().push(commit.value.text.to_string()));
    app.invoke_clear();
    assert_eq!(app.get_text(), "");
    assert_eq!(*changes.borrow(), [""]);
    app.invoke_undo();
    assert_eq!(app.get_text(), "query");
}
#[test]
fn window_strings_localize_constraints_and_typed_codec_failures() {
    let app = fixture();
    let strings = app.global::<kit_slint::ui::KitStrings>();
    strings.set_required_message("Obligatoire".into());
    strings.set_incomplete("Terminez la saisie".into());
    strings.set_invalid_syntax("Format incorrect".into());
    app.set_required(true);
    assert!(!app.invoke_report());
    assert_eq!(app.get_validation_message(), "Obligatoire");
    app.set_required(false);
    let spec = slint_kit::FieldSpec::codec(
        |text| {
            if text == "?" {
                slint_kit::DraftValue::Incomplete
            } else {
                slint_kit::DraftValue::Invalid(slint_kit::ParseIssue::InvalidSyntax)
            }
        },
        |text: &String| text.clone(),
    );
    kit_slint::install_field_specs(
        &app,
        [spec.named(slint_kit::FieldSpecKey::new("code").unwrap())],
    )
    .unwrap();
    app.set_spec_key("code".into());
    key(&app, "?");
    key(&app, Key::Return);
    assert_eq!(app.get_validation_message(), "Terminez la saisie");
    app.set_text("corrected".into());
    slint::platform::update_timers_and_animations();
    assert!(
        app.get_validation_message().is_empty(),
        "host correction drops stale codec failure"
    );
}
#[test]
fn escape_policy_and_keyboard_hook_run_before_native_editing() {
    let app = fixture();
    app.set_escape_behavior(TextEscapeBehavior::Clear);
    key(&app, "query");
    key(&app, Key::Escape);
    assert_eq!(app.get_text(), "");
    app.set_intercept_key(true);
    key(&app, "ignored");
    assert_eq!(app.get_text(), "");
    app.set_intercept_key(false);
    app.set_escape_behavior(TextEscapeBehavior::Cancel);
    let cancelled = Rc::new(RefCell::new(0));
    let observed = cancelled.clone();
    app.on_cancelled(move || *observed.borrow_mut() += 1);
    key(&app, Key::Escape);
    assert_eq!(*cancelled.borrow(), 1);
}
#[test]
fn character_count_defaults_to_graphemes_without_host_wiring() {
    let app = fixture();
    key(&app, "a👩‍👩‍👧‍👦e\u{301}");
    assert_eq!(app.get_count(), 3);
}
#[test]
fn accessible_hint_keeps_help_and_required_text_when_invalid() {
    use i_slint_backend_testing::ElementHandle;
    let app = fixture();
    app.set_required(true);
    app.set_description("Use your legal name".into());
    app.set_placeholder("Ada".into());
    app.invoke_report();
    let editor = ElementHandle::find_by_accessible_label(&app, "Name")
        .find(|element| {
            element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
        })
        .unwrap();
    assert_eq!(editor.accessible_placeholder_text().as_deref(), Some("Ada"));
    let description = editor.accessible_description().unwrap();
    assert!(description.contains("Use your legal name"));
    assert!(description.contains("Required"));
    assert!(description.contains("Please fill out this field."));
}

#[test]
fn public_kit_key_result_can_be_returned_from_a_rust_text_field_hook() {
    let app = fixture();
    app.on_handle_key(|event| {
        if event.text == "x" {
            slint_kit::ui::KitKeyResult::Accept
        } else {
            slint_kit::ui::KitKeyResult::Reject
        }
    });
    key(&app, "x");
    assert_eq!(app.get_text().as_str(), "");
    key(&app, "y");
    assert_eq!(app.get_text().as_str(), "y");
}

fn menu_item(
    app: &TextFieldBehaviorFixture,
    label: &str,
) -> i_slint_backend_testing::ElementHandle {
    let requested = label.to_owned();
    i_slint_backend_testing::ElementQuery::from_root(app)
        .match_inherits("KitOption")
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing text command {label}"))
}
fn menu_visible(app: &TextFieldBehaviorFixture) -> bool {
    i_slint_backend_testing::ElementQuery::from_root(app)
        .match_inherits("TextEditMenu")
        .find_first()
        .is_some()
}
fn right_click_editor(app: &TextFieldBehaviorFixture) {
    use i_slint_backend_testing::{AccessibleRole, ElementHandle};
    let editor = ElementHandle::find_by_accessible_label(app, "Name")
        .find(|element| element.accessible_role() == Some(AccessibleRole::TextInput))
        .unwrap();
    editor.mock_single_click(slint::platform::PointerEventButton::Right);
    slint::platform::update_timers_and_animations();
}
#[test]
fn editing_menu_uses_native_commands_and_preserves_selection_focus_and_history() {
    use slint::platform::PointerEventButton;
    let app = fixture();
    app.set_policy(CommitPolicy::OnBlur);
    key(&app, "a😀b");
    app.invoke_selection(1, 5);
    let commits = Rc::new(RefCell::new(0));
    let output = commits.clone();
    app.on_committed(move |_| *output.borrow_mut() += 1);
    right_click_editor(&app);
    assert!(menu_visible(&app));
    assert!(
        app.get_has_focus(),
        "the owned menu retains logical field focus"
    );
    assert_eq!((app.get_anchor(), app.get_cursor()), (1, 5));
    assert_eq!(*commits.borrow(), 0, "opening a menu is not a field blur");
    menu_item(&app, "Copy").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.invoke_read_clipboard(), "😀");
    assert_eq!(app.get_text(), "a😀b");
    assert!(!menu_visible(&app));
    assert!(app.get_has_focus());
    right_click_editor(&app);
    menu_item(&app, "Cut").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_text(), "ab");
    key(&app, Key::Menu);
    menu_item(&app, "Undo").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_text(), "a😀b");
    key(&app, Key::Menu);
    menu_item(&app, "Redo").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_text(), "ab");
    app.invoke_selection(1, 1);
    app.set_clipboard_text("XY".into());
    app.invoke_prepare_paste();
    key(&app, Key::Menu);
    menu_item(&app, "Paste").mock_single_click(PointerEventButton::Left);
    assert_eq!(app.get_text(), "aXYb");
    key(&app, Key::Menu);
    key(&app, Key::End);
    key(&app, Key::Return);
    assert_eq!((app.get_anchor(), app.get_cursor()), (0, 4));
    assert_eq!(*commits.borrow(), 0);
}
#[test]
fn editing_menu_keyboard_dismissal_disabled_readonly_and_password_are_guarded() {
    use slint::platform::{Key, PointerEventButton};
    let app = fixture();
    app.set_text("secret".into());
    app.invoke_select_all();
    app.set_access(FieldAccess::ReadOnly);
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(&app, Key::F10);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    assert!(menu_visible(&app));
    for label in ["Undo", "Redo", "Cut", "Paste"] {
        let item = menu_item(&app, label);
        assert_eq!(item.accessible_enabled(), Some(false), "{label}");
        item.invoke_accessible_default_action();
        assert_eq!(app.get_text(), "secret");
    }
    assert_eq!(menu_item(&app, "Copy").accessible_enabled(), Some(true));
    key(&app, Key::Home);
    key(&app, Key::Return);
    assert_eq!(
        app.invoke_read_clipboard(),
        "secret",
        "keyboard skips disabled edits and copies read-only selection"
    );
    key(&app, Key::Menu);
    key(&app, Key::Escape);
    assert!(!menu_visible(&app));
    assert!(app.get_has_focus());
    app.set_access(FieldAccess::Editable);
    app.set_purpose(TextPurpose::Password);
    app.set_clipboard_text("sentinel".into());
    app.invoke_prepare_paste();
    app.invoke_select_all();
    key(&app, Key::Menu);
    assert_eq!(menu_item(&app, "Cut").accessible_enabled(), Some(false));
    assert_eq!(menu_item(&app, "Copy").accessible_enabled(), Some(false));
    key(&app, Key::Escape);
    app.invoke_copy();
    app.invoke_cut();
    assert_eq!(app.get_text(), "secret");
    assert_eq!(app.invoke_read_clipboard(), "sentinel");
    app.set_intercept_key(true);
    key(&app, Key::Menu);
    assert!(
        !menu_visible(&app),
        "the public keyboard hook keeps priority"
    );
    app.set_intercept_key(false);
    key(&app, Key::Menu);
    assert!(menu_visible(&app));
    app.set_access(FieldAccess::ReadOnly);
    slint::platform::update_timers_and_animations();
    assert!(
        !menu_visible(&app),
        "access changes discard stale menu availability"
    );
    assert!(app.get_has_focus());
    app.set_access(FieldAccess::Disabled);
    right_click_editor(&app);
    assert!(!menu_visible(&app));
    app.set_access(FieldAccess::Editable);
    app.set_context_menu(TextContextMenu::Disabled);
    right_click_editor(&app);
    assert!(!menu_visible(&app));
    app.set_context_menu(TextContextMenu::Standard);
    app.invoke_focus_field();
    key(&app, Key::Menu);
    assert!(menu_visible(&app));
    key(&app, Key::Tab);
    assert!(!menu_visible(&app));
    assert!(
        !app.get_has_focus(),
        "Tab continues normal focus navigation"
    );
    app.invoke_focus_field();
    key(&app, Key::Menu);
    app.invoke_release_field();
    slint::platform::update_timers_and_animations();
    assert!(!menu_visible(&app));
    assert!(!app.get_has_focus());
    // Slint restores parent focus on the dismissal press; the next press reaches the outside target.
    app.invoke_focus_field();
    key(&app, Key::Menu);
    i_slint_backend_testing::ElementQuery::from_root(&app)
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Other"))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert!(!menu_visible(&app));
    assert!(
        app.get_has_focus(),
        "Slint popup dismissal restores parent focus"
    );
    i_slint_backend_testing::ElementQuery::from_root(&app)
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Other"))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert!(
        !app.get_has_focus(),
        "a subsequent outside click keeps its own focus"
    );
}
#[test]
fn menu_and_public_paste_keep_phone_and_numeric_filters() {
    use slint::platform::PointerEventButton;
    let app = fixture();
    for (purpose, input_type, original, rejected, accepted) in [
        (TextPurpose::Telephone, false, "514", "Ada", " (555)-0123"),
        (TextPurpose::Plain, true, "12", "NaN", ".5"),
    ] {
        app.set_purpose(purpose);
        app.set_decimal_input(input_type);
        app.set_text(original.into());
        app.invoke_selection(
            original.len().try_into().unwrap(),
            original.len().try_into().unwrap(),
        );
        app.set_clipboard_text(rejected.into());
        app.invoke_prepare_paste();
        app.invoke_paste();
        assert_eq!(app.get_text(), original);
        key(&app, Key::Menu);
        menu_item(&app, "Paste").mock_single_click(PointerEventButton::Left);
        assert_eq!(app.get_text(), original);
        app.set_clipboard_text(accepted.into());
        app.invoke_prepare_paste();
        app.invoke_paste();
        assert_eq!(app.get_text(), format!("{original}{accepted}"));
    }
}
#[test]
fn editing_menu_renders_in_both_themes_at_double_text_scale() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = TextFieldBehaviorFixture::new().unwrap();
    kit_slint::install(&app);
    app.show().unwrap();
    app.set_text("Selectable native text".into());
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            app.invoke_focus_field();
            app.invoke_select_all();
            key(&app, Key::Menu);
            assert!(menu_visible(&app));
            let last = menu_item(&app, "Select all");
            let origin = last.absolute_position();
            let size = last.size();
            assert!(origin.x >= 0. && origin.y >= 0.);
            assert!(origin.x + size.width <= 600. && origin.y + size.height <= 400.);
            let snapshot = app.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &snapshot,
                &format!("text-menu-{name}-{scale}"),
            );
            key(&app, Key::Escape);
            for (state, access, purpose) in [
                ("readonly", FieldAccess::ReadOnly, TextPurpose::Plain),
                ("password", FieldAccess::Editable, TextPurpose::Password),
            ] {
                app.set_access(access);
                app.set_purpose(purpose);
                app.invoke_focus_field();
                app.invoke_select_all();
                key(&app, Key::Menu);
                assert_eq!(menu_item(&app, "Cut").accessible_enabled(), Some(false));
                crate::choice_presentation::save_capture(
                    &app.window().take_snapshot().unwrap(),
                    &format!("text-menu-{state}-{name}-{scale}"),
                );
                key(&app, Key::Escape);
            }
            app.set_access(FieldAccess::Editable);
            app.set_purpose(TextPurpose::Plain);
            app.invoke_focus_field();
            app.invoke_select_all();
            key(&app, Key::Menu);
            let copy = menu_item(&app, "Copy");
            let mut click = Box::pin(copy.single_click(slint::platform::PointerEventButton::Left));
            let mut context = std::task::Context::from_waker(std::task::Waker::noop());
            assert!(click.as_mut().poll(&mut context).is_pending());
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("text-menu-pressed-{name}-{scale}"),
            );
            i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(50));
            assert!(click.as_mut().poll(&mut context).is_ready());
            assert!(!menu_visible(&app));
        }
    }
}

#[test]
fn native_text_pointer_reference_preserves_selection_through_plain_wrappers() {
    use i_slint_backend_testing::{
        AccessibleRole, ElementHandle, TestingBackend, TestingBackendOptions,
    };
    use slint::platform::PointerEventButton;
    #[derive(Clone, Copy, Debug)]
    enum Probe {
        Direct,
        Wrapped,
        Scrolled,
    }
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = NativeTextHitFixture::new().unwrap();
    app.set_text(
        (0..20)
            .map(|_| "A selectable overflowing line\n")
            .collect::<String>()
            .into(),
    );
    app.show().unwrap();
    let snapshot = app.window().take_snapshot().unwrap();
    crate::choice_presentation::save_capture(&snapshot, "native-text-pointer-reference");
    let mut results = Vec::new();
    for (probe, label) in [
        (Probe::Direct, "Direct"),
        (Probe::Wrapped, "Wrapped"),
        (Probe::Scrolled, "Scrolled"),
    ] {
        let editor = ElementHandle::find_by_accessible_label(&app, label)
            .find(|e| e.accessible_role() == Some(AccessibleRole::TextInput))
            .unwrap();
        let origin = editor.absolute_position();
        let start = slint::LogicalPosition::new(origin.x + 4., origin.y + 8.);
        let end = slint::LogicalPosition::new(origin.x + 140., origin.y + 30.);
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position: start });
        app.window().dispatch_event(WindowEvent::PointerPressed {
            position: start,
            button: PointerEventButton::Left,
        });
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position: end });
        app.window().dispatch_event(WindowEvent::PointerReleased {
            position: end,
            button: PointerEventButton::Left,
        });
        let offsets = match probe {
            Probe::Direct => (app.get_direct_cursor(), app.get_direct_anchor()),
            Probe::Wrapped => (app.get_wrapped_cursor(), app.get_wrapped_anchor()),
            Probe::Scrolled => (app.get_scrolled_cursor(), app.get_scrolled_anchor()),
        };
        results.push((probe, offsets));
    }
    for (probe, (cursor, anchor)) in results {
        assert_ne!(cursor, anchor, "native {probe:?} pointer reference");
    }
}
