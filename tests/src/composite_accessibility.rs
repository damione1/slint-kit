// SPDX-License-Identifier: GPL-3.0-or-later
//! Public semantic projection plus native lifecycle; no screen-reader verdict.
use crate::{ChoiceDirection, ChoiceFlow, CompositeAccessibilityFixture, TabActivation};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle,
    platform::{Key, WindowEvent},
};
use slint_kit::{Choices, SelectOption};

fn options() -> Vec<SelectOption<u32>> {
    (0..5)
        .map(|key| {
            let mut row = SelectOption::new(key, format!("Choice {key}"));
            row.disabled = key == 1;
            row
        })
        .collect()
}
fn fixture() -> (CompositeAccessibilityFixture, Choices<u32>) {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let app = CompositeAccessibilityFixture::new().unwrap();
    slint_kit::install(&app);
    let choices = Choices::from_options(options()).unwrap();
    app.set_options(choices.model());
    app.set_bar_selected(choices.selection(&0).unwrap());
    app.set_tabs_selected(choices.selection(&0).unwrap());
    app.show().unwrap();
    settle(&app);
    (app, choices)
}
fn settle(app: &CompositeAccessibilityFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &CompositeAccessibilityFixture, text: impl Into<slint::SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn component(app: &CompositeAccessibilityFixture, name: &str) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_inherits(name)
        .find_first()
        .unwrap()
}
fn assert_positions(group: &ElementHandle, option: &str, reversed: bool) {
    let rows = group.query_descendants().match_inherits(option).find_all();
    assert_eq!(rows.len(), 5);
    for (physical, row) in rows.iter().enumerate() {
        let logical = if reversed { 4 - physical } else { physical };
        assert_eq!(row.accessible_item_index(), Some(logical));
        assert_eq!(row.accessible_item_count(), Some(5));
        assert_eq!(
            row.accessible_label().as_deref(),
            Some(format!("Choice {logical}").as_str())
        );
    }
}
fn capture(app: &CompositeAccessibilityFixture) {
    let Some(directory) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    use std::io::Write;
    for (name, base) in [
        ("light", app.get_light_theme()),
        ("dark", app.get_dark_theme()),
    ] {
        for scale in [1, 2] {
            let mut theme = base.clone();
            theme.text_scale = if scale == 1 { 1. } else { 2. };
            app.set_theme(theme);
            app.invoke_focus_radio();
            key(app, Key::End);
            let image = app.window().take_snapshot().unwrap();
            let mut file = std::fs::File::create(
                std::path::PathBuf::from(&directory)
                    .join(format!("composite-static-focus-{name}-{scale}.ppm")),
            )
            .unwrap();
            write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
            for pixel in image.as_slice() {
                file.write_all(&[pixel.r, pixel.g, pixel.b]).unwrap();
            }
        }
    }
}

