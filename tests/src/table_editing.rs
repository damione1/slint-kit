// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed in-place editing of `KitDataTable` field cells: the cell cursor,
//! Enter / Escape / Tab, the draft kept apart from the accepted value, the
//! invalid, read-only and disabled states, identity fencing across a
//! reordered projection and the single viewport editor. These are headless
//! contract proofs; they do not qualify IME or a screen reader.
use super::base_data::{delegates, down, fixture, key, labelled, text, up};
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::Key;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_kit::{
    CellCodec, CellContent, DraftValue, FieldAccess, FieldSpec, FieldSpecKey, KeyedModel,
    TextValue, table_cells,
};
use std::{cell::RefCell, rc::Rc};

const TITLE: i32 = 3;
const QUANTITY: i32 = 5;
const OWNER: i32 = 8;
const RESERVED_OWNER: &str = "forbidden";
const RESERVED_MESSAGE: &str = "Reserved owner";

type Commits = Rc<RefCell<Vec<(i32, i32, i32, FieldCommit)>>>;

fn quantity_key() -> FieldSpecKey {
    FieldSpecKey::new("quantity").unwrap()
}
fn owner_key() -> FieldSpecKey {
    FieldSpecKey::new("owner").unwrap()
}
fn owner_spec() -> FieldSpec<TextValue, &'static str> {
    FieldSpec::new(
        |text| {
            if text.is_empty() {
                DraftValue::Empty
            } else {
                DraftValue::Parsed(TextValue::new(text))
            }
        },
        |value| value.as_str().into(),
        |value| {
            if value.as_str() == RESERVED_OWNER {
                Err(RESERVED_MESSAGE)
            } else {
                Ok(())
            }
        },
        |error| (*error).into(),
    )
}
fn column(token: i32, label: &str) -> TableColumn {
    TableColumn {
        visibility: Default::default(),
        source: Default::default(),
        token,
        label: label.into(),
        sortable: false,
        sort: DataSortDirection::Unsorted,
        alignment: DataAlignment::Leading,
        width: ColumnWidth::default(),
    }
}
fn columns() -> ModelRc<TableColumn> {
    ModelRc::new(VecModel::from(vec![
        column(TITLE, "Title"),
        column(QUANTITY, "Quantity"),
        column(OWNER, "Owner"),
    ]))
}
fn record(token: i32, title: &str, quantity: i64, owner: &str, access: FieldAccess) -> TableRow {
    TableRow {
        token,
        cells: ModelRc::default(),
        content: table_cells([
            CellContent::text(title),
            CellContent::typed_field(
                quantity_key(),
                &FieldSpec::integer(),
                Some(&quantity),
                access,
            ),
            CellContent::typed_field(
                owner_key(),
                &owner_spec(),
                Some(&TextValue::new(owner)),
                access,
            ),
        ]),
        selected: false,
        disabled: false,
    }
}
fn records() -> Vec<TableRow> {
    vec![
        record(1, "Field notes", 12, "Elliot", FieldAccess::Editable),
        record(2, "Design system", 4, "Darlene", FieldAccess::Editable),
        record(3, "Launch plan", 30, "Angela", FieldAccess::Editable),
    ]
}
/// A window whose table navigates its rows itself and records commits and
/// activations, with the quantity and owner codecs installed.
struct Editing {
    window: BaseDataFixture,
    commits: Commits,
    activations: Rc<RefCell<Vec<(i32, i32)>>>,
}
fn editing(rows: Vec<TableRow>) -> Editing {
    let window = fixture();
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    let count = i32::try_from(rows.len()).unwrap();
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    let weak = window.as_weak();
    window.on_navigate(move |collection, step| {
        assert_eq!(collection, DataFixtureCollection::Table);
        let window = weak.upgrade().unwrap();
        let active = window.get_table_active();
        window.set_table_active(match step {
            NavigationStep::Next => (active + 1).min(count - 1),
            NavigationStep::Previous => (active - 1).max(0),
            NavigationStep::First | NavigationStep::PagePrevious => 0,
            NavigationStep::Last | NavigationStep::PageNext => count - 1,
        });
    });
    let commits: Commits = Rc::default();
    let recorded = commits.clone();
    window.on_cell_committed(move |row, column, generation, commit| {
        recorded
            .borrow_mut()
            .push((row, column, generation, commit));
    });
    let activations = Rc::new(RefCell::new(Vec::new()));
    let recorded = activations.clone();
    window.on_table_activated(move |token, generation| {
        recorded.borrow_mut().push((token, generation));
    });
    window.invoke_focus_table();
    key(&window, Key::DownArrow);
    assert_eq!(window.get_table_active(), 0);
    Editing {
        window,
        commits,
        activations,
    }
}
fn table(window: &BaseDataFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitDataTable")
        .find_first()
        .unwrap()
}
fn editors(window: &BaseDataFixture) -> Vec<ElementHandle> {
    table(window)
        .query_descendants()
        .match_inherits("KitTextField")
        .find_all()
}
fn table_label(window: &BaseDataFixture) -> String {
    table(window).accessible_label().unwrap().to_string()
}
fn flush() {
    slint::platform::update_timers_and_animations();
}
fn shift_tab(window: &BaseDataFixture) {
    down(window, Key::Shift);
    key(window, Key::Tab);
    up(window, Key::Shift);
}
/// Moves the cursor to `column` of the active row and opens its editor.
fn open(window: &BaseDataFixture, column: i32) {
    window.set_table_active_column(column);
    key(window, Key::Return);
    flush();
    assert!(window.get_table_editing(), "Enter opens the editor");
    assert_eq!(editors(window).len(), 1, "one editor mounts");
}
/// Proves the table's key scope has the keyboard: Down reaches `navigate`.
fn table_has_focus(window: &BaseDataFixture) -> bool {
    let before = window.get_table_active();
    key(window, Key::DownArrow);
    let moved = window.get_table_active() != before;
    if moved {
        key(window, Key::UpArrow);
    }
    moved
}

#[test]
fn the_cell_cursor_moves_with_left_and_right_and_enter_edits_only_an_editable_field() {
    let Editing {
        window,
        commits,
        activations,
    } = editing(records());
    assert_eq!(window.get_table_active_column(), -1);
    assert_eq!(table_label(&window), "Products fixture: Field notes");
    key(&window, Key::Return);
    assert_eq!(
        &*activations.borrow(),
        &[(1, 1)],
        "Enter on the row activates it"
    );
    assert!(!window.get_table_editing());

    key(&window, Key::RightArrow);
    assert_eq!(window.get_table_active_column(), 0);
    assert_eq!(table_label(&window), "Products fixture: Field notes, Title");
    key(&window, Key::Return);
    assert_eq!(
        activations.borrow().len(),
        2,
        "Enter on a text cell activates the row"
    );
    assert!(editors(&window).is_empty());

    key(&window, Key::RightArrow);
    key(&window, Key::RightArrow);
    key(&window, Key::RightArrow);
    assert_eq!(
        window.get_table_active_column(),
        2,
        "the cursor stops at the last column"
    );
    key(&window, Key::LeftArrow);
    key(&window, Key::LeftArrow);
    key(&window, Key::LeftArrow);
    key(&window, Key::LeftArrow);
    assert_eq!(
        window.get_table_active_column(),
        -1,
        "the cursor returns to the whole row"
    );

    window.set_table_active_column(1);
    key(&window, Key::Return);
    assert!(window.get_table_editing());
    assert_eq!(
        activations.borrow().len(),
        2,
        "editing is not an activation"
    );
    let editor = labelled(&window, "Quantity, Field notes");
    assert_eq!(editor.accessible_value().as_deref(), Some("12"));
    assert!(commits.borrow().is_empty());
}

#[test]
fn escape_discards_the_draft_and_keeps_the_accepted_value() {
    let Editing {
        window, commits, ..
    } = editing(records());
    open(&window, 1);
    text(&window, "7");
    let editor = labelled(&window, "Quantity, Field notes");
    assert_eq!(
        editor.accessible_value().as_deref(),
        Some("7"),
        "typing replaces the selected value"
    );
    let row = delegates(&window, "KitDataTable").remove(0);
    assert!(
        row.query_descendants()
            .match_predicate(|element| element.accessible_label().as_deref() == Some("12"))
            .find_all()
            .is_empty(),
        "the accepted value is not painted beside the open editor"
    );
    key(&window, Key::Escape);
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
    assert!(commits.borrow().is_empty(), "Escape never commits");
    let row = delegates(&window, "KitDataTable").remove(0);
    assert_eq!(
        row.query_descendants()
            .match_predicate(|element| element.accessible_label().as_deref() == Some("12"))
            .find_all()
            .len(),
        1,
        "the cell shows the accepted value again"
    );
    assert!(
        table_has_focus(&window),
        "Escape returns the keyboard to the table"
    );
    flush();
    assert!(
        commits.borrow().is_empty(),
        "the blur of the closed editor commits nothing"
    );
}

#[test]
fn enter_commits_once_with_the_armed_identity_and_the_codec_accepts_it() {
    let Editing {
        window, commits, ..
    } = editing(records());
    key(&window, Key::DownArrow);
    open(&window, 1);
    text(&window, "7");
    key(&window, Key::Return);
    flush();
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
    let recorded = commits.borrow();
    assert_eq!(
        recorded.len(),
        1,
        "one commit per Enter, the closing blur adds none"
    );
    let (row, column, generation, commit) = &recorded[0];
    assert_eq!((*row, *column, *generation), (2, QUANTITY, 1));
    assert_eq!(commit.value.text, "7");
    assert_eq!(FieldSpec::integer().accept(commit.clone()), Ok(Some(7)));
    drop(recorded);
    assert!(table_has_focus(&window));
}

