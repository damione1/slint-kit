// SPDX-License-Identifier: GPL-3.0-or-later
use crate::ShortcutFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, SharedString};
use slint_kit::ui::FieldAccess;

fn settle() {
    slint::platform::update_timers_and_animations();
}

fn fixture() -> ShortcutFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = ShortcutFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle();
    app.window().take_snapshot().unwrap();
    app
}

fn action(app: &ShortcutFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap()
}

fn click(app: &ShortcutFixture, label: &str) {
    let item = action(app, label);
    let p = item.absolute_position();
    let size = item.size();
    let position = LogicalPosition::new(p.x + size.width / 2., p.y + size.height / 2.);
    for event in [
        WindowEvent::PointerMoved { position },
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        app.window().dispatch_event(event);
    }
    settle();
}

fn down(app: &ShortcutFixture, text: impl Into<SharedString>) {
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
}

fn up(app: &ShortcutFixture, text: impl Into<SharedString>) {
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}

fn stroke(app: &ShortcutFixture, text: impl Into<SharedString>) {
    let text = text.into();
    down(app, text.clone());
    up(app, text);
    settle();
}

#[test]
fn logical_chords_modifiers_and_separate_clear_preserve_the_controlled_value() {
    let app = fixture();
    stroke(&app, Key::Tab);
    assert!(app.get_focused());
    down(&app, Key::Return);
    app.window().dispatch_event(WindowEvent::KeyPressRepeated {
        text: Key::Return.into(),
    });
    up(&app, Key::Return);
    assert!(app.get_recording());
    assert_eq!(
        action(&app, "First").accessible_value().as_deref(),
        Some("Press a key…")
    );
    assert_eq!(app.get_presses(), 0);

    for (index, logical) in [
        SharedString::from("é"),
        Key::Delete.into(),
        Key::Backspace.into(),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            click(&app, "First");
        }
        down(&app, Key::Shift);
        down(&app, logical.clone());
        app.window().dispatch_event(WindowEvent::KeyPressRepeated {
            text: logical.clone(),
        });
        up(&app, logical.clone());
        up(&app, Key::Shift);
        let event = app.get_event();
        assert_eq!(event.text, logical);
        assert!(event.modifiers.shift && !event.repeat);
        assert_eq!(app.get_presses(), i32::try_from(index + 1).unwrap());
        assert_eq!((app.get_clears(), app.get_releases()), (0, 0));
        assert_eq!(app.get_value(), "F2");
        assert!(app.invoke_finish(app.get_proposal()));
        assert!(!app.invoke_finish(app.get_proposal()));
    }
    app.set_value("Accepted".into());
    settle();
    assert!(app.get_focused());
    stroke(&app, Key::Space);
    down(&app, Key::ShiftR);
    assert_eq!(app.get_releases(), 0);
    app.window().dispatch_event(WindowEvent::KeyPressRepeated {
        text: Key::ShiftR.into(),
    });
    up(&app, Key::ShiftR);
    assert_eq!(app.get_event().text, SharedString::from(Key::ShiftR));
    assert_eq!(app.get_releases(), 1);
    assert!(app.invoke_finish(app.get_proposal()));
    app.set_modifier_only(false);
    stroke(&app, Key::Space);
    stroke(&app, Key::Shift);
    assert!(app.get_recording());
    assert_eq!(app.get_releases(), 1);
    stroke(&app, Key::Escape);
    assert!(!app.get_recording() && app.get_focused());
    assert_eq!(app.get_value(), "Accepted");
    action(&app, "Clear First").invoke_accessible_default_action();
    assert_eq!(app.get_clears(), 1);
    assert_eq!(app.get_value(), "Accepted");
    assert!(app.invoke_finish(app.get_proposal()));
    app.set_value("".into());
    settle();
    assert!(app.get_focused());
    assert_eq!(
        action(&app, "First").accessible_value().as_deref(),
        Some("Unassigned")
    );
    assert!(
        ElementHandle::find_by_accessible_label(&app, "Clear First")
            .next()
            .is_none()
    );
}

