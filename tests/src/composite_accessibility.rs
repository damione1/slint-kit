// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{AccessibleComposite, CompositeAccessibilityFixture, ui};
use i_slint_backend_testing::{
    AccessibleRole, ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions,
};
use slint::{ComponentHandle, Model, ModelRc, VecModel, platform::WindowEvent};
use slint_kit::{Choices, SelectOption};

fn fixture() -> (CompositeAccessibilityFixture, Choices<u8>) {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = CompositeAccessibilityFixture::new().unwrap();
    slint_kit::install(&app);
    let mut blocked = SelectOption::new(2, "Blocked");
    blocked.disabled = true;
    let choices = Choices::from_options(vec![
        SelectOption::new(1, "Alpha"),
        blocked,
        SelectOption::new(3, "Gamma"),
        SelectOption::new(4, "Delta"),
        SelectOption::new(5, "Epsilon"),
    ])
    .unwrap();
    app.set_options(choices.model());
    app.set_selected(choices.selection(&1).unwrap());
    app.show().unwrap();
    (app, choices)
}
fn key(app: &CompositeAccessibilityFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    slint::platform::update_timers_and_animations();
}
fn component(app: &CompositeAccessibilityFixture, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
fn delegated_row(
    app: &CompositeAccessibilityFixture,
    control: AccessibleComposite,
) -> ElementHandle {
    let (name, role, index) = match control {
        AccessibleComposite::MenuBar => (
            "KitMenuBar",
            AccessibleRole::ListItem,
            app.get_bar_delegate(),
        ),
        AccessibleComposite::Tabs => ("KitTabs", AccessibleRole::Tab, app.get_tabs_delegate()),
        AccessibleComposite::Radio => (
            "KitRadioGroup",
            AccessibleRole::RadioButton,
            app.get_radio_delegate(),
        ),
        AccessibleComposite::Segments => (
            "KitSegmentedControl",
            AccessibleRole::RadioButton,
            app.get_segments_delegate(),
        ),
        AccessibleComposite::Toggles => (
            "KitToggleGroup",
            AccessibleRole::Button,
            app.get_toggles_delegate(),
        ),
        AccessibleComposite::Menu
        | AccessibleComposite::Rating
        | AccessibleComposite::Select
        | AccessibleComposite::Combobox
        | AccessibleComposite::Multiple
        | AccessibleComposite::Surface => panic!("no nonvirtual delegation oracle for {control:?}"),
    };
    // Observe actual instantiated row order from the public testing API; do not
    // reproduce the source-to-layout index formula in the expected result.
    let rows = component(app, name)
        .query_descendants()
        .match_accessible_role(role)
        .find_all();
    rows[usize::try_from(index).expect("current enabled choice has a delegate")].clone()
}
fn count_rows(container: &ElementHandle, expected: usize) {
    assert_eq!(container.accessible_item_count(), Some(expected));
    for row in container
        .query_descendants()
        .match_inherits("KitOption")
        .find_all()
    {
        assert_eq!(row.accessible_item_count(), Some(expected));
        assert!(
            row.accessible_item_index()
                .is_some_and(|index| index < expected)
        );
    }
}

fn capture_themes(app: &CompositeAccessibilityFixture, control: AccessibleComposite) {
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
            slint::platform::update_timers_and_animations();
            let image = app.window().take_snapshot().unwrap();
            let mut output = std::fs::File::create(std::path::PathBuf::from(&directory).join(
                format!("composite-accessibility-{control:?}-{name}-{scale}.ppm"),
            ))
            .unwrap();
            write!(output, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
            for pixel in image.as_slice() {
                output.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
            }
        }
    }
}