#[test]
fn an_invalid_draft_stays_in_the_editor_with_its_error_until_corrected_or_cancelled() {
    let Editing {
        window, commits, ..
    } = editing(records());
    open(&window, 1);
    text(&window, "abc");
    key(&window, Key::Return);
    assert!(
        window.get_table_editing(),
        "an invalid draft is not committed"
    );
    assert_eq!(editors(&window).len(), 1);
    assert_eq!(
        window.get_table_edit_error(),
        "Enter a value in the expected format."
    );
    assert!(commits.borrow().is_empty());
    text(&window, "1");
    assert_eq!(
        window.get_table_edit_error(),
        "",
        "editing clears the last error"
    );
    key(&window, Key::Escape);
    assert!(!window.get_table_editing());

    open(&window, 2);
    text(&window, RESERVED_OWNER);
    key(&window, Key::Return);
    assert!(
        window.get_table_editing(),
        "a domain rejection keeps the draft"
    );
    assert_eq!(window.get_table_edit_error(), RESERVED_MESSAGE);
    assert!(commits.borrow().is_empty());
    key(&window, Key::Escape);
    assert_eq!(window.get_table_edit_error(), "");
    assert!(commits.borrow().is_empty());
}

#[test]
fn read_only_disabled_and_non_interactive_cells_never_open_an_editor() {
    let mut rows = vec![
        record(1, "Read only", 12, "Elliot", FieldAccess::ReadOnly),
        record(2, "Disabled", 4, "Darlene", FieldAccess::Disabled),
        record(3, "Disabled row", 30, "Angela", FieldAccess::Editable),
    ];
    rows[2].disabled = true;
    let Editing {
        window,
        commits,
        activations,
    } = editing(rows);
    window.set_table_active_column(1);
    key(&window, Key::Return);
    assert!(
        !window.get_table_editing(),
        "a read-only field does not open"
    );
    assert_eq!(
        &*activations.borrow(),
        &[(1, 1)],
        "Enter falls back to the row activation"
    );
    key(&window, Key::DownArrow);
    key(&window, Key::Return);
    assert!(
        !window.get_table_editing(),
        "a disabled field does not open"
    );
    assert_eq!(activations.borrow().len(), 2);
    key(&window, Key::DownArrow);
    key(&window, Key::Return);
    assert!(!window.get_table_editing(), "a disabled row does not open");
    assert_eq!(
        activations.borrow().len(),
        2,
        "a disabled row does not activate"
    );
    window.set_rows(ModelRc::new(VecModel::from(records())));
    window.set_table_interactive(false);
    window.invoke_focus_table();
    key(&window, Key::Return);
    assert!(
        !window.get_table_editing(),
        "a non-interactive table does not edit"
    );
    assert!(editors(&window).is_empty());
    assert!(commits.borrow().is_empty());
}

#[test]
fn an_open_editor_closes_when_the_table_stops_being_interactive() {
    let Editing {
        window, commits, ..
    } = editing(records());
    open(&window, 1);
    text(&window, "7");
    window.set_table_interactive(false);
    flush();
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
    assert!(commits.borrow().is_empty());
}

#[test]
fn a_new_generation_ends_the_session_without_committing() {
    let Editing {
        window, commits, ..
    } = editing(records());
    key(&window, Key::DownArrow);
    open(&window, 1);
    text(&window, "7");
    // The host sorts while the user types: the projection changes.
    let mut reordered = records();
    reordered.reverse();
    window.set_rows(ModelRc::new(VecModel::from(reordered)));
    window.set_table_generation(2);
    flush();
    assert!(
        !window.get_table_editing(),
        "a new generation closes the editor"
    );
    assert!(editors(&window).is_empty());
    assert!(
        commits.borrow().is_empty(),
        "the stale draft is not committed anywhere"
    );
    assert!(table_has_focus(&window));
    // Editing the same position now edits the row that is there, under the
    // new generation.
    open(&window, 1);
    text(&window, "9");
    key(&window, Key::Return);
    let recorded = commits.borrow();
    assert_eq!(recorded.len(), 1);
    assert_eq!(
        (recorded[0].0, recorded[0].1, recorded[0].2),
        (2, QUANTITY, 2)
    );
}

#[test]
fn keyed_model_cell_commits_reject_a_stale_generation_and_name_the_key() {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct RecordId(u32);
    let model = KeyedModel::new([
        (RecordId(7), records().remove(0)),
        (RecordId(9), records().remove(1)),
    ])
    .unwrap();
    let received = Rc::new(RefCell::new(Vec::new()));
    let recorded = received.clone();
    let handler = model.cell_commits(move |key, column, commit| {
        recorded
            .borrow_mut()
            .push((key, column, commit.value.text.to_string()));
    });
    let token = model.model().row_data(1).unwrap().token;
    let commit = FieldCommit {
        value: FieldValue {
            present: true,
            text: "7".into(),
        },
        ..FieldCommit::default()
    };
    let stale = model.generation();
    model.sort_by(|left, right| right.0.cmp(&left.0));
    handler(token, QUANTITY, stale, commit.clone());
    assert!(
        received.borrow().is_empty(),
        "a commit armed before the sort is rejected"
    );
    handler(token, QUANTITY, model.generation(), commit);
    assert_eq!(
        &*received.borrow(),
        &[(RecordId(9), QUANTITY, "7".to_string())],
        "the token still names its key after the reorder"
    );
}

#[test]
fn tab_commits_a_valid_draft_and_moves_the_cursor_while_an_invalid_one_stays() {
    let Editing {
        window, commits, ..
    } = editing(records());
    open(&window, 1);
    text(&window, "8");
    key(&window, Key::Tab);
    flush();
    assert!(
        !window.get_table_editing(),
        "Tab commits and leaves the editor"
    );
    assert_eq!(
        window.get_table_active_column(),
        2,
        "Tab moves the cursor to the next cell"
    );
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(commits.borrow()[0].1, QUANTITY);
    assert!(table_has_focus(&window));

    key(&window, Key::Return);
    assert!(window.get_table_editing(), "Enter opens the next cell");
    shift_tab(&window);
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(
        window.get_table_active_column(),
        1,
        "Shift+Tab moves the cursor back"
    );
    assert_eq!(commits.borrow().len(), 2);
    assert_eq!(commits.borrow()[1].1, OWNER);
    assert_eq!(commits.borrow()[1].3.value.text, "Elliot");

    key(&window, Key::Return);
    text(&window, "x");
    key(&window, Key::Tab);
    assert!(
        window.get_table_editing(),
        "an invalid draft keeps the editor open"
    );
    assert_eq!(
        window.get_table_active_column(),
        1,
        "and the cursor does not move"
    );
    assert_eq!(commits.borrow().len(), 2);
    assert_ne!(window.get_table_edit_error(), "");
}

#[test]
fn focus_loss_commits_a_valid_draft_once_and_drops_an_invalid_one() {
    let Editing {
        window, commits, ..
    } = editing(records());
    open(&window, 1);
    text(&window, "7");
    window.invoke_focus_paste();
    flush();
    assert!(!window.get_table_editing(), "blur ends the session");
    assert_eq!(
        commits.borrow().len(),
        1,
        "a valid draft commits once on blur"
    );
    assert_eq!(commits.borrow()[0].3.value.text, "7");
    assert!(editors(&window).is_empty());
    text(&window, "kept focus");
    assert_eq!(
        window.get_pasted(),
        "kept focus",
        "blur commits must preserve the destination focus"
    );

    window.invoke_focus_table();
    open(&window, 1);
    text(&window, "x");
    window.invoke_focus_paste();
    flush();
    assert!(!window.get_table_editing(), "blur drops an invalid draft");
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(window.get_table_edit_error(), "");
}

#[test]
fn only_the_edited_cell_mounts_an_editor_across_a_large_model() {
    let rows = (1..=10_000)
        .map(|index| {
            record(
                index,
                &format!("Record {index}"),
                i64::from(index),
                "Owner",
                FieldAccess::Editable,
            )
        })
        .collect::<Vec<_>>();
    let Editing {
        window, commits, ..
    } = editing(rows);
    open(&window, 1);
    assert_eq!(editors(&window).len(), 1);
    assert!(delegates(&window, "KitDataTable").len() < 40);
    key(&window, Key::Escape);
    key(&window, Key::End);
    assert_eq!(window.get_table_active(), 9_999);
    open(&window, 2);
    assert_eq!(
        editors(&window).len(),
        1,
        "the editor follows the edited cell, not the viewport"
    );
    assert!(delegates(&window, "KitDataTable").len() < 40);
    let editor = labelled(&window, "Owner, Record 10000");
    assert_eq!(editor.accessible_value().as_deref(), Some("Owner"));
    text(&window, "Maya");
    key(&window, Key::Return);
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(
        (commits.borrow()[0].0, commits.borrow()[0].1),
        (10_000, OWNER)
    );
}

#[test]
fn field_cells_project_once_and_format_their_typed_value() {
    let plain = TableCell::from(CellContent::field("Note", FieldAccess::ReadOnly));
    assert_eq!(plain.kind, TableCellKind::Field);
    assert_eq!(plain.text, "Note");
    assert_eq!(plain.spec, "");
    assert_eq!(plain.access, ui::FieldAccess::ReadOnly);
    let typed = TableCell::from(CellContent::typed_field(
        quantity_key(),
        &FieldSpec::integer(),
        Some(&1_234),
        FieldAccess::Editable,
    ));
    assert_eq!(
        typed.text, "1234",
        "the codec's canonical draft, not a display projection"
    );
    assert_eq!(typed.spec, "quantity");
    assert_eq!(typed.access, ui::FieldAccess::Editable);
    let absent = TableCell::from(CellContent::typed_field(
        quantity_key(),
        &FieldSpec::integer(),
        None,
        FieldAccess::Disabled,
    ));
    assert_eq!(absent.text, "", "an absent value is an empty draft");
    assert_eq!(absent.access, ui::FieldAccess::Disabled);
    assert_eq!(
        CellContent::field("x", FieldAccess::Editable),
        CellContent::Field {
            text: "x".into(),
            codec: CellCodec::PlainText,
            constraints: None,
            access: FieldAccess::Editable
        }
    );
}

