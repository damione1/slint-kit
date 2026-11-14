// SPDX-License-Identifier: GPL-3.0-or-later
//! Toggle, action and link cells of `KitDataTable`: typed projection,
//! activation from the cell cursor and the pointer, the `cell-action`
//! fence (generation, access, disabled rows) and the guarantee that a cell
//! activation never selects or opens its row. Headless contract proofs; they
//! do not qualify a screen reader.
use super::base_data::{fixture, key};
use super::*;
use i_slint_backend_testing::ElementHandle;
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{LogicalPosition, Model, ModelRc, VecModel};
use slint_kit::{CellContent, FieldAccess, table_cells};
use std::{cell::RefCell, rc::Rc};

type Actions = Rc<RefCell<Vec<(i32, i32, i32)>>>;
type Gestures = Rc<RefCell<Vec<(i32, RowGesture)>>>;

fn column(token: i32, label: &str) -> TableColumn {
    TableColumn {
        visibility: Default::default(),
        source: Default::default(),
        token,
        label: label.into(),
        sortable: false,
        sort: DataSortDirection::Unsorted,
        alignment: DataAlignment::Leading,
        width: ColumnWidth {
            sizing: ColumnSizing::Fixed,
            basis: 160.,
            ..ColumnWidth::default()
        },
    }
}
fn glyph() -> IconGlyph {
    IconGlyph {
        stroke: "M2 12 L22 12".into(),
        fill: "".into(),
        viewbox: 24.,
    }
}
fn row(token: i32, name: &str, access: FieldAccess, disabled: bool) -> TableRow {
    TableRow {
        token,
        cells: ModelRc::default(),
        content: table_cells([
            CellContent::text(name),
            CellContent::toggle(false, format!("Done {name}"), access),
            CellContent::action(glyph(), format!("Open {name}"), access),
            CellContent::link(format!("Docs {name}"), access),
        ]),
        selected: false,
        disabled,
    }
}
struct Table {
    window: BaseDataFixture,
    actions: Actions,
    gestures: Gestures,
}
fn load(rows: Vec<TableRow>) -> Table {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(vec![
        column(1, "Name"),
        column(2, "Done"),
        column(3, "Open"),
        column(4, "Docs"),
    ])));
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window.set_table_generation(7);
    let actions: Actions = Rc::default();
    let recorded = actions.clone();
    window.on_cell_action(move |cell| {
        recorded
            .borrow_mut()
            .push((cell.row, cell.column, cell.generation));
    });
    let gestures: Gestures = Rc::default();
    let observed = gestures.clone();
    window.on_table_gesture(move |token, _, gesture| observed.borrow_mut().push((token, gesture)));
    Table {
        window,
        actions,
        gestures,
    }
}
fn cursor(table: &Table, row: i32, column: i32) {
    table.window.set_table_active(row);
    table.window.set_table_active_column(column);
    table.window.invoke_focus_table();
}
fn cell(table: &Table, row: usize, column: usize) -> ElementHandle {
    let per_row = 4;
    ElementHandle::find_by_element_id(&table.window, "KitDataTable::content-cell")
        .nth(row * per_row + column)
        .expect("cell")
}
fn click(table: &Table, target: &ElementHandle) {
    let origin = target.absolute_position();
    let position = LogicalPosition::new(
        origin.x + target.size().width / 2.,
        origin.y + target.size().height / 2.,
    );
    table
        .window
        .window()
        .dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
    slint::platform::update_timers_and_animations();
    table
        .window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(50));
    slint::platform::update_timers_and_animations();
}

#[test]
fn interactive_cells_convert_once_into_their_flat_projection() {
    let cells = table_cells([
        CellContent::toggle(true, "Done", FieldAccess::Editable),
        CellContent::toggle(false, "Done", FieldAccess::ReadOnly),
        CellContent::action(glyph(), "Open", FieldAccess::Editable),
        CellContent::text_action("Archive", FieldAccess::Disabled),
        CellContent::link("Docs", FieldAccess::Editable),
    ]);
    let cell = |index| cells.row_data(index).expect("cell");
    assert_eq!(cell(0).kind, TableCellKind::Toggle);
    assert_eq!(cell(0).check, CheckState::Checked);
    assert_eq!(cell(1).check, CheckState::Unchecked);
    assert_eq!(cell(1).access, ui::FieldAccess::ReadOnly);
    assert_eq!(cell(2).kind, TableCellKind::Action);
    assert_eq!(cell(2).glyph, glyph());
    assert_eq!(cell(3).glyph, IconGlyph::default());
    assert_eq!(cell(3).access, ui::FieldAccess::Disabled);
    assert_eq!(cell(4).kind, TableCellKind::Link);
    assert_eq!(cell(4).text, "Docs");
}

