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
