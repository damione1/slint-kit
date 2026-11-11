// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{SelectionNamesFixture, ui};
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{ComponentHandle, Model};
use slint_kit::{Choices, SelectOption};

fn fixture() -> (SelectionNamesFixture, Choices<u8>) {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = SelectionNamesFixture::new().unwrap();
    slint_kit::install(&app);
    let choices = Choices::from_options(vec![
        SelectOption::new(1, "Alpha"),
        SelectOption::new(2, "Beta"),
        SelectOption::new(3, ""),
    ])
    .unwrap();
    app.set_options(choices.model());
    app.show().unwrap();
    (app, choices)
}
fn settle(app: &SelectionNamesFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
}
fn owner(app: &SelectionNamesFixture, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
fn control(app: &SelectionNamesFixture, name: &str) -> ElementHandle {
    let root = owner(app, name);
    // Combobox owns the role on its root; Select shares a trigger.
    if root.accessible_role() == Some(AccessibleRole::Combobox) {
        root
    } else {
        root.query_descendants()
            .match_accessible_role(AccessibleRole::Combobox)
            .find_first()
            .unwrap()
    }
}
fn capture(app: &SelectionNamesFixture, state: &str) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    std::fs::create_dir_all(&directory).unwrap();
    for (name, base) in [
        ("light", app.global::<ui::KitThemes>().get_light()),
        ("dark", app.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            theme.reduced_motion = true;
            app.set_theme(theme);
            let image = app.window().take_snapshot().unwrap();
            let mut output = std::fs::File::create(
                std::path::PathBuf::from(&directory)
                    .join(format!("selection-names-{state}-{name}-{scale}.ppm")),
            )
            .unwrap();
            write!(output, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
            for pixel in image.as_slice() {
                output.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
            }
        }
    }
}

#[test]
fn selection_names_hints_and_values_are_independent_including_an_empty_option_label() {
    let (app, choices) = fixture();
    let strings = app.global::<ui::KitStrings>();
    strings.set_select_label("Espace".into());
    strings.set_select_placeholder("Choisir un espace".into());
    assert_eq!(
        control(&app, "KitSelect").accessible_label().as_deref(),
        Some("Espace")
    );
    assert_eq!(
        control(&app, "KitSelect")
            .accessible_placeholder_text()
            .as_deref(),
        Some("Choisir un espace")
    );
    assert_eq!(
        control(&app, "KitSelect").accessible_value().as_deref(),
        Some("")
    );
    app.set_label("Destination".into());
    app.set_placeholder("Pick a workspace".into());
    capture(&app, "empty");
    for (key, value) in [(1, "Alpha"), (2, "Beta"), (3, "")] {
        app.set_selected(choices.selection(&key).unwrap());
        settle(&app);
        assert_eq!(
            app.get_combo_text(),
            value,
            "the native editor projects the accepted label, including empty labels"
        );
        let single = control(&app, "KitSelect");
        assert_eq!(single.accessible_label().as_deref(), Some("Destination"));
        assert_eq!(
            single.accessible_placeholder_text().as_deref(),
            Some("Pick a workspace")
        );
        assert_eq!(single.accessible_value().as_deref(), Some(value));
        let combo = control(&app, "KitCombobox");
        assert_eq!(combo.accessible_label().as_deref(), Some("Contributor"));
        assert_eq!(combo.accessible_value().as_deref(), Some(value));
        if key == 2 {
            capture(&app, "chosen");
        }
    }
    assert_eq!(app.get_availability(), ui::SelectionAvailability::Available);
    assert!(
        !control(&app, "KitSelect")
            .query_descendants()
            .match_accessible_role(AccessibleRole::Text)
            .find_all()
            .iter()
            .any(|text| text.accessible_label().as_deref() == Some("Pick a workspace")),
        "a valid empty label is not replaced by the visible hint"
    );
    app.invoke_override_single_label("External current value".into());
    assert_eq!(
        control(&app, "KitSelect").accessible_value().as_deref(),
        Some("External current value")
    );
    assert_eq!(
        app.get_selected(),
        choices.selection(&3).unwrap(),
        "display text is not selection identity"
    );
    app.invoke_override_value_label("Current item: three".into());
    assert_eq!(
        control(&app, "KitSelect").accessible_value().as_deref(),
        Some("Current item: three")
    );
    assert_eq!(
        control(&app, "KitSelect").accessible_label().as_deref(),
        Some("Destination")
    );
    assert_eq!(app.get_selected(), choices.selection(&3).unwrap());
}

#[test]
fn multiple_selection_value_reports_localized_count_and_labels_after_controlled_acknowledgment() {
    let (app, choices) = fixture();
    app.set_multi_label("Members".into());
    let multi = control(&app, "KitMultiSelect");
    assert_eq!(multi.accessible_label().as_deref(), Some("Members"));
    assert_eq!(multi.accessible_value().as_deref(), Some(""));
    assert_eq!(
        multi.accessible_placeholder_text().as_deref(),
        Some("Add a selection")
    );
    app.set_ownership(ui::Ownership::Controlled);
    app.set_many(choices.selections(&[1, 2]).unwrap());
    assert_eq!(
        multi.accessible_value().as_deref(),
        Some("2 selected: Alpha, Beta")
    );
    let strings = app.global::<ui::KitStrings>();
    strings.set_selection_label_separator(" / ".into());
    strings.set_multi_selection_value_format("{labels} — {count} choix".into());
    assert_eq!(
        multi.accessible_value().as_deref(),
        Some("Alpha / Beta — 2 choix")
    );
    capture(&app, "multiple");
    let remove = owner(&app, "KitMultiSelect")
        .query_descendants()
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Remove Alpha"))
        .find_first()
        .unwrap();
    remove.invoke_accessible_default_action();
    assert_eq!(
        app.get_many().row_count(),
        2,
        "controlled removal waits for the host"
    );
    assert_eq!(
        multi.accessible_value().as_deref(),
        Some("Alpha / Beta — 2 choix")
    );
    app.set_many(choices.selections(&[2]).unwrap());
    assert_eq!(multi.accessible_value().as_deref(), Some("Beta — 1 choix"));
    for access in [ui::FieldAccess::ReadOnly, ui::FieldAccess::Disabled] {
        app.set_access(access);
        assert_eq!(multi.accessible_label().as_deref(), Some("Members"));
        assert_eq!(multi.accessible_value().as_deref(), Some("Beta — 1 choix"));
    }
    app.set_many(Default::default());
    assert_eq!(multi.accessible_value().as_deref(), Some(""));
}

#[test]
fn unavailable_selection_shows_a_localized_message_and_the_theme_warning_glyph() {
    let (app, choices) = fixture();
    app.set_ownership(ui::Ownership::Controlled);
    app.set_label("Destination".into());
    app.set_selected(choices.selection(&1).unwrap());
    app.global::<ui::KitStrings>()
        .set_selection_unavailable_label("Choice removed; select another".into());
    app.set_options(slint::ModelRc::new(slint::VecModel::from(vec![
        choices.model().row_data(1).unwrap(),
    ])));
    assert_eq!(
        app.get_availability(),
        ui::SelectionAvailability::Unavailable
    );
    let single = control(&app, "KitSelect");
    assert_eq!(single.accessible_label().as_deref(), Some("Destination"));
    assert!(
        single
            .accessible_description()
            .is_some_and(|s| s.contains("Choice removed; select another"))
    );
    assert!(
        single
            .query_descendants()
            .match_accessible_role(AccessibleRole::Text)
            .find_all()
            .iter()
            .any(|e| e.accessible_label().as_deref() == Some("Choice removed; select another")),
        "the message is visible text, not merely an accessible description"
    );
    assert!(
        single
            .query_descendants()
            .match_inherits("KitIcon")
            .find_first()
            .is_some()
    );
    let mut theme = app.get_theme();
    theme.reduced_motion = true;
    app.set_theme(theme.clone());
    let before = app.window().take_snapshot().unwrap();
    let original = theme.icons.warning.clone();
    theme.icons.warning = theme.icons.check.clone();
    app.set_theme(theme.clone());
    let changed = app.window().take_snapshot().unwrap();
    assert_ne!(
        before.as_slice(),
        changed.as_slice(),
        "changing only the warning token changes the painted glyph"
    );
    theme.icons.warning = original;
    app.set_theme(theme);
    capture(&app, "unavailable");
    assert_eq!(
        app.get_selected(),
        choices.selection(&1).unwrap(),
        "warning presentation preserves the host's unavailable identity"
    );
}

#[test]
fn editable_combobox_keeps_name_hint_draft_and_retained_selection_separate() {
    let (app, choices) = fixture();
    app.set_selected(choices.selection(&1).unwrap());
    settle(&app);
    assert_eq!(app.get_combo_text(), "Alpha");
    assert_eq!(
        app.get_combo_label(),
        "Alpha",
        "accept the native selected projection before replacing remote options"
    );
    let combo = control(&app, "KitCombobox");
    assert_eq!(combo.accessible_label().as_deref(), Some("Contributor"));
    assert_eq!(
        combo.accessible_placeholder_text().as_deref(),
        Some("Search contributors")
    );
    assert_eq!(combo.accessible_value().as_deref(), Some("Alpha"));
    app.set_options(slint::ModelRc::new(slint::VecModel::from(vec![
        choices.model().row_data(1).unwrap(),
    ])));
    settle(&app);
    assert_eq!(app.get_combo_text(), "Alpha");
    assert_eq!(app.get_combo_label(), "Alpha");
    assert_eq!(
        app.get_combo_availability(),
        ui::SelectionAvailability::Available
    );
    assert_eq!(combo.accessible_value().as_deref(), Some("Alpha"));
    slint::platform::update_timers_and_animations();
    app.invoke_focus_combo();
    for _ in 0.."Alpha".len() {
        app.window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed {
                text: slint::platform::Key::RightArrow.into(),
            });
        app.window()
            .dispatch_event(slint::platform::WindowEvent::KeyReleased {
                text: slint::platform::Key::RightArrow.into(),
            });
    }
    for text in ["x", "y"] {
        app.window()
            .dispatch_event(slint::platform::WindowEvent::KeyPressed { text: text.into() });
        app.window()
            .dispatch_event(slint::platform::WindowEvent::KeyReleased { text: text.into() });
    }
    assert_eq!(app.get_combo_text(), "Alphaxy");
    app.invoke_separate_combo_query("server-page-query".into());
    assert_eq!(app.get_combo_query(), "server-page-query");
    assert_eq!(app.get_combo_label(), "Alpha");
    assert_eq!(combo.accessible_label().as_deref(), Some("Contributor"));
    assert_eq!(
        combo.accessible_placeholder_text().as_deref(),
        Some("Search contributors")
    );
    assert_eq!(combo.accessible_value().as_deref(), Some("Alphaxy"));
}
