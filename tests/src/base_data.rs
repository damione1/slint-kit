// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

pub(crate) fn fixture() -> BaseDataFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.show().unwrap();
    window
}
#[test]
fn empty_content_uses_the_current_host_supplied_label() {
    let window = fixture();
    for label in ["Aucune collection disponible", "No available collections"] {
        window.set_content_empty_label(label.into());
        let message = labelled(&window, label);
        assert!(message.size().width > 0. && message.size().height > 0.);
        assert!(
            ElementQuery::from_root(&window)
                .match_predicate(
                    |element| element.accessible_label().as_deref() == Some("No articles found")
                )
                .find_all()
                .is_empty()
        );
    }
}

#[test]
fn content_thumbnail_uses_an_icon_only_while_the_host_image_is_empty() {
    let window = fixture();
    let articles = Rc::new(VecModel::from(items(1)));
    window.set_items(ModelRc::from(articles.clone()));
    let icons = || content_icons(&window);
    assert_eq!(
        icons().len(),
        1,
        "an absent thumbnail needs a visible fallback"
    );
    let mut article = articles.row_data(0).unwrap();
    article.thumbnail = slint::Image::from_rgb8(slint::SharedPixelBuffer::new(1, 1));
    articles.set_row_data(0, article.clone());
    assert!(icons().is_empty(), "a real thumbnail replaces the fallback");
    article.thumbnail = Default::default();
    articles.set_row_data(0, article);
    assert_eq!(icons().len(), 1, "removing an image restores the fallback");
}
pub(crate) fn labelled(window: &BaseDataFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
pub(crate) fn delegates(window: &BaseDataFixture, component: &str) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits(component)
        .find_first()
        .unwrap()
        .query_descendants()
        .match_inherits("DataAction")
        .find_all()
}
pub(crate) fn down(window: &BaseDataFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
}
pub(crate) fn up(window: &BaseDataFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
pub(crate) fn key(window: &BaseDataFixture, value: Key) {
    down(window, value);
    up(window, value);
}
pub(crate) fn text(window: &BaseDataFixture, value: &str) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: value.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: value.into() });
}
fn shortcut(window: &BaseDataFixture, letter: &str) {
    down(window, Key::Control);
    text(window, letter);
    up(window, Key::Control);
}
// A header cell is the first element carrying the header's accessible name:
// the sortable button, or the static text wrapper that precedes its label.
fn header(window: &BaseDataFixture, label: &str) -> Option<ElementHandle> {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
                && matches!(
                    element.accessible_role(),
                    Some(
                        i_slint_backend_testing::AccessibleRole::Button
                            | i_slint_backend_testing::AccessibleRole::Text
                    )
                )
        })
        .find_first()
}
fn columns() -> ModelRc<TableColumn> {
    ModelRc::new(VecModel::from(column_set()))
}
pub(crate) fn column_set() -> Vec<TableColumn> {
    vec![
        TableColumn {
            visibility: Default::default(),
            source: Default::default(),
            token: 17,
            label: "Product".into(),
            sortable: true,
            sort: DataSortDirection::Ascending,
            alignment: DataAlignment::Leading,
            width: ColumnWidth::default(),
        },
        TableColumn {
            visibility: Default::default(),
            source: Default::default(),
            token: 91,
            label: "Quantity".into(),
            sortable: false,
            sort: DataSortDirection::Unsorted,
            alignment: DataAlignment::Trailing,
            width: ColumnWidth::default(),
        },
    ]
}
pub(crate) fn rows(start: i32, count: i32) -> Vec<TableRow> {
    (0..count)
        .map(|index| TableRow {
            token: start + index,
            cells: ModelRc::new(VecModel::from(vec![
                format!("Product {}", start + index).into(),
                index.to_string().into(),
            ])),
            content: ModelRc::default(),
            selected: false,
            disabled: false,
        })
        .collect()
}
pub(crate) fn items(count: i32) -> Vec<ContentItem> {
    (0..count)
        .map(|index| ContentItem {
            token: index + 1,
            title: format!("Story {}", index + 1).into(),
            excerpt: "Original article excerpt".into(),
            metadata: "Design · 4 minutes".into(),
            link_label: "Read story".into(),
            thumbnail: Default::default(),
            selected: false,
            disabled: false,
        })
        .collect()
}
fn lines(count: i32) -> Vec<TextLine> {
    (0..count)
        .map(|index| TextLine {
            token: index + 1,
            number: index + 1,
            text: format!("Source line {:05}", index + 1).into(),
            selected: false,
        })
        .collect()
}

#[test]
fn large_collections_virtualize_and_end_reveals_the_last_record() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 10_000))));
    window.set_items(ModelRc::new(VecModel::from(items(10_000))));
    window.set_lines(ModelRc::new(VecModel::from(lines(50_000))));
    for component in ["KitDataTable", "KitContentList", "KitVirtualTextView"] {
        let visible = delegates(&window, component);
        assert!(!visible.is_empty(), "{component} must mount visible rows");
        assert!(
            visible.len() < 40,
            "{component} mounted {} delegates",
            visible.len()
        );
    }
    let weak = window.as_weak();
    window.on_navigate(move |collection, step| {
        assert_eq!(step, NavigationStep::Last);
        let window = weak.upgrade().unwrap();
        match collection {
            DataFixtureCollection::Table => window.set_table_active(9_999),
            DataFixtureCollection::Content => window.set_content_active(9_999),
            DataFixtureCollection::Document => window.set_line_active(49_999),
        }
    });
    window.invoke_focus_table();
    key(&window, Key::End);
    let visible = delegates(&window, "KitDataTable");
    assert!(visible.len() < 40);
    assert!(
        visible
            .iter()
            .any(|row| row.accessible_label().as_deref() == Some("Product 10000"))
    );
    assert!(window.get_table_scroll() < -400_000.);
    window.invoke_focus_content();
    key(&window, Key::End);
    let visible = delegates(&window, "KitContentList");
    assert!(visible.len() < 40);
    assert!(visible.iter().any(|row| {
        row.accessible_label()
            .is_some_and(|label| label.starts_with("Story 10000,"))
    }));
    assert!(window.get_content_scroll() < -1_000_000.);
    window.invoke_focus_document();
    key(&window, Key::End);
    let visible = delegates(&window, "KitVirtualTextView");
    assert!(visible.len() < 40);
    assert!(
        visible
            .iter()
            .any(|row| row.accessible_label().as_deref() == Some("50000: Source line 50000"))
    );
    assert!(window.get_line_scroll() < -1_000_000.);
}

