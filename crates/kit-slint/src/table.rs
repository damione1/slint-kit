// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Typed cells of `KitDataTable`. The Slint `TableCell` struct is a flat
//! projection; `CellContent` is the sum type behind it, converted once here
//! so only the fields of a cell's kind ever carry meaning.
use crate::{FieldSpec, FieldSpecKey, ui};
use kit_core::{CheckState, FieldAccess};
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

/// A canonical cell-data index, independent of visible column order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnDataIndex(i32);
impl TryFrom<usize> for ColumnDataIndex {
    type Error = std::num::TryFromIntError;
    fn try_from(index: usize) -> Result<Self, Self::Error> {
        i32::try_from(index).map(Self)
    }
}
impl From<ColumnDataIndex> for ui::TableColumnSource {
    fn from(index: ColumnDataIndex) -> Self {
        Self {
            mode: ui::ColumnSourceMode::DataIndex,
            index: index.0,
        }
    }
}

/// The codec of an editable cell: the key of a `FieldSpec` installed with
/// `install_field_specs`, which parses, validates and displays the value in
/// the editor and at rest, or plain text when the cell has none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CellCodec {
    /// Free text: every draft is accepted as typed.
    PlainText,
    /// The installed codec named by this key.
    Spec(FieldSpecKey),
}

/// What one data-table cell shows. Every variant carries the words a reader
/// gets for the cell: a badge shows its `text`, a check mark exposes its
/// `text` as the accessible label of the painted `state`, a field shows its
/// accepted value.
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
    /// A value the user may edit in place. `text` is the accepted value in
    /// the canonical draft form of `codec`; `access` says whether Enter opens
    /// the editor (`Editable`), or the cell only shows its value (`ReadOnly`),
    /// or shows it dimmed (`Disabled`).
    Field {
        text: SharedString,
        codec: CellCodec,
        access: FieldAccess,
        constraints: Option<ui::KitInputConstraints>,
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
    /// A free-text field holding `text`.
    pub fn field(text: impl Into<SharedString>, access: FieldAccess) -> Self {
        Self::Field {
            text: text.into(),
            codec: CellCodec::PlainText,
            access,
            constraints: None,
        }
    }
    /// A field holding a typed value, formatted once by the codec that the
    /// host installed under `key`; an absent value is an empty draft. The
    /// commit the table reports for it round-trips through the same
    /// `spec.accept(commit)`.
    pub fn typed_field<T: 'static, E: 'static>(
        key: FieldSpecKey,
        spec: &FieldSpec<T, E>,
        value: Option<&T>,
        access: FieldAccess,
    ) -> Self {
        Self::Field {
            text: value.map_or_else(SharedString::default, |value| spec.format(value).into()),
            codec: CellCodec::Spec(key),
            access,
            constraints: None,
        }
    }
    /// A constrained editor using the shared HTML engine and native numeric filtering.
    /// Use -1 for absent min/max lengths, as on KitTextField.
    pub fn constrained_field(
        text: impl Into<SharedString>,
        codec: CellCodec,
        access: FieldAccess,
        constraints: ui::KitInputConstraints,
    ) -> Self {
        Self::Field {
            text: text.into(),
            codec,
            access,
            constraints: Some(constraints),
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
            CellContent::Field {
                text,
                codec,
                access,
                constraints,
            } => Self {
                kind: ui::TableCellKind::Field,
                constrained: constraints.is_some(),
                constraints: constraints.unwrap_or_default(),
                text,
                access: access.into(),
                spec: match codec {
                    CellCodec::PlainText => SharedString::default(),
                    CellCodec::Spec(key) => key.as_str().into(),
                },
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
