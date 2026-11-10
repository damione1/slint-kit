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