#[test]
fn replacing_a_model_generation_cancels_the_armed_table_row() {
    let window = fixture();
    window.set_columns(columns());
    let model = Rc::new(VecModel::from(rows(1, 10_000)));
    window.set_rows(ModelRc::from(model.clone()));
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_table_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    let first = delegates(&window, "KitDataTable").remove(0);
    let origin = first.absolute_position();
    let position = LogicalPosition::new(
        origin.x + first.size().width / 2.,
        origin.y + first.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    model.set_vec(rows(40_001, 10_000));
    window.set_table_generation(2);
    assert_eq!(
        delegates(&window, "KitDataTable")[0]
            .accessible_label()
            .as_deref(),
        Some("Product 40001")
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert!(
        activated.borrow().is_empty(),
        "release must not retarget a newly reused row"
    );
    labelled(&window, "Product 40001").mock_single_click(PointerEventButton::Left);
    assert_eq!(&*activated.borrow(), &[(40_001, 2)]);
}

#[test]
fn native_line_edit_commit_and_copy_leave_the_other_49999_lines_unchanged() {
    let window = fixture();
    let original = lines(50_000);
    let model = Rc::new(VecModel::from(original.clone()));
    window.set_lines(ModelRc::from(model.clone()));
    window.set_line_active(2);
    window.set_line_draft(original[2].text.clone());
    let accepted = model.clone();
    let weak = window.as_weak();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    window.on_line_committed(move |token, generation, text| {
        assert_eq!((token, generation), (3, 1));
        let mut line = accepted.row_data(2).unwrap();
        line.text = text.clone();
        accepted.set_row_data(2, line);
        observed.borrow_mut().push(text);
        weak.upgrade().unwrap().set_line_generation(2);
    });
    let editor = ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Edit selected line"))
        .find_first()
        .unwrap();
    editor.mock_single_click(PointerEventButton::Left);
    shortcut(&window, "a");
    text(&window, "Updated é漢👩‍💻");
    key(&window, Key::Return);
    assert_eq!(commits.borrow().len(), 1);
    assert_eq!(model.row_data(2).unwrap().text.as_str(), "Updated é漢👩‍💻");
    assert!(
        editor.is_valid(),
        "committing does not replace the native editor"
    );
    for (index, line) in original.iter().enumerate().filter(|(index, _)| *index != 2) {
        assert_eq!(
            model.row_data(index).unwrap().text,
            line.text,
            "unexpected edit to line {}",
            index + 1
        );
    }
    let copied = Rc::new(RefCell::new(Vec::new()));
    let observed = copied.clone();
    window.on_line_copied(move |token, generation| observed.borrow_mut().push((token, generation)));
    labelled(&window, "Copy").invoke_accessible_default_action();
    assert_eq!(&*copied.borrow(), &[(3, 2)]);
    window.invoke_focus_paste();
    shortcut(&window, "v");
    assert_eq!(window.get_pasted().as_str(), "Updated é漢👩‍💻");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ColumnKey {
    Product,
    Quantity,
}
impl TryFrom<i32> for ColumnKey {
    type Error = ();
    fn try_from(token: i32) -> Result<Self, Self::Error> {
        match token {
            17 => Ok(Self::Product),
            91 => Ok(Self::Quantity),
            _ => Err(()),
        }
    }
}

// The table tests below are contract proofs in the headless testing backend:
// they check the names, roles, enabled states and actions the kit publishes.
// They do not qualify a screen reader; that needs a named OS, backend, device
// and version.
#[test]
fn headers_emit_stable_column_tokens_and_respect_sortability() {
    let window = fixture();
    window.set_columns(columns());
    let sorted = Rc::new(RefCell::new(Vec::new()));
    let observed = sorted.clone();
    window.on_sort_requested(move |token| {
        observed
            .borrow_mut()
            .push(ColumnKey::try_from(token).unwrap())
    });
    // The assistive action, the pointer and the keyboard follow one rule:
    // a sortable header sorts its column, a static header never does.
    labelled(&window, "Product, ascending").invoke_accessible_default_action();
    labelled(&window, "Quantity").invoke_accessible_default_action();
    assert_eq!(&*sorted.borrow(), &[ColumnKey::Product]);
    labelled(&window, "Product, ascending").mock_single_click(PointerEventButton::Left);
    labelled(&window, "Quantity").mock_single_click(PointerEventButton::Left);
    assert_eq!(&*sorted.borrow(), &[ColumnKey::Product; 2]);
    window.invoke_focus_table();
    down(&window, Key::Shift);
    key(&window, Key::Tab);
    up(&window, Key::Shift);
    key(&window, Key::Return);
    assert_eq!(&*sorted.borrow(), &[ColumnKey::Product; 3]);
    key(&window, Key::Tab);
    key(&window, Key::Return);
    assert_eq!(
        &*sorted.borrow(),
        &[ColumnKey::Product; 3],
        "a static header is not a tab stop"
    );
}

#[test]
fn sort_direction_is_announced_with_host_supplied_words() {
    let window = fixture();
    let model = Rc::new(VecModel::from(column_set()));
    window.set_columns(ModelRc::from(model.clone()));
    let sortable = header(&window, "Product, ascending").expect("sortable header");
    assert_eq!(
        sortable.accessible_role(),
        Some(i_slint_backend_testing::AccessibleRole::Button)
    );
    assert_eq!(sortable.accessible_enabled(), Some(true));
    let fixed = header(&window, "Quantity").expect("static header");
    assert_eq!(
        fixed.accessible_role(),
        Some(i_slint_backend_testing::AccessibleRole::Text),
        "a header that cannot sort is static text, not a button"
    );
    assert_eq!(
        fixed.accessible_enabled(),
        None,
        "static text carries no enabled state"
    );
    window.set_table_ascending_label("croissant".into());
    window.set_table_descending_label("décroissant".into());
    assert!(header(&window, "Product, croissant").is_some());
    assert!(header(&window, "Product, ascending").is_none());
    let mut column = model.row_data(0).unwrap();
    column.sort = DataSortDirection::Descending;
    model.set_row_data(0, column.clone());
    assert!(header(&window, "Product, décroissant").is_some());
    column.sort = DataSortDirection::Unsorted;
    model.set_row_data(0, column);
    assert!(header(&window, "Product").is_some());
    assert!(
        header(&window, "Quantity").is_some(),
        "an unsorted column carries no direction word"
    );
}

#[test]
fn a_blocked_table_rejects_sorting_and_activation_from_every_input() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_active(0);
    let sorted = Rc::new(RefCell::new(Vec::new()));
    let observed = sorted.clone();
    window.on_sort_requested(move |token| {
        observed
            .borrow_mut()
            .push(ColumnKey::try_from(token).unwrap())
    });
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_table_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    let exercise = |window: &BaseDataFixture| {
        labelled(window, "Product, ascending").invoke_accessible_default_action();
        labelled(window, "Product, ascending").mock_single_click(PointerEventButton::Left);
        labelled(window, "Product 2").invoke_accessible_default_action();
        labelled(window, "Product 2").mock_single_click(PointerEventButton::Left);
        window.invoke_focus_table();
        key(window, Key::Return);
    };
    window.set_table_interactive(false);
    assert_eq!(
        labelled(&window, "Product, ascending").accessible_enabled(),
        Some(false)
    );
    assert_eq!(
        labelled(&window, "Quantity").accessible_enabled(),
        None,
        "static text is not a disabled control: it has no enabled state"
    );
    assert_eq!(
        labelled(&window, "Product 1").accessible_enabled(),
        Some(false)
    );
    exercise(&window);
    assert!(sorted.borrow().is_empty());
    assert!(activated.borrow().is_empty());
    window.set_table_interactive(true);
    assert_eq!(
        labelled(&window, "Product, ascending").accessible_enabled(),
        Some(true)
    );
    assert_eq!(
        labelled(&window, "Product 1").accessible_enabled(),
        Some(true)
    );
    exercise(&window);
    assert_eq!(&*sorted.borrow(), &[ColumnKey::Product; 2]);
    assert_eq!(&*activated.borrow(), &[(2, 1), (2, 1), (1, 1)]);
}

#[test]
fn focus_active_row_and_selection_are_three_separate_facts() {
    let window = fixture();
    window.set_columns(columns());
    let mut table_rows = rows(1, 3);
    table_rows[0].selected = true;
    window.set_rows(ModelRc::new(VecModel::from(table_rows)));
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_table_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    // Without an active row the table is named by its label alone.
    labelled(&window, "Products fixture");
    window.set_table_active(2);
    labelled(&window, "Products fixture: Product 3");
    assert_eq!(
        labelled(&window, "Product 1").accessible_item_selected(),
        Some(true)
    );
    assert_eq!(
        labelled(&window, "Product 3").accessible_item_selected(),
        Some(false),
        "the active row is not the selected row"
    );
    // Keyboard activation targets the active row, not the selected one.
    window.invoke_focus_table();
    key(&window, Key::Return);
    assert_eq!(&*activated.borrow(), &[(3, 1)]);
    // Pointer activation reports a cursor request; the host controls the active
    // row and selection independently.
    labelled(&window, "Product 2").mock_single_click(PointerEventButton::Left);
    assert_eq!(&*activated.borrow(), &[(3, 1), (2, 1)]);
    labelled(&window, "Products fixture: Product 3, Quantity");
    assert_eq!(
        labelled(&window, "Product 2").accessible_item_selected(),
        Some(false)
    );
    assert_eq!(
        labelled(&window, "Product 1").accessible_item_selected(),
        Some(true)
    );
    // Clearing the active row removes it from the name and leaves selection.
    window.set_table_active(-1);
    labelled(&window, "Products fixture");
    assert_eq!(
        labelled(&window, "Product 1").accessible_item_selected(),
        Some(true)
    );
}

#[test]
fn disabled_records_reject_keyboard_and_assistive_activation() {
    let window = fixture();
    window.set_columns(columns());
    let mut table_rows = rows(1, 2);
    table_rows[1].disabled = true;
    window.set_rows(ModelRc::new(VecModel::from(table_rows)));
    let mut articles = items(2);
    articles[1].disabled = true;
    window.set_items(ModelRc::new(VecModel::from(articles)));
    window.set_table_active(1);
    window.set_content_active(1);
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_table_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    let observed = activated.clone();
    window.on_content_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    window.invoke_focus_table();
    key(&window, Key::Return);
    key(&window, Key::Space);
    labelled(&window, "Product 2").invoke_accessible_default_action();
    window.invoke_focus_content();
    key(&window, Key::Return);
    key(&window, Key::Space);
    labelled(&window, "Story 2, Design · 4 minutes").invoke_accessible_default_action();
    assert!(activated.borrow().is_empty());
}

#[test]
fn virtual_collections_expose_selection_and_position_without_changing_identity() {
    let window = fixture();
    window.set_columns(columns());
    let table_rows = Rc::new(VecModel::from(rows(41, 3)));
    let articles = Rc::new(VecModel::from(items(3)));
    let document = Rc::new(VecModel::from(lines(3)));
    window.set_rows(ModelRc::from(table_rows.clone()));
    window.set_items(ModelRc::from(articles.clone()));
    window.set_lines(ModelRc::from(document.clone()));
    for label in [
        "Product 42",
        "Story 2, Design · 4 minutes",
        "2: Source line 00002",
    ] {
        let record = labelled(&window, label);
        assert_eq!(record.accessible_item_selectable(), Some(true));
        assert_eq!(record.accessible_item_selected(), Some(false));
        assert_eq!(record.accessible_item_index(), Some(1));
        assert_eq!(record.accessible_item_count(), Some(3));
    }
    let mut row = table_rows.row_data(1).unwrap();
    row.selected = true;
    table_rows.set_row_data(1, row);
    let mut article = articles.row_data(1).unwrap();
    article.selected = true;
    articles.set_row_data(1, article);
    let mut line = document.row_data(1).unwrap();
    line.selected = true;
    document.set_row_data(1, line);
    for label in [
        "Product 42",
        "Story 2, Design · 4 minutes",
        "2: Source line 00002",
    ] {
        assert_eq!(
            labelled(&window, label).accessible_item_selected(),
            Some(true)
        );
    }
}

#[test]
fn enlarged_line_editor_keeps_actions_and_wrapped_errors_below_the_viewport() {
    let window = fixture();
    window.set_lines(ModelRc::new(VecModel::from(lines(10))));
    window.set_line_active(0);
    window.set_line_draft("A native editable draft".into());
    window.set_line_error("Give the selected line a title before saving your changes.".into());
    let editor = labelled(&window, "Edit selected line");
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        window
            .window()
            .set_size(slint::LogicalSize::new(480., 1200.));
        let document = labelled(&window, "Document fixture");
        let bounds = document.absolute_position();
        let size = document.size();
        let viewport = ElementHandle::find_by_element_id(&window, "KitVirtualTextView::list")
            .next()
            .unwrap();
        assert!(viewport.size().height > 0.);
        for label in [
            "Edit selected line",
            "Save line",
            "Copy",
            "Give the selected line a title before saving your changes.",
        ] {
            let control = labelled(&window, label);
            let position = control.absolute_position();
            let control_size = control.size();
            assert!(position.y >= viewport.absolute_position().y + viewport.size().height - 1.);
            assert!(
                position.x >= bounds.x - 1.
                    && position.x + control_size.width <= bounds.x + size.width + 1.
            );
            assert!(position.y + control_size.height <= bounds.y + size.height + 1.);
        }
        assert!(
            editor.is_valid(),
            "responsive layout must retain the native editor"
        );
    }
}

#[test]
fn table_header_and_document_viewport_start_at_their_top_edge() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 100))));
    window.set_lines(ModelRc::new(VecModel::from(lines(100))));
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let table = labelled(&window, "Products fixture");
        let header = labelled(&window, "Product, ascending");
        assert!(
            (header.absolute_position().y - table.absolute_position().y).abs() < 1.,
            "table header must not be implicitly centered behind rows"
        );
        assert!(header.size().height >= window.get_theme().spacing.medium.height);
        let label = header
            .query_descendants()
            .match_type_name("Text")
            .find_first()
            .unwrap();
        assert!(label.size().width > 40. && label.size().height >= 14. * scale);
        assert!(label.computed_opacity() > 0.99);
        let sort_icon = header
            .query_descendants()
            .match_inherits("KitIcon")
            .find_first()
            .unwrap();
        assert!(
            (sort_icon.absolute_position().y + sort_icon.size().height / 2.
                - header.absolute_position().y
                - header.size().height / 2.)
                .abs()
                < 1.
        );
        let first_row = delegates(&window, "KitDataTable").remove(0);
        assert!(
            first_row.absolute_position().y >= header.absolute_position().y + header.size().height
        );
        let document = labelled(&window, "Document fixture");
        let viewport = ElementHandle::find_by_element_id(&window, "KitVirtualTextView::list")
            .next()
            .unwrap();
        assert!(
            (viewport.absolute_position().y - document.absolute_position().y).abs() < 1.,
            "a shortened document viewport must stay above its edit footer"
        );
        let editor = labelled(&window, "Edit selected line");
        assert!(
            editor.absolute_position().y >= viewport.absolute_position().y + viewport.size().height
        );
    }
}

