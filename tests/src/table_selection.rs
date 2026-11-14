// SPDX-License-Identifier: GPL-3.0-or-later
//! The selection column of `KitDataTable`: a kit-painted checkbox per row and
//! a tri-state select-all header driven by `KeyedModel`, plus the selection
//! mode that decides whether rows are announced as selectable. These are
//! headless contract proofs; they do not qualify a screen reader.
use super::*;
use crate::base_data::{column_set, down, fixture, key, labelled, rows, text, up};
use kit_core::{CheckState, RowGesture, SelectionMode};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use slint_kit::{KeyedModel, ui};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Record(u32);

fn table_row(label: &str, disabled: bool) -> ui::TableRow {
    ui::TableRow {
        cells: ModelRc::new(VecModel::from(vec![label.into(), "1".into()])),
        disabled,
        ..Default::default()
    }
}
fn model(disabled: &[u32]) -> KeyedModel<Record, ui::TableRow> {
    KeyedModel::new((1..=4).map(|n| {
        (
            Record(n),
            table_row(&format!("Product {n}"), disabled.contains(&n)),
        )
    }))
    .unwrap()
}
fn token(model: &KeyedModel<Record, ui::TableRow>, record: u32) -> i32 {
    let index = model.index_of(&Record(record)).unwrap();
    model
        .model()
        .row_data(usize::try_from(index).unwrap())
        .unwrap()
        .token
}
fn click_at(window: &BaseDataFixture, x: f32, y: f32) {
    let position = LogicalPosition::new(x, y);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}
// The checkbox cell is the first, row-high square of a row.
fn click_row_checkbox(window: &BaseDataFixture, label: &str) {
    let row = labelled(window, label);
    let origin = row.absolute_position();
    let side = row.size().height;
    click_at(window, origin.x + side / 2., origin.y + side / 2.);
}
fn selecting_fixture() -> BaseDataFixture {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 4))));
    window.set_table_selection_mode(ui::SelectionMode::Multiple);
    window.set_table_selection_column(true);
    window
}

#[test]
fn the_select_all_state_is_checked_mixed_or_unchecked_over_enabled_rows() {
    let model = model(&[4]);
    let generation = model.generation();
    assert_eq!(model.selection_state(), CheckState::Unchecked);
    model.toggle(token(&model, 1), generation);
    assert_eq!(model.selection_state(), CheckState::Mixed);
    model.select_all();
    // The disabled row stays unselected and does not keep the header mixed.
    assert_eq!(model.selected_keys(), vec![Record(1), Record(2), Record(3)]);
    assert_eq!(model.selection_state(), CheckState::Checked);
    model.toggle(token(&model, 2), generation);
    assert_eq!(model.selection_state(), CheckState::Mixed);
    model.clear_selection();
    assert_eq!(model.selection_state(), CheckState::Unchecked);
    // Only disabled rows: nothing can be selected.
    let none = KeyedModel::new([(Record(1), table_row("Locked", true))]).unwrap();
    assert_eq!(none.selection_state(), CheckState::Unchecked);
}

#[test]
fn select_all_toggle_selects_when_partial_and_clears_when_complete() {
    let model = model(&[]);
    let toggle = model.select_all_toggle();
    toggle();
    assert_eq!(model.selection_state(), CheckState::Checked);
    toggle();
    assert_eq!(model.selection_state(), CheckState::Unchecked);
    model.toggle(token(&model, 2), model.generation());
    toggle();
    assert_eq!(model.selection_state(), CheckState::Checked);
}

#[test]
fn selection_mode_none_only_moves_the_active_row() {
    let model = model(&[]);
    let generation = model.generation();
    for gesture in [
        RowGesture::Select,
        RowGesture::Toggle,
        RowGesture::Extend,
        RowGesture::Open,
    ] {
        assert_eq!(
            model.gesture(SelectionMode::None, gesture, token(&model, 3), generation),
            Some(Record(3))
        );
    }
    assert!(model.selected_keys().is_empty());
    assert_eq!(model.active_index(), 2);
    // A stale generation is still fenced.
    assert_eq!(
        model.gesture(
            SelectionMode::None,
            RowGesture::Select,
            token(&model, 1),
            generation + 1
        ),
        None
    );
}

#[test]
fn rows_are_selectable_only_while_a_selection_mode_is_set() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    for (mode, selectable) in [
        (ui::SelectionMode::None, false),
        (ui::SelectionMode::Single, true),
        (ui::SelectionMode::Multiple, true),
    ] {
        window.set_table_selection_mode(mode);
        assert_eq!(
            labelled(&window, "Product 2").accessible_item_selectable(),
            Some(selectable)
        );
    }
}

