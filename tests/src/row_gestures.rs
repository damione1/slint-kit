// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use crate::base_data::{column_set, fixture, key, labelled, rows};
use kit_core::{RowGesture, SelectionMode};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use slint_kit::{KeyedModel, ui};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Record(u32);

fn table_row(label: &str, disabled: bool) -> ui::TableRow {
    ui::TableRow {
        cells: ModelRc::new(VecModel::from(vec![label.into()])),
        disabled,
        ..Default::default()
    }
}
fn model(count: u32) -> KeyedModel<Record, ui::TableRow> {
    KeyedModel::new((1..=count).map(|n| (Record(n), table_row(&format!("Row {n}"), false))))
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
fn click(window: &BaseDataFixture, label: &str, modifier: Option<Key>, double: bool) {
    let element = labelled(window, label);
    let origin = element.absolute_position();
    let position = LogicalPosition::new(
        origin.x + element.size().width / 2.,
        origin.y + element.size().height / 2.,
    );
    if let Some(modifier) = modifier {
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: modifier.into(),
        });
    }
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    for _ in 0..if double { 2 } else { 1 } {
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
    if let Some(modifier) = modifier {
        window.window().dispatch_event(WindowEvent::KeyReleased {
            text: modifier.into(),
        });
    }
}

#[test]
fn keyed_model_keeps_several_rows_selected_and_extends_from_an_anchor() {
    let model = model(6);
    let generation = model.generation();
    let t = |n| token(&model, n);
    model.select(t(2), generation);
    model.toggle(t(4), generation);
    assert_eq!(model.selected_keys(), vec![Record(2), Record(4)]);
    // A range pivots around the last plain or toggled row (4), not the last extension.
    model.select_range(t(6), generation);
    assert_eq!(model.selected_keys(), vec![Record(4), Record(5), Record(6)]);
    model.select_range(t(3), generation);
    assert_eq!(model.selected_keys(), vec![Record(3), Record(4)]);
    model.toggle(t(4), generation);
    assert_eq!(model.selected_keys(), vec![Record(3)]);
    model.select_all();
    assert_eq!(model.selected_keys().len(), 6);
    model.clear_selection();
    assert!(model.selected_keys().is_empty());
    // Without an anchor an extension selects the row alone.
    model.select_range(t(5), generation);
    assert_eq!(model.selected_keys(), vec![Record(5)]);
}

#[test]
fn bulk_selection_skips_disabled_rows_and_rejects_stale_gestures() {
    let model = KeyedModel::new([
        (Record(1), table_row("a", false)),
        (Record(2), table_row("b", true)),
        (Record(3), table_row("c", false)),
    ])
    .unwrap();
    let generation = model.generation();
    let first = token(&model, 1);
    model.select(first, generation);
    model.select_range(token(&model, 3), generation);
    assert_eq!(model.selected_keys(), vec![Record(1), Record(3)]);
    model.select_all();
    assert_eq!(model.selected_keys(), vec![Record(1), Record(3)]);
    assert_eq!(model.toggle(token(&model, 2), generation), None);
    assert_eq!(model.toggle(first, generation + 1), None);
    assert_eq!(model.selected_keys(), vec![Record(1), Record(3)]);
    // Removing the anchor forgets it.
    model.remove_by_key(&Record(1));
    model.select_range(token(&model, 3), model.generation());
    assert_eq!(model.selected_keys(), vec![Record(3)]);
}

#[test]
fn the_mode_decides_how_a_gesture_changes_the_selection() {
    let single = model(4);
    let generation = single.generation();
    let s = |n| token(&single, n);
    single.gesture(SelectionMode::Single, RowGesture::Select, s(1), generation);
    single.gesture(SelectionMode::Single, RowGesture::Toggle, s(2), generation);
    single.gesture(SelectionMode::Single, RowGesture::Extend, s(4), generation);
    assert_eq!(single.selected_keys(), vec![Record(4)]);
    let multiple = model(4);
    let generation = multiple.generation();
    let t = |n| token(&multiple, n);
    multiple.gesture(
        SelectionMode::Multiple,
        RowGesture::Select,
        t(1),
        generation,
    );
    multiple.gesture(
        SelectionMode::Multiple,
        RowGesture::Toggle,
        t(3),
        generation,
    );
    assert_eq!(multiple.selected_keys(), vec![Record(1), Record(3)]);
    // Opening a selected row keeps the selection; opening another row replaces it.
    multiple.gesture(SelectionMode::Multiple, RowGesture::Open, t(1), generation);
    assert_eq!(multiple.selected_keys(), vec![Record(1), Record(3)]);
    multiple.gesture(SelectionMode::Multiple, RowGesture::Open, t(2), generation);
    assert_eq!(multiple.selected_keys(), vec![Record(2)]);
}