#[test]
fn dragging_away_from_a_row_preserves_composite_arrow_navigation() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    let observed = Rc::new(RefCell::new(Vec::new()));
    let recorded = observed.clone();
    window.on_navigate(move |collection, step| recorded.borrow_mut().push((collection, step)));
    let row = delegates(&window, "KitDataTable").remove(0);
    let origin = row.absolute_position();
    let position = LogicalPosition::new(
        origin.x + row.size().width / 2.,
        origin.y + row.size().height / 2.,
    );
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    let outside = LogicalPosition::new(0., 0.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position: outside });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position: outside,
            button: PointerEventButton::Left,
        });
    key(&window, Key::DownArrow);
    assert_eq!(
        &*observed.borrow(),
        &[(DataFixtureCollection::Table, NavigationStep::Next)]
    );
}

#[test]
fn table_key_hook_can_handle_a_shortcut_before_builtin_navigation() {
    let window = fixture();
    let handled = Rc::new(RefCell::new(Vec::new()));
    let recorded = handled.clone();
    window.on_table_key_event(move |event| {
        recorded.borrow_mut().push(event.text);
        slint_kit::ui::KitKeyResult::Accept
    });
    let navigated = Rc::new(RefCell::new(Vec::new()));
    let recorded = navigated.clone();
    window.on_navigate(move |collection, step| recorded.borrow_mut().push((collection, step)));
    window.invoke_focus_table();
    key(&window, Key::DownArrow);
    assert_eq!(handled.borrow().len(), 1);
    assert!(navigated.borrow().is_empty());
}