#[test]
fn stale_tickets_cannot_cross_fields_revisions_focus_or_window_lifetime() {
    let app = fixture();
    click(&app, "First");
    stroke(&app, "k");
    let old = app.get_proposal();
    click(&app, "First");
    stroke(&app, "p");
    assert!(!app.invoke_finish(old));
    let pending = app.get_proposal();
    app.set_generation(2);
    assert!(!app.invoke_finish(pending));
    settle();
    click(&app, "First");
    stroke(&app, "q");
    let pending = app.get_proposal();
    app.set_value("Remote".into());
    assert!(!app.invoke_finish(pending));
    settle();
    click(&app, "Second");
    stroke(&app, "x");
    let other = app.get_proposal();
    app.set_mounted(false);
    settle();
    app.set_mounted(true);
    settle();
    click(&app, "Second");
    stroke(&app, "y");
    assert_ne!(app.get_proposal(), other);
    click(&app, "First");
    stroke(&app, "z");
    assert!(!app.invoke_finish(other));

    for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
        click(&app, "First");
        assert!(app.get_recording());
        app.set_access(access);
        settle();
        assert!(!app.get_recording());
        let first = action(&app, "First");
        first.invoke_accessible_default_action();
        action(&app, "Clear First").invoke_accessible_default_action();
        assert_eq!(app.get_clears(), 0);
        assert!(!app.get_recording());
        if access == FieldAccess::ReadOnly {
            assert_eq!(first.accessible_read_only(), Some(true));
        } else {
            assert_eq!(first.accessible_enabled(), Some(false));
        }
        app.set_access(FieldAccess::Editable);
        settle();
    }
    click(&app, "First");
    app.set_modal(true);
    settle();
    app.invoke_focus_modal();
    let presses = app.get_presses();
    stroke(&app, "m");
    action(&app, "First").invoke_accessible_default_action();
    assert!(!app.get_focused() && !app.get_recording());
    assert_eq!(app.get_presses(), presses);
    app.set_modal(false);
    settle();
    assert!(!app.get_focused());
    click(&app, "First");
    app.window()
        .dispatch_event(WindowEvent::WindowActiveChanged(false));
    assert!(!app.get_recording());
    app.window()
        .dispatch_event(WindowEvent::WindowActiveChanged(true));
    stroke(&app, "m");
    assert_eq!(app.get_presses(), presses);
    click(&app, "First");
    stroke(&app, "k");
    let pending = app.get_proposal();
    app.hide().unwrap();
    assert!(!app.invoke_finish(pending));
    let weak = app.as_weak();
    drop(app);
    assert!(weak.upgrade().is_none());
}

#[test]
fn hidden_and_restored_window_rejects_an_old_proposal() {
    let app = fixture();
    click(&app, "First");
    stroke(&app, "k");
    let pending = app.get_proposal();
    assert_eq!(app.get_presses(), 1);
    slint_kit::hide_window(&app).unwrap();
    app.show().unwrap();
    settle();
    assert!(!app.invoke_finish(pending));
}

#[test]
fn retained_panel_rejects_an_old_proposal_after_becoming_visible() {
    let app = fixture();
    click(&app, "First");
    stroke(&app, "k");
    let pending = app.get_proposal();
    assert_eq!(app.get_presses(), 1);
    app.set_panel_active(false);
    settle();
    app.set_panel_active(true);
    settle();
    assert!(!app.invoke_finish(pending));

    click(&app, "First");
    stroke(&app, "q");
    let ticket = app.get_proposal();
    app.set_panel_active(false);
    assert!(!app.invoke_finish(ticket));
}

#[test]
fn rejected_proposal_rearms_without_a_click_and_hidden_targets_ignore_events() {
    let app = fixture();
    click(&app, "First");
    stroke(&app, "k");
    let rejected = app.get_proposal();
    assert!(app.invoke_retry(rejected.clone()));
    assert!(app.get_focused() && app.get_recording());
    stroke(&app, Key::F2);
    assert_eq!(app.get_presses(), 2);
    assert!(!app.invoke_retry(rejected.clone()));
    assert!(!app.invoke_finish(rejected));
    assert!(app.invoke_finish(app.get_proposal()));

    click(&app, "First");
    app.set_panel_active(false);
    stroke(&app, "h");
    app.set_panel_active(true);
    settle();
    assert!(!app.get_recording());
    stroke(&app, "p");
    assert_eq!(app.get_presses(), 2);

    click(&app, "First");
    slint_kit::hide_window(&app).unwrap();
    stroke(&app, "h");
    app.show().unwrap();
    stroke(&app, "p");
    assert_eq!(app.get_presses(), 2);
    assert!(!app.get_recording());
}

#[test]
fn accepted_close_invalidates_tickets_and_a_veto_preserves_the_host_session() {
    use slint::CloseRequestResponse;
    use slint_kit::ui::KitFieldService;
    use std::{cell::Cell, rc::Rc};

    let app = fixture();
    let close_allowed = Rc::new(Cell::new(false));
    let policy = close_allowed.clone();
    let weak = app.as_weak();
    app.window().on_close_requested(move || {
        if !policy.get() {
            return CloseRequestResponse::KeepWindowShown;
        }
        weak.unwrap()
            .global::<KitFieldService>()
            .invoke_invalidate_shortcut_session();
        CloseRequestResponse::HideWindow
    });
    click(&app, "First");
    stroke(&app, "k");
    let ticket = app.get_proposal();
    app.window().dispatch_event(WindowEvent::CloseRequested);
    assert!(app.window().is_visible());
    assert!(app.invoke_retry(ticket));
    stroke(&app, Key::F3);
    let ticket = app.get_proposal();
    close_allowed.set(true);
    app.window().dispatch_event(WindowEvent::CloseRequested);
    assert!(!app.window().is_visible());
    app.show().unwrap();
    assert!(!app.invoke_finish(ticket));
}

#[test]
fn clear_action_aligns_with_a_wrapped_capture_at_double_text_scale() {
    use slint_kit::ui::{KitThemeContext, KitThemes};
    let app = fixture();
    let mut theme = app.global::<KitThemes>().get_dark();
    theme.text_scale = 2.;
    app.global::<KitThemeContext>().set_current(theme);
    app.set_multiline(true);
    settle();
    click(&app, "First");
    let first = action(&app, "First");
    let clear = action(&app, "Clear First");
    assert!(first.size().height > clear.size().height);
    let center = |item: &ElementHandle| item.absolute_position().y + item.size().height / 2.;
    assert!((center(&first) - center(&clear)).abs() <= 1.);
}
