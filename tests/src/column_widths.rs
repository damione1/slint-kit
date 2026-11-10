// SPDX-License-Identifier: GPL-3.0-or-later
//! Column width policies of `KitDataTable`: geometry of fill, fixed and
//! bounded columns, header/row alignment under horizontal scroll, and the
//! pointer, keyboard and assistive resize contract. These are headless
//! contract proofs; they do not qualify a screen reader.
use super::base_data::{column_set, delegates, down, fixture, key, labelled, rows, up};
use super::*;
use i_slint_backend_testing::{AccessibleRole, ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

const PRODUCT: i32 = 17;

fn fill() -> ColumnWidth {
    ColumnWidth::default()
}
fn fixed(basis: f32) -> ColumnWidth {
    ColumnWidth {
        sizing: ColumnSizing::Fixed,
        basis,
        ..ColumnWidth::default()
    }
}
fn bounded(width: ColumnWidth, minimum: f32, maximum: f32) -> ColumnWidth {
    ColumnWidth {
        minimum,
        ceiling: ColumnCeiling::Bounded,
        maximum,
        ..width
    }
}
fn resizable(width: ColumnWidth) -> ColumnWidth {
    ColumnWidth {
        resizable: true,
        ..width
    }
}
fn with_widths(product: ColumnWidth, quantity: ColumnWidth) -> Vec<TableColumn> {
    let mut columns = column_set();
    columns[0].width = product;
    columns[1].width = quantity;
    columns
}
fn header_cells(window: &BaseDataFixture) -> Vec<ElementHandle> {
    ElementHandle::find_by_element_id(window, "KitDataTable::header-cell").collect()
}
fn row_cells(window: &BaseDataFixture, index: usize) -> Vec<ElementHandle> {
    delegates(window, "KitDataTable")
        .remove(index)
        .query_descendants()
        .match_id("KitDataTable::row-cell")
        .find_all()
}
fn widths(cells: &[ElementHandle]) -> Vec<f32> {
    cells.iter().map(|cell| cell.size().width).collect()
}
fn near(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < 1.
}
fn assert_aligned(window: &BaseDataFixture) {
    let header = header_cells(window);
    for index in 0..2 {
        let row = row_cells(window, index);
        assert_eq!(header.len(), row.len(), "one row cell per header cell");
        for (header_cell, row_cell) in header.iter().zip(&row) {
            assert!(
                near(
                    header_cell.absolute_position().x,
                    row_cell.absolute_position().x
                ) && near(header_cell.size().width, row_cell.size().width),
                "header {:?}/{} and row {:?}/{} must share x and width",
                header_cell.absolute_position(),
                header_cell.size().width,
                row_cell.absolute_position(),
                row_cell.size().width
            );
        }
    }
}
fn visible_width(window: &BaseDataFixture) -> f32 {
    ElementHandle::find_by_element_id(window, "KitDataTable::list")
        .next()
        .expect("table viewport")
        .query_descendants()
        .match_type_name("Flickable")
        .find_first()
        .expect("viewport flickable")
        .size()
        .width
}
fn handle(window: &BaseDataFixture, label: &str) -> Option<ElementHandle> {
    let requested = label.to_owned();
    ElementQuery::from_root(window)
        .match_accessible_role(AccessibleRole::Spinbox)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested.as_str())
        })
        .find_first()
}
fn center(element: &ElementHandle) -> LogicalPosition {
    let origin = element.absolute_position();
    LogicalPosition::new(
        origin.x + element.size().width / 2.,
        origin.y + element.size().height / 2.,
    )
}
fn pointer_move(window: &BaseDataFixture, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
}
fn pointer_down(window: &BaseDataFixture, position: LogicalPosition) {
    pointer_move(window, position);
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
}
fn pointer_up(window: &BaseDataFixture, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}
fn observe_resizes(window: &BaseDataFixture) -> Rc<RefCell<Vec<(i32, f32)>>> {
    let requested = Rc::new(RefCell::new(Vec::new()));
    let observed = requested.clone();
    window.on_resize_requested(move |token, width| observed.borrow_mut().push((token, width)));
    requested
}

