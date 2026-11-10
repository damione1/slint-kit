// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed presentation cells of `KitDataTable`: text, badge and check cells
//! painted by the kit inside the virtualized row, next to the plain string
//! rows that keep working unchanged. These are headless contract proofs;
//! they do not qualify a screen reader.
use super::base_data::{delegates, fixture, key, rows};
use super::*;
use i_slint_backend_testing::ElementHandle;
use slint::platform::Key;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_kit::{CellContent, CheckState, KeyedRow, table_cells};

const TITLE: i32 = 3;
const STATUS: i32 = 5;
const DONE: i32 = 8;

fn column(token: i32, label: &str, alignment: DataAlignment, width: ColumnWidth) -> TableColumn {
    TableColumn {
        token,
        label: label.into(),
        sortable: false,
        sort: DataSortDirection::Unsorted,
        alignment,
        width,
    }
}
fn columns() -> ModelRc<TableColumn> {
    ModelRc::new(VecModel::from(vec![
        column(
            TITLE,
            "Title",
            DataAlignment::Leading,
            ColumnWidth::default(),
        ),
        column(
            STATUS,
            "Status",
            DataAlignment::Center,
            ColumnWidth {
                sizing: ColumnSizing::Fixed,
                basis: 160.,
                ..ColumnWidth::default()
            },
        ),
        column(
            DONE,
            "Done",
            DataAlignment::Trailing,
            ColumnWidth::default(),
        ),
    ]))
}
fn typed_row(token: i32, title: &str, status: (&str, Tone), done: (CheckState, &str)) -> TableRow {
    TableRow {
        token,
        cells: ModelRc::default(),
        content: table_cells([
            CellContent::text(title),
            CellContent::badge(status.0, status.1),
            CellContent::check(done.0, done.1),
        ]),
        selected: false,
        disabled: false,
    }
}
fn row(window: &BaseDataFixture, label: &str) -> ElementHandle {
    delegates(window, "KitDataTable")
        .into_iter()
        .find(|row| row.accessible_label().as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing row {label}"))
}
fn descendants(row: &ElementHandle, id: &str) -> Vec<ElementHandle> {
    row.query_descendants().match_id(id).find_all()
}
fn components(row: &ElementHandle, component: &str) -> Vec<ElementHandle> {
    row.query_descendants().match_inherits(component).find_all()
}
fn header_cells(window: &BaseDataFixture) -> Vec<ElementHandle> {
    ElementHandle::find_by_element_id(window, "KitDataTable::header-cell").collect()
}
fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 1.
}

#[test]
fn typed_rows_paint_badges_and_check_marks_while_string_rows_stay_text() {
    let window = fixture();
    window.set_columns(columns());
    let mut data = vec![typed_row(
        1,
        "Field notes",
        ("Review", Tone::Warning),
        (CheckState::Checked, "Done"),
    )];
    data.extend(rows(2, 1));
    window.set_rows(ModelRc::new(VecModel::from(data)));

    let typed = row(&window, "Field notes");
    assert_eq!(
        descendants(&typed, "KitDataTable::content-cell").len(),
        3,
        "one typed cell per content entry"
    );
    assert!(
        descendants(&typed, "KitDataTable::row-cell").is_empty(),
        "a typed row paints no string cells"
    );
    let badges = components(&typed, "KitBadge");
    assert_eq!(badges.len(), 1);
    assert_eq!(badges[0].accessible_label().as_deref(), Some("Review"));
    let marks = components(&typed, "CheckMark");
    assert_eq!(marks.len(), 1);
    assert_eq!(
        marks[0].accessible_label().as_deref(),
        Some("Done"),
        "the host's words name the check mark"
    );
    assert_eq!(components(&marks[0], "KitIcon").len(), 1);

    let plain = row(&window, "Product 2");
    assert_eq!(
        descendants(&plain, "KitDataTable::row-cell").len(),
        3,
        "string rows keep one text cell per declared column"
    );
    let padded = descendants(&plain, "KitDataTable::row-cell").remove(2);
    assert!(
        padded
            .query_descendants()
            .match_type_name("Text")
            .find_all()
            .iter()
            .all(|text| text.accessible_label().as_deref().unwrap_or("").is_empty())
    );
    assert!(descendants(&plain, "KitDataTable::content-cell").is_empty());
    assert!(components(&plain, "KitBadge").is_empty());
    assert!(components(&plain, "CheckMark").is_empty());
}

#[test]
fn check_marks_render_each_state_distinctly() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(vec![
        typed_row(
            1,
            "Unchecked",
            ("Draft", Tone::Neutral),
            (CheckState::Unchecked, "Not done"),
        ),
        typed_row(
            2,
            "Checked",
            ("Ready", Tone::Success),
            (CheckState::Checked, "Done"),
        ),
        typed_row(
            3,
            "Mixed",
            ("Partial", Tone::Information),
            (CheckState::Mixed, "Partly done"),
        ),
    ])));
    for (label, icons, bars) in [("Unchecked", 0, 0), ("Checked", 1, 0), ("Mixed", 0, 1)] {
        let mark = components(&row(&window, label), "CheckMark").remove(0);
        assert_eq!(
            components(&mark, "KitIcon").len(),
            icons,
            "{label}: check icon"
        );
        assert_eq!(
            descendants(&mark, "CheckMark::mixed-bar").len(),
            bars,
            "{label}: mixed bar"
        );
    }
}