#[test]
fn document_editor_actions_accept_localized_labels() {
    let window = fixture();
    window.set_lines(ModelRc::new(VecModel::from(lines(1))));
    window.set_line_active(0);
    window.set_document_editor_label("Modifier la ligne".into());
    window.set_document_save_label("Enregistrer".into());
    window.set_document_copy_label("Copier".into());
    for label in ["Modifier la ligne", "Enregistrer", "Copier"] {
        assert!(labelled(&window, label).size().width > 0.);
    }
}

#[test]
fn blocking_a_table_preserves_row_selection_and_striping_presentation() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_columns(columns());
    let mut records = rows(1, 3);
    records[0].selected = true;
    window.set_rows(ModelRc::new(VecModel::from(records)));
    window.show().unwrap();
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            let before = window.window().take_snapshot().unwrap();
            window.set_table_interactive(false);
            let after = window.window().take_snapshot().unwrap();
            assert_eq!(
                before.as_bytes(),
                after.as_bytes(),
                "inert table paint must remain stable"
            );
            assert_eq!(
                labelled(&window, "Product 1").accessible_enabled(),
                Some(false)
            );
            crate::choice_presentation::save_capture(
                &after,
                &format!("table-inert-{name}-{scale}"),
            );
            window.set_table_interactive(true);
        }
    }
}

#[test]
fn header_tab_order_matches_the_visual_order_in_each_theme_and_scale() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_active(0);
    window.show().unwrap();
    let sorted = Rc::new(std::cell::Cell::new(0));
    let observed = sorted.clone();
    window.on_sort_requested(move |token| {
        assert_eq!(token, 17);
        observed.set(observed.get() + 1);
    });
    let navigated = Rc::new(std::cell::Cell::new(0));
    let observed = navigated.clone();
    window.on_navigate(move |collection, step| {
        assert_eq!(collection, DataFixtureCollection::Table);
        assert_eq!(step, NavigationStep::Next);
        observed.set(observed.get() + 1);
    });
    let mut expected = 0;
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            window.invoke_focus_table();
            down(&window, Key::Shift);
            key(&window, Key::Tab);
            up(&window, Key::Shift);
            key(&window, Key::Return);
            expected += 1;
            assert_eq!(
                sorted.get(),
                expected,
                "the sortable header precedes the body"
            );
            assert_eq!(navigated.get(), expected - 1);
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-tab-order-header-{name}-{scale}"),
            );
            key(&window, Key::Tab);
            key(&window, Key::DownArrow);
            assert_eq!(
                navigated.get(),
                expected,
                "Tab from the last header enters the body"
            );
            assert_eq!(sorted.get(), expected);
            crate::choice_presentation::save_capture(
                &window.window().take_snapshot().unwrap(),
                &format!("table-tab-order-body-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn roving_headers_sort_resize_and_keep_accessible_context_in_an_empty_table() {
    let window = fixture();
    let mut data = column_set();
    data[0].width = ColumnWidth {
        sizing: ColumnSizing::Fixed,
        basis: 200.,
        resizable: true,
        ..Default::default()
    };
    let columns = Rc::new(VecModel::from(data));
    window.set_columns(ModelRc::from(columns.clone()));
    window.set_table_header_focus(ui::TableHeaderFocus::Roving);
    window.set_table_header_label("En-têtes".into());
    let sorted = Rc::new(RefCell::new(Vec::new()));
    let observed = sorted.clone();
    window.on_sort_requested(move |token| observed.borrow_mut().push(token));
    let resized = Rc::new(RefCell::new(Vec::new()));
    let observed = resized.clone();
    window.on_resize_requested(move |token, width| observed.borrow_mut().push((token, width)));
    window.invoke_focus_table();
    key(&window, Key::UpArrow);
    let table = ElementQuery::from_root(&window)
        .match_inherits("KitDataTable")
        .find_first()
        .unwrap();
    assert!(
        table
            .accessible_label()
            .unwrap()
            .contains("En-têtes, Product, ascending")
    );
    assert_eq!(table.accessible_value().as_deref(), Some(""));
    assert_eq!(table.accessible_description().as_deref(), Some(""));
    key(&window, Key::F2);
    assert!(!window.get_table_editing());
    key(&window, Key::Return);
    key(&window, Key::Space);
    table.invoke_accessible_default_action();
    assert_eq!(&*sorted.borrow(), &[17, 17, 17]);
    down(&window, Key::Alt);
    key(&window, Key::RightArrow);
    up(&window, Key::Alt);
    slint::platform::update_timers_and_animations();
    assert_eq!(
        &*resized.borrow(),
        &[(17, 200. + window.get_theme().spacing.unit * 4.)]
    );
    key(&window, Key::RightArrow);
    assert!(
        table
            .accessible_label()
            .unwrap()
            .contains("En-têtes, Quantity")
    );
    key(&window, Key::Return);
    table.invoke_accessible_default_action();
    assert_eq!(sorted.borrow().len(), 3, "static headers do not sort");
    columns.remove(1);
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_table_active_column(), 0);
    window.invoke_resize_table_column(-10000.);
    slint::platform::update_timers_and_animations();
    assert_eq!(resized.borrow().last().unwrap().1, 140.);
    window.set_table_interactive(false);
    window.invoke_sort_table_column();
    window.invoke_resize_table_column(40.);
    slint::platform::update_timers_and_animations();
    assert_eq!(sorted.borrow().len(), 3);
    assert_eq!(resized.borrow().len(), 2);
    window.set_table_interactive(true);
    columns.remove(0);
    slint::platform::update_timers_and_animations();
    assert!(!table.accessible_label().unwrap().contains("En-têtes"));
    assert!(!window.get_table_editing());
}

