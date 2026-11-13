// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{
    OptionItem, OptionItemKind, OptionItemsComboboxFixture, OptionItemsFixture, OptionRow,
};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, ModelRc, SharedString, VecModel,
    platform::{Key, WindowEvent},
};
use slint_kit::ui::FieldAccess;
use std::{cell::OnceCell, cell::RefCell, rc::Rc};

thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }

fn backend() {
    BACKEND.with(|backend| {
        backend.get_or_init(|| {
            slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
                mock_time: true,
                renderer_name: Some("software".into()),
                ..Default::default()
            })))
            .unwrap()
        });
    });
}

// The group and separator rows are deliberately not disabled in the model: the
// presentation alone must make them inert.
fn rows_without_disabled_flags() -> ModelRc<OptionRow> {
    ModelRc::new(VecModel::from(
        [
            (1, "Citrus"),
            (2, "Lemon"),
            (3, ""),
            (4, "Stone fruit"),
            (5, "Peach"),
        ]
        .map(|(token, label)| OptionRow {
            token,
            generation: 1,
            label: label.into(),
            ..Default::default()
        })
        .to_vec(),
    ))
}

fn items() -> ModelRc<OptionItem> {
    ModelRc::new(VecModel::from(vec![
        OptionItem {
            token: 1,
            kind: OptionItemKind::Group,
            ..Default::default()
        },
        OptionItem {
            token: 2,
            trailing_text: "Ctrl+L".into(),
            ..Default::default()
        },
        OptionItem {
            token: 3,
            kind: OptionItemKind::Separator,
            ..Default::default()
        },
        OptionItem {
            token: 4,
            kind: OptionItemKind::Group,
            ..Default::default()
        },
    ]))
}

fn settle(window: &impl ComponentHandle) {
    slint::platform::update_timers_and_animations();
    window.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}

fn key(window: &impl ComponentHandle, text: impl Into<SharedString>) {
    let text = text.into();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(window);
}

fn select() -> OptionItemsFixture {
    backend();
    let window = OptionItemsFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(rows_without_disabled_flags());
    window.set_items(items());
    window.show().unwrap();
    settle(&window);
    window
}

fn element(window: &impl ComponentHandle, label: &str) -> Option<ElementHandle> {
    let label = label.to_string();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
}

fn open_select(window: &OptionItemsFixture) {
    window.invoke_focus_control();
    key(window, Key::DownArrow);
    assert!(window.get_state().open);
}

#[test]
fn group_and_separator_rows_are_inert_and_trailing_text_is_described() {
    let window = select();
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let target = chosen.clone();
    window.on_chosen(move |row| target.borrow_mut().push(row.token));
    open_select(&window);
    let group = element(&window, "Citrus").expect("group row");
    assert_eq!(group.accessible_enabled(), Some(false));
    group.invoke_accessible_default_action();
    settle(&window);
    assert!(chosen.borrow().is_empty());
    let lemon = element(&window, "Lemon").expect("option row");
    assert_eq!(lemon.accessible_enabled(), Some(true));
    assert_eq!(lemon.accessible_description().as_deref(), Some("Ctrl+L"));
    lemon.invoke_accessible_default_action();
    settle(&window);
    assert_eq!(*chosen.borrow(), vec![2]);
    assert_eq!(window.get_selected().token, 2);
}

#[test]
fn a_token_without_an_item_is_a_plain_option() {
    let window = select();
    open_select(&window);
    let peach = element(&window, "Peach").expect("plain row");
    assert_eq!(peach.accessible_enabled(), Some(true));
    assert_eq!(peach.accessible_description().as_deref(), Some(""));
}

#[test]
fn clear_button_empties_the_selection_and_reports_it() {
    let window = select();
    window.set_clearable(true);
    settle(&window);
    assert!(element(&window, "Clear selection").is_none());
    window.set_selected(slint_kit::ui::SelectionRef {
        present: true,
        token: 5,
    });
    settle(&window);
    let cleared = Rc::new(RefCell::new(0));
    let counter = cleared.clone();
    window.on_cleared(move || *counter.borrow_mut() += 1);
    let button = element(&window, "Clear selection").expect("clear button");
    button.invoke_accessible_default_action();
    settle(&window);
    assert_eq!(*cleared.borrow(), 1);
    assert!(!window.get_selected().present);
    assert!(element(&window, "Clear selection").is_none());
}

#[test]
fn clear_button_is_hidden_when_not_clearable_or_not_editable() {
    let window = select();
    window.set_selected(slint_kit::ui::SelectionRef {
        present: true,
        token: 5,
    });
    settle(&window);
    assert!(element(&window, "Clear selection").is_none());
    window.set_clearable(true);
    settle(&window);
    assert!(element(&window, "Clear selection").is_some());
    window.set_access(FieldAccess::ReadOnly);
    settle(&window);
    assert!(element(&window, "Clear selection").is_none());
}

#[test]
fn combobox_group_rows_are_inert_too() {
    backend();
    let window = OptionItemsComboboxFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_options(rows_without_disabled_flags());
    window.set_items(items());
    window.show().unwrap();
    settle(&window);
    window.invoke_focus_control();
    key(&window, Key::DownArrow);
    assert!(window.get_state().open);
    let group = element(&window, "Stone fruit").expect("group row");
    assert_eq!(group.accessible_enabled(), Some(false));
    group.invoke_accessible_default_action();
    settle(&window);
    assert!(!window.get_selected().present);
    let lemon = element(&window, "Lemon").expect("option row");
    assert_eq!(lemon.accessible_description().as_deref(), Some("Ctrl+L"));
    lemon.invoke_accessible_default_action();
    settle(&window);
    assert_eq!(window.get_selected().token, 2);
}