#[test]
fn editors_and_cell_cursor_fit_the_row_at_every_density_and_text_scale() {
    let Editing { window, .. } = editing(records());
    let light = window.get_theme();
    for (name, theme) in [("light", light), ("dark", window.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for size in [
                ControlSize::Compact,
                ControlSize::Medium,
                ControlSize::Comfortable,
            ] {
                window.set_table_size(size);
                open(&window, 1);
                let editor = editors(&window).remove(0);
                let row = delegates(&window, "KitDataTable").remove(0);
                assert!(editor.absolute_position().y >= row.absolute_position().y);
                assert!(
                    editor.absolute_position().y + editor.size().height
                        <= row.absolute_position().y + row.size().height
                );
                assert_eq!(editor.computed_opacity(), 1.);
                key(&window, Key::Escape);
            }
        }
        let _ = name;
    }
}

#[test]
fn active_cell_announces_its_value_and_localized_editability() {
    let Editing { window, .. } = editing(records());
    window.set_table_active_column(1);
    assert_eq!(table(&window).accessible_value().as_deref(), Some("12"));
    assert_eq!(
        table(&window).accessible_description().as_deref(),
        Some("editable")
    );
    window.set_table_active_column(0);
    assert_eq!(
        table(&window).accessible_value().as_deref(),
        Some("Field notes")
    );
    assert_eq!(
        table(&window).accessible_description().as_deref(),
        Some("read only")
    );
}

#[test]
fn plain_text_editor_rendering_keeps_native_editing_in_light_dark_and_large_text() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_columns(ModelRc::new(VecModel::from(vec![column(OWNER, "Owner")])));
    window.set_rows(ModelRc::new(VecModel::from(vec![TableRow {
        token: 1,
        content: table_cells([CellContent::field("Original", FieldAccess::Editable)]),
        ..Default::default()
    }])));
    window.set_table_active(0);
    window.set_table_active_column(0);
    window.show().unwrap();
    let committed = Rc::new(RefCell::new(Vec::new()));
    let recorded = committed.clone();
    window.on_cell_committed(move |_, _, _, commit| recorded.borrow_mut().push(commit));
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for size in [
                ControlSize::Compact,
                ControlSize::Medium,
                ControlSize::Comfortable,
            ] {
                window.set_table_size(size);
                window.invoke_focus_table();
                key(&window, Key::Return);
                text(&window, "Native draft");
                assert_eq!(editors(&window).len(), 1);
                let snapshot = window.window().take_snapshot().unwrap();
                crate::choice_presentation::save_capture(
                    &snapshot,
                    &format!("table-editor-{name}-{scale}-{size:?}"),
                );
                key(&window, Key::Return);
                assert!(!window.get_table_editing());
                assert_eq!(
                    committed.borrow().last().unwrap().value.text,
                    "Native draft"
                );
            }
        }
    }
}

#[test]
fn f2_and_public_edit_commands_share_the_native_plain_text_editor() {
    let Editing {
        window, commits, ..
    } = editing(records());
    window.set_table_active_column(2);
    key(&window, Key::F2);
    assert!(window.get_table_editing());
    text(&window, "Maya");
    window.invoke_commit_table();
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(commits.borrow().last().unwrap().3.value.text, "Maya");
    window.invoke_edit_table();
    flush();
    assert!(window.get_table_editing());
    window.invoke_cancel_table();
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(commits.borrow().len(), 1);
}

#[test]
fn pointer_cell_cursor_and_double_click_open_the_pointed_field() {
    use slint::{
        LogicalPosition,
        platform::{PointerEventButton, WindowEvent},
    };
    let Editing { window, .. } = editing(records());
    let cell = ElementHandle::find_by_element_id(&window, "KitDataTable::content-cell")
        .nth(2)
        .unwrap();
    let origin = cell.absolute_position();
    let position = LogicalPosition::new(
        origin.x + cell.size().width / 2.,
        origin.y + cell.size().height / 2.,
    );
    for _ in 0..2 {
        window.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        flush();
        window
            .window()
            .dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(50));
        flush();
    }
    assert_eq!(window.get_table_active_column(), 2);
    assert!(window.get_table_editing());
    assert_eq!(editors(&window).len(), 1);
}

#[test]
fn cell_cursor_reveals_horizontal_overflow_before_opening_an_editor() {
    let Editing { window, .. } = editing(records());
    let columns = columns()
        .iter()
        .map(|mut column| {
            column.width = ColumnWidth {
                sizing: ColumnSizing::Fixed,
                basis: 500.,
                ..Default::default()
            };
            column
        })
        .collect::<Vec<_>>();
    window.set_columns(ModelRc::new(VecModel::from(columns)));
    window.set_table_active_column(2);
    // A model replacement invalidates native layout. Resolve the real column
    // geometry before testing its reveal; timer delivery alone does not paint
    // a frame in the mock backend.
    let row = delegates(&window, "KitDataTable").remove(0);
    let cells = row
        .query_descendants()
        .match_inherits("TableCellView")
        .find_all();
    assert!(!cells.is_empty());
    for cell in cells {
        assert!((cell.size().width - 500.).abs() < 1.);
    }
    flush();
    assert!(
        window.get_table_scroll_x() < 0.,
        "the resolved cell is revealed before F2"
    );
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
    key(&window, Key::F2);
    let editor = editors(&window).remove(0);
    let bounds = table(&window);
    assert!(editor.absolute_position().x >= bounds.absolute_position().x - 1.);
    assert!(
        editor.absolute_position().x + editor.size().width
            <= bounds.absolute_position().x + bounds.size().width + 1.
    );
}

#[test]
fn table_fields_refuse_to_open_before_the_field_service_is_installed() {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDataFixture::new().unwrap();
    window.set_columns(ModelRc::new(VecModel::from(vec![column(OWNER, "Owner")])));
    window.set_rows(ModelRc::new(VecModel::from(vec![TableRow {
        token: 1,
        content: table_cells([CellContent::field("Original", FieldAccess::Editable)]),
        ..Default::default()
    }])));
    window.set_table_active(0);
    window.set_table_active_column(0);
    window.show().unwrap();
    window.invoke_focus_table();
    key(&window, Key::Return);
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
}

fn constrained_records() -> Vec<TableRow> {
    let mut records = records();
    records[0].content = table_cells([
        CellContent::text("Field notes"),
        CellContent::constrained_field(
            "12",
            CellCodec::Spec(quantity_key()),
            FieldAccess::Editable,
            ui::KitInputConstraints {
                kind: ui::HtmlInputKind::Number,
                required: true,
                min: "0".into(),
                max: "20".into(),
                min_length: -1,
                max_length: -1,
                ..Default::default()
            },
        ),
    ]);
    records
}

#[test]
fn constrained_cells_filter_numbers_and_keep_rejected_domain_values_open() {
    let records = constrained_records();
    let Editing {
        window, commits, ..
    } = editing(records);
    open(&window, 1);
    text(&window, "x");
    assert_eq!(
        labelled(&window, "Quantity, Field notes")
            .accessible_value()
            .as_deref(),
        Some("12")
    );
    text(&window, "25");
    key(&window, Key::Return);
    assert!(window.get_table_editing());
    assert!(commits.borrow().is_empty());
    key(&window, Key::Escape);
    key(&window, Key::F2);
    text(&window, "8");
    window.on_validate_table_edit(|_, _, _, _| "The record is locked".into());
    key(&window, Key::Return);
    assert!(window.get_table_editing());
    assert_eq!(window.get_table_edit_error(), "The record is locked");
    assert!(commits.borrow().is_empty());
}

#[test]
fn constrained_host_rejections_can_retry_the_same_draft_without_retyping() {
    let Editing {
        window, commits, ..
    } = editing(constrained_records());
    let submit: [fn(&BaseDataFixture); 3] = [
        |window| key(window, Key::Return),
        |window| key(window, Key::Tab),
        |window| window.invoke_commit_table(),
    ];
    for submit in submit {
        open(&window, 1);
        text(&window, "8");
        let locked = Rc::new(std::cell::Cell::new(true));
        let observed = locked.clone();
        let verdicts = Rc::new(std::cell::Cell::new(0));
        let observed_verdicts = verdicts.clone();
        window.on_validate_table_edit(move |_, _, _, _| {
            observed_verdicts.set(observed_verdicts.get() + 1);
            if observed.get() {
                "The record is locked".into()
            } else {
                "".into()
            }
        });
        submit(&window);
        flush();
        assert!(window.get_table_editing());
        assert!(commits.borrow().is_empty());
        assert_eq!(verdicts.get(), 1);
        locked.set(false);
        submit(&window);
        flush();
        assert!(!window.get_table_editing());
        assert_eq!(verdicts.get(), 2);
        assert_eq!(commits.borrow().len(), 1);
        assert_eq!(commits.borrow()[0].3.value.text.as_str(), "8");
        commits.borrow_mut().clear();
    }
}

#[test]
fn constrained_cells_preserve_the_default_step_base_and_persistent_custom_error() {
    let mut rows = records();
    let constraints = ui::KitInputConstraints {
        kind: ui::HtmlInputKind::Number,
        default_value: "1".into(),
        step: "2".into(),
        min_length: -1,
        max_length: -1,
        ..Default::default()
    };
    rows[0].content = table_cells([
        CellContent::text("Field notes"),
        CellContent::constrained_field(
            "3",
            CellCodec::PlainText,
            FieldAccess::Editable,
            constraints.clone(),
        ),
    ]);
    let Editing {
        window, commits, ..
    } = editing(rows);
    open(&window, 1);
    text(&window, "2");
    key(&window, Key::Return);
    assert!(
        window.get_table_editing(),
        "the step base is 1, so 2 is invalid"
    );
    assert!(commits.borrow().is_empty());
    key(&window, Key::Escape);
    key(&window, Key::F2);
    text(&window, "5");
    key(&window, Key::Return);
    assert!(!window.get_table_editing());
    assert_eq!(commits.borrow().len(), 1);
    commits.borrow_mut().clear();
    let mut rows = records();
    rows[0].content = table_cells([
        CellContent::text("Field notes"),
        CellContent::constrained_field(
            "3",
            CellCodec::PlainText,
            FieldAccess::Editable,
            ui::KitInputConstraints {
                custom_error: "Approval is pending".into(),
                ..constraints
            },
        ),
    ]);
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    open(&window, 1);
    for _ in 0..2 {
        key(&window, Key::Return);
        assert!(window.get_table_editing());
        assert_eq!(window.get_table_edit_error(), "Approval is pending");
        assert!(commits.borrow().is_empty());
    }
}

