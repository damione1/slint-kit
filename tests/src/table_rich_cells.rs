// SPDX-License-Identifier: GPL-3.0-or-later
//! Icon, progress and tone-coloured text cells of `KitDataTable`. Headless
//! contract proofs of structure, labels and values; they do not qualify a
//! screen reader or the rendered colours on a device.
use super::base_data::{delegates, fixture};
use super::*;
use i_slint_backend_testing::ElementHandle;
use slint::{Model, ModelRc, VecModel};
use slint_kit::{CellContent, ProgressFraction, table_cells};

fn column(token: i32, label: &str, alignment: DataAlignment) -> TableColumn {
    TableColumn {
        visibility: Default::default(),
        source: Default::default(),
        token,
        label: label.into(),
        sortable: false,
        sort: DataSortDirection::Unsorted,
        alignment,
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
fn fraction(value: f32) -> ProgressFraction {
    ProgressFraction::try_from(value).expect("a fraction from 0 to 1")
}
fn rich_row(token: i32, name: &str) -> TableRow {
    TableRow {
        token,
        cells: ModelRc::default(),
        content: table_cells([
            CellContent::tone_text(name, Tone::Danger),
            CellContent::icon(glyph(), "Warning", Tone::Warning),
            CellContent::icon(glyph(), "", Tone::Neutral),
            CellContent::progress(fraction(0.4), "Upload", Tone::Success),
        ]),
        selected: false,
        disabled: false,
    }
}
fn load(rows: Vec<TableRow>) -> BaseDataFixture {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(vec![
        column(1, "Amount", DataAlignment::Leading),
        column(2, "Alert", DataAlignment::Center),
        column(3, "Mark", DataAlignment::Trailing),
        column(4, "Upload", DataAlignment::Leading),
    ])));
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window
}
fn row_named(window: &BaseDataFixture, label: &str) -> ElementHandle {
    delegates(window, "KitDataTable")
        .into_iter()
        .find(|row| row.accessible_label().as_deref() == Some(label))
        .unwrap_or_else(|| panic!("missing row {label}"))
}
fn components(row: &ElementHandle, component: &str) -> Vec<ElementHandle> {
    row.query_descendants().match_inherits(component).find_all()
}

#[test]
fn progress_fraction_accepts_only_a_number_from_zero_to_one() {
    for valid in [0., 0.4, 1.] {
        assert_eq!(
            ProgressFraction::try_from(valid).map(ProgressFraction::get),
            Ok(valid)
        );
    }
    for invalid in [-0.01, 1.01, f32::NAN, f32::INFINITY] {
        assert!(ProgressFraction::try_from(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn icons_are_named_when_the_host_gives_words() {
    let window = load(vec![rich_row(1, "-12.00")]);
    let row = row_named(&window, "-12.00");
    let icons = components(&row, "KitIcon");
    assert_eq!(icons.len(), 2, "one icon per icon cell");
    let named = row
        .query_descendants()
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Warning"))
        .find_all();
    assert_eq!(
        named.len(),
        1,
        "only the informative icon carries the words"
    );
}

#[test]
fn progress_cells_expose_their_percentage_and_label() {
    let window = load(vec![rich_row(1, "-12.00")]);
    let row = row_named(&window, "-12.00");
    let bars: Vec<_> = row
        .query_descendants()
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Upload"))
        .find_all();
    assert_eq!(bars.len(), 1);
    assert_eq!(bars[0].accessible_value().as_deref(), Some("40"));
}

#[test]
fn typed_cells_convert_once_into_their_flat_projection() {
    let cells = table_cells([
        CellContent::tone_text("-1", Tone::Danger),
        CellContent::icon(glyph(), "Warning", Tone::Warning),
        CellContent::progress(fraction(0.25), "Upload", Tone::Success),
    ]);
    let cell = |index| cells.row_data(index).expect("cell");
    assert_eq!(cell(0).kind, TableCellKind::Text);
    assert_eq!(cell(0).tone, Tone::Danger);
    assert_eq!(cell(1).kind, TableCellKind::Icon);
    assert_eq!(cell(1).glyph, glyph());
    assert_eq!(cell(2).kind, TableCellKind::Progress);
    assert!((cell(2).fraction - 0.25).abs() < f32::EPSILON);
}