#[test]
fn pointer_modifiers_and_double_click_become_typed_table_gestures() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 5))));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = seen.clone();
    window.on_table_gesture(move |token, _, gesture| {
        observed
            .borrow_mut()
            .push((token, RowGesture::from(gesture)))
    });
    click(&window, "Product 1", None, false);
    click(&window, "Product 2", Some(Key::Control), false);
    click(&window, "Product 3", Some(Key::Shift), false);
    click(&window, "Product 4", Some(Key::Meta), false);
    assert_eq!(
        &*seen.borrow(),
        &[
            (1, RowGesture::Select),
            (2, RowGesture::Toggle),
            (3, RowGesture::Extend),
            (4, RowGesture::Toggle)
        ]
    );
    seen.borrow_mut().clear();
    click(&window, "Product 1", None, true);
    assert_eq!(seen.borrow().last(), Some(&(1, RowGesture::Open)));
    // Keyboard: Space toggles and Enter opens the active row.
    seen.borrow_mut().clear();
    window.set_table_active(1);
    window.invoke_focus_table();
    key(&window, Key::Space);
    key(&window, Key::Return);
    assert_eq!(
        &*seen.borrow(),
        &[(2, RowGesture::Toggle), (2, RowGesture::Open)]
    );
}

fn right_click(window: &BaseDataFixture, label: &str) -> LogicalPosition {
    let element = labelled(window, label);
    let origin = element.absolute_position();
    let position = LogicalPosition::new(
        origin.x + element.size().width / 2.,
        origin.y + element.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Right,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Right,
        });
    position
}

#[test]
fn a_secondary_click_reports_the_row_and_the_pointer_point_in_table_coordinates() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    let mut data = rows(1, 5);
    data[3].disabled = true;
    window.set_rows(ModelRc::new(VecModel::from(data)));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = seen.clone();
    window.on_table_context(move |token, _, x, y| observed.borrow_mut().push((token, x, y)));
    let gestures = Rc::new(RefCell::new(0));
    let counted = gestures.clone();
    window.on_table_gesture(move |_, _, _| *counted.borrow_mut() += 1);
    let table = labelled(&window, "Products fixture").absolute_position();
    let point = right_click(&window, "Product 2");
    {
        let seen = seen.borrow();
        assert_eq!(seen.len(), 1);
        let (token, x, y) = seen[0];
        assert_eq!(token, 2);
        assert!((x - (point.x - table.x)).abs() < 1.5, "x {x}");
        assert!((y - (point.y - table.y)).abs() < 1.5, "y {y}");
    }
    // The secondary button selects and activates nothing; a disabled row reports nothing.
    assert_eq!(*gestures.borrow(), 0);
    right_click(&window, "Product 4");
    assert_eq!(seen.borrow().len(), 1);
}

#[test]
fn the_menu_key_reports_the_active_row_inside_the_table() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 5))));
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = seen.clone();
    window.on_table_context(move |token, _, x, y| observed.borrow_mut().push((token, x, y)));
    let table = labelled(&window, "Products fixture");
    window.set_table_active(2);
    window.invoke_focus_table();
    key(&window, Key::Menu);
    let row = labelled(&window, "Product 3");
    let expected = row.absolute_position().y + row.size().height / 2. - table.absolute_position().y;
    let seen = seen.borrow();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, 3);
    assert!(
        (seen[0].2 - expected).abs() < 1.5,
        "y {} vs {expected}",
        seen[0].2
    );
    assert!(seen[0].1 > 0. && seen[0].1 < table.size().width);
}

#[test]
fn context_requests_resolve_the_row_key_and_drop_stale_generations() {
    let model = model(3);
    let generation = model.generation();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let observed = seen.clone();
    let handler = model.context_requests(move |key, x, y| observed.borrow_mut().push((key, x, y)));
    handler(token(&model, 2), generation, 10., 20.);
    handler(token(&model, 3), generation + 1, 1., 2.);
    assert_eq!(&*seen.borrow(), &[(Record(2), 10., 20.)]);
    assert!(model.selected_keys().is_empty());
}