#[test]
fn constrained_email_and_url_cells_and_rust_acceptance_reject_arbitrary_nonempty_text() {
    let Editing {
        window, commits, ..
    } = editing(records());
    for (kind, initial, invalid, valid) in [
        (
            ui::HtmlInputKind::Email,
            "initial@example.test",
            "not-an-email",
            "ada@example.test",
        ),
        (
            ui::HtmlInputKind::Url,
            "https://initial.example.test",
            "not-a-url",
            "https://example.test/path",
        ),
    ] {
        let constraints = ui::KitInputConstraints {
            kind,
            required: true,
            min_length: -1,
            max_length: -1,
            ..Default::default()
        };
        let mut rows = records();
        rows[0].content = table_cells([
            CellContent::text("Field notes"),
            CellContent::text("12"),
            CellContent::constrained_field(
                initial,
                CellCodec::PlainText,
                FieldAccess::Editable,
                constraints.clone(),
            ),
        ]);
        window.set_rows(ModelRc::new(VecModel::from(rows)));
        window.set_table_active(0);
        open(&window, 2);
        text(&window, invalid);
        key(&window, Key::Return);
        assert!(window.get_table_editing());
        assert!(!window.get_table_edit_error().is_empty());
        assert!(commits.borrow().is_empty());
        let forged = FieldCommit {
            value: ui::FieldValue {
                present: true,
                text: invalid.into(),
            },
            constraints: constraints.clone(),
            ..Default::default()
        };
        assert!(
            owner_spec().accept(forged).is_err(),
            "Rust acceptance must enforce intrinsic constraints too"
        );
        let valid_commit = FieldCommit {
            value: ui::FieldValue {
                present: true,
                text: valid.into(),
            },
            constraints,
            ..Default::default()
        };
        assert!(owner_spec().accept(valid_commit).is_ok());
        key(&window, Key::Escape);
        key(&window, Key::F2);
        text(&window, valid);
        key(&window, Key::Return);
        assert!(!window.get_table_editing());
        assert_eq!(
            commits.borrow().last().unwrap().3.value.text.as_str(),
            valid
        );
        commits.borrow_mut().clear();
    }
}
#[test]
fn constrained_cell_rejections_render_and_keep_native_editor_geometry_in_each_theme_and_scale() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(constrained_records())));
    window.set_table_active(0);
    window.show().unwrap();
    let commits = Rc::new(RefCell::new(0));
    let observed = commits.clone();
    window.on_cell_committed(move |_, _, _, _| *observed.borrow_mut() += 1);
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            window.on_validate_table_edit(|_, _, _, _| "".into());
            window.invoke_focus_table();
            open(&window, 1);
            text(&window, "25");
            key(&window, Key::Return);
            assert!(window.get_table_editing());
            assert!(!window.get_table_edit_error().is_empty());
            let editor = editors(&window).remove(0);
            let row = delegates(&window, "KitDataTable").remove(0);
            assert!(
                editor.absolute_position().y >= row.absolute_position().y
                    && editor.absolute_position().y + editor.size().height
                        <= row.absolute_position().y + row.size().height
            );
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-constraints-intrinsic-{name}-{scale}"),
            );
            key(&window, Key::Escape);
            key(&window, Key::F2);
            text(&window, "8");
            window.on_validate_table_edit(|_, _, _, _| "The record is locked".into());
            key(&window, Key::Return);
            assert!(window.get_table_editing());
            assert_eq!(window.get_table_edit_error(), "The record is locked");
            assert_eq!(*commits.borrow(), 0);
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-constraints-domain-{name}-{scale}"),
            );
            key(&window, Key::Escape);
        }
    }
}

#[test]
fn scrolling_or_shrinking_an_edited_row_outside_the_viewport_cancels_without_ghost_editors() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::{LogicalPosition, platform::WindowEvent};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    let records = (1..=1000)
        .map(|token| {
            record(
                token,
                &format!("Row {token}"),
                12,
                "Owner",
                FieldAccess::Editable,
            )
        })
        .collect::<Vec<_>>();
    window.set_rows(ModelRc::new(VecModel::from(records)));
    window.set_table_active(0);
    window.show().unwrap();
    flush();
    window.window().take_snapshot().unwrap();
    let commits = Rc::new(std::cell::Cell::new(0));
    let observed = commits.clone();
    window.on_cell_committed(move |_, _, _, _| observed.set(observed.get() + 1));
    let navigated = Rc::new(std::cell::Cell::new(0));
    let observed = navigated.clone();
    window.on_navigate(move |collection, step| {
        assert_eq!(collection, DataFixtureCollection::Table);
        assert_eq!(step, NavigationStep::Next);
        observed.set(observed.get() + 1);
    });
    let mut expected_navigation = 0;
    let wheel = |delta| {
        let bounds = table(&window);
        let position = LogicalPosition::new(
            bounds.absolute_position().x + window.get_theme().spacing.unit * 2.,
            bounds.absolute_position().y + bounds.size().height / 2.,
        );
        window
            .window()
            .dispatch_event(WindowEvent::PointerScrolled {
                position,
                delta_x: 0.,
                delta_y: delta,
            });
        flush();
    };
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            window.set_table_height(240.);
            flush();
            window.window().take_snapshot().unwrap();
            window.set_table_active(0);
            window.invoke_focus_table();
            open(&window, 2);
            text(&window, "Unsaved");
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-scroll-session-editing-{name}-{scale}"),
            );
            wheel(-10000.);
            let closed = window.window().take_snapshot().unwrap();
            flush();
            assert!(window.get_table_scroll() < 0.);
            assert!(!window.get_table_editing());
            assert!(editors(&window).is_empty());
            assert_eq!(commits.get(), 0);
            crate::choice_presentation::save_capture(
                &closed,
                &format!("table-scroll-session-closed-{name}-{scale}"),
            );
            key(&window, Key::DownArrow);
            expected_navigation += 1;
            assert_eq!(
                navigated.get(),
                expected_navigation,
                "scroll cancellation returns keyboard ownership to the table"
            );
            wheel(100000.);
            window.window().take_snapshot().unwrap();
            assert_eq!(window.get_table_scroll(), 0.);
            assert!(editors(&window).is_empty());
            assert_eq!(table(&window).accessible_value().as_deref(), Some("Owner"));
            window.set_table_active(4);
            open(&window, 2);
            text(&window, "Resize draft");
            window.window().take_snapshot().unwrap();
            window.set_table_height(160.);
            flush();
            window.window().take_snapshot().unwrap();
            flush();
            assert!(
                !window.get_table_editing(),
                "shrinking the viewport must also close an offscreen editor"
            );
            assert!(editors(&window).is_empty());
            assert_eq!(commits.get(), 0);
            window.invoke_focus_paste();
            window.set_pasted("".into());
            text(&window, "Destination");
            assert_eq!(window.get_pasted(), "Destination");
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SessionAddress {
    row: i32,
    column: i32,
    generation: i32,
}
#[derive(Debug)]
struct ObservedSessionEnd {
    address: SessionAddress,
    outcome: ui::TableEditOutcome,
    message: String,
}
#[derive(Default)]
struct SessionObserver {
    started: Rc<RefCell<Vec<SessionAddress>>>,
    ended: Rc<RefCell<Vec<ObservedSessionEnd>>>,
}
fn observe_sessions(window: &BaseDataFixture) -> SessionObserver {
    let observer = SessionObserver::default();
    let observed = observer.started.clone();
    window.on_table_edit_started(move |row, column, generation| {
        observed.borrow_mut().push(SessionAddress {
            row,
            column,
            generation,
        });
    });
    let observed = observer.ended.clone();
    window.on_table_edit_ended(move |row, column, generation, outcome, message| {
        observed.borrow_mut().push(ObservedSessionEnd {
            address: SessionAddress {
                row,
                column,
                generation,
            },
            outcome,
            message: message.to_string(),
        });
    });
    observer
}

#[test]
fn edit_session_events_name_the_armed_cell_and_distinguish_unchanged_cancel_and_commit() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    let address = SessionAddress {
        row: 1,
        column: QUANTITY,
        generation: 1,
    };
    open(&window, 1);
    assert_eq!(
        observed.started.borrow().as_slice(),
        std::slice::from_ref(&address)
    );
    assert_eq!(
        (
            window.get_table_edit_row(),
            window.get_table_edit_column(),
            window.get_table_edit_generation()
        ),
        (1, QUANTITY, 1)
    );
    assert_eq!(window.get_table_edit_draft(), "12");
    text(&window, "7");
    assert_eq!(window.get_table_edit_draft(), "7");
    key(&window, Key::Escape);
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::Cancelled
    );
    assert_eq!(observed.ended.borrow()[0].address, address);
    assert!(commits.borrow().is_empty());
    open(&window, 1);
    window.invoke_focus_paste();
    flush();
    assert_eq!(
        observed.ended.borrow()[1].outcome,
        ui::TableEditOutcome::Unchanged
    );
    assert!(observed.ended.borrow()[1].message.is_empty());
    assert!(commits.borrow().is_empty());
    text(&window, "Destination");
    assert_eq!(window.get_pasted(), "Destination");
    window.invoke_focus_table();
    open(&window, 1);
    text(&window, "8");
    key(&window, Key::Return);
    assert_eq!(
        observed.ended.borrow()[2].outcome,
        ui::TableEditOutcome::Committed
    );
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(commits.borrow()[0].3.value.text, "8");
    open(&window, 1);
    key(&window, Key::Return);
    assert_eq!(
        observed.ended.borrow()[3].outcome,
        ui::TableEditOutcome::Unchanged
    );
    assert_eq!(commits.borrow().len(), 2);
    window.set_table_unchanged_edit_policy(ui::TableUnchangedEditPolicy::Skip);
    open(&window, 1);
    key(&window, Key::Return);
    assert_eq!(
        observed.ended.borrow()[4].outcome,
        ui::TableEditOutcome::Unchanged
    );
    assert_eq!(commits.borrow().len(), 2);
    assert_eq!(observed.started.borrow().len(), 5);
    assert_eq!(observed.ended.borrow().len(), 5);
    assert!(
        observed
            .ended
            .borrow()
            .iter()
            .all(|event| event.address == address)
    );
}

