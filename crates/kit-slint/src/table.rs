// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Typed presentation cells of `KitDataTable`. The Slint `TableCell` struct
//! is a flat projection; `CellContent` is the sum type behind it, converted
//! once here so only the fields of a cell's kind ever carry meaning.
use crate::ui;
use kit_core::CheckState;
use slint::{ModelRc, SharedString, VecModel};

impl From<CheckState> for ui::CheckState {
    fn from(value: CheckState) -> Self {
        match value {
            CheckState::Unchecked => Self::Unchecked,
            CheckState::Checked => Self::Checked,
            CheckState::Mixed => Self::Mixed,
        }
    }
}

/// What one data-table cell shows. Every variant carries the words a reader
/// gets for the cell: a badge shows its `text`, a check mark exposes its
/// `text` as the accessible label of the painted `state`.
#[derive(Clone, Debug, PartialEq)]
pub enum CellContent {
    /// Plain text, as a `TableRow.cells` string renders.
    Text(SharedString),
    /// A `KitBadge` with a semantic tone.
    Badge { text: SharedString, tone: ui::Tone },
    /// A read-only check mark; `text` names the state in the host's words.
    Check {
        state: CheckState,
        text: SharedString,
    },
}

impl CellContent {
    /// Plain text from anything that converts into a `SharedString`.
    pub fn text(text: impl Into<SharedString>) -> Self {
        Self::Text(text.into())
    }
    /// A badge with the given tone.
    pub fn badge(text: impl Into<SharedString>, tone: ui::Tone) -> Self {
        Self::Badge {
            text: text.into(),
            tone,
        }
    }
    /// A check mark whose `text` is its accessible meaning.
    pub fn check(state: CheckState, text: impl Into<SharedString>) -> Self {
        Self::Check {
            state,
            text: text.into(),
        }
    }
}

impl From<CellContent> for ui::TableCell {
    fn from(value: CellContent) -> Self {
        match value {
            CellContent::Text(text) => Self {
                kind: ui::TableCellKind::Text,
                text,
                ..Self::default()
            },
            CellContent::Badge { text, tone } => Self {
                kind: ui::TableCellKind::Badge,
                text,
                tone,
                ..Self::default()
            },
            CellContent::Check { state, text } => Self {
                kind: ui::TableCellKind::Check,
                text,
                check: state.into(),
                ..Self::default()
            },
        }
    }
}

/// The `TableRow.content` model of one row, in column order.
pub fn table_cells(cells: impl IntoIterator<Item = CellContent>) -> ModelRc<ui::TableCell> {
    ModelRc::new(VecModel::from(
        cells
            .into_iter()
            .map(ui::TableCell::from)
            .collect::<Vec<_>>(),
    ))
}
