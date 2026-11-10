// SPDX-License-Identifier: GPL-3.0-or-later
use super::{OptionRow, SearchPresentation, SelectGeometryFixture, SelectState};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::Cell, rc::Rc};

fn rows(count: usize) -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(
        (0..count)
            .map(|index| OptionRow {
                token: index as i32,
                generation: 1,
                label: format!("Workspace {index}").into(),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ))
}

fn surface(window: &SelectGeometryFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitSelectSurface")
        .find_first()
        .expect("open popup surface")
}

#[test]
fn short_select_popups_fit_their_rows_and_large_models_stay_bounded() {
    i_slint_backend_testing::init_no_event_loop();
    for combobox in [false, true] {
        let window = SelectGeometryFixture::new().unwrap();
        window.set_combobox(combobox);
        window.set_rows(rows(3));
        window.show().unwrap();
        let _ = ElementQuery::from_root(&window)
            .match_inherits(if combobox { "KitCombobox" } else { "KitSelect" })
            .find_first()
            .unwrap()
            .size();
        window.set_state(SelectState {
            open: true,
            ..Default::default()
        });
        slint::platform::update_timers_and_animations();
        let height = surface(&window).size().height;
        let expected_three = if combobox { 140.0..200.0 } else { 190.0..250.0 };
        assert!(
            expected_three.contains(&height),
            "three rows need a compact popup: {height}"
        );
        window.set_rows(rows(1));
        let height = surface(&window).size().height;
        let expected_one = if combobox { 55.0..110.0 } else { 90.0..150.0 };
        assert!(
            expected_one.contains(&height),
            "filtered popup follows its content: {height}"
        );
        window.set_rows(rows(0));
        let minimum_empty = if combobox { 55.0 } else { 90.0 };
        assert!(
            surface(&window).size().height >= minimum_empty,
            "empty state keeps a usable viewport"
        );
        window.set_rows(rows(200));
        assert!(
            surface(&window).size().height <= 320.0,
            "large lists keep the finite viewport"
        );
    }
}

#[test]
fn combobox_labels_use_the_full_left_aligned_text_column() {
    i_slint_backend_testing::init_no_event_loop();
    let window = SelectGeometryFixture::new().unwrap();
    window.set_combobox(true);
    window.set_selected_label("".into());
    window.set_rows(rows(3));
    window.show().unwrap();
    let combobox = ElementQuery::from_root(&window)
        .match_inherits("KitCombobox")
        .find_first()
        .unwrap();
    let _ = combobox.size();
    window.set_state(SelectState {
        open: true,
        ..Default::default()
    });
    slint::platform::update_timers_and_animations();
    // Resolve the popup viewport before querying virtualized row delegates.
    let _ = surface(&window).size();
    for list in ElementQuery::from_root(&window)
        .match_inherits("ListView")
        .find_all()
    {
        let _ = list.size();
    }
    let row = ElementQuery::from_root(&window)
        .match_inherits("KitOption")
        .find_first()
        .unwrap();
    let text = row
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();
    assert!(text.size().width > row.size().width * 0.7);
    assert!(
        (text.absolute_position().x - row.absolute_position().x - window.get_row_padding()).abs()
            < 1.0
    );
    let placeholder = combobox
        .query_descendants()
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Workspace"))
        .find_first()
        .expect("inline combobox field exposes its placeholder");
    assert!(placeholder.size().width > 100.0);
    let search = combobox
        .query_descendants()
        .match_type_name("TextInput")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Workspace"))
        .find_first()
        .unwrap();
    window.set_query("a very long query ".repeat(30).into());
    assert!(
        search.is_valid(),
        "query updates preserve the native editor"
    );
}

#[test]
fn combobox_renders_descriptions_loading_with_results_custom_geometry_and_end_reached() {
    i_slint_backend_testing::init_no_event_loop();
    let window = SelectGeometryFixture::new().unwrap();
    window.set_combobox(true);
    window.set_popup_height(220.0);
    window.set_row_height(56.0);
    let reached = Rc::new(Cell::new(0));
    let observed = reached.clone();
    window.on_end_reached(move || observed.set(observed.get() + 1));
    window.set_rows(ModelRc::new(VecModel::from(vec![
        OptionRow {
            token: 1,
            generation: 1,
            label: "Primary".into(),
            description: "Default workspace".into(),
            disabled: false,
        },
        OptionRow {
            token: 2,
            generation: 1,
            label: "Staging".into(),
            description: "Preview environment".into(),
            disabled: false,
        },
    ])));
    window.set_search_state(SearchPresentation::Loading);
    window.set_status_text("Loading more workspaces…".into());
    window.show().unwrap();
    let _ = ElementQuery::from_root(&window)
        .match_inherits("KitCombobox")
        .find_first()
        .unwrap()
        .size();
    window.set_state(SelectState {
        open: true,
        ..Default::default()
    });
    slint::platform::update_timers_and_animations();
    let popup_surface = surface(&window);
    assert!(popup_surface.size().height <= 220.0);
    for list in ElementQuery::from_root(&window)
        .match_inherits("ListView")
        .find_all()
    {
        let _ = list.size();
    }
    let first_row = ElementQuery::from_root(&window)
        .match_inherits("KitOption")
        .find_first()
        .unwrap();
    assert!((first_row.size().height - 56.0).abs() < 1.0);
    assert!(
        first_row
            .query_descendants()
            .match_type_name("Text")
            .find_all()
            .iter()
            .any(|t| t.accessible_label().as_deref() == Some("Default workspace")),
        "combobox option renders row.description"
    );
    assert!(
        popup_surface
            .query_descendants()
            .match_inherits("KitSpinner")
            .find_first()
            .is_some(),
        "popup shows spinner while search-state is loading even when rows exist"
    );
    assert!(
        reached.get() >= 1,
        "end-reached fires when visible rows reach the end of the list"
    );
}

#[test]
fn combobox_defaults_follow_theme_metrics_and_window_strings_with_the_native_editor_focused() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::platform::{Key, WindowEvent};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = crate::ComboboxDefaultsFixture::new().unwrap();
    slint_kit::install(&app);
    app.global::<crate::ui::KitStrings>()
        .set_busy_label("Chargement…".into());
    app.show().unwrap();
    app.invoke_focus_editor();
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::DownArrow.into(),
    });
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::DownArrow.into(),
    });
    let mut normal_height = 0.;
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            slint::platform::update_timers_and_animations();
            assert_eq!(app.get_loading_label(), "Chargement…");
            assert!(app.get_editor_focused());
            assert!(
                (app.get_popup_height()
                    - 6. * app.get_row_height()
                    - 2. * app.get_expected_inset())
                .abs()
                    < 0.01
            );
            if scale == 1. {
                normal_height = app.get_row_height();
            } else {
                assert!(app.get_row_height() > normal_height);
            }
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("combobox-defaults-{name}-{scale}"),
            );
        }
    }
}