#[test]
fn rejected_blur_and_superseding_projection_report_the_last_error_and_original_identity() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    open(&window, 1);
    text(&window, "x");
    window.invoke_focus_paste();
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::RejectedOnBlur
    );
    assert!(!observed.ended.borrow()[0].message.is_empty());
    assert!(commits.borrow().is_empty());
    window.invoke_focus_table();
    open(&window, 2);
    window.set_table_generation(2);
    flush();
    assert_eq!(
        observed.ended.borrow()[1].outcome,
        ui::TableEditOutcome::Superseded
    );
    assert_eq!(
        observed.ended.borrow()[1].address,
        SessionAddress {
            row: 1,
            column: OWNER,
            generation: 1
        }
    );
    open(&window, 2);
    let model = window.get_rows();
    let mut record = model.row_data(0).unwrap();
    record.disabled = true;
    model.set_row_data(0, record);
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(
        observed.ended.borrow()[2].outcome,
        ui::TableEditOutcome::Superseded
    );
    assert_eq!(
        observed.ended.borrow()[2].address,
        SessionAddress {
            row: 1,
            column: OWNER,
            generation: 2
        }
    );
    assert!(commits.borrow().is_empty());
}

#[test]
fn keep_open_blur_retains_the_native_invalid_draft_until_explicit_cancel() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    window.set_table_edit_blur_policy(ui::TableEditBlurPolicy::KeepOpen);
    open(&window, 1);
    text(&window, "x");
    window.invoke_focus_paste();
    flush();
    assert!(window.get_table_editing());
    assert!(observed.ended.borrow().is_empty());
    assert_eq!(window.get_table_edit_draft(), "x");
    assert!(!window.get_table_edit_error().is_empty());
    text(&window, "y");
    assert_eq!(
        window.get_table_edit_draft(),
        "xy",
        "focus retention preserves the native caret"
    );
    assert_eq!(window.get_pasted(), "");
    for _ in 0..3 {
        down(&window, Key::Control);
        text(&window, "z");
        up(&window, Key::Control);
    }
    assert_eq!(
        window.get_table_edit_draft(),
        "12",
        "native undo reaches the value before the rejected blur"
    );
    assert!(observed.ended.borrow().is_empty());
    window.invoke_cancel_table();
    flush();
    assert_eq!(observed.ended.borrow().len(), 1);
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::Cancelled
    );
    assert!(commits.borrow().is_empty());
    window.invoke_focus_paste();
    text(&window, "Destination");
    assert_eq!(window.get_pasted(), "Destination");
}

#[test]
fn an_end_observer_can_open_another_cell_without_redirecting_the_accepted_commit() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let opened = Rc::new(std::cell::Cell::new(false));
    let observed = opened.clone();
    let weak = window.as_weak();
    window.on_table_edit_ended(move |row, column, generation, outcome, _| {
        if outcome == ui::TableEditOutcome::Committed && !observed.replace(true) {
            assert_eq!((row, column, generation), (1, QUANTITY, 1));
            let window = weak.upgrade().unwrap();
            window.set_table_active(1);
            window.set_table_active_column(2);
            window.invoke_edit_table();
        }
    });
    open(&window, 1);
    text(&window, "8");
    key(&window, Key::Return);
    flush();
    assert!(opened.get());
    assert!(window.get_table_editing());
    assert_eq!(
        (window.get_table_edit_row(), window.get_table_edit_column()),
        (2, OWNER)
    );
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(
        (
            commits.borrow()[0].0,
            commits.borrow()[0].1,
            commits.borrow()[0].2
        ),
        (1, QUANTITY, 1)
    );
    assert_eq!(commits.borrow()[0].3.value.text, "8");
    text(&window, "Next owner");
    assert_eq!(window.get_table_edit_draft(), "Next owner");
    key(&window, Key::Escape);
    assert_eq!(commits.borrow().len(), 1);
}

#[test]
fn session_bounds_follow_native_geometry_and_viewport_exit_reports_one_closed_outcome() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::{LogicalPosition, platform::WindowEvent};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    let rows = (1..=1000)
        .map(|token| {
            record(
                token,
                &format!("Row {token}"),
                12,
                "Owner",
                FieldAccess::Editable,
            )
        })
        .collect::<Vec<_>>();
    window.set_rows(ModelRc::new(VecModel::from(rows)));
    window.set_table_active(0);
    window.set_table_edit_blur_policy(ui::TableEditBlurPolicy::KeepOpen);
    window.show().unwrap();
    flush();
    window.window().take_snapshot().unwrap();
    let observed = observe_sessions(&window);
    let commits = Rc::new(std::cell::Cell::new(0));
    let counted = commits.clone();
    window.on_cell_committed(move |_, _, _, _| counted.set(counted.get() + 1));
    let wheel = |delta| {
        let view = table(&window);
        window
            .window()
            .dispatch_event(WindowEvent::PointerScrolled {
                position: LogicalPosition::new(
                    view.absolute_position().x + 8.,
                    view.absolute_position().y + view.size().height / 2.,
                ),
                delta_x: 0.,
                delta_y: delta,
            });
        flush();
    };
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            window.set_columns(columns());
            flush();
            window.window().take_snapshot().unwrap();
            window.invoke_focus_table();
            open(&window, 1);
            text(&window, "x");
            key(&window, Key::Return);
            window.window().take_snapshot().unwrap();
            flush();
            let before = window.get_table_edit_bounds();
            assert!(before.valid && before.width > 0. && before.height > 0.);
            let native = editors(&window).remove(0);
            assert!((before.x - native.absolute_position().x).abs() < 0.5);
            assert!((before.y - native.absolute_position().y).abs() < 0.5);
            assert!((before.width - native.size().width).abs() < 0.5);
            assert!((before.height - native.size().height).abs() < 0.5);
            let columns = window.get_columns();
            let mut first = columns.row_data(0).unwrap();
            first.width = ColumnWidth {
                sizing: ColumnSizing::Fixed,
                basis: 200.,
                ..Default::default()
            };
            columns.set_row_data(0, first);
            flush();
            window.window().take_snapshot().unwrap();
            flush();
            let after = window.get_table_edit_bounds();
            let native = editors(&window).remove(0);
            assert!(after.valid);
            assert!((after.x - before.x).abs() > 1.);
            assert!((after.x - native.absolute_position().x).abs() < 0.5);
            assert!((after.width - native.size().width).abs() < 0.5);
            let invalid = window.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &invalid,
                &format!("table-session-observer-invalid-{name}-{scale}"),
            );
            let previous = observed.ended.borrow().len();
            wheel(-10000.);
            window.window().take_snapshot().unwrap();
            flush();
            assert!(!window.get_table_editing());
            assert!(editors(&window).is_empty());
            assert_eq!(observed.ended.borrow().len(), previous + 1);
            assert_eq!(
                observed.ended.borrow().last().unwrap().outcome,
                ui::TableEditOutcome::ViewportExit
            );
            assert!(!observed.ended.borrow().last().unwrap().message.is_empty());
            assert_eq!(commits.get(), 0);
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-session-observer-ended-{name}-{scale}"),
            );
            wheel(100000.);
            window.window().take_snapshot().unwrap();
            flush();
        }
    }
    assert_eq!(observed.started.borrow().len(), 4);
    assert_eq!(observed.ended.borrow().len(), 4);
}

#[test]
fn a_verdict_that_replaces_the_projection_ends_the_original_session_as_superseded() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    let weak = window.as_weak();
    window.on_validate_table_edit(move |_, _, _, _| {
        weak.upgrade().unwrap().set_table_generation(2);
        "".into()
    });
    open(&window, 2);
    text(&window, "Changed owner");
    key(&window, Key::Return);
    flush();
    assert!(!window.get_table_editing());
    assert!(commits.borrow().is_empty());
    assert_eq!(observed.ended.borrow().len(), 1);
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::Superseded
    );
    assert_eq!(
        observed.ended.borrow()[0].address,
        SessionAddress {
            row: 1,
            column: OWNER,
            generation: 1
        }
    );
}

#[test]
fn removing_an_edited_column_then_focusing_another_field_keeps_that_destination() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    open(&window, 2);
    text(&window, "Unsaved owner");
    let remaining = vec![column(TITLE, "Title"), column(QUANTITY, "Quantity")];
    window.set_columns(ModelRc::new(VecModel::from(remaining)));
    window.invoke_focus_paste();
    flush();
    assert!(!window.get_table_editing());
    assert!(commits.borrow().is_empty());
    assert_eq!(observed.ended.borrow().len(), 1);
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::Superseded
    );
    text(&window, "Destination");
    assert_eq!(window.get_pasted(), "Destination");
}

