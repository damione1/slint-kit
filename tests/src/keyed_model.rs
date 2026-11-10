// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, Model};
use slint_kit::{KeyedModel, ModelError, ToastBinding, ToastLifetime, ui};
use std::num::NonZeroUsize;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Record(u64);
fn row(label: &str) -> ui::ContentItem {
    ui::ContentItem {
        title: label.into(),
        ..Default::default()
    }
}
#[test]
fn key_models_reject_stale_gestures_duplicate_keys_and_removed_identities() {
    let model = KeyedModel::new([(Record(1), row("first")), (Record(2), row("second"))]).unwrap();
    let first = model.model().row_data(0).unwrap();
    let generation = model.generation();
    model
        .replace([(Record(2), row("renamed")), (Record(1), row("first"))])
        .unwrap();
    assert_eq!(model.model().row_data(1).unwrap().token, first.token);
    assert_eq!(model.accept(first.token, generation), None);
    assert_eq!(
        model.accept(first.token, model.generation()),
        Some(Record(1))
    );
    let generation = model.generation();
    assert_eq!(
        model.replace([(Record(1), row("one")), (Record(1), row("two"))]),
        Err(ModelError::DuplicateKey)
    );
    assert_eq!(model.generation(), generation);
    model.remove(first.token);
    model.insert(Record(1), row("new lifecycle")).unwrap();
    assert_eq!(model.key(first.token), None);
    let another = KeyedModel::new([(Record(1), row("other window"))]).unwrap();
    assert_eq!(
        another.accept(model.model().row_data(0).unwrap().token, model.generation()),
        None
    );
}
#[test]
fn bound_model_and_toast_handlers_do_not_keep_a_window_alive() {
    i_slint_backend_testing::init_no_event_loop();
    let window = crate::AutonomousFixture::new().unwrap();
    let weak = window.as_weak();
    let model = KeyedModel::new([(Record(1), row("record"))]).unwrap();
    model.bind(
        &window,
        crate::AutonomousFixture::set_content,
        crate::AutonomousFixture::set_generation,
        crate::AutonomousFixture::set_active,
    );
    let toasts = ToastBinding::new(NonZeroUsize::new(2).unwrap());
    toasts
        .push(
            Record(1),
            ui::ToastEntry {
                title: "Saved".into(),
                ..Default::default()
            },
            ToastLifetime::Persistent,
        )
        .unwrap();
    let token = toasts.model().row_data(0).unwrap().token;
    assert_eq!(toasts.dismiss(token), Some(Record(1)));
    assert_eq!(toasts.dismiss(token), None);
    drop(window);
    assert!(weak.upgrade().is_none());
    model.insert(Record(2), row("after drop")).unwrap();
}

#[test]
fn sorting_preserves_keys_selection_and_active_row_but_invalidates_old_gestures() {
    use std::{cell::Cell, rc::Rc};
    let model = KeyedModel::new([(Record(2), row("second")), (Record(1), row("first"))]).unwrap();
    let token = model.model().row_data(0).unwrap().token;
    let old_generation = model.generation();
    model.select(token, old_generation).unwrap();
    let direction = Rc::new(Cell::new(ui::DataSortDirection::Unsorted));
    let observed = direction.clone();
    let sort = model.sorting_by(
        |left, right| left.0.cmp(&right.0),
        move |value| observed.set(value),
    );
    sort();
    assert_eq!(direction.get(), ui::DataSortDirection::Ascending);
    assert_eq!(model.active_index(), 1);
    assert_eq!(model.model().row_data(1).unwrap().token, token);
    assert!(model.model().row_data(1).unwrap().selected);
    assert_eq!(model.accept(token, old_generation), None);
    sort();
    assert_eq!(direction.get(), ui::DataSortDirection::Descending);
    assert_eq!(model.active_index(), 0);
    assert_eq!(model.accept(token, model.generation()), Some(Record(2)));
}