#[test]
fn roving_headers_render_sortable_and_static_focus_in_each_theme_and_scale() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_active(0);
    window.set_table_header_focus(ui::TableHeaderFocus::Roving);
    window.show().unwrap();
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    let table = ElementQuery::from_root(&window)
        .match_inherits("KitDataTable")
        .find_first()
        .unwrap();
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            let ink = theme.text_primary;
            window.set_theme(theme);
            slint::platform::update_timers_and_animations();
            window.window().take_snapshot().unwrap();
            window.invoke_focus_table();
            key(&window, Key::UpArrow);
            key(&window, Key::LeftArrow);
            slint::platform::update_timers_and_animations();
            assert_eq!(window.get_table_active_column(), 0);
            assert_eq!(table.accessible_value().as_deref(), Some(""));
            let sortable_frame = window.window().take_snapshot().unwrap();
            let row = delegates(&window, "KitDataTable").remove(0);
            assert!(row.size().height > 2. && row.size().width > 2.);
            let left = f64::from(row.absolute_position().x) + 1.;
            let right = left + f64::from(row.size().width) - 2.;
            let top = f64::from(row.absolute_position().y) + 1.;
            let bottom = top + f64::from(row.size().height) - 2.;
            assert!(top >= 0. && bottom < f64::from(sortable_frame.height()));
            assert!(left >= 0. && right < f64::from(sortable_frame.width()));
            crate::choice_presentation::save_capture(
                &sortable_frame,
                &format!("table-roving-sortable-{name}-{scale}"),
            );
            key(&window, Key::RightArrow);
            slint::platform::update_timers_and_animations();
            assert_eq!(window.get_table_active_column(), 1);
            let static_frame = window.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &static_frame,
                &format!("table-roving-static-{name}-{scale}"),
            );
            let body_pixels = sortable_frame
                .as_slice()
                .iter()
                .zip(static_frame.as_slice())
                .enumerate()
                .filter(|(index, _)| {
                    let offset = u32::try_from(*index).unwrap();
                    let y = f64::from(offset / sortable_frame.width());
                    let x = f64::from(offset % sortable_frame.width());
                    y >= top && y < bottom && x >= left && x < right
                })
                .collect::<Vec<_>>();
            assert!(!body_pixels.is_empty());
            let is_ink = |pixel: &slint::Rgba8Pixel| {
                u32::from(pixel.r.abs_diff(ink.red()))
                    + u32::from(pixel.g.abs_diff(ink.green()))
                    + u32::from(pixel.b.abs_diff(ink.blue()))
                    < 16
            };
            assert!(
                body_pixels.iter().any(|(_, (before, _))| is_ink(before)),
                "the initial row must actually paint its text at {name} {scale}, bounds {left}..{right}/{top}..{bottom}"
            );
            assert!(
                body_pixels.iter().any(|(_, (_, after))| is_ink(after)),
                "the row text must survive the header movement at {name} {scale}"
            );
            let repeated_frame = window.window().take_snapshot().unwrap();
            crate::choice_presentation::save_capture(
                &repeated_frame,
                &format!("diagnostic-header-repeat-{name}-{scale}"),
            );
            assert!(
                body_pixels
                    .iter()
                    .all(|(_, (before, after))| before == after),
                "body changed at {name} {scale}, row y={top}..{bottom}, scroll={}/{}, repeat={}",
                window.get_table_scroll(),
                window.get_table_scroll_x(),
                static_frame.as_bytes() == repeated_frame.as_bytes()
            );
            key(&window, Key::DownArrow);
            assert_eq!(window.get_table_active(), 0);
            assert_eq!(table.accessible_value().as_deref(), Some("0"));
        }
    }
}

#[test]
fn roving_headers_share_one_tab_stop_and_only_the_body_can_open_its_cell_editor() {
    let window = fixture();
    window.set_columns(columns());
    let mut data = rows(1, 1);
    data[0].content = slint_kit::table_cells([
        slint_kit::CellContent::field("Editable", slint_kit::FieldAccess::Editable),
        slint_kit::CellContent::text("7"),
    ]);
    window.set_rows(ModelRc::new(VecModel::from(data)));
    window.set_table_active(0);
    window.set_table_active_column(0);
    window.set_table_header_focus(ui::TableHeaderFocus::Roving);
    let navigation = Rc::new(RefCell::new(Vec::new()));
    let observed = navigation.clone();
    window.on_navigate(move |collection, step| observed.borrow_mut().push((collection, step)));
    window.invoke_focus_paste();
    key(&window, Key::Tab);
    key(&window, Key::DownArrow);
    assert_eq!(
        &*navigation.borrow(),
        &[(DataFixtureCollection::Table, NavigationStep::Next)]
    );
    key(&window, Key::UpArrow);
    key(&window, Key::F2);
    assert!(
        !window.get_table_editing(),
        "the header cursor does not edit the body cell"
    );
    key(&window, Key::Tab);
    key(&window, Key::DownArrow);
    assert_eq!(
        navigation.borrow().last(),
        Some(&(DataFixtureCollection::Content, NavigationStep::Next)),
        "one Tab leaves the whole table from its shared header cursor"
    );
    window.invoke_focus_table();
    key(&window, Key::DownArrow);
    key(&window, Key::F2);
    assert!(
        window.get_table_editing(),
        "the same column is editable in the body"
    );
    key(&window, Key::Escape);
    assert!(!window.get_table_editing());
}

