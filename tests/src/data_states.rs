// SPDX-License-Identifier: GPL-3.0-or-later
use super::base_data::{fixture, items, rows};
use super::*;
use i_slint_backend_testing::ElementQuery;
use slint::{ModelRc, VecModel};
use std::{cell::Cell, rc::Rc};

fn present(window: &BaseDataFixture, label: &str) -> bool {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .is_some()
}

#[test]
fn table_replaces_the_empty_message_while_loading_and_after_a_failure() {
    let window = fixture();
    assert!(present(&window, "No matching records"));
    window.set_table_state(DataState::Loading);
    assert!(present(&window, "Loading"));
    assert!(!present(&window, "No matching records"));
    assert!(!present(&window, "Retry"));
    window.set_table_state(DataState::Failed);
    assert!(present(&window, "The data could not be loaded"));
    assert!(present(&window, "Retry"));
    assert!(!present(&window, "Loading"));
    window.set_table_state(DataState::Ready);
    assert!(present(&window, "No matching records"));
    assert!(!present(&window, "Retry"));
}

#[test]
fn table_retry_action_reports_a_request() {
    let window = fixture();
    let retries = Rc::new(Cell::new(0));
    window.on_table_retry({
        let retries = retries.clone();
        move || retries.set(retries.get() + 1)
    });
    window.set_table_state(DataState::Failed);
    super::base_data::labelled(&window, "Retry").invoke_accessible_default_action();
    assert_eq!(retries.get(), 1);
}

#[test]
fn table_shows_custom_children_only_for_a_ready_empty_table() {
    let window = fixture();
    assert!(!present(&window, "Table custom empty"));
    window.set_table_empty_content(DataEmptyContent::Custom);
    assert!(present(&window, "Table custom empty"));
    assert!(!present(&window, "No matching records"));
    window.set_table_state(DataState::Loading);
    assert!(!present(&window, "Table custom empty"));
    window.set_table_state(DataState::Ready);
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    assert!(!present(&window, "Table custom empty"));
}

#[test]
fn rows_stay_visible_while_the_table_reloads() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_state(DataState::Loading);
    assert!(!present(&window, "Loading"));
    assert!(!super::base_data::delegates(&window, "KitDataTable").is_empty());
}

#[test]
fn content_list_has_the_same_states_and_custom_empty_content() {
    let window = fixture();
    window.set_content_empty_label("Nothing published".into());
    assert!(present(&window, "Nothing published"));
    window.set_content_empty_content(DataEmptyContent::Custom);
    assert!(present(&window, "Content custom empty"));
    assert!(!present(&window, "Nothing published"));
    window.set_content_empty_content(DataEmptyContent::Label);
    let retries = Rc::new(Cell::new(0));
    window.on_content_retry({
        let retries = retries.clone();
        move || retries.set(retries.get() + 1)
    });
    window.set_content_state(DataState::Failed);
    assert!(present(&window, "The articles could not be loaded"));
    super::base_data::labelled(&window, "Retry").invoke_accessible_default_action();
    assert_eq!(retries.get(), 1);
    window.set_content_state(DataState::Loading);
    assert!(present(&window, "Loading"));
    window.set_items(ModelRc::new(VecModel::from(items(2))));
    assert!(!present(&window, "Loading"));
}
