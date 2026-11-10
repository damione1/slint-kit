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

fn assert_combobox_retains_remote_page(ownership: slint_kit::ui::Ownership) {
    let (window, choices) = fixture();
    window.set_ownership(ownership);
    let apricot_ref = choices.selection(&Destination::Apricot).unwrap();
    window.set_combo_selected(apricot_ref.clone());
    // A host projection becomes displayed after the UI processes its changed handlers.
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_combo_selected_label(), "Apricot");
    assert!(window.get_combo_validity().valid);

    // Simulate remote search replacing the options with a page that omits Apricot.
    choices
        .replace(vec![SelectOption::new(Destination::Banana, "Banana")])
        .unwrap();
    assert_eq!(
        choices.selection(&Destination::Apricot),
        Some(apricot_ref.clone()),
        "Choices retains stable token for known keys across remote pages"
    );
    assert_eq!(
        choices.label(&Destination::Apricot).as_deref(),
        Some("Apricot")
    );
    assert_eq!(window.get_combo_state().selected, apricot_ref);
    assert_eq!(window.get_combo_selected_label(), "Apricot");
    assert_eq!(
        window.get_combo_state().selection_availability,
        slint_kit::ui::SelectionAvailability::Available
    );
    assert!(window.get_combo_validity().valid);

    // Returning to a page that includes Apricot preserves its original token.
    choices.replace(options()).unwrap();
    assert_eq!(choices.selection(&Destination::Apricot), Some(apricot_ref));
    assert_eq!(window.get_combo_selected_label(), "Apricot");
}

#[test]
fn combobox_retains_accepted_selection_and_label_across_remote_page_replacements_local() {
    assert_combobox_retains_remote_page(slint_kit::ui::Ownership::Local);
}

#[test]
fn combobox_retains_accepted_selection_and_label_across_remote_page_replacements_controlled() {
    assert_combobox_retains_remote_page(slint_kit::ui::Ownership::Controlled);
}

#[test]
fn combobox_inline_editing_respects_min_query_length_debounce_custom_value_and_clear() {
    let (window, _choices) = fixture();
    window.set_combo_min_query_length(2);
    window.set_combo_query_debounce(50);
    window.set_combo_allow_custom(true);
    window.set_combo_clearable(true);

    let queries = Rc::new(RefCell::new(Vec::new()));
    let observed_queries = queries.clone();
    window.on_combo_query_edited(move |q| observed_queries.borrow_mut().push(q.to_string()));

    let customs = Rc::new(RefCell::new(Vec::new()));
    let observed_customs = customs.clone();
    window.on_combo_custom_chosen(move |val| observed_customs.borrow_mut().push(val.to_string()));

    let cleared = Rc::new(RefCell::new(0usize));
    let observed_cleared = cleared.clone();
    window.on_combo_cleared(move || *observed_cleared.borrow_mut() += 1);

    window.invoke_focus_combo();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "C".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "C".into() });
    assert_eq!(window.get_combo_text(), "C");
    assert_eq!(window.get_combo_query(), "");
    assert!(!window.get_combo_state().open);
    assert!(queries.borrow().is_empty());

    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "u".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "u".into() });
    assert_eq!(window.get_combo_text(), "Cu");
    assert_eq!(window.get_combo_query(), "Cu");
    assert!(window.get_combo_state().open);
    assert!(
        queries.borrow().is_empty(),
        "debounced query waits for timer"
    );

    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(60));
    slint::platform::update_timers_and_animations();
    assert_eq!(&*queries.borrow(), &["Cu"]);
    assert!(
        window.get_combo_validity().valid,
        "allow-custom-value satisfies required with custom text"
    );

    // Pressing Return with no matching option commits the custom value.
    key(&window, Key::Return);
    assert_eq!(&*customs.borrow(), &["Cu"]);

    // Clearing resets text, query, and emits cleared().
    window.invoke_clear_combo();
    assert_eq!(window.get_combo_text(), "");
    assert_eq!(window.get_combo_query(), "");
    assert_eq!(*cleared.borrow(), 1);
}