#[test]
fn programmatic_viewport_commands_end_an_invalid_keep_open_session_once() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(
        (1..=1000)
            .map(|token| {
                record(
                    token,
                    &format!("Row {token}"),
                    12,
                    "Owner",
                    FieldAccess::Editable,
                )
            })
            .collect::<Vec<_>>(),
    )));
    window.set_table_active(0);
    window.set_table_edit_blur_policy(ui::TableEditBlurPolicy::KeepOpen);
    window.show().unwrap();
    flush();
    window.window().take_snapshot().unwrap();
    let observed = observe_sessions(&window);
    let commits = Rc::new(std::cell::Cell::new(0));
    let count = commits.clone();
    window.on_cell_committed(move |_, _, _, _| count.set(count.get() + 1));
    window.invoke_focus_table();
    open(&window, 1);
    text(&window, "x");
    key(&window, Key::Return);
    let error = window.get_table_edit_error();
    assert!(!error.is_empty());
    window.invoke_scroll_table_to(500);
    flush();
    window.window().take_snapshot().unwrap();
    flush();
    assert!(!window.get_table_editing());
    assert!(editors(&window).is_empty());
    assert_eq!(commits.get(), 0);
    assert_eq!(observed.ended.borrow().len(), 1);
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::ViewportExit
    );
    assert_eq!(observed.ended.borrow()[0].message, error.as_str());
    window.invoke_scroll_table_to(0);
    flush();
    window.window().take_snapshot().unwrap();
    assert!(editors(&window).is_empty());
    assert_eq!(observed.ended.borrow().len(), 1);
    assert_eq!(
        window
            .get_rows()
            .row_data(0)
            .unwrap()
            .content
            .row_data(1)
            .unwrap()
            .text,
        "12"
    );
}

#[test]
fn enter_after_a_real_wheel_scroll_reveals_the_active_row_before_mounting_and_focusing_its_editor()
{
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::{LogicalPosition, platform::WindowEvent};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    slint_kit::install_field_specs(
        &window,
        [
            FieldSpec::integer().named(quantity_key()),
            owner_spec().named(owner_key()),
        ],
    )
    .unwrap();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(
        (1..=1000)
            .map(|token| {
                record(
                    token,
                    &format!("Row {token}"),
                    12,
                    "Owner",
                    FieldAccess::Editable,
                )
            })
            .collect::<Vec<_>>(),
    )));
    window.set_table_active(0);
    window.set_table_active_column(1);
    window.show().unwrap();
    let accepted: Commits = Rc::new(RefCell::new(Vec::new()));
    let observed = accepted.clone();
    window.on_cell_committed(move |row, column, generation, commit| {
        observed
            .borrow_mut()
            .push((row, column, generation, commit))
    });
    let frame = || {
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        slint::platform::update_timers_and_animations();
        snapshot
    };
    frame();
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            frame();
            window.invoke_focus_table();
            let table = table(&window);
            let origin = table.absolute_position();
            window
                .window()
                .dispatch_event(WindowEvent::PointerScrolled {
                    position: LogicalPosition::new(
                        origin.x + window.get_theme().spacing.unit * 2.,
                        origin.y + table.size().height / 2.,
                    ),
                    delta_x: 0.,
                    delta_y: -10000.,
                });
            frame();
            assert!(window.get_table_scroll() < 0.);
            assert_eq!(window.get_table_active(), 0);
            assert!(!window.get_table_editing());
            assert!(editors(&window).is_empty());
            // Keep the keyboard owner from before the wheel; do not refocus or call
            // edit-active here, since either would mask an offscreen-start regression.
            key(&window, Key::Return);
            let editing = frame();
            assert!(window.get_table_editing());
            assert_eq!(window.get_table_scroll(), 0.);
            assert_eq!(window.get_table_edit_row(), 1);
            assert_eq!(window.get_table_edit_column(), QUANTITY);
            let editor = editors(&window).remove(0);
            let row = delegates(&window, "KitDataTable").remove(0);
            assert!(editor.absolute_position().y >= row.absolute_position().y);
            assert!(
                editor.absolute_position().y + editor.size().height
                    <= row.absolute_position().y + row.size().height
            );
            assert_eq!(
                labelled(&window, "Quantity, Row 1")
                    .accessible_value()
                    .as_deref(),
                Some("12")
            );
            crate::choice_presentation::save_capture(
                &editing,
                &format!("table-wheel-enter-editing-{name}-{scale}"),
            );
            let before = accepted.borrow().len();
            text(&window, "13");
            key(&window, Key::Return);
            let closed = frame();
            assert!(!window.get_table_editing());
            assert!(editors(&window).is_empty());
            assert_eq!(accepted.borrow().len(), before + 1);
            let values = accepted.borrow();
            let commit = &values[before];
            assert_eq!((commit.0, commit.1, commit.2), (1, QUANTITY, 1));
            assert_eq!(commit.3.value.text, "13");
            drop(values);
            crate::choice_presentation::save_capture(
                &closed,
                &format!("table-wheel-enter-closed-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn editing_a_visible_column_resolves_its_canonical_data_source() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    window.set_columns(ModelRc::new(VecModel::from(vec![owner])));
    open(&window, 0);
    assert_eq!(window.get_table_edit_draft(), "Elliot");
    text(&window, "Maya");
    key(&window, Key::Return);
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(
        (
            commits.borrow()[0].0,
            commits.borrow()[0].1,
            commits.borrow()[0].2
        ),
        (1, OWNER, 1)
    );
    assert_eq!(commits.borrow()[0].3.value.text, "Maya");
    assert_eq!(
        window
            .get_rows()
            .row_data(0)
            .unwrap()
            .content
            .row_data(2)
            .unwrap()
            .text,
        "Elliot",
        "a commit does not mutate canonical data"
    );
}

#[test]
fn hiding_reordering_or_retargeting_an_edited_column_never_redirects_its_draft() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    let mut title = column(TITLE, "Title");
    title.source = slint_kit::ColumnDataIndex::try_from(0usize).unwrap().into();
    let source = Rc::new(VecModel::from(vec![owner.clone(), title.clone()]));
    window.set_columns(source.clone().into());
    enum Mutation {
        Hide,
        Reorder,
        Retarget,
    }
    for (ordinal, action) in [Mutation::Hide, Mutation::Reorder, Mutation::Retarget]
        .into_iter()
        .enumerate()
    {
        source.set_vec(vec![owner.clone(), title.clone()]);
        window.invoke_focus_table();
        open(&window, 0);
        text(&window, "Uncommitted owner");
        match action {
            Mutation::Hide => {
                let mut hidden = owner.clone();
                hidden.visibility = ColumnVisibility::Hidden;
                source.set_row_data(0, hidden);
            }
            Mutation::Reorder => source.set_vec(vec![title.clone(), owner.clone()]),
            Mutation::Retarget => {
                let mut changed = owner.clone();
                changed.source = slint_kit::ColumnDataIndex::try_from(1usize).unwrap().into();
                source.set_row_data(0, changed);
            }
        }
        flush();
        assert!(!window.get_table_editing());
        assert!(commits.borrow().is_empty());
        assert_eq!(observed.ended.borrow().len(), ordinal + 1);
        assert_eq!(
            observed.ended.borrow().last().unwrap().outcome,
            ui::TableEditOutcome::Superseded
        );
        assert_eq!(
            observed.ended.borrow().last().unwrap().address.column,
            OWNER
        );
    }
}

#[test]
fn invalid_or_missing_canonical_sources_are_empty_and_cannot_open_an_editor() {
    let Editing {
        window, commits, ..
    } = editing(records());
    assert!(slint_kit::ColumnDataIndex::try_from(usize::MAX).is_err());
    for index in [-1, 3, i32::MAX] {
        let mut owner = column(OWNER, "Owner");
        owner.source = TableColumnSource {
            mode: ColumnSourceMode::DataIndex,
            index,
        };
        window.set_columns(ModelRc::new(VecModel::from(vec![owner])));
        window.set_table_active_column(0);
        assert_eq!(table(&window).accessible_value().as_deref(), Some(""));
        key(&window, Key::F2);
        assert!(!window.get_table_editing());
        assert!(editors(&window).is_empty());
    }
    assert!(commits.borrow().is_empty());
}

