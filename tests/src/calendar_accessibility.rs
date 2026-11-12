// SPDX-License-Identifier: GPL-3.0-or-later
//! Semantic projections and native focus lifecycle; no screen-reader verdict.
use crate::CalendarAccessibilityFixture;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::ui::{CalendarOutsideDays, ChoiceFlow, FieldAccess};

fn fixture() -> CalendarAccessibilityFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = CalendarAccessibilityFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    settle(&app);
    app
}
fn settle(app: &CalendarAccessibilityFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
}
fn key(app: &CalendarAccessibilityFixture, value: impl Into<slint::SharedString>) {
    let text = value.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn button(app: &CalendarAccessibilityFixture) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, "Open reservation calendar")
        .next()
        .unwrap()
}
fn calendar(app: &CalendarAccessibilityFixture) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits("KitCalendar")
        .find_first()
        .unwrap()
}
fn capture(app: &CalendarAccessibilityFixture, name: &str) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    std::fs::create_dir_all(&directory).unwrap();
    let snapshot = app.window().take_snapshot().unwrap();
    let mut file = std::fs::File::create(
        std::path::PathBuf::from(directory).join(format!("calendar-accessibility-{name}.ppm")),
    )
    .unwrap();
    write!(
        file,
        "P6\n{} {}\n255\n",
        snapshot.width(),
        snapshot.height()
    )
    .unwrap();
    for pixel in snapshot.as_slice() {
        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
    }
}

#[test]
fn calendar_native_navigation_keeps_one_scope_and_exposes_day_positions_and_states() {
    let app = fixture();
    for flow in [ChoiceFlow::LeftToRight, ChoiceFlow::RightToLeft] {
        app.set_flow(flow);
        app.invoke_focus_calendar();
        settle(&app);
        assert!(app.get_calendar_focused());
        let list = calendar(&app)
            .query_descendants()
            .match_accessible_role(AccessibleRole::List)
            .find_first()
            .unwrap();
        assert_eq!(
            list.accessible_label().as_deref(),
            Some("Reservation calendar")
        );
        assert!(
            list.accessible_description()
                .unwrap()
                .contains("Arrow keys")
        );
        let rows = list
            .query_descendants()
            .match_inherits("CalendarOption")
            .find_all();
        assert_eq!(rows.len(), 42);
        for (index, row) in rows.iter().enumerate() {
            assert_eq!(row.accessible_item_index(), Some(index));
            assert_eq!(row.accessible_item_count(), Some(42));
        }
        let initial = app.get_active();
        key(
            &app,
            if flow == ChoiceFlow::LeftToRight {
                Key::RightArrow
            } else {
                Key::LeftArrow
            },
        );
        assert_eq!(app.get_active(), initial + 1);
        assert!(app.get_calendar_focused());
        key(&app, Key::Return);
        assert_eq!(app.get_selected(), app.get_focused());
        app.set_access(FieldAccess::ReadOnly);
        key(&app, Key::DownArrow);
        assert!(app.get_calendar_focused());
        let selected = app.get_selected();
        key(&app, Key::Return);
        assert_eq!(app.get_selected(), selected);
        let row = list
            .query_descendants()
            .match_inherits("CalendarOption")
            .find_first()
            .unwrap();
        assert_eq!(row.accessible_read_only(), Some(true));
        app.set_access(FieldAccess::Editable);
    }
    app.set_outside_days(CalendarOutsideDays::Hide);
    settle(&app);
    assert!(
        calendar(&app)
            .query_descendants()
            .match_inherits("CalendarOption")
            .find_all()
            .len()
            < 42
    );
    app.set_access(FieldAccess::Disabled);
    settle(&app);
    let list = calendar(&app)
        .query_descendants()
        .match_accessible_role(AccessibleRole::List)
        .find_first()
        .unwrap();
    assert_eq!(list.accessible_enabled(), Some(false));
}

#[test]
fn date_picker_accessible_expansion_is_idempotent_guarded_and_reports_live_state() {
    let app = fixture();
    for access in [
        FieldAccess::Editable,
        FieldAccess::ReadOnly,
        FieldAccess::Disabled,
    ] {
        app.set_access(access);
        app.set_open(false);
        settle(&app);
        let trigger = button(&app);
        assert_eq!(trigger.accessible_expandable(), Some(true));
        assert_eq!(trigger.accessible_expanded(), Some(false));
        trigger.invoke_accessible_expand_action();
        settle(&app);
        assert_eq!(app.get_open(), access == FieldAccess::Editable);
        trigger.invoke_accessible_expand_action();
        settle(&app);
        assert_eq!(app.get_open(), access == FieldAccess::Editable);
        assert_eq!(trigger.accessible_expanded(), Some(app.get_open()));
        if access == FieldAccess::Editable {
            key(&app, Key::Escape);
            assert!(!app.get_open());
            assert_eq!(trigger.accessible_expanded(), Some(false));
        }
    }
}

#[test]
fn picker_hint_reaches_the_native_editor_without_replacing_its_draft() {
    let app = fixture();
    app.invoke_focus_editor();
    let editor = ElementQuery::from_root(&app)
        .match_accessible_role(AccessibleRole::TextInput)
        .find_all()
        .into_iter()
        .find(|e| e.accessible_label().as_deref() == Some("Reservation date"))
        .unwrap();
    assert!(
        editor
            .accessible_description()
            .unwrap()
            .contains(app.get_hint().as_str())
    );
    key(&app, "x");
    let draft = editor.accessible_value();
    app.set_hint("Date civile au format ISO".into());
    settle(&app);
    assert!(
        editor
            .accessible_description()
            .unwrap()
            .contains("Date civile au format ISO")
    );
    assert_eq!(editor.accessible_value(), draft);
    let extra = ElementQuery::from_root(&app)
        .match_accessible_role(AccessibleRole::TextInput)
        .find_all()
        .into_iter()
        .find(|e| e.accessible_label().as_deref() == Some("Separate field"))
        .unwrap();
    assert_eq!(
        extra.accessible_description().as_deref(),
        Some("Help rendered by the host")
    );
    for (name, theme) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1, 2] {
            let mut scaled = theme.clone();
            scaled.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(scaled);
            app.invoke_focus_calendar();
            settle(&app);
            capture(&app, &format!("{name}-{scale}"));
        }
    }
}
