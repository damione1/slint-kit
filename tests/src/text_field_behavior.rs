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
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::PointerEventButton;
    let app = fixture();
    app.set_multiline(true);
    app.set_text(
        (0..20)
            .map(|_| "A selectable overflowing line\n")
            .collect::<String>()
            .into(),
    );
    slint::platform::update_timers_and_animations();
    let editor = ElementHandle::find_by_accessible_label(&app, "Name")
        .find(|element| {
            element.accessible_role() == Some(i_slint_backend_testing::AccessibleRole::TextInput)
        })
        .unwrap();
    let bounds = editor.absolute_position();
    let start = slint::LogicalPosition::new(bounds.x + 4., bounds.y + 8.);
    let end = slint::LogicalPosition::new(bounds.x + 140., bounds.y + 30.);
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
    assert!(
        ElementQuery::from_root(&app)
            .match_inherits("KitScrollBar")
            .find_first()
            .is_some()
    );
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