#[test]
fn remote_labels_do_not_replace_a_focused_native_draft_or_its_undo_and_outside_focus_destination() {
    let (window, choices) = fixture();
    let accepted = choices.selection(&Destination::Apricot).unwrap();
    window.set_combo_selected(accepted.clone());
    window.set_acknowledged(accepted);
    slint::platform::update_timers_and_animations();
    window.invoke_focus_combo();
    for _ in 0.."Apricot".len() {
        key(&window, Key::RightArrow);
    }
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "x".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "x".into() });
    let editor = ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Search and select"))
        .find_first()
        .unwrap();
    assert_eq!(window.get_combo_text(), "Apricotx");
    choices
        .replace(vec![
            SelectOption::new(Destination::Apricot, "Renamed"),
            SelectOption::new(Destination::Banana, "Banana"),
        ])
        .unwrap();
    slint::platform::update_timers_and_animations();
    assert_eq!(window.get_combo_selected_label(), "Renamed");
    assert_eq!(window.get_combo_text(), "Apricotx");
    assert!(window.get_combo_focused());
    assert!(editor.is_valid());
    key(&window, Key::Menu);
    ElementQuery::from_root(&window)
        .match_inherits("KitOption")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Undo"))
        .find_first()
        .unwrap()
        .mock_single_click(slint::platform::PointerEventButton::Left);
    assert_eq!(window.get_combo_text(), "Apricot");
    let outside = ElementQuery::from_root(&window)
        .match_inherits("KitNavigationAction")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Renamed"))
        .find_first()
        .unwrap();
    // Slint PopupWindow consumes the outside press while dismissing its child popup.
    // A subsequent click reaches the intended parent control and must retain that focus.
    outside.mock_single_click(slint::platform::PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert!(!window.get_combo_state().open);
    outside.mock_single_click(slint::platform::PointerEventButton::Left);
    slint::platform::update_timers_and_animations();
    assert!(!window.get_combo_focused());
    assert!(window.get_first_focused());
    assert!(!window.get_combo_state().open);
}

#[test]
fn falling_below_the_query_threshold_closes_suggestions_without_replacing_the_native_text() {
    let (window, _choices) = fixture();
    window.set_combo_min_query_length(2);
    window.invoke_focus_combo();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "Ap".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "Ap".into() });
    assert!(window.get_combo_state().open);
    key(&window, Key::Backspace);
    assert_eq!(window.get_combo_text(), "A");
    assert_eq!(window.get_combo_query(), "");
    assert!(!window.get_combo_state().open);
    assert!(window.get_combo_focused());
}

#[test]
fn exposed_combobox_expand_action_is_idempotent_and_respects_access_guards() {
    let (window, _choices) = fixture();
    let combo = ElementQuery::from_root(&window)
        .match_inherits("KitCombobox")
        .find_first()
        .unwrap();
    assert_eq!(combo.accessible_expandable(), Some(true));
    assert!(combo.accessible_description().unwrap().contains("Required"));
    assert!(!window.get_combo_state().open);
    combo.invoke_accessible_expand_action();
    assert!(window.get_combo_state().open);
    assert_eq!(window.get_combo_open_requests(), 1);
    combo.invoke_accessible_expand_action();
    assert!(window.get_combo_state().open);
    assert_eq!(window.get_combo_open_requests(), 1);
    key(&window, Key::Escape);
    assert!(!window.get_combo_state().open);
    for access in [
        slint_kit::ui::FieldAccess::ReadOnly,
        slint_kit::ui::FieldAccess::Disabled,
    ] {
        window.set_access(access);
        assert_eq!(combo.accessible_expandable(), Some(false));
        combo.invoke_accessible_expand_action();
        assert!(!window.get_combo_state().open);
        assert_eq!(window.get_combo_open_requests(), 1);
    }
}

fn assert_combobox_reentrant_remote_choice(ownership: slint_kit::ui::Ownership) {
    let (window, choices) = fixture();
    window.set_ownership(ownership);
    let accepted = choices.selection(&Destination::Apricot).unwrap();
    let weak = window.as_weak();
    let replacement = choices.clone();
    window.on_combo_chosen(move |row| {
        let window = weak.unwrap();
        window.set_combo_selected(slint_kit::ui::SelectionRef {
            present: true,
            token: row.token,
        });
        replacement
            .replace(vec![SelectOption::new(Destination::Banana, "Banana")])
            .unwrap();
    });
    window.invoke_focus_combo();
    key(&window, Key::DownArrow);
    assert!(window.get_combo_state().open);
    key(&window, Key::DownArrow); // skip the disabled row to Apricot
    key(&window, Key::Return);
    assert_eq!(window.get_combo_selected(), accepted);
    assert_eq!(window.get_combo_selected_label(), "Apricot");
    assert!(!window.get_combo_state().open);
}

