// SPDX-License-Identifier: GPL-3.0-or-later
use super::{OptionRow, SelectGeometryFixture, SelectState};
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::{ComponentHandle, ModelRc, VecModel};

fn rows(count: usize) -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(
        (0..count)
            .map(|index| OptionRow {
                token: index as i32,
                generation: 1,
                label: format!("Workspace {index}").into(),
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    ))
}

fn surface(window: &SelectGeometryFixture) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitSelectSurface")
        .find_first()
        .expect("open popup surface")
}

#[test]
fn short_select_popups_fit_their_rows_and_large_models_stay_bounded() {
    i_slint_backend_testing::init_no_event_loop();
    for combobox in [false, true] {
        let window = SelectGeometryFixture::new().unwrap();
        window.set_combobox(combobox);
        window.set_rows(rows(3));
        window.show().unwrap();
        let _ = ElementQuery::from_root(&window)
            .match_inherits(if combobox { "KitCombobox" } else { "KitSelect" })
            .find_first()
            .unwrap()
            .size();
        window.set_state(SelectState {
            open: true,
            ..Default::default()
        });
        slint::platform::update_timers_and_animations();
        let height = surface(&window).size().height;
        assert!(
            (190.0..250.0).contains(&height),
            "three rows need a compact popup: {height}"
        );
        window.set_rows(rows(1));
        let height = surface(&window).size().height;
        assert!(
            (90.0..150.0).contains(&height),
            "filtered popup follows its content: {height}"
        );
        window.set_rows(rows(0));
        assert!(
            surface(&window).size().height >= 90.0,
            "empty state keeps a usable viewport"
        );
        window.set_rows(rows(200));
        assert!(
            surface(&window).size().height <= 320.0,
            "large lists keep the finite viewport"
        );
    }
}

#[test]
fn combobox_labels_use_the_full_left_aligned_text_column() {
    i_slint_backend_testing::init_no_event_loop();
    let window = SelectGeometryFixture::new().unwrap();
    window.set_combobox(true);
    window.set_rows(rows(3));
    window.show().unwrap();
    let _ = ElementQuery::from_root(&window)
        .match_inherits("KitCombobox")
        .find_first()
        .unwrap()
        .size();
    window.set_state(SelectState {
        open: true,
        ..Default::default()
    });
    slint::platform::update_timers_and_animations();
    // Resolve the popup viewport before querying virtualized row delegates.
    let _ = surface(&window).size();
    for list in ElementQuery::from_root(&window)
        .match_inherits("ListView")
        .find_all()
    {
        let _ = list.size();
    }
    let row = ElementQuery::from_root(&window)
        .match_inherits("KitOption")
        .find_first()
        .unwrap();
    let text = row
        .query_descendants()
        .match_type_name("Text")
        .find_first()
        .unwrap();
    assert!(text.size().width > row.size().width * 0.7);
    assert!(
        (text.absolute_position().x - row.absolute_position().x - window.get_row_padding()).abs()
            < 1.0
    );
    let placeholder = ElementQuery::from_root(&window)
        .match_type_name("Text")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Workspace"))
        .find_first()
        .expect("search field exposes its placeholder");
    assert!(placeholder.size().width > 100.0);
    let search = surface(&window)
        .query_descendants()
        .match_type_name("TextInput")
        .find_first()
        .unwrap();
    window.set_query("a very long query ".repeat(30).into());
    assert!(
        search.is_valid(),
        "query updates preserve the native editor"
    );
}