#[test]
fn typed_cells_share_the_header_constraints_and_follow_the_column_alignment() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(vec![typed_row(
        1,
        "Field notes",
        ("Review", Tone::Warning),
        (CheckState::Checked, "Done"),
    )])));
    let header = header_cells(&window);
    let cells = descendants(&row(&window, "Field notes"), "KitDataTable::content-cell");
    assert_eq!(header.len(), cells.len());
    for (header_cell, cell) in header.iter().zip(&cells) {
        assert!(
            near(
                header_cell.absolute_position().x,
                cell.absolute_position().x
            ) && near(header_cell.size().width, cell.size().width),
            "header {:?}/{} and typed cell {:?}/{} must share x and width",
            header_cell.absolute_position(),
            header_cell.size().width,
            cell.absolute_position(),
            cell.size().width
        );
    }
    assert!(
        near(cells[1].size().width, 160.),
        "fixed column keeps its basis"
    );
    let badge = components(&cells[1], "KitBadge").remove(0);
    let cell_center = cells[1].absolute_position().x + cells[1].size().width / 2.;
    let badge_center = badge.absolute_position().x + badge.size().width / 2.;
    assert!(
        near(badge_center, cell_center),
        "a centered column centers its badge: {badge_center} vs {cell_center}"
    );
    let mark = components(&cells[2], "CheckMark").remove(0);
    let mark_end = mark.absolute_position().x + mark.size().width;
    let cell_end = cells[2].absolute_position().x + cells[2].size().width;
    assert!(
        mark_end < cell_end && mark_end > cell_center,
        "a trailing column ends its check mark before the cell's trailing inset"
    );
    assert!(
        mark.absolute_position().y > cells[2].absolute_position().y
            && mark.absolute_position().y + mark.size().height
                < cells[2].absolute_position().y + cells[2].size().height,
        "the mark sits inside the row height"
    );
}

#[test]
fn content_wins_over_strings_for_painting_and_the_row_name() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(vec![TableRow {
        token: 1,
        cells: ModelRc::new(VecModel::from(vec!["Ignored".into(), "0".into()])),
        content: table_cells([
            CellContent::text("Named by content"),
            CellContent::badge("Review", Tone::Accent),
            CellContent::check(CheckState::Unchecked, "Not done"),
        ]),
        selected: false,
        disabled: false,
    }])));
    let typed = row(&window, "Named by content");
    assert!(descendants(&typed, "KitDataTable::row-cell").is_empty());
    assert_eq!(descendants(&typed, "KitDataTable::content-cell").len(), 3);
    assert!(
        delegates(&window, "KitDataTable")
            .iter()
            .all(|row| row.accessible_label().as_deref() != Some("Ignored"))
    );
}

#[test]
fn typed_rows_stay_virtualized() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(
        (1..=10_000)
            .map(|index| {
                typed_row(
                    index,
                    &format!("Record {index}"),
                    ("Review", Tone::Neutral),
                    (CheckState::Checked, "Done"),
                )
            })
            .collect::<Vec<_>>(),
    )));
    let visible = delegates(&window, "KitDataTable");
    assert!(!visible.is_empty());
    assert!(
        visible.len() < 40,
        "typed rows mounted {} delegates",
        visible.len()
    );
    let weak = window.as_weak();
    window.on_navigate(move |_, step| {
        assert_eq!(step, NavigationStep::Last);
        weak.upgrade().unwrap().set_table_active(9_999);
    });
    window.invoke_focus_table();
    key(&window, Key::End);
    let visible = delegates(&window, "KitDataTable");
    assert!(visible.len() < 40);
    assert!(
        visible
            .iter()
            .any(|row| row.accessible_label().as_deref() == Some("Record 10000")),
        "End reveals the last typed row"
    );
}

#[test]
fn cell_content_projects_once_into_the_flat_table_cell() {
    let text = TableCell::from(CellContent::text("Plain"));
    assert_eq!(text.kind, TableCellKind::Text);
    assert_eq!(text.text, "Plain");
    let badge = TableCell::from(CellContent::badge("Review", Tone::Danger));
    assert_eq!(badge.kind, TableCellKind::Badge);
    assert_eq!(badge.text, "Review");
    assert_eq!(badge.tone, Tone::Danger);
    for (state, projected) in [
        (CheckState::Unchecked, ui::CheckState::Unchecked),
        (CheckState::Checked, ui::CheckState::Checked),
        (CheckState::Mixed, ui::CheckState::Mixed),
    ] {
        let check = TableCell::from(CellContent::check(state, "Done"));
        assert_eq!(check.kind, TableCellKind::Check);
        assert_eq!(check.check, projected);
        assert_eq!(check.text, "Done");
    }
    let model = table_cells([CellContent::text("a"), CellContent::text("b")]);
    assert_eq!(model.row_count(), 2);
    assert_eq!(model.row_data(1).unwrap().text, "b");

    let typed = typed_row(
        1,
        "Field notes",
        ("Review", Tone::Warning),
        (CheckState::Checked, "Done"),
    );
    assert_eq!(
        typed.label(),
        "Field notes",
        "the first typed cell names the row"
    );
    assert_eq!(rows(7, 1).remove(0).label(), "Product 7");
    assert_eq!(TableRow::default().label(), "");
}
