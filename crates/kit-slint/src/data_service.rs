// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
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
pub(crate) fn install_data<W: ComponentHandle>(window: &W)
where
    for<'a> ui::KitDataService<'a>: slint::Global<'a, W>,
{
    let service = window.global::<ui::KitDataService>();
    service.on_visible_columns(visible_columns);
    service.set_ready(true);
}
