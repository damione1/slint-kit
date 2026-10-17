// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

fn fixture() -> BaseDataFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = BaseDataFixture::new().unwrap();
    window.show().unwrap();
    window
}
fn labelled(window: &BaseDataFixture, label: &str) -> ElementHandle {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn delegates(window: &BaseDataFixture, component: &str) -> Vec<ElementHandle> {
    ElementQuery::from_root(window)
        .match_inherits(component)
        .find_first()
        .unwrap()
        .query_descendants()
        .match_inherits("DataAction")
        .find_all()
}
fn down(window: &BaseDataFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
}
fn up(window: &BaseDataFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn key(window: &BaseDataFixture, value: Key) {
    down(window, value);
    up(window, value);
}
fn text(window: &BaseDataFixture, value: &str) {
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
fn columns() -> ModelRc<TableColumn> {
    ModelRc::new(VecModel::from(vec![
        TableColumn {
            token: 17,
            label: "Product".into(),
            sortable: true,
            sort: DataSortDirection::Ascending,
            alignment: DataAlignment::Leading,
        },
        TableColumn {
            token: 91,
            label: "Quantity".into(),
            sortable: false,
            sort: DataSortDirection::Unsorted,
            alignment: DataAlignment::Trailing,
        },
    ]))
}
fn rows(start: i32, count: i32) -> Vec<TableRow> {
    (0..count)
        .map(|index| TableRow {
            token: start + index,
            cells: ModelRc::new(VecModel::from(vec![
                format!("Product {}", start + index).into(),
                index.to_string().into(),
            ])),
            selected: false,
            disabled: false,
        })
        .collect()
}
fn items(count: i32) -> Vec<ContentItem> {
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
    let header = labelled(&window, "Product, ascending");
    header.invoke_accessible_default_action();
    labelled(&window, "Quantity").invoke_accessible_default_action();
    assert_eq!(&*sorted.borrow(), &[ColumnKey::Product]);
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
        assert!(header.size().height >= 48.);
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