#[test]
fn combobox_keyboard_choice_keeps_its_label_when_the_host_replaces_the_page_in_chosen_local() {
    assert_combobox_reentrant_remote_choice(slint_kit::ui::Ownership::Local);
}

#[test]
fn combobox_keyboard_choice_keeps_its_label_when_the_host_replaces_the_page_in_chosen_controlled() {
    assert_combobox_reentrant_remote_choice(slint_kit::ui::Ownership::Controlled);
}

fn assert_combobox_native_clear(ownership: slint_kit::ui::Ownership) {
    let (window, choices) = fixture();
    window.set_ownership(ownership);
    let cleared = Rc::new(RefCell::new(0));
    let observed_clear = cleared.clone();
    window.on_combo_cleared(move || *observed_clear.borrow_mut() += 1);
    window.set_combo_selected(choices.selection(&Destination::Apricot).unwrap());
    slint::platform::update_timers_and_animations();
    let queries = Rc::new(RefCell::new(Vec::new()));
    let observed = queries.clone();
    window.on_combo_query_edited(move |query| observed.borrow_mut().push(query.to_string()));
    window.invoke_clear_combo();
    assert_eq!(window.get_combo_text(), "");
    let accepted_until_ack = match ownership {
        slint_kit::ui::Ownership::Local => false,
        slint_kit::ui::Ownership::Controlled => true,
    };
    assert_eq!(window.get_combo_selected().present, accepted_until_ack);
    assert_eq!(*cleared.borrow(), 1);
    assert_eq!(&*queries.borrow(), &[""]);
    key(&window, Key::Menu);
    ElementQuery::from_root(&window)
        .match_inherits("KitOption")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Undo"))
        .find_first()
        .unwrap()
        .mock_single_click(PointerEventButton::Left);
    assert_eq!(
        window.get_combo_text(),
        "Apricot",
        "Undo restores the actual native deletion"
    );
    window.set_combo_selected(Default::default());
    assert!(!window.get_combo_selected().present);
    for access in [
        slint_kit::ui::FieldAccess::ReadOnly,
        slint_kit::ui::FieldAccess::Disabled,
    ] {
        window.set_access(access);
        window.invoke_clear_combo();
        assert_eq!(window.get_combo_text(), "Apricot");
        assert_eq!(*cleared.borrow(), 1);
    }
}
#[test]
fn combobox_public_clear_uses_native_undo_and_one_empty_query_locally() {
    assert_combobox_native_clear(slint_kit::ui::Ownership::Local);
}
#[test]
fn combobox_public_clear_uses_native_undo_and_waits_for_controlled_acknowledgment() {
    assert_combobox_native_clear(slint_kit::ui::Ownership::Controlled);
    }