#[test]
fn default_policy_shares_the_viewport_equally_and_aligns_rows_at_every_text_scale() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(column_set())));
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        let header = widths(&header_cells(&window));
        assert_eq!(header.len(), 2);
        assert!(near(header[0], header[1]), "fill columns share equally");
        assert!(
            near(header[0] + header[1], visible_width(&window)),
            "fill columns cover the viewport: {header:?}"
        );
        assert_aligned(&window);
        assert!(
            handle(&window, "Product, width").is_none()
                && handle(&window, "Quantity, width").is_none(),
            "a column that is not resizable has no handle"
        );
    }
}

#[test]
fn fixed_fill_and_bounded_columns_respect_their_explicit_bounds() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    // A fixed column is exactly its basis; a bounded fill column stops at
    // its maximum and the space it leaves stays empty rather than widening it.
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        fixed(120.),
        bounded(fill(), 0., 200.),
    ))));
    let header = widths(&header_cells(&window));
    assert!(near(header[0], 120.), "fixed column: {header:?}");
    assert!(near(header[1], 200.), "bounded fill column: {header:?}");
    assert_aligned(&window);
    // A fixed basis outside its own bounds is clamped to them.
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        bounded(fixed(500.), 80., 160.),
        fill(),
    ))));
    let header = widths(&header_cells(&window));
    assert!(near(header[0], 160.), "fixed basis clamped: {header:?}");
    assert!(
        near(header[1], visible_width(&window) - 160.),
        "the fill column takes the rest: {header:?}"
    );
    // The table floor applies to fill columns; a fixed column may be narrower.
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        fixed(48.),
        fill(),
    ))));
    let header = widths(&header_cells(&window));
    assert!(
        near(header[0], 48.),
        "fixed below the table floor: {header:?}"
    );
    assert_aligned(&window);
}

#[test]
fn minimums_that_overflow_the_viewport_scroll_with_the_header_aligned() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        fixed(600.),
        bounded(fill(), 500., 800.),
    ))));
    let viewport = visible_width(&window);
    let header = widths(&header_cells(&window));
    assert!(600. + 500. > viewport, "the fixture must overflow");
    assert!(near(header[0], 600.) && near(header[1], 500.), "{header:?}");
    assert_aligned(&window);
    let first_cell_x = header_cells(&window)[0].absolute_position().x;
    let rows_area = center(&delegates(&window, "KitDataTable")[3]);
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: rows_area,
            delta_x: -300.,
            delta_y: 0.,
        });
    assert!(
        window.get_table_scroll_x() < -1.,
        "a horizontal wheel scrolls the grid: {}",
        window.get_table_scroll_x()
    );
    assert!(
        header_cells(&window)[0].absolute_position().x < first_cell_x - 1.,
        "the header follows the horizontal scroll"
    );
    assert_aligned(&window);
}

#[test]
fn a_pointer_drag_previews_the_width_then_requests_one_clamped_width() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    let model = Rc::new(VecModel::from(with_widths(
        resizable(bounded(fixed(200.), 100., 400.)),
        fill(),
    )));
    window.set_columns(ModelRc::from(model.clone()));
    let requested = observe_resizes(&window);
    let grip = handle(&window, "Product, width").expect("resizable column handle");
    assert_eq!(grip.accessible_value().as_deref(), Some("200"));
    assert_eq!(grip.accessible_value_minimum(), Some(100.));
    assert_eq!(grip.accessible_value_maximum(), Some(400.));
    let start = center(&grip);
    pointer_down(&window, start);
    pointer_move(&window, LogicalPosition::new(start.x + 50., start.y));
    assert!(
        near(widths(&header_cells(&window))[0], 250.),
        "the dragged column previews live"
    );
    assert_aligned(&window);
    assert!(
        requested.borrow().is_empty(),
        "nothing is requested before release"
    );
    pointer_move(&window, LogicalPosition::new(start.x + 900., start.y));
    assert!(
        near(widths(&header_cells(&window))[0], 400.),
        "the preview stops at the maximum"
    );
    pointer_up(&window, LogicalPosition::new(start.x + 900., start.y));
    assert_eq!(&*requested.borrow(), &[(PRODUCT, 400.)]);
    assert!(
        near(widths(&header_cells(&window))[0], 200.),
        "without a host change the column returns to its policy width"
    );
    assert_aligned(&window);
    // The host applies the width by changing the column's policy.
    let mut column = model.row_data(0).unwrap();
    column.width.basis = 400.;
    model.set_row_data(0, column);
    assert!(near(widths(&header_cells(&window))[0], 400.));
    assert_aligned(&window);
    // A drag below the minimum is clamped as well, and a release emits once.
    let grip = handle(&window, "Product, width").expect("handle after resize");
    let start = center(&grip);
    pointer_down(&window, start);
    pointer_move(&window, LogicalPosition::new(start.x - 900., start.y));
    pointer_up(&window, LogicalPosition::new(start.x - 900., start.y));
    assert_eq!(&*requested.borrow(), &[(PRODUCT, 400.), (PRODUCT, 100.)]);
}