#[test]
fn projected_columns_keep_geometry_navigation_and_commit_identity_in_both_directions() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    let mut title = column(TITLE, "Title");
    title.source = slint_kit::ColumnDataIndex::try_from(0usize).unwrap().into();
    let mut hidden = column(QUANTITY, "Quantity");
    hidden.source = slint_kit::ColumnDataIndex::try_from(1usize).unwrap().into();
    hidden.visibility = ColumnVisibility::Hidden;
    let columns = Rc::new(VecModel::from(vec![owner.clone(), hidden, title]));
    window.set_columns(columns.clone().into());
    window.set_rows(ModelRc::new(VecModel::from(vec![TableRow {
        token: 101,
        content: table_cells([
            CellContent::text("Canonical project"),
            CellContent::field("Hidden quantity", FieldAccess::Editable),
            CellContent::field("Original owner", FieldAccess::Editable),
        ]),
        ..Default::default()
    }])));
    window.set_table_active(0);
    window.show().unwrap();
    let commits: Commits = Rc::default();
    let recorded = commits.clone();
    window.on_cell_committed(move |row, column, generation, commit| {
        recorded
            .borrow_mut()
            .push((row, column, generation, commit))
    });
    for (direction_name, direction) in [
        ("ltr", ChoiceFlow::LeftToRight),
        ("rtl", ChoiceFlow::RightToLeft),
    ] {
        for (name, base) in [
            ("light", window.global::<KitThemes>().get_light()),
            ("dark", window.global::<KitThemes>().get_dark()),
        ] {
            for scale in [1., 2.] {
                let mut theme = base.clone();
                theme.direction = direction;
                theme.text_scale = scale;
                window.set_theme(theme);
                window.on_validate_table_edit(|_, _, _, _| "".into());
                columns.set_row_data(0, owner.clone());
                window.invoke_focus_table();
                window.set_table_active_column(0);
                window.window().take_snapshot().unwrap();
                flush();
                assert_eq!(
                    table(&window).accessible_value().as_deref(),
                    Some("Original owner")
                );
                let headers: Vec<_> =
                    ElementHandle::find_by_element_id(&window, "KitDataTable::header-cell")
                        .collect();
                let cells: Vec<_> =
                    ElementHandle::find_by_element_id(&window, "KitDataTable::content-cell")
                        .collect();
                assert_eq!(headers.len(), 2);
                assert_eq!(cells.len(), 2);
                for (header, cell) in headers.iter().zip(&cells) {
                    assert!((header.absolute_position().x - cell.absolute_position().x).abs() < 1.);
                    assert!((header.size().width - cell.size().width).abs() < 1.);
                }
                key(&window, Key::RightArrow);
                assert_eq!(window.get_table_active_column(), 1);
                assert_eq!(
                    table(&window).accessible_value().as_deref(),
                    Some("Canonical project")
                );
                key(&window, Key::LeftArrow);
                key(&window, Key::F2);
                assert!(window.get_table_editing());
                flush();
                window.window().take_snapshot().unwrap();
                text(&window, "Projected owner");
                let frame = window.window().take_snapshot().unwrap();
                crate::choice_presentation::save_capture(
                    &frame,
                    &format!("table-column-source-{direction_name}-{name}-{scale}"),
                );
                key(&window, Key::Return);
                let accepted = commits.borrow();
                let last = accepted.last().unwrap();
                assert_eq!((last.0, last.1), (101, OWNER));
                assert_eq!(last.3.value.text, "Projected owner");
                drop(accepted);
                assert_eq!(
                    window
                        .get_rows()
                        .row_data(0)
                        .unwrap()
                        .content
                        .row_data(2)
                        .unwrap()
                        .text,
                    "Original owner"
                );
                let before = commits.borrow().len();
                let did_reopen = Rc::new(std::cell::Cell::new(false));
                let reopened = did_reopen.clone();
                let weak = window.as_weak();
                let source = columns.clone();
                let mut replacement = owner.clone();
                replacement.source = slint_kit::ColumnDataIndex::try_from(1usize).unwrap().into();
                window.on_validate_table_edit(move |row, column, generation, _| {
                    assert_eq!((row, column, generation), (101, OWNER, 1));
                    if !reopened.replace(true) {
                        let window = weak.upgrade().unwrap();
                        window.invoke_cancel_table();
                        source.set_row_data(0, replacement.clone());
                        window.invoke_edit_table();
                    }
                    "".into()
                });
                window.invoke_edit_table();
                window.window().take_snapshot().unwrap();
                text(&window, "Retired owner draft");
                key(&window, Key::Return);
                flush();
                window.window().take_snapshot().unwrap();
                assert!(did_reopen.get());
                assert!(window.get_table_editing());
                assert_eq!(window.get_table_edit_draft(), "Hidden quantity");
                assert_eq!(commits.borrow().len(), before);
                text(&window, "Reopened source");
                assert_eq!(window.get_table_edit_draft(), "Reopened source");
                assert!(window.get_table_edit_bounds().valid);
                crate::choice_presentation::save_capture(
                    &window.window().take_snapshot().unwrap(),
                    &format!("table-column-source-reopened-{direction_name}-{name}-{scale}"),
                );
                key(&window, Key::Return);
                flush();
                assert!(!window.get_table_editing());
                assert_eq!(commits.borrow().len(), before + 1);
                let accepted = commits.borrow();
                let last = accepted.last().unwrap();
                assert_eq!((last.0, last.1, last.2), (101, OWNER, 1));
                assert_eq!(last.3.value.text, "Reopened source");
            }
        }
    }
    assert_eq!(commits.borrow().len(), 16);
}

#[derive(Clone, Copy)]
enum ReopenDuring {
    VerdictCurrent,
    VerdictRetargeted,
    EndRetargeted,
}
fn assert_reentrant_source_reopen(during: ReopenDuring) {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    let columns = Rc::new(VecModel::from(vec![owner.clone()]));
    window.set_columns(columns.clone().into());
    let (data_index, original, accepted) = match during {
        ReopenDuring::VerdictCurrent => (2usize, "Elliot", "Maya"),
        ReopenDuring::VerdictRetargeted | ReopenDuring::EndRetargeted => (1usize, "12", "7"),
    };
    let mut replacement = owner;
    replacement.source = slint_kit::ColumnDataIndex::try_from(data_index)
        .unwrap()
        .into();
    let did_reopen = Rc::new(std::cell::Cell::new(false));
    let reopened = did_reopen.clone();
    let weak = window.as_weak();
    let reopen = move || {
        let window = weak.upgrade().unwrap();
        window.invoke_cancel_table();
        columns.set_row_data(0, replacement.clone());
        window.invoke_edit_table();
    };
    match during {
        ReopenDuring::VerdictCurrent | ReopenDuring::VerdictRetargeted => {
            window.on_validate_table_edit(move |row, column, generation, _| {
                assert_eq!((row, column, generation), (1, OWNER, 1));
                if !reopened.replace(true) {
                    reopen();
                }
                "".into()
            });
        }
        ReopenDuring::EndRetargeted => {
            window.on_table_edit_ended(move |row, column, generation, outcome, _| {
                assert_eq!((row, column, generation), (1, OWNER, 1));
                if outcome == ui::TableEditOutcome::Committed && !reopened.replace(true) {
                    reopen();
                }
            });
        }
    }
    open(&window, 0);
    text(&window, "Retired owner draft");
    key(&window, Key::Return);
    flush();
    assert!(did_reopen.get());
    assert!(
        window.get_table_editing(),
        "the original native callback must leave the reopened session alone"
    );
    assert_eq!(editors(&window).len(), 1);
    assert_eq!(
        (
            window.get_table_edit_row(),
            window.get_table_edit_column(),
            window.get_table_edit_generation()
        ),
        (1, OWNER, 1)
    );
    assert_eq!(window.get_table_edit_draft(), original);
    assert!(
        commits.borrow().is_empty(),
        "an older session/source must not propose a value for the new one"
    );
    text(&window, accepted);
    assert_eq!(
        window.get_table_edit_draft(),
        accepted,
        "the fresh editor owns focus and selects its canonical value"
    );
    // A reopened native editor must not retain the previous session's undo history.
    for _ in 0..3 {
        down(&window, Key::Control);
        text(&window, "z");
        up(&window, Key::Control);
    }
    assert_eq!(window.get_table_edit_draft(), original);
    down(&window, Key::Control);
    text(&window, "a");
    up(&window, Key::Control);
    text(&window, accepted);
    key(&window, Key::Return);
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(
        (
            commits.borrow()[0].0,
            commits.borrow()[0].1,
            commits.borrow()[0].2
        ),
        (1, OWNER, 1)
    );
    assert_eq!(commits.borrow()[0].3.value.text, accepted);
    assert_eq!(observed.started.borrow().len(), 2);
    let row = window.get_rows().row_data(0).unwrap();
    assert_eq!(row.content.row_data(1).unwrap().text, "12");
    assert_eq!(row.content.row_data(2).unwrap().text, "Elliot");
}
#[test]
fn a_verdict_cannot_consume_a_reopened_session_at_the_same_public_address_and_source() {
    assert_reentrant_source_reopen(ReopenDuring::VerdictCurrent);
}
#[test]
fn a_verdict_cannot_redirect_a_commit_into_a_reopened_canonical_source() {
    assert_reentrant_source_reopen(ReopenDuring::VerdictRetargeted);
}
#[test]
fn an_end_observer_cannot_redirect_a_commit_into_a_reopened_canonical_source() {
    assert_reentrant_source_reopen(ReopenDuring::EndRetargeted);
}

#[test]
fn an_end_observer_hiding_the_original_column_suppresses_the_stale_proposal() {
    let Editing {
        window, commits, ..
    } = editing(records());
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    let columns = Rc::new(VecModel::from(vec![owner.clone()]));
    window.set_columns(columns.clone().into());
    let hidden = columns.clone();
    window.on_table_edit_ended(move |row, column, generation, outcome, _| {
        assert_eq!(
            (row, column, generation, outcome),
            (1, OWNER, 1, ui::TableEditOutcome::Committed)
        );
        let mut hidden_owner = owner.clone();
        hidden_owner.visibility = ColumnVisibility::Hidden;
        hidden.set_row_data(0, hidden_owner);
    });
    open(&window, 0);
    text(&window, "Retired hidden owner");
    key(&window, Key::Return);
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(window.get_table_active_column(), -1);
    assert!(commits.borrow().is_empty());
    assert!(editors(&window).is_empty());
}