#[test]
fn active_row_bindings_share_updates_without_accepting_edits_to_a_different_line() {
    i_slint_backend_testing::init_no_event_loop();
    let first = crate::AutonomousFixture::new().unwrap();
    let second = crate::AutonomousFixture::new().unwrap();
    let model = KeyedModel::new([(Record(1), row("first")), (Record(2), row("second"))]).unwrap();
    model.bind_active(&first, |window, row| {
        window.set_email_text(row.map_or_else(Default::default, |row| row.title))
    });
    model.bind_active(&second, |window, row| {
        window.set_email_text(row.map_or_else(Default::default, |row| row.title))
    });
    let token = model.model().row_data(0).unwrap().token;
    model.select(token, model.generation()).unwrap();
    assert_eq!(first.get_email_text(), "first");
    assert_eq!(second.get_email_text(), "first");
    let generation = model.generation();
    assert_eq!(
        model.update_active(token, generation, |row| row.title = "edited".into()),
        Some(Record(1))
    );
    assert_eq!(first.get_email_text(), "edited");
    assert_eq!(second.get_email_text(), "edited");
    model
        .replace([(Record(1), row("edited")), (Record(2), row("second"))])
        .unwrap();
    assert_eq!(
        model.update_active(token, generation, |row| row.title = "stale".into()),
        None
    );
    model.navigate(slint_kit::NavigationStep::Next);
    assert_eq!(
        model.update_active(token, model.generation(), |row| row.title =
            "wrong line".into()),
        None
    );
    assert_eq!(model.active_row().unwrap().title, "second");
    drop(first);
    model.navigate(slint_kit::NavigationStep::Previous);
    assert_eq!(second.get_email_text(), "edited");
}

// The short integration path of a data table: `bind`, `navigation`,
// `activation` and `sorting_by` wired to `KitDataTable`, with the host
// supplying only its data, comparator and filter.
#[test]
fn table_rows_follow_their_key_through_keyboard_selection_sorting_and_filtering() {
    use crate::{DataFixtureCollection, base_data};
    use slint::platform::Key;
    use slint_kit::{CellContent, table_cells};
    use std::{cell::Cell, rc::Rc};
    const PROJECTS: [(Record, &str, u64, bool); 4] = [
        (Record(101), "Atlas", 12, false),
        (Record(203), "Boreal", 3, false),
        (Record(307), "Cedar", 24, true),
        (Record(409), "Delta", 8, false),
    ];
    fn tasks(key: &Record) -> u64 {
        PROJECTS
            .iter()
            .find(|(candidate, ..)| candidate == key)
            .expect("known project")
            .2
    }
    fn rows(show_archived: bool, selected: Option<Record>) -> Vec<(Record, ui::TableRow)> {
        PROJECTS
            .into_iter()
            .filter(|(_, _, _, archived)| show_archived || !archived)
            .map(|(key, name, tasks, archived)| {
                let row = ui::TableRow {
                    content: table_cells([
                        CellContent::text(name),
                        CellContent::text(tasks.to_string()),
                    ]),
                    selected: selected == Some(key),
                    disabled: archived,
                    ..Default::default()
                };
                (key, row)
            })
            .collect()
    }
    fn names(model: &KeyedModel<Record, ui::TableRow>) -> Vec<String> {
        model
            .model()
            .iter()
            .map(|row| row.content.row_data(0).expect("name cell").text.to_string())
            .collect()
    }
    let window = base_data::fixture();
    window.set_columns(slint::ModelRc::new(slint::VecModel::from(vec![
        ui::TableColumn {
            token: 1,
            label: "Project".into(),
            ..Default::default()
        },
        ui::TableColumn {
            token: 2,
            label: "Tasks".into(),
            sortable: true,
            alignment: ui::DataAlignment::Trailing,
            ..Default::default()
        },
    ])));
    let model = KeyedModel::new(rows(true, None)).unwrap();
    model.bind(
        &window,
        crate::BaseDataFixture::set_rows,
        crate::BaseDataFixture::set_table_generation,
        crate::BaseDataFixture::set_table_active,
    );
    let navigate = model.navigation();
    window.on_navigate(move |collection, step| match collection {
        DataFixtureCollection::Table => navigate(step),
        DataFixtureCollection::Content | DataFixtureCollection::Document => {}
    });
    let chosen = Rc::new(Cell::new(None));
    window.on_table_activated(model.activation({
        let chosen = chosen.clone();
        move |key| chosen.set(Some(key))
    }));
    let sort = model.sorting_by(|left, right| tasks(left).cmp(&tasks(right)), |_| {});
    window.on_sort_requested(move |_| sort());

    window.invoke_focus_table();
    base_data::key(&window, Key::DownArrow);
    base_data::key(&window, Key::Return);
    assert_eq!(
        chosen.get(),
        Some(Record(101)),
        "Enter selects the first enabled row"
    );
    let atlas = model.model().row_data(0).unwrap().token;
    let generation = window.get_table_generation();

    window.invoke_sort_requested(2);
    assert_eq!(names(&model), ["Boreal", "Delta", "Atlas", "Cedar"]);
    assert_ne!(
        window.get_table_generation(),
        generation,
        "sorting advances the generation"
    );
    let moved = model.model().row_data(2).unwrap();
    assert!(
        moved.selected && moved.token == atlas,
        "selection follows its key"
    );
    assert_eq!(
        window.get_table_active(),
        2,
        "the active row follows its key"
    );

    // A stale gesture and a disabled row are rejected at the component's callback.
    window.invoke_table_activated(model.model().row_data(1).unwrap().token, generation);
    window.invoke_table_activated(
        model.model().row_data(3).unwrap().token,
        window.get_table_generation(),
    );
    assert_eq!(chosen.get(), Some(Record(101)));

    base_data::key(&window, Key::End);
    assert_eq!(
        window.get_table_active(),
        2,
        "End skips the archived last row"
    );
    base_data::key(&window, Key::Home);
    base_data::key(&window, Key::Return);
    assert_eq!(chosen.get(), Some(Record(203)));

    // Filtering republishes the host's rows: surviving keys keep their token,
    // selection and active position.
    let boreal = model.model().row_data(0).unwrap().token;
    model.replace(rows(false, chosen.get())).unwrap();
    assert_eq!(model.model().row_count(), 3);
    let kept = model.model().row_data(1).unwrap();
    assert!(kept.selected && kept.token == boreal);
    assert_eq!(window.get_table_active(), 1);
    assert_eq!(
        model.accept(boreal, generation),
        None,
        "old generation stays rejected"
    );
}