#[test]
fn keyboard_and_assistive_steps_resize_within_bounds_and_respect_blocking() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    let model = Rc::new(VecModel::from(with_widths(
        resizable(bounded(fixed(200.), 100., 400.)),
        fill(),
    )));
    window.set_columns(ModelRc::from(model.clone()));
    let requested = observe_resizes(&window);
    let step = window.get_theme().spacing.unit * 4.;
    let grip = handle(&window, "Product, width").expect("resizable column handle");
    assert_eq!(grip.accessible_value_step(), Some(step));
    assert_eq!(grip.accessible_enabled(), Some(true));
    // Header controls precede the body in the visual and keyboard order.
    window.invoke_focus_table();
    down(&window, Key::Shift);
    key(&window, Key::Tab);
    up(&window, Key::Shift);
    key(&window, Key::RightArrow);
    assert_eq!(&*requested.borrow(), &[(PRODUCT, 200. + step)]);
    key(&window, Key::LeftArrow);
    assert_eq!(requested.borrow().last(), Some(&(PRODUCT, 200. - step)));
    key(&window, Key::End);
    assert_eq!(requested.borrow().last(), Some(&(PRODUCT, 400.)));
    key(&window, Key::Home);
    assert_eq!(requested.borrow().last(), Some(&(PRODUCT, 100.)));
    // Near the ceiling, one more step is clamped to it.
    let mut column = model.row_data(0).unwrap();
    column.width.basis = 400. - step / 2.;
    model.set_row_data(0, column);
    grip.invoke_accessible_increment_action();
    assert_eq!(requested.borrow().last(), Some(&(PRODUCT, 400.)));
    grip.invoke_accessible_decrement_action();
    assert_eq!(
        requested.borrow().last(),
        Some(&(PRODUCT, 400. - step * 1.5))
    );
    assert_eq!(grip.accessible_value().as_deref(), Some("392"));
    let count = requested.borrow().len();
    // The quantity column is not resizable: no handle, no requests.
    assert!(handle(&window, "Quantity, width").is_none());
    // A blocked table disables the handle for keys, pointer and assistive actions.
    window.set_table_interactive(false);
    let grip = handle(&window, "Product, width").expect("handle stays in the tree");
    assert_eq!(grip.accessible_enabled(), Some(false));
    grip.invoke_accessible_increment_action();
    let start = center(&grip);
    pointer_down(&window, start);
    pointer_move(&window, LogicalPosition::new(start.x + 40., start.y));
    pointer_up(&window, LogicalPosition::new(start.x + 40., start.y));
    assert_eq!(
        requested.borrow().len(),
        count,
        "a blocked table never resizes"
    );
    assert!(near(widths(&header_cells(&window))[0], 400. - step / 2.));
}