#[test]
fn roving_headers_after_disabling_an_individual_header_do_not_restore_stale_focus() {
    let window = fixture();
    window.set_columns(columns());
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_active(0);
    let sorted = Rc::new(std::cell::Cell::new(0));
    let observed = sorted.clone();
    window.on_sort_requested(move |_| observed.set(observed.get() + 1));
    window.invoke_focus_table();
    down(&window, Key::Shift);
    key(&window, Key::Tab);
    up(&window, Key::Shift);
    key(&window, Key::Return);
    assert_eq!(sorted.get(), 1);
    window.set_table_interactive(false);
    slint::platform::update_timers_and_animations();
    key(&window, Key::Return);
    key(&window, Key::Space);
    assert_eq!(sorted.get(), 1);
    window.set_table_header_focus(ui::TableHeaderFocus::Roving);
    window.set_table_interactive(true);
    slint::platform::update_timers_and_animations();
    key(&window, Key::Return);
    assert_eq!(
        sorted.get(),
        1,
        "reenabling does not restore the old header focus"
    );
    let navigated = Rc::new(std::cell::Cell::new(0));
    let observed = navigated.clone();
    window.on_navigate(move |collection, step| {
        assert_eq!(collection, DataFixtureCollection::Table);
        assert_eq!(step, NavigationStep::Next);
        observed.set(observed.get() + 1);
    });
    key(&window, Key::Tab);
    key(&window, Key::DownArrow);
    assert_eq!(navigated.get(), 1);
    key(&window, Key::UpArrow);
    key(&window, Key::Return);
    assert_eq!(sorted.get(), 2, "the new shared cursor can sort normally");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PageRecord(usize);

#[test]
fn page_keys_follow_all_collection_viewports_after_resize_and_text_scaling() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint_kit::KeyedModel;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.set_columns(columns());
    let table_model = KeyedModel::new(
        rows(1, 100)
            .into_iter()
            .enumerate()
            .map(|(index, row)| (PageRecord(index), row)),
    )
    .unwrap();
    let content_model = KeyedModel::new(
        items(100)
            .into_iter()
            .enumerate()
            .map(|(index, row)| (PageRecord(index), row)),
    )
    .unwrap();
    let line_model = KeyedModel::new(
        lines(100)
            .into_iter()
            .enumerate()
            .map(|(index, row)| (PageRecord(index), row)),
    )
    .unwrap();
    table_model.bind(
        &window,
        BaseDataFixture::set_rows,
        BaseDataFixture::set_table_generation,
        BaseDataFixture::set_table_active,
    );
    content_model.bind(
        &window,
        BaseDataFixture::set_items,
        BaseDataFixture::set_content_generation,
        BaseDataFixture::set_content_active,
    );
    line_model.bind(
        &window,
        BaseDataFixture::set_lines,
        BaseDataFixture::set_line_generation,
        BaseDataFixture::set_line_active,
    );
    let table_navigation = table_model.navigation(&window, BaseDataFixture::get_table_page_rows);
    let content_navigation =
        content_model.navigation(&window, BaseDataFixture::get_content_page_rows);
    let line_navigation = line_model.navigation(&window, BaseDataFixture::get_line_page_rows);
    window.on_navigate(move |collection, step| match collection {
        DataFixtureCollection::Table => table_navigation(step),
        DataFixtureCollection::Content => content_navigation(step),
        DataFixtureCollection::Document => line_navigation(step),
    });
    window.show().unwrap();
    window
        .window()
        .set_size(slint::LogicalSize::new(960., 1700.));
    let mut density_changed_page = false;
    for (theme_name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            let mut small_pages = None;
            for large in [false, true] {
                window.set_table_size(ControlSize::Medium);
                window.set_table_height(if large { 400. } else { 240. });
                window.set_content_height(if large { 500. } else { 250. });
                window.set_document_height(if large { 500. } else { 320. });
                window.set_document_access(TextViewAccess::EditLine);
                slint::platform::update_timers_and_animations();
                window.window().take_snapshot().unwrap();
                let pages = [
                    window.get_table_page_rows(),
                    window.get_content_page_rows(),
                    window.get_line_page_rows(),
                ];
                assert!(pages.iter().all(|page| *page > 0));
                if let Some(previous) = small_pages {
                    for (current, previous) in pages.into_iter().zip(previous) {
                        assert!(
                            current > previous,
                            "a larger viewport must expose a larger page"
                        );
                    }
                } else {
                    small_pages = Some(pages);
                }
                for (collection, name, page) in [
                    (DataFixtureCollection::Table, "table", pages[0]),
                    (DataFixtureCollection::Content, "content", pages[1]),
                    (DataFixtureCollection::Document, "document", pages[2]),
                ] {
                    match collection {
                        DataFixtureCollection::Table => {
                            table_model.navigate(slint_kit::NavigationStep::First, 1);
                            window.invoke_focus_table();
                        }
                        DataFixtureCollection::Content => {
                            content_model.navigate(slint_kit::NavigationStep::First, 1);
                            window.invoke_focus_content();
                        }
                        DataFixtureCollection::Document => {
                            line_model.navigate(slint_kit::NavigationStep::First, 1);
                            window.invoke_focus_document();
                        }
                    }
                    slint::platform::update_timers_and_animations();
                    window.window().take_snapshot().unwrap();
                    key(&window, Key::PageDown);
                    slint::platform::update_timers_and_animations();
                    let snapshot = window.window().take_snapshot().unwrap();
                    let actual = match collection {
                        DataFixtureCollection::Table => table_model.active_index(),
                        DataFixtureCollection::Content => content_model.active_index(),
                        DataFixtureCollection::Document => line_model.active_index(),
                    };
                    assert_eq!(
                        actual, page,
                        "{name} PageDown must use its current visible page"
                    );
                    if large {
                        crate::choice_presentation::save_capture(
                            &snapshot,
                            &format!("collection-page-{name}-{theme_name}-{scale}"),
                        );
                    }
                    key(&window, Key::PageUp);
                    slint::platform::update_timers_and_animations();
                    let actual = match collection {
                        DataFixtureCollection::Table => table_model.active_index(),
                        DataFixtureCollection::Content => content_model.active_index(),
                        DataFixtureCollection::Document => line_model.active_index(),
                    };
                    assert_eq!(actual, 0, "{name} PageUp must use the same viewport page");
                }
                if large {
                    window.set_table_size(ControlSize::Compact);
                    slint::platform::update_timers_and_animations();
                    window.window().take_snapshot().unwrap();
                    let compact_page = window.get_table_page_rows();
                    let compact_row_height =
                        f64::from(delegates(&window, "KitDataTable")[0].size().height);
                    let compact_viewport_height = f64::from(
                        ElementHandle::find_by_element_id(&window, "KitDataTable::list")
                            .next()
                            .unwrap()
                            .query_descendants()
                            .match_type_name("Flickable")
                            .find_first()
                            .unwrap()
                            .size()
                            .height,
                    );
                    assert!(compact_row_height > 0. && compact_viewport_height > 0.);
                    assert_eq!(
                        f64::from(compact_page),
                        (compact_viewport_height / compact_row_height)
                            .floor()
                            .max(1.)
                    );
                    table_model.navigate(slint_kit::NavigationStep::First, 1);
                    window.invoke_focus_table();
                    key(&window, Key::PageDown);
                    let compact_destination = table_model.active_index();
                    assert_eq!(compact_destination, compact_page);
                    window.set_table_size(ControlSize::Comfortable);
                    slint::platform::update_timers_and_animations();
                    window.window().take_snapshot().unwrap();
                    let comfortable_page = window.get_table_page_rows();
                    let comfortable_row_height =
                        f64::from(delegates(&window, "KitDataTable")[0].size().height);
                    let comfortable_viewport_height = f64::from(
                        ElementHandle::find_by_element_id(&window, "KitDataTable::list")
                            .next()
                            .unwrap()
                            .query_descendants()
                            .match_type_name("Flickable")
                            .find_first()
                            .unwrap()
                            .size()
                            .height,
                    );
                    assert!(comfortable_row_height > compact_row_height);
                    assert!(comfortable_viewport_height > 0.);
                    assert_eq!(
                        f64::from(comfortable_page),
                        (comfortable_viewport_height / comfortable_row_height)
                            .floor()
                            .max(1.)
                    );
                    assert!(compact_page >= comfortable_page);
                    table_model.navigate(slint_kit::NavigationStep::First, 1);
                    window.invoke_focus_table();
                    key(&window, Key::PageDown);
                    let comfortable_destination = table_model.active_index();
                    assert_eq!(comfortable_destination, comfortable_page);
                    if compact_page > comfortable_page {
                        assert_ne!(compact_destination, comfortable_destination);
                        density_changed_page = true;
                    }
                }
                let edit_pages = window.get_line_page_rows();
                window.set_document_access(TextViewAccess::ReadOnly);
                slint::platform::update_timers_and_animations();
                window.window().take_snapshot().unwrap();
                assert!(
                    window.get_line_page_rows() > edit_pages,
                    "hiding the edit footer must enlarge the document page"
                );
            }
        }
    }
    assert!(
        density_changed_page,
        "at least one density change must alter the page and actual navigation distance"
    );
}

struct ScrollProbe {
    name: &'static str,
    scroll_to: fn(&BaseDataFixture, i32),
    restore: fn(&BaseDataFixture, f32, f32),
    top: fn(&BaseDataFixture),
    reveal: fn(&BaseDataFixture),
    first: fn(&BaseDataFixture) -> i32,
    count: fn(&BaseDataFixture) -> i32,
    page: fn(&BaseDataFixture) -> i32,
    end_near: fn(&BaseDataFixture) -> bool,
    x: fn(&BaseDataFixture) -> f32,
    y: fn(&BaseDataFixture) -> f32,
    active: fn(&BaseDataFixture) -> i32,
    set_active: fn(&BaseDataFixture, i32),
    near_events: Rc<std::cell::Cell<usize>>,
}