#[test]
fn static_composite_delegation_matches_physical_order_while_positions_keep_logical_order() {
    let (app, choices) = fixture();
    for (flow, reversed) in [
        (ChoiceFlow::LeftToRight, false),
        (ChoiceFlow::RightToLeft, true),
    ] {
        app.set_flow(flow);
        app.set_direction(ChoiceDirection::Horizontal);
        settle(&app);
        app.invoke_focus_bar();
        key(&app, Key::End);
        assert_eq!(app.get_bar_active(), choices.selection(&4).unwrap().token);
        assert_eq!(app.get_bar_delegate(), if reversed { 0 } else { 4 });
        assert_eq!(app.get_bar_selected(), choices.selection(&0).unwrap());
        assert_positions(
            &component(&app, "KitMenuBar"),
            "KitNavigationAction",
            reversed,
        );
        app.invoke_focus_tabs();
        key(&app, Key::End);
        assert_eq!(app.get_tabs_active(), choices.selection(&4).unwrap().token);
        assert_eq!(app.get_tabs_delegate(), if reversed { 0 } else { 4 });
        assert_eq!(app.get_tabs_selected(), choices.selection(&0).unwrap());
        assert_positions(&component(&app, "KitTabs"), "KitNavigationAction", reversed);
        app.set_activation(TabActivation::Automatic);
        key(&app, Key::Home);
        assert_eq!(
            app.get_tabs_selected(),
            choices.selection(&0).unwrap(),
            "controlled tabs await host acknowledgment"
        );
        app.set_activation(TabActivation::Manual);
        app.invoke_focus_radio();
        key(&app, Key::Home);
        key(
            &app,
            if reversed {
                Key::LeftArrow
            } else {
                Key::RightArrow
            },
        );
        assert_eq!(
            app.get_radio_active(),
            choices.selection(&2).unwrap().token,
            "disabled row is skipped"
        );
        assert_positions(&component(&app, "KitRadioGroup"), "ChoiceOption", reversed);
        app.invoke_focus_toggles();
        key(&app, Key::End);
        assert_eq!(
            app.get_toggles_active(),
            choices.selection(&4).unwrap().token
        );
        assert_positions(&component(&app, "KitToggleGroup"), "ChoiceOption", reversed);
        app.invoke_focus_segments();
        key(&app, Key::End);
        assert_eq!(
            app.get_segments_delegate(),
            4,
            "the second row maps its materialized grid slot"
        );
        key(
            &app,
            if reversed {
                Key::RightArrow
            } else {
                Key::LeftArrow
            },
        );
        assert_eq!(
            app.get_segments_active(),
            choices.selection(&3).unwrap().token
        );
        assert_eq!(
            app.get_segments_delegate(),
            if reversed { 5 } else { 3 },
            "a partial RTL row includes its materialized unused slot"
        );
    }
    app.set_direction(ChoiceDirection::Vertical);
    app.set_flow(ChoiceFlow::LeftToRight);
    settle(&app);
    assert_positions(&component(&app, "KitRadioGroup"), "ChoiceOption", false);
    assert_positions(&component(&app, "KitToggleGroup"), "ChoiceOption", false);
    let surface = component(&app, "KitSelectSurface");
    assert_eq!(
        surface.accessible_label().as_deref(),
        Some("Composed choices")
    );
    assert_positions(&surface, "KitOption", false);
    let rating = component(&app, "KitRating");
    let scores = rating
        .query_descendants()
        .match_inherits("SegmentedOption")
        .find_all();
    assert_eq!(scores.len(), 5);
    for (index, score) in scores.iter().enumerate() {
        assert_eq!(score.accessible_item_index(), Some(index));
        assert_eq!(score.accessible_item_count(), Some(5));
    }
    for (id, label) in [
        ("select", "Select choices"),
        ("combo", "Editable choices"),
        ("multi", "Multiple choices"),
    ] {
        let host = ElementQuery::from_root(&app)
            .match_id(format!("CompositeAccessibilityFixture::{id}"))
            .find_first()
            .unwrap();
        let expander = if host.accessible_expandable() == Some(true) {
            host.clone()
        } else {
            host.query_descendants()
                .match_predicate(|item| item.accessible_expandable() == Some(true))
                .find_first()
                .unwrap()
        };
        expander.invoke_accessible_expand_action();
        settle(&app);
        let popup = host
            .query_descendants()
            .match_inherits("KitSelectSurface")
            .find_first()
            .unwrap();
        assert_eq!(popup.accessible_label().as_deref(), Some(label));
        assert_positions(&popup, "KitOption", false);
        if id == "combo" {
            key(&app, "Ch");
            assert_eq!(
                app.get_combo_text(),
                "Ch",
                "popup option metadata must preserve native editing"
            );
        }
        key(&app, Key::Escape);
    }
    capture(&app);
}