#[test]
fn nonvirtual_composites_delegate_to_the_actual_active_row_after_navigation_and_projection() {
    let (app, choices) = fixture();
    for control in [
        AccessibleComposite::MenuBar,
        AccessibleComposite::Tabs,
        AccessibleComposite::Radio,
        AccessibleComposite::Segments,
        AccessibleComposite::Toggles,
    ] {
        app.set_control(control);
        app.set_flow(ui::ChoiceFlow::RightToLeft);
        app.set_direction(ui::ChoiceDirection::Horizontal);
        app.set_ownership(ui::Ownership::Controlled);
        app.set_selected(choices.selection(&1).unwrap());
        app.invoke_focus_current();
        use slint::platform::Key;
        // Bar/tabs keep their existing physical key policy. Choice controls
        // map Left to next in right-to-left flow; no key behavior changes here.
        key(
            &app,
            if matches!(
                control,
                AccessibleComposite::Radio | AccessibleComposite::Segments
            ) {
                Key::LeftArrow
            } else {
                Key::RightArrow
            },
        );
        let active = delegated_row(&app, control);
        assert_eq!(
            active.accessible_label().as_deref(),
            Some("Gamma"),
            "{control:?}: skip disabled, follow stable token"
        );
        assert_eq!(active.accessible_item_index(), Some(2));
        assert_eq!(active.accessible_item_count(), Some(5));
        assert_eq!(
            app.get_selected(),
            choices.selection(&1).unwrap(),
            "controlled navigation never acknowledges itself"
        );
        capture_themes(&app, control);
        if matches!(
            control,
            AccessibleComposite::Radio | AccessibleComposite::Segments
        ) {
            app.set_direction(ui::ChoiceDirection::Vertical);
            assert_eq!(
                delegated_row(&app, control).accessible_label().as_deref(),
                Some("Gamma")
            );
        }
    }
    // Keep a live stable choice while its visual/model position changes.
    app.set_control(AccessibleComposite::Tabs);
    app.invoke_focus_current();
    key(&app, slint::platform::Key::RightArrow);
    let rows = choices.model().iter().collect::<Vec<_>>();
    app.set_options(ModelRc::new(VecModel::from(vec![
        rows[2].clone(),
        rows[0].clone(),
        rows[1].clone(),
        rows[3].clone(),
        rows[4].clone(),
    ])));
    assert_eq!(
        delegated_row(&app, AccessibleComposite::Tabs)
            .accessible_label()
            .as_deref(),
        Some("Gamma")
    );
    assert_eq!(
        delegated_row(&app, AccessibleComposite::Tabs).accessible_item_index(),
        Some(0)
    );
    app.set_access(ui::FieldAccess::Disabled);
    assert_eq!(app.get_tabs_delegate(), -1);
    app.set_access(ui::FieldAccess::Editable);
    app.invoke_focus_reference();
    app.set_flow(ui::ChoiceFlow::LeftToRight);
    assert!(app.get_reference_focused());
    key(&app, "X");
    assert!(
        app.get_reference_text().contains('X'),
        "AT projection updates cannot take native editor focus"
    );
}

#[test]
fn accessibility_positions_keep_virtualized_model_ordinals_and_native_search_editing() {
    let (app, _choices) = fixture();
    let many = Choices::from_options(
        (0..2000u32)
            .map(|key| SelectOption::new(key, format!("Row {key}")))
            .collect::<Vec<_>>(),
    )
    .unwrap();
    app.set_options(many.model());
    app.set_control(AccessibleComposite::Menu);
    app.invoke_focus_current();
    key(&app, slint::platform::Key::End);
    app.window().take_snapshot().unwrap();
    let menu = component(&app, "KitMenu");
    let rows = menu
        .query_descendants()
        .match_inherits("KitNavigationAction")
        .find_all();
    assert!(
        rows.len() < 100,
        "preserve the virtualized view instead of instantiating all semantic rows"
    );
    assert!(
        rows.iter()
            .any(|row| row.accessible_label().as_deref() == Some("Row 1999")
                && row.accessible_item_index() == Some(1999))
    );
    assert_eq!(menu.accessible_item_count(), Some(2000));
    for row in rows {
        assert_eq!(row.accessible_item_count(), Some(2000));
    }
    capture_themes(&app, AccessibleComposite::Menu);
    app.set_options(
        Choices::from_options(vec![
            SelectOption::new(1u8, "Alpha"),
            SelectOption::new(2u8, "Beta"),
        ])
        .unwrap()
        .model(),
    );
    for (control, name) in [
        (AccessibleComposite::Select, "KitSelect"),
        (AccessibleComposite::Multiple, "KitMultiSelect"),
        (AccessibleComposite::Combobox, "KitCombobox"),
    ] {
        app.set_control(control);
        app.invoke_focus_current();
        key(&app, slint::platform::Key::DownArrow);
        let surface = component(&app, name)
            .query_descendants()
            .match_inherits("KitSelectSurface")
            .find_first()
            .unwrap();
        assert_eq!(surface.accessible_role(), Some(AccessibleRole::List));
        assert!(surface.accessible_label().is_some_and(|s| !s.is_empty()));
        count_rows(&surface, 2);
        capture_themes(&app, control);
        if control == AccessibleComposite::Combobox {
            key(&app, "Al");
            assert_eq!(app.get_query(), "Al");
            let editor = surface
                .query_descendants()
                .match_accessible_role(AccessibleRole::TextInput)
                .find_first()
                .unwrap();
            editor.set_accessible_selection_offsets(0, 2);
            key(&app, slint::platform::Key::DownArrow);
            key(&app, "Q");
            assert_eq!(
                app.get_query(),
                "Q",
                "arrow navigation preserves native editor focus and selected replacement"
            );
            assert_eq!(editor.accessible_value().as_deref(), Some("Q"));
        }
        key(&app, slint::platform::Key::Escape);
    }
}