#[test]
fn active_binding_preserves_drafts_when_other_rows_or_only_order_change() {
    i_slint_backend_testing::init_no_event_loop();
    let window = crate::AutonomousFixture::new().unwrap();
    let model = KeyedModel::new([(Record(1), row("first")), (Record(2), row("second"))]).unwrap();
    model.bind_active(&window, |window, row| {
        window.set_email_text(row.map_or_else(Default::default, |row| row.title))
    });
    model.select(model.model().row_data(0).unwrap().token, model.generation());
    window.set_email_text("unsaved draft".into());
    model.insert(Record(3), row("third")).unwrap();
    assert_eq!(window.get_email_text(), "unsaved draft", "insertion");
    let second = model.model().row_data(1).unwrap();
    model.update(second.token, model.generation(), |row| {
        row.title = "updated second".into()
    });
    assert_eq!(window.get_email_text(), "unsaved draft", "other row update");
    model.sort_by(|left, right| right.0.cmp(&left.0));
    assert_eq!(window.get_email_text(), "unsaved draft");
    let first = model.active_row().unwrap();
    model.update(first.token, model.generation(), |row| {
        row.title = "updated first".into()
    });
    assert_eq!(window.get_email_text(), "updated first");
    model.remove(first.token);
    assert_eq!(window.get_email_text(), "");
}

#[test]
fn domain_updates_preserve_generation_and_can_reenable_or_remove_disabled_records() {
    let mut disabled = row("disabled");
    disabled.disabled = true;
    let model = KeyedModel::new([(Record(1), disabled), (Record(2), row("live"))]).unwrap();
    let token = model.model().row_data(0).unwrap().token;
    let generation = model.generation();
    assert_eq!(model.accept(token, generation), None);
    assert!(model.update_by_key(&Record(1), |row| {
        row.disabled = false;
        row.title = "enabled".into();
    }));
    assert_eq!(model.generation(), generation);
    assert_eq!(model.accept(token, generation), Some(Record(1)));
    assert_eq!(model.get(&Record(1)).unwrap().title, "enabled");
    assert_eq!(model.len(), 2);
    model.update_by_key(&Record(1), |row| row.disabled = true);
    assert!(model.remove_by_key(&Record(1)));
    assert_eq!(model.get(&Record(1)), None);
    assert!(!model.is_empty());
}