#[test]
fn an_unbounded_resizable_column_reports_no_maximum_and_grows_the_grid() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 20))));
    let model = Rc::new(VecModel::from(with_widths(resizable(fixed(200.)), fill())));
    window.set_columns(ModelRc::from(model.clone()));
    let requested = observe_resizes(&window);
    let grip = handle(&window, "Product, width").expect("resizable column handle");
    assert_eq!(grip.accessible_value_minimum(), Some(140.));
    assert_eq!(
        grip.accessible_value_maximum(),
        None,
        "no ceiling is not a number"
    );
    let viewport = visible_width(&window);
    let start = center(&grip);
    pointer_down(&window, start);
    pointer_move(
        &window,
        LogicalPosition::new(start.x + viewport / 2., start.y),
    );
    let header = header_cells(&window);
    assert!(
        near(header[0].size().width, 200. + viewport / 2.),
        "an unbounded preview passes the bounded range: {}",
        header[0].size().width
    );
    assert_aligned(&window);
    pointer_up(
        &window,
        LogicalPosition::new(start.x + viewport / 2., start.y),
    );
    assert_eq!(&*requested.borrow(), &[(PRODUCT, 200. + viewport / 2.)]);
    // A width past the viewport widens the grid instead of being clamped,
    // so the table scrolls horizontally.
    let mut column = model.row_data(0).unwrap();
    column.width.basis = viewport + 100.;
    model.set_row_data(0, column);
    assert!(near(widths(&header_cells(&window))[0], viewport + 100.));
    window
        .window()
        .dispatch_event(WindowEvent::PointerScrolled {
            position: center(&delegates(&window, "KitDataTable")[3]),
            delta_x: -300.,
            delta_y: 0.,
        });
    assert!(
        window.get_table_scroll_x() < -1.,
        "the grid grew past the viewport"
    );
    assert_aligned(&window);
}

#[test]
fn width_and_order_changes_keep_row_identity_and_selection() {
    let window = fixture();
    let columns = Rc::new(VecModel::from(with_widths(resizable(fixed(200.)), fill())));
    window.set_columns(ModelRc::from(columns.clone()));
    let mut table_rows = rows(1, 3);
    table_rows[1].selected = true;
    let row_model = Rc::new(VecModel::from(table_rows));
    window.set_rows(ModelRc::from(row_model.clone()));
    window.set_table_active(1);
    let activated = Rc::new(RefCell::new(Vec::new()));
    let observed = activated.clone();
    window.on_table_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation))
    });
    let selected = || {
        delegates(&window, "KitDataTable")
            .iter()
            .filter(|row| row.accessible_item_selected() == Some(true))
            .map(|row| row.accessible_label().unwrap().to_string())
            .collect::<Vec<_>>()
    };
    assert_eq!(selected(), ["Product 2"]);
    // Change the widths, then reorder the columns and every row's cells as
    // the application would. Row tokens, selection and the active row stay.
    let mut product = columns.row_data(0).unwrap();
    product.width.basis = 320.;
    columns.set_row_data(0, product);
    assert!(near(widths(&header_cells(&window))[0], 320.));
    let mut reordered = with_widths(resizable(fixed(320.)), fill());
    reordered.swap(0, 1);
    columns.set_vec(reordered);
    for index in 0..row_model.row_count() {
        let mut row = row_model.row_data(index).unwrap();
        let cells: Vec<slint::SharedString> = row.cells.iter().collect();
        row.cells = ModelRc::new(VecModel::from(vec![cells[1].clone(), cells[0].clone()]));
        row_model.set_row_data(index, row);
    }
    assert!(
        labelled(&window, "Quantity").absolute_position().x
            < labelled(&window, "Product, ascending")
                .absolute_position()
                .x,
        "the quantity column now leads"
    );
    assert!(
        near(widths(&header_cells(&window))[1], 320.),
        "the width followed the column key"
    );
    assert_aligned(&window);
    assert_eq!(
        delegates(&window, "KitDataTable")
            .iter()
            .filter(|row| row.accessible_item_selected() == Some(true))
            .count(),
        1,
        "exactly one row remains selected"
    );
    assert_eq!(
        row_model.row_data(1).unwrap().token,
        2,
        "the selected row keeps its token"
    );
    window.invoke_focus_table();
    key(&window, Key::Return);
    assert_eq!(&*activated.borrow(), &[(2, 1)]);
}

