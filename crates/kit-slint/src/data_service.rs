// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{ColumnDataIndex, ui};
use slint::{ComponentHandle, Model, ModelExt, ModelRc, ModelTracker};

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
pub(crate) fn install_data<W: ComponentHandle>(window: &W)
where
    for<'a> ui::KitDataService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitDataService>();
    service.on_visible_columns(visible_columns);
    service.on_source_current(source_current);
    service.set_ready(true);
}