#[test]
fn enter_and_space_activate_the_cell_under_the_cursor_and_nothing_else() {
    let table = load(vec![row(11, "a", FieldAccess::Editable, false)]);
    for column in 1..=3 {
        cursor(&table, 0, column);
        key(&table.window, Key::Return);
        key(&table.window, Key::Space);
    }
    assert_eq!(
        &*table.actions.borrow(),
        &[
            (11, 2, 7),
            (11, 2, 7),
            (11, 3, 7),
            (11, 3, 7),
            (11, 4, 7),
            (11, 4, 7)
        ]
    );
    assert!(
        table.gestures.borrow().is_empty(),
        "a cell activation neither selects nor opens the row"
    );
    cursor(&table, 0, 0);
    key(&table.window, Key::Space);
    assert_eq!(
        table.gestures.borrow().as_slice(),
        &[(11, RowGesture::Toggle)]
    );
    assert_eq!(
        table.actions.borrow().len(),
        6,
        "a text cell is not an action"
    );
}

#[test]
fn a_pointer_click_activates_the_cell_without_selecting_the_row() {
    let table = load(vec![row(11, "a", FieldAccess::Editable, false)]);
    for column in 1..=3 {
        click(&table, &cell(&table, 0, column));
    }
    assert_eq!(
        &*table.actions.borrow(),
        &[(11, 2, 7), (11, 3, 7), (11, 4, 7)]
    );
    assert!(table.gestures.borrow().is_empty());
    assert_eq!(table.window.get_table_active_column(), 3);
    click(&table, &cell(&table, 0, 0));
    assert_eq!(table.actions.borrow().len(), 3);
    assert_eq!(
        table.gestures.borrow().as_slice(),
        &[(11, RowGesture::Select)]
    );
}

#[test]
fn read_only_disabled_and_disabled_row_cells_report_nothing() {
    let table = load(vec![
        row(1, "ro", FieldAccess::ReadOnly, false),
        row(2, "off", FieldAccess::Disabled, false),
        row(3, "row", FieldAccess::Editable, true),
    ]);
    for index in 0..3 {
        for column in 1..=3 {
            cursor(&table, index, column);
            key(&table.window, Key::Return);
            key(&table.window, Key::Space);
            click(&table, &cell(&table, index as usize, column as usize));
        }
    }
    assert!(table.actions.borrow().is_empty());
}

#[test]
fn a_non_interactive_table_makes_the_cells_inert() {
    let table = load(vec![row(1, "a", FieldAccess::Editable, false)]);
    table.window.set_table_interactive(false);
    cursor(&table, 0, 1);
    key(&table.window, Key::Return);
    click(&table, &cell(&table, 0, 2));
    assert!(table.actions.borrow().is_empty());
}

#[test]
fn the_toggle_names_its_state_and_the_row_keeps_the_host_state() {
    let table = load(vec![row(1, "a", FieldAccess::Editable, false)]);
    let toggle = ElementHandle::find_by_accessible_label(&table.window, "Done a")
        .next()
        .expect("toggle");
    assert_eq!(toggle.accessible_checked(), Some(false));
    let rows = table.window.get_rows();
    let mut first = rows.row_data(0).unwrap();
    first.content = table_cells([
        CellContent::text("a"),
        CellContent::toggle(true, "Done a", FieldAccess::Editable),
        CellContent::text(""),
        CellContent::text(""),
    ]);
    rows.set_row_data(0, first);
    let toggle = ElementHandle::find_by_accessible_label(&table.window, "Done a")
        .next()
        .expect("toggle");
    assert_eq!(toggle.accessible_checked(), Some(true));
}