#[test]
fn a_fixed_resizable_column_cannot_hide_its_own_handle() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 2))));
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        resizable(fixed(200.)),
        fill(),
    ))));
    let requested = observe_resizes(&window);
    let grip = handle(&window, "Product, width").unwrap();
    let start = center(&grip);
    pointer_down(&window, start);
    let end = LogicalPosition::new(start.x - 1000., start.y);
    pointer_move(&window, end);
    assert!(header_cells(&window)[0].size().width >= 140.);
    assert!(handle(&window, "Product, width").unwrap().size().width > 0.);
    pointer_up(&window, end);
    assert_eq!(requested.borrow().last(), Some(&(PRODUCT, 140.)));
}

#[test]
fn resize_click_does_not_freeze_fill_and_escape_cancels_the_preview() {
    let window = fixture();
    window.set_rows(ModelRc::new(VecModel::from(rows(1, 2))));
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        resizable(fill()),
        fill(),
    ))));
    let requested = observe_resizes(&window);
    let grip = handle(&window, "Product, width").unwrap();
    assert!(grip.size().width >= 24.);
    let original = header_cells(&window)[0].size().width;
    let start = center(&grip);
    pointer_down(&window, start);
    pointer_up(&window, start);
    assert!(requested.borrow().is_empty());
    pointer_down(&window, start);
    pointer_move(&window, LogicalPosition::new(start.x + 60., start.y));
    assert!(header_cells(&window)[0].size().width > original);
    key(&window, Key::Escape);
    pointer_up(&window, LogicalPosition::new(start.x + 60., start.y));
    assert!(requested.borrow().is_empty());
    assert!(near(header_cells(&window)[0].size().width, original));
}

#[test]
fn sparse_and_extra_cells_always_share_the_complete_header_grid() {
    let window = fixture();
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        fill(),
        bounded(fill(), 180., 300.),
    ))));
    let mut records = rows(1, 2);
    records[0].cells = ModelRc::new(VecModel::from(vec!["Short".into()]));
    records[1].cells = ModelRc::new(VecModel::from(vec![
        "Full".into(),
        "Second".into(),
        "Ignored".into(),
    ]));
    window.set_rows(ModelRc::new(VecModel::from(records)));
    assert_aligned(&window);
    assert_eq!(row_cells(&window, 0).len(), 2);
    assert_eq!(row_cells(&window, 1).len(), 2);
}

#[test]
fn unnamed_rows_and_resizers_use_current_localized_names_and_keep_their_actions() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        resizable(bounded(fixed(180.), 140., 220.)),
        fill(),
    ))));
    window.set_rows(ModelRc::new(VecModel::from(vec![TableRow {
        token: 41,
        ..Default::default()
    }])));
    window.set_table_active(0);
    window.show().unwrap();
    let requested = observe_resizes(&window);
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
            frame();
            window.invoke_focus_table();
            for (row_name, resize_name) in [
                ("Unnamed row", "column width"),
                ("Ligne sans nom", "largeur"),
            ] {
                window.set_table_unnamed_row_label(row_name.into());
                window.set_table_resize_label(resize_name.into());
                frame();
                assert!(table.accessible_label().unwrap().contains(row_name));
                assert_eq!(
                    delegates(&window, "KitDataTable")[0]
                        .accessible_label()
                        .as_deref(),
                    Some(row_name)
                );
                let expected = format!("Product, {resize_name}");
                let grip = handle(&window, &expected)
                    .expect("a localized name still exposes the resize control");
                assert!(handle(&window, "Product, width").is_none());
                let before = requested.borrow().len();
                grip.invoke_accessible_increment_action();
                frame();
                assert_eq!(requested.borrow().len(), before + 1);
                let (token, width) = requested.borrow()[before];
                assert_eq!(token, PRODUCT);
                assert!(width > 180. && width <= 220.);
                grip.invoke_accessible_decrement_action();
                frame();
                assert_eq!(requested.borrow().len(), before + 2);
                let (token, width) = requested.borrow()[before + 1];
                assert_eq!(token, PRODUCT);
                assert!((140.0..180.0).contains(&width));
            }
            crate::choice_presentation::save_capture(
                &frame(),
                &format!("table-localized-empty-row-{name}-{scale}"),
            );
        }
    }
}

