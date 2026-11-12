// SPDX-License-Identifier: GPL-3.0-or-later
use crate::TabsPanelsFixture;
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::{
    Choices, SelectOption,
    ui::{ChoiceFlow, Ownership, TabActivation},
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination {
    Profile,
    Unavailable,
    Settings,
}
struct Harness {
    app: TabsPanelsFixture,
    choices: Rc<Choices<Destination>>,
    proposals: Rc<RefCell<Vec<Destination>>>,
}
fn fixture() -> Harness {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = TabsPanelsFixture::new().unwrap();
    slint_kit::install(&app);
    let mut unavailable = SelectOption::new(Destination::Unavailable, "Unavailable");
    unavailable.disabled = true;
    let choices = Rc::new(
        Choices::from_options(vec![
            SelectOption::new(Destination::Profile, "Profile"),
            unavailable,
            SelectOption::new(Destination::Settings, "Settings"),
        ])
        .unwrap(),
    );
    app.set_options(choices.model());
    app.set_profile(choices.selection(&Destination::Profile).unwrap());
    app.set_settings(choices.selection(&Destination::Settings).unwrap());
    app.set_selected(choices.selection(&Destination::Profile).unwrap());
    let proposals = Rc::new(RefCell::new(Vec::new()));
    let (source, recorded) = (choices.clone(), proposals.clone());
    app.on_chosen(move |row| {
        recorded
            .borrow_mut()
            .push(source.accept(row).expect("current typed destination"))
    });
    app.show().unwrap();
    settle(&app);
    Harness {
        app,
        choices,
        proposals,
    }
}
fn settle(app: &TabsPanelsFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
}
fn key(app: &TabsPanelsFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn modified(app: &TabsPanelsFixture, modifier: Key, value: Key) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: modifier.into(),
    });
    key(app, value);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: modifier.into(),
    });
    settle(app);
}
fn capture(app: &TabsPanelsFixture, name: &str) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    std::fs::create_dir_all(&directory).unwrap();
    let image = app.window().take_snapshot().unwrap();
    let mut file = std::fs::File::create(
        std::path::PathBuf::from(directory).join(format!("tabs-panels-{name}.ppm")),
    )
    .unwrap();
    write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
    for pixel in image.as_slice() {
        file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
    }
}

#[test]
fn horizontal_tabs_preserve_parent_shortcuts_scroll_keys_and_escape() {
    let harness = fixture();
    let app = &harness.app;
    app.invoke_focus_tabs();
    key(app, Key::Home);
    let initial = app.get_tabs_state().active;
    for value in [Key::UpArrow, Key::DownArrow, Key::PageUp, Key::PageDown] {
        key(app, value);
    }
    key(app, "s");
    key(app, Key::Escape);
    assert_eq!(app.get_vertical_keys(), 4);
    assert_eq!(app.get_shortcuts(), 1);
    assert_eq!(app.get_escapes(), 1);
    assert_eq!(app.get_tabs_state().active, initial);
    assert!(harness.proposals.borrow().is_empty());
    modified(app, Key::Control, Key::RightArrow);
    assert_eq!(app.get_shortcuts(), 2);
    assert_eq!(app.get_tabs_state().active, initial);
    app.invoke_focus_bar();
    key(app, Key::Home);
    let bar = app.get_bar_state().active;
    key(app, Key::DownArrow);
    key(app, Key::UpArrow);
    assert_eq!(app.get_bar_state().active, bar);
    assert_eq!(app.get_vertical_keys(), 6);
}

#[test]
fn manual_and_automatic_navigation_skip_disabled_rows_and_follow_rtl() {
    let harness = fixture();
    let app = &harness.app;
    for flow in [ChoiceFlow::LeftToRight, ChoiceFlow::RightToLeft] {
        app.set_flow(flow);
        app.set_activation(TabActivation::Manual);
        app.set_selected(harness.choices.selection(&Destination::Profile).unwrap());
        app.invoke_focus_tabs();
        key(app, Key::Home);
        key(
            app,
            if flow == ChoiceFlow::LeftToRight {
                Key::RightArrow
            } else {
                Key::LeftArrow
            },
        );
        assert_eq!(
            app.get_tabs_state().active.token,
            harness
                .choices
                .selection(&Destination::Settings)
                .unwrap()
                .token
        );
        assert!(app.get_profile_active());
        assert!(!app.get_settings_active());
        key(app, Key::Space);
        assert!(!app.get_profile_active());
        assert!(app.get_settings_active());
        assert_eq!(
            harness.proposals.borrow().last(),
            Some(&Destination::Settings)
        );
        app.set_activation(TabActivation::Automatic);
        key(app, Key::Home);
        assert!(app.get_profile_active());
        assert!(!app.get_settings_active());
        key(
            app,
            if flow == ChoiceFlow::LeftToRight {
                Key::RightArrow
            } else {
                Key::LeftArrow
            },
        );
        assert!(app.get_settings_active());
    }
    app.set_interactive(false);
    let selected = app.get_selected();
    key(app, Key::Home);
    key(app, Key::Return);
    assert_eq!(app.get_selected(), selected);
}