#[test]
fn collection_scroll_commands_restore_bounded_ranges_and_signal_near_end_without_selecting() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use std::cell::Cell;
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    let counts = [
        Rc::new(Cell::new(0)),
        Rc::new(Cell::new(0)),
        Rc::new(Cell::new(0)),
    ];
    let observed = counts.clone();
    window.on_near_end(move |collection| {
        let counter = match collection {
            DataFixtureCollection::Table => &observed[0],
            DataFixtureCollection::Content => &observed[1],
            DataFixtureCollection::Document => &observed[2],
        };
        counter.set(counter.get() + 1);
    });
    let mut headers = column_set();
    for column in &mut headers {
        column.width = ColumnWidth {
            sizing: ColumnSizing::Fixed,
            basis: 700.,
            ..Default::default()
        };
    }
    window.set_columns(ModelRc::new(VecModel::from(headers)));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 1000))));
    window.set_items(ModelRc::new(VecModel::from(items(1000))));
    window.set_lines(ModelRc::new(VecModel::from(lines(1000))));
    window.set_table_active(0);
    window.set_content_active(0);
    window.set_line_active(0);
    window.show().unwrap();
    let probes = [
        ScrollProbe {
            name: "table",
            scroll_to: BaseDataFixture::invoke_scroll_table_to,
            restore: BaseDataFixture::invoke_restore_table_scroll,
            top: BaseDataFixture::invoke_table_to_top,
            reveal: BaseDataFixture::invoke_reveal_table,
            first: BaseDataFixture::get_table_first_visible,
            count: BaseDataFixture::get_table_visible_rows,
            page: BaseDataFixture::get_table_page_rows,
            end_near: BaseDataFixture::get_table_end_near,
            x: BaseDataFixture::get_table_scroll_x,
            y: BaseDataFixture::get_table_scroll,
            active: BaseDataFixture::get_table_active,
            set_active: BaseDataFixture::set_table_active,
            near_events: counts[0].clone(),
        },
        ScrollProbe {
            name: "content",
            scroll_to: BaseDataFixture::invoke_scroll_content_to,
            restore: BaseDataFixture::invoke_restore_content_scroll,
            top: BaseDataFixture::invoke_content_to_top,
            reveal: BaseDataFixture::invoke_reveal_content,
            first: BaseDataFixture::get_content_first_visible,
            count: BaseDataFixture::get_content_visible_rows,
            page: BaseDataFixture::get_content_page_rows,
            end_near: BaseDataFixture::get_content_end_near,
            x: BaseDataFixture::get_content_scroll_x,
            y: BaseDataFixture::get_content_scroll,
            active: BaseDataFixture::get_content_active,
            set_active: BaseDataFixture::set_content_active,
            near_events: counts[1].clone(),
        },
        ScrollProbe {
            name: "document",
            scroll_to: BaseDataFixture::invoke_scroll_line_to,
            restore: BaseDataFixture::invoke_restore_line_scroll,
            top: BaseDataFixture::invoke_line_to_top,
            reveal: BaseDataFixture::invoke_reveal_line,
            first: BaseDataFixture::get_line_first_visible,
            count: BaseDataFixture::get_line_visible_rows,
            page: BaseDataFixture::get_line_page_rows,
            end_near: BaseDataFixture::get_line_end_near,
            x: BaseDataFixture::get_line_scroll_x,
            y: BaseDataFixture::get_line_scroll,
            active: BaseDataFixture::get_line_active,
            set_active: BaseDataFixture::set_line_active,
            near_events: counts[2].clone(),
        },
    ];
    let frame = || {
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        slint::platform::update_timers_and_animations();
        snapshot
    };
    frame();
    for (theme_name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            frame();
            for probe in &probes {
                (probe.top)(&window);
                frame();
                assert_eq!((probe.first)(&window), 0);
                (probe.scroll_to)(&window, 500);
                let snapshot = frame();
                let offset = (probe.y)(&window);
                assert!(offset < 0.);
                assert_eq!((probe.first)(&window), 500);
                assert!((probe.count)(&window) >= (probe.page)(&window));
                assert!((probe.count)(&window) <= (probe.page)(&window) + 1);
                assert_eq!((probe.active)(&window), 0);
                crate::choice_presentation::save_capture(
                    &snapshot,
                    &format!("collection-scroll-{}-{theme_name}-{scale}", probe.name),
                );
                (probe.restore)(&window, 0., 0.);
                frame();
                assert_eq!((probe.first)(&window), 0);
                (probe.restore)(&window, -1_000_000., offset);
                frame();
                assert_eq!((probe.first)(&window), 500);
                assert!(((probe.y)(&window) - offset).abs() < 0.01);
                let horizontal = (probe.x)(&window);
                assert!(horizontal <= 0.);
                (probe.restore)(&window, 0., 0.);
                frame();
                (probe.restore)(&window, horizontal, offset);
                frame();
                assert_eq!((probe.x)(&window), horizontal);
                (probe.scroll_to)(&window, 950);
                frame();
                assert!(!(probe.end_near)(&window));
                let before = probe.near_events.get();
                window.set_near_end_rows(100);
                frame();
                assert!((probe.end_near)(&window));
                assert_eq!(probe.near_events.get(), before + 1);
                window.set_near_end_rows(-1);
                frame();
                assert!(!(probe.end_near)(&window));
                window.set_near_end_rows(3);
                frame();
                (probe.scroll_to)(&window, i32::MAX);
                frame();
                assert!((probe.end_near)(&window));
                assert_eq!((probe.first)(&window) + (probe.count)(&window), 1000);
                let at_end = probe.near_events.get();
                (probe.scroll_to)(&window, i32::MAX);
                frame();
                assert_eq!(
                    probe.near_events.get(),
                    at_end,
                    "near-end is an edge, not repeated polling"
                );
                (probe.top)(&window);
                frame();
                assert!(!(probe.end_near)(&window));
                assert_eq!((probe.x)(&window), 0.);
                assert_eq!((probe.y)(&window), 0.);
                (probe.scroll_to)(&window, 999);
                frame();
                assert_eq!(probe.near_events.get(), at_end + 1);
                assert_eq!((probe.active)(&window), 0);
                (probe.scroll_to)(&window, -100);
                frame();
                assert_eq!((probe.y)(&window), 0.);
                (probe.set_active)(&window, 700);
                frame();
                (probe.top)(&window);
                frame();
                (probe.reveal)(&window);
                frame();
                assert!((probe.first)(&window) <= 700);
                assert!((probe.first)(&window) + (probe.count)(&window) > 700);
                assert_eq!((probe.active)(&window), 700);
                (probe.set_active)(&window, 0);
                frame();
            }
        }
    }
    window.set_rows(ModelRc::default());
    window.set_items(ModelRc::default());
    window.set_lines(ModelRc::default());
    frame();
    for probe in &probes {
        (probe.scroll_to)(&window, i32::MAX);
        (probe.reveal)(&window);
        frame();
        assert_eq!((probe.first)(&window), -1);
        assert_eq!((probe.count)(&window), 0);
        assert!(!(probe.end_near)(&window));
        assert_eq!((probe.y)(&window), 0.);
    }
}