#[test]
fn focus_projection_rejects_stale_disabled_and_ambiguous_identity() {
    let (app, choices) = fixture();
    let service = app.global::<ui::KitSelectionService>();
    let model = choices.model();
    let generation = model.row_data(0).unwrap().generation;
    assert_eq!(
        service.invoke_option_index(model.clone(), choices.selection(&3).unwrap(), generation),
        2
    );
    assert_eq!(
        service.invoke_option_index(
            model.clone(),
            choices.selection(&3).unwrap(),
            generation + 1
        ),
        -1
    );
    assert_eq!(
        service.invoke_option_index(model.clone(), choices.selection(&2).unwrap(), generation),
        -1
    );
    let row = model.row_data(0).unwrap();
    let duplicate = ModelRc::new(VecModel::from(vec![row.clone(), row]));
    assert_eq!(
        service.invoke_option_index(duplicate, choices.selection(&1).unwrap(), generation),
        -1
    );
    let rows = ModelRc::new(VecModel::from(vec![
        ui::ChoiceRow {
            token: 17,
            label: "Stable".into(),
            ..Default::default()
        },
        ui::ChoiceRow {
            token: 23,
            label: "Blocked".into(),
            disabled: true,
            ..Default::default()
        },
    ]));
    assert_eq!(service.invoke_choice_index(rows.clone(), 17), 0);
    assert_eq!(service.invoke_choice_index(rows.clone(), 23), -1);
    assert_eq!(service.invoke_choice_index(rows.clone(), -1), -1);
    assert_eq!(
        service.invoke_choice_index(
            ModelRc::new(VecModel::from(vec![
                rows.row_data(0).unwrap(),
                rows.row_data(0).unwrap()
            ])),
            17
        ),
        -1
    );
}

#[test]
fn surface_and_rating_composition_publish_set_positions_and_preserve_native_editor_focus() {
    let (app, _choices) = fixture();
    app.set_control(AccessibleComposite::Surface);
    app.invoke_focus_current();
    let surface = component(&app, "KitSelectSurface");
    assert_eq!(
        surface.accessible_label().as_deref(),
        Some("Standalone options")
    );
    count_rows(&surface, 5);
    capture_themes(&app, AccessibleComposite::Surface);
    app.set_control(AccessibleComposite::Rating);
    let rating = component(&app, "KitRating");
    let group = rating
        .query_descendants()
        .match_inherits("KitSegmentedControl")
        .find_first()
        .unwrap();
    assert_eq!(group.accessible_item_count(), Some(5));
    let three = group
        .query_descendants()
        .match_predicate(|row| {
            row.accessible_role() == Some(AccessibleRole::RadioButton)
                && row.accessible_label().as_deref() == Some("3")
        })
        .find_first()
        .unwrap();
    assert_eq!(three.accessible_item_index(), Some(2));
    assert_eq!(three.accessible_item_count(), Some(5));
    three.invoke_accessible_default_action();
    assert_eq!(app.get_score().step, 3);
    assert!(app.get_score().present);
    capture_themes(&app, AccessibleComposite::Rating);
    app.invoke_focus_reference();
    app.set_access(ui::FieldAccess::ReadOnly);
    three.invoke_accessible_default_action();
    assert!(app.get_reference_focused());
    assert_eq!(app.get_score().step, 3);
}