#[test]
fn optional_typeahead_keeps_space_activation_and_automatic_acceptance() {
    let harness = fixture();
    let app = &harness.app;
    app.set_typeahead_enabled(true);
    app.invoke_focus_tabs();
    key(app, Key::Home);
    key(app, "s");
    assert_eq!(app.get_shortcuts(), 0);
    assert!(app.get_profile_active());
    key(app, Key::Space);
    assert!(app.get_settings_active());
    app.set_activation(TabActivation::Automatic);
    key(app, Key::Home);
    key(app, "s");
    assert!(app.get_settings_active());
    app.set_ownership(Ownership::Controlled);
    app.set_selected(harness.choices.selection(&Destination::Profile).unwrap());
    app.invoke_focus_tabs();
    key(app, Key::End);
    assert!(
        app.get_profile_active(),
        "controlled selection waits for its owner"
    );
    assert_eq!(
        harness.proposals.borrow().last(),
        Some(&Destination::Settings)
    );
    app.set_selected(harness.choices.selection(&Destination::Settings).unwrap());
    settle(app);
    assert!(app.get_settings_active());
}

#[test]
fn tab_traversal_enters_only_the_selected_panel_and_hidden_editors_keep_undo() {
    let harness = fixture();
    let app = &harness.app;
    assert!(app.get_profile_active());
    assert_eq!(app.get_settings_height(), 0.);
    app.invoke_focus_tabs();
    key(app, Key::Tab);
    assert!(app.get_profile_focused());
    assert!(!app.get_settings_focused());
    key(app, Key::Tab);
    assert!(app.get_after_focused());
    modified(app, Key::Shift, Key::Tab);
    assert!(app.get_profile_focused());
    app.invoke_select_profile();
    key(app, "edited");
    assert_eq!(app.get_profile_text(), "edited");
    app.invoke_focus_tabs();
    key(app, Key::End);
    key(app, Key::Return);
    assert!(app.get_settings_active());
    assert_eq!(app.get_profile_height(), 0.);
    key(app, Key::Tab);
    assert!(app.get_settings_focused());
    app.invoke_focus_tabs();
    key(app, Key::Home);
    key(app, Key::Return);
    app.invoke_focus_profile();
    app.invoke_undo_profile();
    settle(app);
    // Slint's native replacement records insertion and deletion separately.
    // Preserve both native entries instead of introducing a kit undo engine.
    assert_eq!(app.get_profile_text(), "");
    app.invoke_undo_profile();
    settle(app);
    assert_eq!(
        app.get_profile_text(),
        "Profile original",
        "the same native editor and undo history survive panel switches"
    );
}

#[test]
fn panel_identity_survives_reordering_and_controlled_unavailability() {
    let harness = fixture();
    let app = &harness.app;
    let profile = harness.choices.selection(&Destination::Profile).unwrap();
    harness
        .choices
        .replace(vec![
            SelectOption::new(Destination::Settings, "Settings"),
            SelectOption::new(Destination::Profile, "Profile"),
        ])
        .unwrap();
    settle(app);
    assert_eq!(app.get_selected(), profile);
    assert!(app.get_profile_active());
    assert!(!app.get_settings_active());
    let panels = ElementQuery::from_root(app)
        .match_accessible_role(AccessibleRole::TabPanel)
        .find_all();
    assert_eq!(panels.len(), 1);
    assert_eq!(
        panels[0].accessible_label().as_deref(),
        Some("Profile panel")
    );
    app.set_ownership(Ownership::Controlled);
    harness
        .choices
        .replace(vec![SelectOption::new(Destination::Settings, "Settings")])
        .unwrap();
    settle(app);
    assert!(!app.get_profile_active());
    assert!(!app.get_settings_active());
    assert_eq!(app.get_profile_height(), 0.);
    assert_eq!(app.get_settings_height(), 0.);
    harness
        .choices
        .replace(vec![SelectOption::new(Destination::Profile, "Profile")])
        .unwrap();
    settle(app);
    assert!(app.get_profile_active());
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(theme);
            app.invoke_focus_tabs();
            settle(app);
            capture(app, &format!("{name}-{scale}"));
        }
    }
    assert!(
        ElementHandle::find_by_accessible_label(app, "Profile panel")
            .next()
            .is_some()
    );
}

#[test]
fn programmatic_selection_never_edits_a_hidden_native_draft() {
    let harness = fixture();
    let app = &harness.app;
    app.invoke_focus_profile();
    assert!(app.get_profile_focused());
    let original = app.get_profile_text();
    app.set_selected(harness.choices.selection(&Destination::Settings).unwrap());
    settle(app);
    key(app, "x");
    assert_eq!(app.get_profile_text(), original);
    assert_eq!(app.get_settings_text(), "Settings original");
}

#[test]
fn an_opted_in_panel_scope_precedes_its_native_children() {
    let harness = fixture();
    let app = &harness.app;
    app.set_static_panel_stop(true);
    app.invoke_focus_tabs();
    key(app, Key::Tab);
    assert!(!app.get_tabs_focused());
    assert!(!app.get_profile_focused());
    assert!(!app.get_settings_focused());
    assert!(!app.get_after_focused());
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(theme);
            settle(app);
            capture(app, &format!("scope-{name}-{scale}"));
        }
    }
    key(app, Key::Tab);
    assert!(app.get_profile_focused());
}
