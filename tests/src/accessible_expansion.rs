// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{AccessibleExpansionFixture, ExpansionControl, ui};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{ComponentHandle, Model};
use slint_kit::{Choices, SelectOption};

fn fixture(control: ExpansionControl) -> (AccessibleExpansionFixture, Choices<u8>) {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = AccessibleExpansionFixture::new().unwrap();
    slint_kit::install(&app);
    app.set_control(control);
    let mut blocked = SelectOption::new(3, "Unavailable section");
    blocked.disabled = true;
    let mut first = SelectOption::new(1, "First section");
    first.description = "The first disclosed region".into();
    let mut second = SelectOption::new(2, "Second section");
    second.description = "The second disclosed region".into();
    let choices = Choices::from_options(vec![first, second, blocked]).unwrap();
    app.set_options(choices.model());
    app.show().unwrap();
    (app, choices)
}
fn trigger(app: &AccessibleExpansionFixture) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits("KitSelectTrigger")
        .find_first()
        .expect("shared select trigger")
}
fn named(app: &AccessibleExpansionFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn assert_expandable(element: &ElementHandle, expanded: bool) {
    assert_eq!(element.accessible_expandable(), Some(true));
    assert_eq!(element.accessible_expanded(), Some(expanded));
}
fn capture(app: &AccessibleExpansionFixture, name: &str) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    slint::platform::update_timers_and_animations();
    let snapshot = app.window().take_snapshot().unwrap();
    std::fs::create_dir_all(&directory).unwrap();
    let mut file =
        std::fs::File::create(std::path::PathBuf::from(directory).join(format!("{name}.ppm")))
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
fn capture_themes(app: &AccessibleExpansionFixture, control: &str) {
    for (theme_name, base) in [
        ("light", app.global::<ui::KitThemes>().get_light()),
        ("dark", app.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.reduced_motion = true;
            theme.text_scale = scale;
            app.set_theme(theme);
            capture(
                app,
                &format!("accessible-expand-{control}-{theme_name}-{scale}"),
            );
        }
    }
}

#[test]
fn select_family_accessible_expand_opens_without_toggling_and_preserves_access_and_host_state() {
    let (app, _choices) = fixture(ExpansionControl::Trigger);
    for control in [
        ExpansionControl::Trigger,
        ExpansionControl::Select,
        ExpansionControl::Combobox,
        ExpansionControl::Multiple,
    ] {
        app.set_control(control);
        app.set_access(ui::FieldAccess::Editable);
        app.set_ownership(ui::Ownership::Local);
        app.set_opened(false);
        app.set_requests(0);
        let element = trigger(&app);
        assert_expandable(&element, false);
        element.invoke_accessible_expand_action();
        assert!(app.get_opened(), "{control:?}: expands");
        assert_eq!(app.get_requests(), 1);
        assert_expandable(&element, true);
        element.invoke_accessible_expand_action();
        assert_eq!(app.get_requests(), 1, "expand is idempotent while open");
        capture_themes(&app, &format!("{control:?}"));
        element.invoke_accessible_default_action();
        assert!(
            !app.get_opened(),
            "ordinary activation retains toggle behavior"
        );
        for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
            app.set_access(access);
            let count = app.get_requests();
            element.invoke_accessible_expand_action();
            assert_eq!(app.get_requests(), count, "{control:?}: {access:?}");
            assert!(!app.get_opened());
        }
        app.set_access(ui::FieldAccess::Editable);
        app.set_ownership(ui::Ownership::Controlled);
        app.set_requests(0);
        element.invoke_accessible_expand_action();
        assert_eq!(app.get_requests(), 1);
        assert!(app.get_requested());
        assert!(
            !app.get_opened(),
            "host retains closed state until acknowledgment"
        );
        app.set_opened(true);
        element.invoke_accessible_expand_action();
        assert_eq!(app.get_requests(), 1);
        element.invoke_accessible_default_action();
        assert_eq!(app.get_requests(), 2);
        assert!(!app.get_requested());
        assert!(app.get_opened(), "close is also a host proposal");
        app.set_opened(false);
    }
    app.set_control(ExpansionControl::Trigger);
    app.set_busy(true);
    let count = app.get_requests();
    trigger(&app).invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), count, "busy trigger cannot expand");
}

#[test]
fn collapse_accessible_expand_preserves_local_controlled_and_disabled_behavior() {
    let (app, _choices) = fixture(ExpansionControl::Collapse);
    let element = named(&app, "Expandable details");
    assert_expandable(&element, false);
    element.invoke_accessible_expand_action();
    assert!(app.get_opened());
    assert_expandable(&element, true);
    element.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 1);
    capture_themes(&app, "collapse");
    element.invoke_accessible_default_action();
    assert!(!app.get_opened());
    app.set_access(ui::FieldAccess::Disabled);
    element.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 2);
    app.set_access(ui::FieldAccess::Editable);
    app.set_ownership(ui::Ownership::Controlled);
    element.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 3);
    assert!(app.get_requested());
    assert!(!app.get_opened());
    app.set_opened(true);
    element.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 3);
    element.invoke_accessible_default_action();
    assert!(app.get_opened());
    assert!(!app.get_requested());
}

#[test]
fn accordion_accessible_expand_uses_stable_sections_and_existing_expansion_policies() {
    let (app, choices) = fixture(ExpansionControl::Accordion);
    let first = named(&app, "First section");
    let second = named(&app, "Second section");
    let blocked = named(&app, "Unavailable section");
    assert_expandable(&first, false);
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_expanded().row_data(0), choices.selection(&1));
    assert_expandable(&first, true);
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 1);
    second.invoke_accessible_expand_action();
    assert_eq!(app.get_expanded().row_count(), 1);
    assert_eq!(app.get_expanded().row_data(0), choices.selection(&2));
    assert_expandable(&first, false);
    assert_expandable(&second, true);
    app.set_mode(ui::ExpansionMode::Multiple);
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_expanded().row_count(), 2);
    capture_themes(&app, "accordion");
    blocked.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 3);
    assert_expandable(&blocked, false);
    app.set_expanded(Default::default());
    app.set_access(ui::FieldAccess::Disabled);
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 3);
    assert_eq!(app.get_expanded().row_count(), 0);
    app.set_access(ui::FieldAccess::Editable);
    app.set_ownership(ui::Ownership::Controlled);
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 4);
    assert!(app.get_requested());
    assert_eq!(app.get_expanded().row_count(), 0);
    app.set_expanded(choices.selections(&[1]).unwrap());
    first.invoke_accessible_expand_action();
    assert_eq!(app.get_requests(), 4);
    first.invoke_accessible_default_action();
    assert_eq!(app.get_requests(), 5);
    assert!(!app.get_requested());
    assert_eq!(
        app.get_expanded().row_count(),
        1,
        "host retains acknowledged value"
    );
}