#[test]
fn composite_keyboard_policies_wrap_closed_select_grid_tab_activation_and_carousel_keys() {
    let (window, choices) = fixture();
    let apple = choices.selection(&Destination::Apple).unwrap();
    let apricot = choices.selection(&Destination::Apricot).unwrap();
    let banana = choices.selection(&Destination::Banana).unwrap();

    // KitMenu wraps by default and respects wrap=false when configured.
    window.invoke_focus_first();
    key(&window, Key::UpArrow);
    assert_eq!(window.get_first_state().active, banana);
    key(&window, Key::DownArrow);
    assert_eq!(window.get_first_state().active, apple);
    window.set_first_wrap(false);
    key(&window, Key::UpArrow);
    assert_eq!(window.get_first_state().active, apple);
    assert!(window.get_first_page_rows() >= 1);

    // Closed KitSelect opens and moves active on Home, End, and printable typeahead.
    assert!(!window.get_popup_state().open);
    window.invoke_focus_popup();
    key(&window, Key::End);
    assert!(window.get_popup_state().open);
    assert_eq!(window.get_popup_state().active, banana);
    key(&window, Key::Escape);
    assert!(!window.get_popup_state().open);

    window.invoke_focus_popup();
    key(&window, Key::Home);
    assert!(window.get_popup_state().open);
    assert_eq!(window.get_popup_state().active, apple);
    key(&window, Key::Escape);
    assert!(!window.get_popup_state().open);

    window.invoke_focus_popup();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "b".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "b".into() });
    assert!(window.get_popup_state().open);
    assert_eq!(window.get_popup_state().active, banana);
    assert!(window.get_popup_page_rows() >= 1);
    key(&window, Key::Escape);

    // KitTabs supports manual and automatic activation plus RTL flow.
    window.invoke_focus_tabs();
    key(&window, Key::RightArrow);
    assert_eq!(window.get_tabs_state().active, apricot);
    assert!(!window.get_tabs_selected().present);
    window.set_tab_activation(slint_kit::ui::TabActivation::Automatic);
    key(&window, Key::RightArrow);
    assert_eq!(window.get_tabs_selected(), banana);
    window.set_nav_flow(slint_kit::ui::ChoiceFlow::RightToLeft);
    key(&window, Key::LeftArrow);
    assert_eq!(
        window.get_tabs_selected(),
        apple,
        "RTL LeftArrow advances to next tab and wraps"
    );

    // KitMenuBar mirrors Left/Right in RTL flow.
    window.invoke_focus_menubar();
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_menubar_state().active, apricot);

    // KitCarousel handles arrow keys, Home/End, and RTL flow.
    window.invoke_focus_carousel();
    key(&window, Key::LeftArrow);
    assert_eq!(window.get_carousel_state().active, apricot);
    key(&window, Key::End);
    assert_eq!(window.get_carousel_state().active, banana);
    key(&window, Key::Home);
    assert_eq!(window.get_carousel_state().active, apple);

    // Wrapped 2-column KitSegmentedControl navigates vertically with DownArrow/UpArrow.
    window.set_nav_flow(slint_kit::ui::ChoiceFlow::LeftToRight);
    window.invoke_focus_segments();
    key(&window, Key::DownArrow);
    assert_eq!(
        window.get_segments_selected(),
        apricot,
        "Down keeps the rendered column despite the disabled second cell"
    );
    key(&window, Key::UpArrow);
    assert_eq!(window.get_segments_selected(), apple);
    window.set_nav_flow(slint_kit::ui::ChoiceFlow::RightToLeft);
    key(&window, Key::UpArrow);
    assert_eq!(
        window.get_segments_selected(),
        apricot,
        "RTL vertical wrapping keeps the same visual column"
    );
    key(&window, Key::DownArrow);
    assert_eq!(window.get_segments_selected(), apple);

    // KitToggleGroup supports vertical direction and wraps.
    window.set_toggle_direction(slint_kit::ui::ChoiceDirection::Vertical);
    window.invoke_focus_toggles();
    key(&window, Key::UpArrow);
    assert_eq!(window.get_toggles_active(), banana.token);
}

#[test]
fn source_reconciliation_clears_typeahead_before_space_activation() {
    let (window, choices) = fixture();
    let received = Rc::new(RefCell::new(Vec::new()));
    let output = received.clone();
    let source = choices.clone();
    window.on_first_chosen(move |row| output.borrow_mut().push(source.accept(row).unwrap()));
    window.invoke_focus_first();
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "a".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "a".into() });
    let generation = window.get_first_state().generation;
    choices.replace(options()).unwrap();
    slint::platform::update_timers_and_animations();
    assert_ne!(window.get_first_state().generation, generation);
    let active = window.get_first_state().active;
    key(&window, Key::Space);
    assert_eq!(
        received.borrow().len(),
        1,
        "Space activates after the canonical query was reset, even within its former timeout"
    );
    assert_eq!(window.get_first_state().selected, active);
}

#[test]
fn carousel_navigation_does_not_intercept_native_child_editing_or_form_keys() {
    let (window, _choices) = fixture();
    let activations = Rc::new(RefCell::new(0));
    let output = activations.clone();
    window.on_carousel_activated(move || *output.borrow_mut() += 1);
    let before = window.get_carousel_state().active;
    window.invoke_focus_carousel_editor();
    key(&window, Key::Home);
    key(&window, Key::LeftArrow);
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: "x".into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: "x".into() });
    assert_eq!(
        window.get_carousel_editor_text(),
        "xab",
        "the editor retains its native caret and insertion"
    );
    assert_eq!(window.get_carousel_state().active, before);
    window.invoke_focus_carousel_checkbox();
    key(&window, Key::Return);
    assert_eq!(
        window.get_carousel_checkbox_state(),
        slint_kit::ui::CheckState::Unchecked
    );
    assert_eq!(
        *activations.borrow(),
        0,
        "a child's rejected Return must reach the enclosing form without activating a slide"
    );
    key(&window, Key::Space);
    assert_eq!(
        window.get_carousel_checkbox_state(),
        slint_kit::ui::CheckState::Checked
    );
    assert_eq!(*activations.borrow(), 0);
    window.invoke_focus_carousel();
    key(&window, Key::End);
    assert_ne!(
        window.get_carousel_state().active,
        before,
        "the carousel's own focus target still handles navigation"
    );
}