#[test]
fn body_cell_cursor_clamps_after_column_removal_and_never_restores_an_obsolete_cell() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    let columns = Rc::new(VecModel::from(column_set()));
    window.set_columns(ModelRc::from(columns.clone()));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 3))));
    window.set_table_active(0);
    window.show().unwrap();
    let frame = || {
        slint::platform::update_timers_and_animations();
        let snapshot = window.window().take_snapshot().unwrap();
        slint::platform::update_timers_and_animations();
        snapshot
    };
    frame();
    let table = ElementQuery::from_root(&window)
        .match_inherits("KitDataTable")
        .find_first()
        .unwrap();
    for (name, theme) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            let mut theme = theme.clone();
            theme.text_scale = scale;
            window.set_theme(theme);
            columns.set_vec(column_set());
            frame();
            window.invoke_focus_table();
            window.set_table_active_column(1);
            frame();
            assert_eq!(table.accessible_value().as_deref(), Some("0"));
            columns.remove(1);
            let surviving = frame();
            assert_eq!(window.get_table_active_column(), 0);
            assert_eq!(table.accessible_value().as_deref(), Some("Product 1"));
            assert!(!table.accessible_label().unwrap().contains("Quantity"));
            crate::choice_presentation::save_capture(
                &surviving,
                &format!("table-column-clamp-surviving-{name}-{scale}"),
            );
            columns.remove(0);
            let empty = frame();
            assert_eq!(window.get_table_active_column(), -1);
            assert_eq!(table.accessible_value().as_deref(), Some(""));
            assert_eq!(window.get_table_active(), 0);
            window.invoke_edit_table();
            frame();
            assert!(!window.get_table_editing());
            crate::choice_presentation::save_capture(
                &empty,
                &format!("table-column-clamp-empty-{name}-{scale}"),
            );
            columns.push(column_set().remove(0));
            frame();
            assert_eq!(window.get_table_active_column(), -1);
            key(&window, Key::RightArrow);
            frame();
            assert_eq!(window.get_table_active_column(), 0);
            assert_eq!(table.accessible_value().as_deref(), Some("Product 1"));
            columns.push(column_set().remove(1));
            frame();
            window.set_table_active_column(100);
            frame();
            assert_eq!(window.get_table_active_column(), 1);
            window.set_table_active_column(-20);
            frame();
            assert_eq!(window.get_table_active_column(), -1);
        }
    }
}

fn content_icons(window: &BaseDataFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("KitContentList")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_inherits("KitIcon")
        .find_all()
}

fn content_link(window: &BaseDataFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("DataAction")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Read story"))
        .find_first()
        .expect("missing link")
}

fn content_rows(window: &BaseDataFixture) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits("KitContentList")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_accessible_role(i_slint_backend_testing::AccessibleRole::ListItem)
        .find_all()
}

#[test]
fn content_row_name_is_built_from_its_non_empty_parts() {
    let window = fixture();
    let mut articles = items(2);
    articles[0].metadata = "".into();
    articles[1].title = "".into();
    window.set_items(ModelRc::from(Rc::new(VecModel::from(articles))));
    assert_eq!(
        content_rows(&window)[0].accessible_label().as_deref(),
        Some("Story 1")
    );
    assert_eq!(
        content_rows(&window)[1].accessible_label().as_deref(),
        Some("Design · 4 minutes")
    );
    assert_eq!(
        content_rows(&window)[0].accessible_description().as_deref(),
        Some("Original article excerpt"),
        "the excerpt is the row's description, not part of its name"
    );
}

#[test]
fn content_thumbnail_policy_chooses_image_icon_or_nothing() {
    let window = fixture();
    let mut articles = items(1);
    articles[0].thumbnail = slint::Image::from_rgb8(slint::SharedPixelBuffer::new(1, 1));
    window.set_items(ModelRc::from(Rc::new(VecModel::from(articles))));
    let icons = || content_icons(&window).len();
    assert_eq!(icons(), 0, "the default shows the host image");
    window.set_content_thumbnail(ContentThumbnail::Icon);
    assert_eq!(icons(), 1, "icon ignores the host image");
    window.set_content_thumbnail(ContentThumbnail::None);
    assert_eq!(icons(), 0);
}

#[test]
fn content_row_height_follows_size_and_excerpt_lines() {
    let window = fixture();
    window.set_items(ModelRc::from(Rc::new(VecModel::from(items(3)))));
    let height = |window: &BaseDataFixture| content_rows(window)[0].size().height;
    let medium = height(&window);
    window.set_content_control_size(ControlSize::Compact);
    let compact = height(&window);
    window.set_content_control_size(ControlSize::Comfortable);
    let comfortable = height(&window);
    assert!(compact < medium && medium < comfortable);
    window.set_content_control_size(ControlSize::Medium);
    assert_eq!(height(&window), medium);
    window.set_content_excerpt_lines(4);
    assert!(
        height(&window) > medium,
        "more excerpt lines make taller rows"
    );
    window.set_content_excerpt_lines(0);
    assert!(height(&window) < medium, "the minimum is one excerpt line");
}

#[test]
fn content_link_is_a_separate_action_from_the_row() {
    let window = fixture();
    window.set_items(ModelRc::from(Rc::new(VecModel::from(items(2)))));
    let links = Rc::new(RefCell::new(Vec::new()));
    let rows = Rc::new(RefCell::new(Vec::new()));
    let observed = links.clone();
    window
        .on_content_link(move |token, generation| observed.borrow_mut().push((token, generation)));
    let observed = rows.clone();
    window.on_content_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    window.set_content_generation(7);
    let link = content_link(&window);
    assert_eq!(
        link.accessible_role(),
        Some(i_slint_backend_testing::AccessibleRole::Button)
    );
    link.invoke_accessible_default_action();
    assert_eq!(*links.borrow(), vec![(1, 7)]);
    assert!(rows.borrow().is_empty(), "the link does not open the row");
    let origin = link.absolute_position();
    let position = LogicalPosition::new(
        origin.x + link.size().width / 2.,
        origin.y + link.size().height / 2.,
    );
    for event in [
        WindowEvent::PointerMoved { position },
        WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        },
        WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        },
    ] {
        window.window().dispatch_event(event);
    }
    assert_eq!(links.borrow().len(), 2);
    assert!(rows.borrow().is_empty());
    let mut articles = items(1);
    articles[0].link_label = "".into();
    window.set_items(ModelRc::from(Rc::new(VecModel::from(articles))));
    assert!(
        ElementQuery::from_root(&window)
            .match_predicate(|element| element.accessible_label().as_deref() == Some("Read story"))
            .find_all()
            .is_empty(),
        "an empty link label draws no link"
    );
}

#[test]
fn a_non_interactive_content_list_ignores_rows_links_and_keys() {
    let window = fixture();
    window.set_items(ModelRc::from(Rc::new(VecModel::from(items(2)))));
    window.set_content_active(0);
    let events = Rc::new(RefCell::new(0));
    let observed = events.clone();
    window.on_content_activated(move |_, _| *observed.borrow_mut() += 1);
    let observed = events.clone();
    window.on_content_link(move |_, _| *observed.borrow_mut() += 1);
    window.set_content_interactive(false);
    let row = labelled(&window, "Story 1, Design · 4 minutes");
    assert_eq!(row.accessible_enabled(), Some(false));
    assert_eq!(content_link(&window).accessible_enabled(), Some(false));
    row.invoke_accessible_default_action();
    row.mock_single_click(PointerEventButton::Left);
    content_link(&window).invoke_accessible_default_action();
    window.invoke_focus_content();
    key(&window, Key::Return);
    key(&window, Key::Space);
    assert_eq!(*events.borrow(), 0);
    window.set_content_interactive(true);
    assert_eq!(row.accessible_enabled(), Some(true));
    row.invoke_accessible_default_action();
    assert_eq!(*events.borrow(), 1);
}