#[test]
fn persistent_composite_scopes_survive_model_replacement_space_and_single_tab_exit() {
    let (app, choices) = fixture();
    for horizontal in [false, true] {
        app.set_direction(if horizontal {
            ChoiceDirection::Horizontal
        } else {
            ChoiceDirection::Vertical
        });
        app.invoke_focus_radio();
        key(&app, Key::End);
        assert!(app.get_radio_focused());
        app.set_direction(if horizontal {
            ChoiceDirection::Vertical
        } else {
            ChoiceDirection::Horizontal
        });
        settle(&app);
        let count = app.get_chosen_count();
        key(&app, Key::Space);
        assert_eq!(
            app.get_chosen_count(),
            count + 1,
            "an orientation change preserves the owned native keyboard scope"
        );
        choices.replace(options()).unwrap();
        settle(&app);
        let count = app.get_chosen_count();
        key(&app, Key::Space);
        assert_eq!(
            app.get_chosen_count(),
            count + 1,
            "Space still reaches the native persistent radio owner"
        );
        assert!(app.get_radio_focused());
        key(&app, Key::Tab);
        assert!(
            app.get_segments_focused(),
            "Tab leaves radio at one composite stop"
        );
        key(&app, Key::Backtab);
        assert!(app.get_radio_focused());
        app.invoke_focus_toggles();
        key(&app, Key::End);
        app.set_direction(if horizontal {
            ChoiceDirection::Horizontal
        } else {
            ChoiceDirection::Vertical
        });
        settle(&app);
        let count = app.get_chosen_count();
        key(&app, Key::Space);
        assert_eq!(
            app.get_chosen_count(),
            count + 1,
            "toggle orientation does not lose the next native key"
        );
        choices.replace(options()).unwrap();
        settle(&app);
        let count = app.get_chosen_count();
        key(&app, Key::Space);
        assert_eq!(
            app.get_chosen_count(),
            count + 1,
            "Space survives a new toggle projection"
        );
        assert!(app.get_toggles_focused());
    }
    app.invoke_focus_bar();
    key(&app, Key::End);
    choices.replace(options()).unwrap();
    settle(&app);
    let count = app.get_chosen_count();
    key(&app, Key::Space);
    assert_eq!(app.get_chosen_count(), count + 1);
    assert!(app.get_bar_focused());
}

#[test]
fn static_composite_host_redirection_and_background_replacement_preserve_native_editing() {
    let (app, choices) = fixture();
    app.set_redirect(true);
    let focus_actions: [fn(&CompositeAccessibilityFixture); 5] = [
        CompositeAccessibilityFixture::invoke_focus_radio,
        CompositeAccessibilityFixture::invoke_focus_toggles,
        CompositeAccessibilityFixture::invoke_focus_bar,
        CompositeAccessibilityFixture::invoke_focus_tabs,
        CompositeAccessibilityFixture::invoke_focus_segments,
    ];
    for focus in focus_actions {
        focus(&app);
        key(&app, Key::End);
        assert!(
            app.get_editor_focused(),
            "the host navigation callback owns the final native focus"
        );
        assert!(
            !app.get_radio_focused()
                && !app.get_toggles_focused()
                && !app.get_bar_focused()
                && !app.get_tabs_focused()
                && !app.get_segments_focused()
        );
        choices.replace(options()).unwrap();
        app.set_direction(if app.get_direction() == ChoiceDirection::Horizontal {
            ChoiceDirection::Vertical
        } else {
            ChoiceDirection::Horizontal
        });
        settle(&app);
        assert!(
            app.get_editor_focused(),
            "background reconciliation cannot reclaim the editor"
        );
    }
    app.invoke_select_editor();
    assert_eq!((app.get_editor_anchor(), app.get_editor_cursor()), (1, 4));
    key(&app, "X");
    assert_eq!(app.get_editor_text(), "nXve");
    app.invoke_undo_editor();
    assert_eq!(app.get_editor_text(), "nve");
    app.invoke_undo_editor();
    assert_eq!(app.get_editor_text(), "native");
}