#[derive(Clone, Copy)]
enum EndProjection {
    ReorderExplicitSource,
    HidePreviousPositional,
}
fn assert_end_projection_keeps_the_original_source(change: EndProjection) {
    let Editing {
        window, commits, ..
    } = editing(records());
    let initial = columns();
    let mut original: Vec<_> = (0..initial.row_count())
        .map(|index| initial.row_data(index).unwrap())
        .collect();
    match change {
        EndProjection::ReorderExplicitSource => {
            for (index, column) in original.iter_mut().enumerate() {
                column.source = slint_kit::ColumnDataIndex::try_from(index).unwrap().into();
            }
        }
        EndProjection::HidePreviousPositional => {}
    }
    let source = Rc::new(VecModel::from(original));
    window.set_columns(source.clone().into());
    let projected = source.clone();
    window.on_table_edit_ended(move |row, column, generation, outcome, _| {
        assert_eq!(
            (row, column, generation, outcome),
            (1, OWNER, 1, ui::TableEditOutcome::Committed)
        );
        match change {
            EndProjection::ReorderExplicitSource => {
                let mut reordered: Vec<_> = (0..projected.row_count())
                    .map(|index| projected.row_data(index).unwrap())
                    .collect();
                reordered.swap(0, 2);
                projected.set_vec(reordered);
            }
            EndProjection::HidePreviousPositional => {
                let mut previous = projected.row_data(0).unwrap();
                previous.visibility = ColumnVisibility::Hidden;
                projected.set_row_data(0, previous);
            }
        }
    });
    open(&window, 2);
    text(&window, "Maya");
    key(&window, Key::Return);
    flush();
    assert!(!window.get_table_editing());
    assert_eq!(
        commits.borrow().len(),
        1,
        "moving an intact source must preserve its accepted proposal"
    );
    assert_eq!(
        (
            commits.borrow()[0].0,
            commits.borrow()[0].1,
            commits.borrow()[0].2
        ),
        (1, OWNER, 1)
    );
    assert_eq!(commits.borrow()[0].3.value.text, "Maya");
    let row = window.get_rows().row_data(0).unwrap();
    assert_eq!(row.content.row_data(2).unwrap().text, "Elliot");
}
#[test]
fn an_end_observer_reordering_a_column_keeps_its_explicit_source_and_commit_token() {
    assert_end_projection_keeps_the_original_source(EndProjection::ReorderExplicitSource);
}
#[test]
fn an_end_observer_hiding_a_previous_column_keeps_the_original_positional_source() {
    assert_end_projection_keeps_the_original_source(EndProjection::HidePreviousPositional);
}

#[derive(Debug, PartialEq, Eq)]
enum ImmediateCommitRequest {
    Keyboard,
    Programmatic,
}
fn assert_immediate_commit_after_source_replacement(request: ImmediateCommitRequest) {
    let Editing {
        window, commits, ..
    } = editing(records());
    let observed = observe_sessions(&window);
    let mut owner = column(OWNER, "Owner");
    owner.source = slint_kit::ColumnDataIndex::try_from(2usize).unwrap().into();
    let source = Rc::new(VecModel::from(vec![owner.clone()]));
    window.set_columns(source.clone().into());
    let verdicts = Rc::new(std::cell::Cell::new(0));
    let called = verdicts.clone();
    window.on_validate_table_edit(move |_, _, _, _| {
        called.set(called.get() + 1);
        "".into()
    });
    open(&window, 0);
    // This draft is valid under both installed codecs, so only identity
    // fencing can prevent a stale business verdict.
    text(&window, "7");
    owner.source = slint_kit::ColumnDataIndex::try_from(1usize).unwrap().into();
    source.set_row_data(0, owner);
    // Deliberately no timer/frame/flush between replacement and request.
    match request {
        ImmediateCommitRequest::Keyboard => key(&window, Key::Return),
        ImmediateCommitRequest::Programmatic => window.invoke_commit_table(),
    }
    flush();
    assert_eq!(verdicts.get(), 0);
    assert!(commits.borrow().is_empty());
    assert_eq!(
        observed.ended.borrow()[0].outcome,
        ui::TableEditOutcome::Superseded
    );
    if window.get_table_editing() {
        // If Return reaches the table after supersession, opening the new
        // source is legitimate; it must start from its canonical value.
        assert_eq!(request, ImmediateCommitRequest::Keyboard);
        assert_eq!(window.get_table_edit_draft(), "12");
        window.invoke_cancel_table();
    }
}
#[test]
fn an_immediate_keyboard_commit_after_source_replacement_never_calls_a_stale_verdict() {
    assert_immediate_commit_after_source_replacement(ImmediateCommitRequest::Keyboard);
}
#[test]
fn an_immediate_programmatic_commit_after_source_replacement_never_calls_a_stale_verdict() {
    assert_immediate_commit_after_source_replacement(ImmediateCommitRequest::Programmatic);
}

#[test]
fn horizontally_scrolled_cell_arrows_keep_the_ring_inside_its_cell_at_every_density() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint::{LogicalPosition, platform::WindowEvent};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    let wide = columns()
        .iter()
        .map(|mut column| {
            column.width = ColumnWidth {
                sizing: ColumnSizing::Fixed,
                basis: 500.,
                ..Default::default()
            };
            column
        })
        .collect::<Vec<_>>();
    window.set_columns(ModelRc::new(VecModel::from(wide)));
    window.set_rows(ModelRc::new(VecModel::from(records())));
    window.set_table_active(0);
    window.show().unwrap();
    let frame = || {
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        slint::platform::update_timers_and_animations();
        snapshot
    };
    frame();
    let viewport = ElementHandle::find_by_element_id(&window, "KitDataTable::list")
        .next()
        .expect("table viewport")
        .query_descendants()
        .match_type_name("Flickable")
        .find_first()
        .expect("native body viewport");
    let committed: Commits = Rc::default();
    let observed = committed.clone();
    window.on_cell_committed(move |row, column, generation, commit| {
        observed
            .borrow_mut()
            .push((row, column, generation, commit));
    });
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            for (size_name, size) in [
                ("compact", ControlSize::Compact),
                ("medium", ControlSize::Medium),
                ("comfortable", ControlSize::Comfortable),
            ] {
                window.set_table_size(size);
                window.invoke_focus_table();
                // Establish the far column, then move the actual native viewport
                // with a horizontal wheel event before each keyboard action.
                window.set_table_active_column(2);
                frame();
                let right_scroll = window.get_table_scroll_x();
                assert!(
                    right_scroll < -250.,
                    "the fixture must have enough real horizontal overflow"
                );
                for (direction, expected, edge, wheel_delta) in [
                    (Key::LeftArrow, 1, "left", 250.),
                    (Key::RightArrow, 2, "right", -250.),
                ] {
                    let active_before_wheel = window.get_table_active_column();
                    let scroll_before_wheel = window.get_table_scroll_x();
                    let row_before_wheel = delegates(&window, "KitDataTable").remove(0);
                    let origin = viewport.absolute_position();
                    window
                        .window()
                        .dispatch_event(WindowEvent::PointerScrolled {
                            position: LogicalPosition::new(
                                origin.x + viewport.size().width / 2.,
                                row_before_wheel.absolute_position().y
                                    + row_before_wheel.size().height / 2.,
                            ),
                            delta_x: wheel_delta,
                            delta_y: 0.,
                        });
                    frame();
                    let wheel_scroll = window.get_table_scroll_x();
                    assert!(
                        (wheel_scroll - scroll_before_wheel).abs() > 1.,
                        "the wheel must actually scroll before {edge}"
                    );
                    assert!(
                        wheel_scroll < -1.,
                        "each arrow begins with a horizontally scrolled viewport"
                    );
                    assert_eq!(
                        window.get_table_active_column(),
                        active_before_wheel,
                        "wheel motion must not select a different cell"
                    );
                    assert_eq!(window.get_table_active(), 0);
                    assert!(!window.get_table_editing());
                    assert!(editors(&window).is_empty());
                    // Do not refocus or reveal the table between wheel and key.
                    key(&window, direction);
                    let snapshot = frame();
                    assert_eq!(window.get_table_active(), 0);
                    assert_eq!(window.get_table_active_column(), expected);
                    assert!(!window.get_table_editing());
                    assert!(
                        editors(&window).is_empty(),
                        "measure the cell frame, never an editor frame"
                    );
                    let row = delegates(&window, "KitDataTable").remove(0);
                    let cells = row
                        .query_descendants()
                        .match_inherits("TableCellView")
                        .find_all();
                    assert!(!cells.is_empty() && cells.len() <= 3);
                    let total_rings = cells
                        .iter()
                        .map(|cell| {
                            cell.query_descendants()
                                .match_inherits("FocusOutline")
                                .find_all()
                                .len()
                        })
                        .sum::<usize>();
                    assert_eq!(total_rings, 1, "only the active cell owns a frame");
                    // Offscreen cells need not appear in the realized test tree.
                    // Identify the active cell by its unique actual frame, never
                    // by treating a visible-child ordinal as a model column.
                    let cell = cells
                        .iter()
                        .find(|cell| {
                            !cell
                                .query_descendants()
                                .match_inherits("FocusOutline")
                                .find_all()
                                .is_empty()
                        })
                        .expect("one visible cell owns the active frame");
                    let rings = cell
                        .query_descendants()
                        .match_inherits("FocusOutline")
                        .find_all();
                    assert_eq!(rings.len(), 1, "the active cell has its own visible ring");
                    let ring = &rings[0];
                    assert!(ring.size().width > 0. && ring.size().height > 0.);
                    assert_eq!(ring.computed_opacity(), 1.);
                    assert!((ring.size().width - cell.size().width).abs() < 1.);
                    assert!((ring.size().height - cell.size().height).abs() < 1.);
                    assert!((ring.absolute_position().x - cell.absolute_position().x).abs() < 1.);
                    assert!((ring.absolute_position().y - cell.absolute_position().y).abs() < 1.);
                    assert!(ring.absolute_position().y >= row.absolute_position().y - 1.);
                    assert!(
                        ring.absolute_position().y + ring.size().height
                            <= row.absolute_position().y + row.size().height + 1.
                    );
                    assert!(ring.absolute_position().x >= viewport.absolute_position().x - 1.);
                    assert!(
                        ring.absolute_position().x + ring.size().width
                            <= viewport.absolute_position().x + viewport.size().width + 1.
                    );
                    if expected == 1 {
                        assert!(window.get_table_scroll_x() > right_scroll);
                        assert_eq!(table(&window).accessible_value().as_deref(), Some("12"));
                    } else {
                        assert!((window.get_table_scroll_x() - right_scroll).abs() < 1.);
                        assert_eq!(table(&window).accessible_value().as_deref(), Some("Elliot"));
                    }
                    crate::choice_presentation::save_capture(
                        &snapshot,
                        &format!("table-scrolled-arrow-{edge}-{size_name}-{name}-{scale}"),
                    );
                }
            }
        }
    }
    assert!(
        committed.borrow().is_empty(),
        "scrolling and cell arrows never submit a value"
    );
}
