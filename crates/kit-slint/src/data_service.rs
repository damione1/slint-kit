// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{ColumnDataIndex, ui};
use kit_core::{TYPEAHEAD_TIMEOUT_MS, typeahead_search};
use slint::{ComponentHandle, DataTransfer, Model, ModelExt, ModelRc, ModelTracker, SharedString};
use std::rc::Rc;

// Map canonical positions before filtering. Reuse the source tracker so row
// changes update existing header delegates instead of replacing the projection.
struct CanonicalColumns(ModelRc<ui::TableColumn>);
impl Model for CanonicalColumns {
    type Data = ui::TableColumn;
    fn row_count(&self) -> usize {
        self.0.row_count()
    }
    fn row_data(&self, index: usize) -> Option<Self::Data> {
        let mut column = self.0.row_data(index)?;
        match column.source.mode {
            ui::ColumnSourceMode::Position => {
                column.source = ui::TableColumnSource {
                    mode: ui::ColumnSourceMode::DataIndex,
                    index: i32::try_from(index).ok()?,
                };
            }
            ui::ColumnSourceMode::DataIndex => {}
        }
        Some(column)
    }
    fn model_tracker(&self) -> &dyn ModelTracker {
        self.0.model_tracker()
    }
}
fn visible_columns(columns: ModelRc<ui::TableColumn>) -> ModelRc<ui::TableColumn> {
    ModelRc::new(
        CanonicalColumns(columns).filter(|column| match column.visibility {
            ui::ColumnVisibility::Hidden => false,
            ui::ColumnVisibility::Visible => true,
        }),
    )
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct ColumnToken(i32);
struct SourceAddress {
    column: ColumnToken,
    data: ColumnDataIndex,
}
fn source_current(columns: ModelRc<ui::TableColumn>, token: i32, data: i32) -> bool {
    let Some(data) = ColumnDataIndex::from_projection(data) else {
        return false;
    };
    let address = SourceAddress {
        column: ColumnToken(token),
        data,
    };
    columns.model_tracker().track_row_count_changes();
    let mut matched = false;
    for position in 0..columns.row_count() {
        columns.model_tracker().track_row_data_changes(position);
        let Some(column) = columns.row_data(position) else {
            continue;
        };
        if ColumnToken(column.token) != address.column {
            continue;
        }
        // A repeated token is an ambiguous address, never a second chance
        // to accept a source the original column no longer owns.
        if matched {
            return false;
        }
        matched = true;
        match column.visibility {
            ui::ColumnVisibility::Hidden => return false,
            ui::ColumnVisibility::Visible => {}
        }
        let current = match column.source.mode {
            ui::ColumnSourceMode::Position => ColumnDataIndex::try_from(position).ok(),
            ui::ColumnSourceMode::DataIndex => {
                ColumnDataIndex::from_projection(column.source.index)
            }
        };
        if current != Some(address.data) {
            return false;
        }
    }
    matched
}
// Only a kit-created in-process payload can reorder columns. It carries the
// source model and its exact revision, so a different model or changed projection
// cannot reinterpret an old token. No string or external MIME token is trusted.
struct ColumnDrag {
    model: ModelRc<ui::TableColumn>,
    revision: Vec<ui::TableColumn>,
    token: ColumnToken,
}
fn column_revision(columns: &ModelRc<ui::TableColumn>) -> Vec<ui::TableColumn> {
    columns.model_tracker().track_row_count_changes();
    (0..columns.row_count())
        .filter_map(|index| columns.row_data_tracked(index))
        .collect()
}
fn unique_visible_column(columns: &[ui::TableColumn], token: ColumnToken) -> bool {
    let mut matches = columns.iter().filter(|column| column.token == token.0);
    let Some(column) = matches.next() else {
        return false;
    };
    matches.next().is_none() && column.visibility == ui::ColumnVisibility::Visible
}
fn column_drag(columns: ModelRc<ui::TableColumn>, token: i32) -> DataTransfer {
    let revision = column_revision(&columns);
    let token = ColumnToken(token);
    if !unique_visible_column(&revision, token) {
        return DataTransfer::default();
    }
    let mut data = DataTransfer::default();
    data.set_user_data(Rc::new(ColumnDrag {
        model: columns,
        revision,
        token,
    }));
    data
}
fn column_drop(
    columns: ModelRc<ui::TableColumn>,
    data: DataTransfer,
    target: i32,
) -> ui::TableColumnDrop {
    let rejected = ui::TableColumnDrop::default();
    let Some(payload) = data.user_data() else {
        return rejected;
    };
    let Some(drag) = payload.downcast_ref::<ColumnDrag>() else {
        return rejected;
    };
    let target = ColumnToken(target);
    if drag.model != columns || drag.token == target {
        return rejected;
    }
    let current = column_revision(&columns);
    if current != drag.revision
        || !unique_visible_column(&current, drag.token)
        || !unique_visible_column(&current, target)
    {
        return rejected;
    }
    ui::TableColumnDrop {
        valid: true,
        token: drag.token.0,
    }
}
/// The text a row is found by when typing: its first cell, as the row's
/// accessible name is read. A row without cells has no name to match.
fn row_name(row: &ui::TableRow) -> SharedString {
    row.content
        .row_data(0)
        .map(|cell| cell.text)
        .or_else(|| row.cells.row_data(0))
        .unwrap_or_default()
}
/// A key worth searching by: one printable character. Control characters and
/// the private-use code points Slint gives to named keys (arrows, function
/// keys) are not text.
fn typeahead_text(text: &str) -> bool {
    let mut chars = text.chars();
    matches!(
        (chars.next(), chars.next()),
        (Some(c), None) if !c.is_control() && !('\u{E000}'..='\u{F8FF}').contains(&c)
    )
}
fn session_clock(session: &ui::TableTypeahead) -> Option<u64> {
    session
        .has_timestamp
        .then(|| u64::try_from(session.timestamp).ok())
        .flatten()
}
fn typeahead_active(session: ui::TableTypeahead, now: i32) -> bool {
    let (Ok(now), Some(last)) = (u64::try_from(now), session_clock(&session)) else {
        return false;
    };
    !session.query.is_empty() && now >= last && now - last < TYPEAHEAD_TIMEOUT_MS
}
fn typeahead(
    rows: ModelRc<ui::TableRow>,
    active: i32,
    session: ui::TableTypeahead,
    text: SharedString,
    now: i32,
) -> ui::TableTypeaheadResult {
    let unchanged = |session| ui::TableTypeaheadResult {
        row: -1,
        handled: false,
        session,
    };
    let Ok(now_ms) = u64::try_from(now) else {
        return unchanged(session);
    };
    if !typeahead_text(&text) {
        return unchanged(session);
    }
    rows.model_tracker().track_row_count_changes();
    let snapshot: Vec<(SharedString, bool)> = (0..rows.row_count())
        .filter_map(|index| rows.row_data_tracked(index))
        .map(|row| (row_name(&row), row.disabled))
        .collect();
    let mut query = session.query.to_string();
    let mut last_ms = session_clock(&session);
    let found = typeahead_search(
        &mut query,
        &mut last_ms,
        TYPEAHEAD_TIMEOUT_MS,
        &text,
        now_ms,
        usize::try_from(active).ok(),
        snapshot
            .iter()
            .map(|(name, disabled)| (name.as_str(), *disabled)),
    );
    ui::TableTypeaheadResult {
        handled: !query.is_empty(),
        row: found
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1),
        session: ui::TableTypeahead {
            query: query.into(),
            has_timestamp: last_ms.is_some(),
            timestamp: last_ms.and_then(|ms| i32::try_from(ms).ok()).unwrap_or(0),
        },
    }
}
/// The position of the first (or last) visible column whose cell in `row` is
/// an editable field, or -1 when the row has none. `columns` is the visible
/// projection the table paints, so a position is a painted column.
fn edge_editable_column(columns: ModelRc<ui::TableColumn>, row: ui::TableRow, last: bool) -> i32 {
    let editable = |position: usize| {
        let Some(column) = columns.row_data_tracked(position) else {
            return false;
        };
        let data = match column.source.mode {
            ui::ColumnSourceMode::Position => i32::try_from(position).ok(),
            ui::ColumnSourceMode::DataIndex => Some(column.source.index),
        };
        data.and_then(|index| usize::try_from(index).ok())
            .and_then(|index| row.content.row_data(index))
            .is_some_and(|cell| {
                cell.kind == ui::TableCellKind::Field && cell.access == ui::FieldAccess::Editable
            })
    };
    columns.model_tracker().track_row_count_changes();
    let count = columns.row_count();
    let found = if last {
        (0..count).rev().find(|&position| editable(position))
    } else {
        (0..count).find(|&position| editable(position))
    };
    found
        .and_then(|position| i32::try_from(position).ok())
        .unwrap_or(-1)
}
pub(crate) fn install_data<W: ComponentHandle>(window: &W)
where
    for<'a> ui::KitDataService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitDataService>();
    service.on_visible_columns(visible_columns);
    service.on_source_current(source_current);
    service.on_column_drag(column_drag);
    service.on_column_drop(column_drop);
    service.on_typeahead(typeahead);
    service.on_typeahead_active(typeahead_active);
    service.on_edge_editable_column(edge_editable_column);
    service.set_ready(true);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::VecModel;

    fn columns() -> Rc<VecModel<ui::TableColumn>> {
        Rc::new(VecModel::from(vec![
            ui::TableColumn {
                token: 17,
                ..Default::default()
            },
            ui::TableColumn {
                token: 91,
                ..Default::default()
            },
        ]))
    }
    #[test]
    fn column_drag_uses_typed_payload_and_exact_source_identity() {
        let source = columns();
        let model = ModelRc::from(source);
        let drag = column_drag(model.clone(), 17);
        assert_eq!(
            column_drop(model.clone(), drag.clone(), 91),
            ui::TableColumnDrop {
                valid: true,
                token: 17
            }
        );
        assert!(!column_drop(model.clone(), drag.clone(), 17).valid);
        assert!(!column_drop(ModelRc::from(columns()), drag, 91).valid);
        assert!(
            !column_drop(
                model,
                DataTransfer::from(slint::SharedString::from("17")),
                91
            )
            .valid
        );
    }
    #[test]
    fn changed_columns_reject_an_in_flight_drag_even_with_the_same_tokens() {
        let source = columns();
        let model = ModelRc::from(source.clone());
        let drag = column_drag(model.clone(), 17);
        let mut target = source.row_data(1).unwrap();
        target.label = "Replaced".into();
        source.set_row_data(1, target);
        assert!(!column_drop(model.clone(), drag, 91).valid);
        let fresh = column_drag(model.clone(), 17);
        assert!(column_drop(model.clone(), fresh.clone(), 91).valid);
        source.push(ui::TableColumn {
            token: 200,
            ..Default::default()
        });
        assert!(!column_drop(model, fresh, 91).valid);
    }
    #[test]
    fn hidden_missing_and_ambiguous_column_identities_cannot_move() {
        let source = columns();
        let model = ModelRc::from(source.clone());
        assert!(column_drag(model.clone(), 200).is_empty());
        let mut target = source.row_data(1).unwrap();
        target.visibility = ui::ColumnVisibility::Hidden;
        source.set_row_data(1, target);
        assert!(column_drag(model.clone(), 91).is_empty());
        assert!(!column_drop(model.clone(), column_drag(model.clone(), 17), 91).valid);
        source.push(ui::TableColumn {
            token: 17,
            ..Default::default()
        });
        assert!(column_drag(model, 17).is_empty());
    }
}
