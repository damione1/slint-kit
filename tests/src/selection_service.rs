// SPDX-License-Identifier: GPL-3.0-or-later
use crate::SelectionServiceFixture;
use i_slint_backend_testing::ElementQuery;
use slint::{
    ComponentHandle, LogicalPosition, Model,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::{Choices, SelectOption};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Destination {
    Apple,
    Apricot,
    Banana,
    Blocked,
}

fn options() -> Vec<SelectOption<Destination>> {
    let mut options = vec![
        SelectOption::new(Destination::Apple, "Apple"),
        SelectOption::new(Destination::Blocked, "Blocked"),
        SelectOption::new(Destination::Apricot, "Apricot"),
        SelectOption::new(Destination::Banana, "Banana"),
    ];
    options[1].disabled = true;
    options
}
fn key(window: &SelectionServiceFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn fixture() -> (SelectionServiceFixture, Choices<Destination>) {
    i_slint_backend_testing::init_no_event_loop();
    let window = SelectionServiceFixture::new().unwrap();
    slint_kit::install(&window);
    let choices = Choices::from_options(options()).unwrap();
    window.set_options(choices.model());
    window.show().unwrap();
    (window, choices)
}

#[test]
fn autonomous_instances_navigate_skip_disabled_and_emit_typed_source_values() {
    let (window, choices) = fixture();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    let source = choices.clone();
    window.on_first_chosen(move |row| output.borrow_mut().push(source.accept(row).unwrap()));
    window.invoke_focus_first();
    key(&window, Key::DownArrow);
    key(&window, Key::Return);
    assert_eq!(*received.borrow(), [Destination::Apricot]);
    assert_eq!(
        window.get_first_state().selected,
        choices.selection(&Destination::Apricot).unwrap()
    );
    assert!(!window.get_second_state().selected.present);
    window.invoke_focus_second();
    key(&window, Key::End);
    key(&window, Key::Return);
    assert_eq!(
        window.get_second_state().selected,
        choices.selection(&Destination::Banana).unwrap()
    );
    assert_eq!(
        window.get_first_state().selected,
        choices.selection(&Destination::Apricot).unwrap()
    );
    window.invoke_search_first("Banana".into());
    assert_eq!(
        window.get_first_state().selected,
        choices.selection(&Destination::Apricot).unwrap(),
        "filtering keeps the accepted key"
    );
    assert_eq!(
        window.get_first_state().selection_availability,
        slint_kit::ui::SelectionAvailability::Available
    );
    assert_eq!(
        window.get_first_state().active_index,
        0,
        "active geometry uses filtered rows"
    );
}

#[test]
fn composed_keyboard_routes_programmatic_navigation_through_the_shared_session() {
    let (window, choices) = fixture();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    let source = choices.clone();
    window.on_keyboard_chosen(move |row| output.borrow_mut().push(source.accept(row).unwrap()));
    window.invoke_move_keyboard(slint_kit::ui::NavigationStep::First);
    window.invoke_move_keyboard(slint_kit::ui::NavigationStep::Next);
    window.invoke_choose_keyboard();
    assert_eq!(&*received.borrow(), &[Destination::Apricot]);
    window.invoke_type_keyboard("b".into());
    window.invoke_choose_keyboard();
    assert_eq!(
        &*received.borrow(),
        &[Destination::Apricot, Destination::Banana]
    );
    assert_eq!(
        window.get_keyboard_state().selected,
        choices.selection(&Destination::Banana).unwrap()
    );
    assert!(
        !window.get_first_state().selected.present,
        "composition keeps a separate session"
    );
}

#[test]
fn choices_reconcile_stable_keys_and_reject_stale_disabled_or_duplicate_values() {
    let (_window, choices) = fixture();
    let original = choices.model().row_data(0).unwrap();
    let token = choices.selection(&Destination::Apple).unwrap();
    let mut reversed = options();
    reversed.reverse();
    choices.replace(reversed).unwrap();
    assert_eq!(
        choices.selection(&Destination::Apple).unwrap(),
        token,
        "token is not a visual index"
    );
    assert_eq!(choices.accept(original), None);
    let current = choices
        .model()
        .iter()
        .find(|row| row.token == token.token)
        .unwrap();
    assert_eq!(choices.accept(current), Some(Destination::Apple));
    let blocked = choices.model().iter().find(|row| row.disabled).unwrap();
    assert_eq!(choices.accept(blocked), None);
    let before: Vec<_> = choices.model().iter().collect();
    assert_eq!(
        choices.replace(vec![
            SelectOption::new(Destination::Apple, "One"),
            SelectOption::new(Destination::Apple, "Duplicate")
        ]),
        Err(slint_kit::ChoiceError::DuplicateKey)
    );
    assert_eq!(
        choices.model().iter().collect::<Vec<_>>(),
        before,
        "rejected replacement is atomic"
    );
}

#[test]
fn model_replacement_between_pointer_down_and_up_never_commits_a_recycled_row() {
    let (window, choices) = fixture();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    window.on_first_chosen(move |row| output.borrow_mut().push(row));
    let element = ElementQuery::from_root(&window)
        .match_inherits("KitNavigationAction")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Apple"))
        .find_first()
        .unwrap();
    let origin = element.absolute_position();
    let position = LogicalPosition::new(
        origin.x + element.size().width / 2.0,
        origin.y + element.size().height / 2.0,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    let mut reordered = options();
    reordered.reverse();
    choices.replace(reordered).unwrap();
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert!(received.borrow().is_empty());
    assert!(!window.get_first_state().selected.present);
}

#[test]
fn controlled_selection_waits_for_acknowledgment_and_radio_arrows_choose_locally() {
    let (window, choices) = fixture();
    window.set_ownership(slint_kit::ui::Ownership::Controlled);
    window.invoke_focus_first();
    key(&window, Key::End);
    key(&window, Key::Return);
    assert!(!window.get_first_state().selected.present);
    window.set_acknowledged(choices.selection(&Destination::Banana).unwrap());
    assert_eq!(
        window.get_first_state().selected,
        choices.selection(&Destination::Banana).unwrap()
    );
    window.invoke_focus_radio();
    key(&window, Key::DownArrow);
    assert_eq!(
        window.get_radio_selected(),
        choices.selection(&Destination::Apricot).unwrap()
    );
    window.set_access(slint_kit::ui::FieldAccess::ReadOnly);
    key(&window, Key::DownArrow);
    assert_eq!(
        window.get_radio_selected(),
        choices.selection(&Destination::Apricot).unwrap()
    );
}

#[test]
fn independent_sources_never_reinterpret_another_sources_tokens_or_generations() {
    let first = Choices::new([(Destination::Apple, "Apple")]).unwrap();
    let second = Choices::new([(Destination::Banana, "Banana")]).unwrap();
    let old = first.model().row_data(0).unwrap();
    let current = second.model().row_data(0).unwrap();
    assert_ne!(old.token, current.token);
    assert_ne!(old.generation, current.generation);
    assert_eq!(second.accept(old.clone()), None);
    first
        .replace(vec![SelectOption::new(Destination::Apple, "Renamed")])
        .unwrap();
    assert_eq!(first.model().row_data(0).unwrap().token, old.token);
    assert_eq!(first.accept(old), None);
    assert!(first.selections(&[Destination::Banana]).is_none());
}

#[test]
fn accordion_local_expansion_and_controlled_proposals_wait_for_acknowledgment() {
    let (window, choices) = fixture();
    let mut theme = window.get_theme();
    theme.reduced_motion = true;
    window.set_theme(theme);
    window.set_ownership(slint_kit::ui::Ownership::Controlled);
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    let current = choices.clone();
    window.on_expansion_changed(move |row, expanded| {
        if let Some(key) = current.accept(row) {
            output.borrow_mut().push((key, expanded));
        }
    });
    let accordions = ElementQuery::from_root(&window)
        .match_inherits("KitAccordion")
        .find_all();
    assert_eq!(accordions.len(), 2);
    let trigger = |index: usize| {
        accordions[index]
            .query_descendants()
            .match_inherits("KitNavigationAction")
            .match_predicate(|element| element.accessible_label().as_deref() == Some("Apple"))
            .find_first()
            .unwrap()
    };
    trigger(0).mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_expanded().row_count(), 0);
    assert_eq!(&*received.borrow(), &[(Destination::Apple, true)]);
    let refs = choices.selections(&[Destination::Apple]).unwrap();
    window.set_expanded(refs);
    trigger(0).mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_expanded().row_count(), 1);
    assert_eq!(
        &*received.borrow(),
        &[(Destination::Apple, true), (Destination::Apple, false)]
    );
    trigger(1).mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_local_expanded_count(), 1);
    trigger(1).mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_local_expanded_count(), 0);
    trigger(1).mock_single_click(PointerEventButton::Left);
    accordions[1]
        .query_descendants()
        .match_inherits("KitNavigationAction")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Apricot"))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        window.get_local_expanded().row_data(0).unwrap(),
        choices.selection(&Destination::Apricot).unwrap(),
        "single mode opens the second section and closes the previous section"
    );
    window.set_expansion_mode(slint_kit::ui::ExpansionMode::Multiple);
    trigger(1).mock_single_click(PointerEventButton::Left);
    assert_eq!(
        window.get_local_expanded_count(),
        2,
        "multiple mode preserves the other section"
    );
    accordions[1]
        .query_descendants()
        .match_inherits("KitNavigationAction")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Blocked"))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        window.get_local_expanded_count(),
        2,
        "disabled sections cannot expand"
    );
    key(&window, Key::DownArrow);
    key(&window, Key::Return);
    assert_eq!(
        window.get_local_expanded_count(),
        1,
        "keyboard focus survives expansion and closes the next section"
    );
}