#[test]
fn table_header_body_and_cell_reveal_share_the_measured_viewport_in_each_direction() {
    use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
    use slint_kit::{CellContent, FieldAccess};
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .unwrap();
    let window = BaseDataFixture::new().unwrap();
    kit_slint::install(&window);
    window.set_columns(ModelRc::new(VecModel::from(with_widths(
        fixed(600.),
        fixed(600.),
    ))));
    window.set_rows(ModelRc::new(VecModel::from(
        (1..=200)
            .map(|token| TableRow {
                token,
                content: slint_kit::table_cells([
                    CellContent::text(format!("Row {token}")),
                    CellContent::field("Original", FieldAccess::Editable),
                ]),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    )));
    window.set_table_active(0);
    window.show().unwrap();
    let frame = || {
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap();
        slint::platform::update_timers_and_animations();
        window.window().take_snapshot().unwrap()
    };
    frame();
    let table = ElementQuery::from_root(&window)
        .match_inherits("KitDataTable")
        .find_first()
        .unwrap();
    let viewport = table
        .query_descendants()
        .match_type_name("Flickable")
        .find_first()
        .unwrap();
    let body_cells = || {
        delegates(&window, "KitDataTable")
            .remove(0)
            .query_descendants()
            .match_id("KitDataTable::content-cell")
            .find_all()
    };
    let assert_aligned = || {
        let headers = header_cells(&window);
        let cells = body_cells();
        assert_eq!(headers.len(), 2);
        assert_eq!(cells.len(), 2);
        for (header, cell) in headers.iter().zip(&cells) {
            assert!(near(
                header.absolute_position().x,
                cell.absolute_position().x
            ));
            assert!(near(header.size().width, cell.size().width));
        }
    };
    for (name, base) in [
        ("light", window.get_theme()),
        ("dark", window.get_dark_theme()),
    ] {
        for scale in [1., 2.] {
            for (flow_name, direction) in [
                ("ltr", ui::ChoiceFlow::LeftToRight),
                ("rtl", ui::ChoiceFlow::RightToLeft),
            ] {
                let mut theme = base.clone();
                theme.text_scale = scale;
                theme.direction = direction;
                theme.reduced_motion = true;
                window.set_theme(theme);
                window.invoke_restore_table_scroll(0., 0.);
                window.set_table_active_column(-1);
                window.invoke_focus_table();
                frame();
                let left = viewport.absolute_position().x;
                let right = left + viewport.size().width;
                assert!(viewport.size().width < table.size().width);
                if direction == ui::ChoiceFlow::RightToLeft {
                    assert!(left > table.absolute_position().x + 1.);
                } else {
                    assert!(near(left, table.absolute_position().x));
                }
                assert_aligned();
                key(&window, Key::RightArrow);
                key(&window, Key::RightArrow);
                frame();
                assert_eq!(window.get_table_active_column(), 1);
                assert!(window.get_table_scroll_x() < 0.);
                assert_aligned();
                key(&window, Key::LeftArrow);
                frame();
                assert_eq!(window.get_table_active_column(), 0);
                assert_aligned();
                let first = body_cells().remove(0);
                assert!(first.absolute_position().x >= left - 1.);
                assert!(near(first.absolute_position().x, left));
                key(&window, Key::RightArrow);
                frame();
                assert_aligned();
                let last = body_cells().remove(1);
                assert!(last.absolute_position().x + last.size().width <= right + 1.);
                key(&window, Key::F2);
                frame();
                assert!(window.get_table_editing());
                let editor = table
                    .query_descendants()
                    .match_type_name("TextInput")
                    .match_predicate(|e| e.accessible_label().as_deref() == Some("Quantity, Row 1"))
                    .find_first()
                    .unwrap();
                assert!(editor.absolute_position().x >= left - 1.);
                assert!(editor.absolute_position().x + editor.size().width <= right + 1.);
                super::base_data::text(&window, "Viewport caret");
                frame();
                assert_eq!(editor.accessible_value().as_deref(), Some("Viewport caret"));
                assert_eq!(window.get_table_edit_draft(), "Viewport caret");
                crate::choice_presentation::save_capture(
                    &frame(),
                    &format!("table-viewport-{flow_name}-{name}-{scale}"),
                );
                key(&window, Key::Escape);
                frame();
                assert!(!window.get_table_editing());
            }
        }
    }
}