#[test]
fn the_column_exists_only_for_multiple_selection_and_keeps_header_and_rows_aligned() {
    let window = selecting_fixture();
    let header = labelled(&window, "Select all rows");
    assert_eq!(
        header.accessible_role(),
        Some(i_slint_backend_testing::AccessibleRole::Checkbox)
    );
    assert_eq!(header.accessible_checkable(), Some(true));
    let product_header = labelled(&window, "Product, ascending");
    let with_column = product_header.absolute_position().x;
    let row_side = labelled(&window, "Product 1").size().height;
    assert_eq!(header.size().width, row_side);
    // The first column starts one checkbox cell later in header and rows alike.
    window.set_table_selection_column(false);
    assert!(
        i_slint_backend_testing::ElementQuery::from_root(&window)
            .match_accessible_role(i_slint_backend_testing::AccessibleRole::Checkbox)
            .find_all()
            .is_empty()
    );
    let without_column = labelled(&window, "Product, ascending")
        .absolute_position()
        .x;
    assert_eq!(with_column - without_column, row_side);
    // Single selection has no select-all, so no column either.
    window.set_table_selection_column(true);
    window.set_table_selection_mode(ui::SelectionMode::Single);
    assert!(
        i_slint_backend_testing::ElementQuery::from_root(&window)
            .match_accessible_role(i_slint_backend_testing::AccessibleRole::Checkbox)
            .find_all()
            .is_empty()
    );
}

#[test]
fn the_header_reports_select_all_and_names_a_partial_selection() {
    let window = selecting_fixture();
    let requests = Rc::new(RefCell::new(0));
    let observed = requests.clone();
    window.on_table_select_all(move || *observed.borrow_mut() += 1);
    let header = labelled(&window, "Select all rows");
    assert_eq!(header.accessible_checked(), Some(false));
    let origin = header.absolute_position();
    click_at(
        &window,
        origin.x + header.size().width / 2.,
        origin.y + header.size().height / 2.,
    );
    assert_eq!(*requests.borrow(), 1);
    // Keyboard: Space on the focused header, and Control+A from the rows.
    window.invoke_focus_table();
    down(&window, Key::Control);
    text(&window, "a");
    up(&window, Key::Control);
    assert_eq!(*requests.borrow(), 2);
    window.set_table_selection_state(ui::CheckState::Mixed);
    let header = labelled(&window, "Select all rows");
    assert_eq!(header.accessible_checked(), Some(false));
    assert_eq!(
        header.accessible_description().as_deref(),
        Some("Some rows selected")
    );
    window.set_table_selection_state(ui::CheckState::Checked);
    assert_eq!(
        labelled(&window, "Select all rows").accessible_checked(),
        Some(true)
    );
}

#[test]
fn a_checkbox_click_is_a_toggle_gesture_and_never_replaces_the_selection() {
    let window = selecting_fixture();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = seen.clone();
    window.on_table_gesture(move |token, _, gesture| {
        observed
            .borrow_mut()
            .push((token, RowGesture::from(gesture)))
    });
    click_row_checkbox(&window, "Product 2");
    click_row_checkbox(&window, "Product 3");
    assert_eq!(
        &*seen.borrow(),
        &[(2, RowGesture::Toggle), (3, RowGesture::Toggle)]
    );
    // Shift on the checkbox extends the selection; Space still toggles.
    down(&window, Key::Shift);
    click_row_checkbox(&window, "Product 4");
    up(&window, Key::Shift);
    window.set_table_active(1);
    window.invoke_focus_table();
    key(&window, Key::Space);
    assert_eq!(
        &seen.borrow()[2..],
        &[(4, RowGesture::Extend), (2, RowGesture::Toggle)]
    );
}

#[test]
fn a_disabled_row_ignores_its_checkbox() {
    let window = selecting_fixture();
    let mut all = rows(1, 4);
    all[1].disabled = true;
    window.set_rows(ModelRc::new(VecModel::from(all)));
    let seen = Rc::new(RefCell::new(0));
    let observed = seen.clone();
    window.on_table_gesture(move |_, _, _| *observed.borrow_mut() += 1);
    click_row_checkbox(&window, "Product 2");
    assert_eq!(*seen.borrow(), 0);
    click_row_checkbox(&window, "Product 1");
    assert_eq!(*seen.borrow(), 1);
}

#[test]
fn a_bound_model_drives_the_column_end_to_end() {
    let window = selecting_fixture();
    let model = model(&[]);
    window.set_rows(model.model());
    model.bind_selection_state(&window, BaseDataFixture::set_table_selection_state);
    window.on_table_select_all({
        let toggle = model.select_all_toggle();
        move || toggle()
    });
    window.on_table_gesture({
        let model = model.clone();
        move |token, generation, gesture| {
            model.gesture(
                SelectionMode::Multiple,
                RowGesture::from(gesture),
                token,
                generation,
            );
        }
    });
    window.set_table_generation(model.generation());
    let header_state = |window: &BaseDataFixture| window.get_table_selection_state();
    assert_eq!(header_state(&window), ui::CheckState::Unchecked);
    click_row_checkbox(&window, "Product 1");
    assert_eq!(model.selected_keys(), vec![Record(1)]);
    assert_eq!(header_state(&window), ui::CheckState::Mixed);
    click_row_checkbox(&window, "Product 2");
    assert_eq!(model.selected_keys(), vec![Record(1), Record(2)]);
    let header = labelled(&window, "Select all rows");
    let origin = header.absolute_position();
    click_at(&window, origin.x + 4., origin.y + 4.);
    assert_eq!(header_state(&window), ui::CheckState::Checked);
    assert_eq!(model.selected_keys().len(), 4);
    let origin = labelled(&window, "Select all rows").absolute_position();
    click_at(&window, origin.x + 4., origin.y + 4.);
    assert_eq!(header_state(&window), ui::CheckState::Unchecked);
    assert!(model.selected_keys().is_empty());
}
