// SPDX-License-Identifier: GPL-3.0-or-later
use crate::CollapseGroupFixture;
use i_slint_backend_testing::{ElementHandle, TestingBackend, TestingBackendOptions};
use slint::ComponentHandle;
use slint_kit::ui::Ownership;

fn fixture() -> CollapseGroupFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = CollapseGroupFixture::new().unwrap();
    slint_kit::install(&app);
    app.show().unwrap();
    app
}
fn header(app: &CollapseGroupFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(app, label)
        .next()
        .unwrap_or_else(|| panic!("missing {label}"))
}
fn open_states(app: &CollapseGroupFixture) -> (bool, bool) {
    (app.get_first_open(), app.get_second_open())
}

#[test]
fn group_opens_one_section_and_keeps_the_others_closed() {
    let app = fixture();
    assert_eq!(open_states(&app), (true, false));
    header(&app, "Second section").invoke_accessible_default_action();
    assert_eq!(app.get_open(), 1);
    assert_eq!(open_states(&app), (false, true));
    assert_eq!(
        header(&app, "Second section").accessible_expanded(),
        Some(true)
    );
    assert_eq!(
        header(&app, "First section").accessible_expanded(),
        Some(false)
    );
    assert_eq!(app.get_requests(), 1);
}

#[test]
fn collapsible_group_closes_the_open_section() {
    let app = fixture();
    header(&app, "First section").invoke_accessible_default_action();
    assert_eq!(app.get_open(), -1);
    assert_eq!(open_states(&app), (false, false));
}

#[test]
fn non_collapsible_group_keeps_one_section_open() {
    let app = fixture();
    app.set_collapsible(false);
    header(&app, "First section").invoke_accessible_default_action();
    assert_eq!(app.get_open(), 0);
    assert_eq!(open_states(&app), (true, false));
    assert_eq!(app.get_requests(), 0);
    header(&app, "Second section").invoke_accessible_expand_action();
    assert_eq!(app.get_open(), 1);
}

#[test]
fn controlled_group_waits_for_the_host() {
    let app = fixture();
    app.set_ownership(Ownership::Controlled);
    header(&app, "Second section").invoke_accessible_default_action();
    assert_eq!(app.get_requests(), 1);
    assert_eq!(app.get_open(), 0);
    app.set_open(1);
    assert_eq!(open_states(&app), (false, true));
}

#[test]
fn standalone_collapse_ignores_group_state() {
    let app = fixture();
    header(&app, "Standalone section").invoke_accessible_default_action();
    assert!(app.get_alone_open());
    assert_eq!(open_states(&app), (true, false));
}
