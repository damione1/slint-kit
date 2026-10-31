// SPDX-License-Identifier: GPL-3.0-or-later
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, PointerEventButton, WindowEvent};
use slint::{ComponentHandle, LogicalPosition, LogicalSize, ModelRc, VecModel};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

fn fixture() -> Part2Fixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = Part2Fixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    window
}
fn element(window: &Part2Fixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    let requested_label = label.clone();
    ElementQuery::from_root(window)
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(requested_label.as_str())
        })
        .find_first()
        .unwrap_or_else(|| panic!("missing element {label}"))
}
fn editor(window: &Part2Fixture, label: &str) -> ElementHandle {
    let label = label.to_owned();
    ElementQuery::from_root(window)
        .match_type_name("TextInput")
        .match_predicate(move |element| {
            element.accessible_label().as_deref() == Some(label.as_str())
        })
        .find_first()
        .expect("native editor with supplied label")
}
fn down<W: ComponentHandle>(window: &W, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
}
fn up<W: ComponentHandle>(window: &W, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
}
fn key<W: ComponentHandle>(window: &W, key: Key) {
    down(window, key);
    up(window, key);
}
fn text<W: ComponentHandle>(window: &W, text: &str) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: text.into() });
}
fn click(window: &Part2Fixture, position: LogicalPosition) {
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
}

#[test]
fn native_field_label_focus_read_only_disabled_and_commit_are_distinct() {
    let window = fixture();
    let edits = Rc::new(Cell::new(0));
    let observed = edits.clone();
    window.on_name_edited(move |_| observed.set(observed.get() + 1));
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    window.on_name_committed(move |value| observed.borrow_mut().push(value.to_string()));
    let label = ElementQuery::from_root(&window)
        .match_inherits("KitFieldLabel")
        .find_first()
        .unwrap();
    label.mock_single_click(PointerEventButton::Left);
    assert!(window.get_name_focused(), "label focuses the actual editor");
    text(&window, "é漢🦊");
    assert_eq!(window.get_name_text().as_str(), "é漢🦊");
    assert_eq!(edits.get(), 1);
    key(&window, Key::Return);
    assert_eq!(&*commits.borrow(), &["é漢🦊"]);
    window.set_access(FieldAccess::ReadOnly);
    window.invoke_focus_name();
    assert!(window.get_name_focused());
    text(&window, "x");
    assert_eq!(window.get_name_text().as_str(), "é漢🦊");
    window.set_access(FieldAccess::Disabled);
    window.invoke_focus_name();
    text(&window, "y");
    assert_eq!(window.get_name_text().as_str(), "é漢🦊");
    assert_eq!(
        editor(&window, "Full name").accessible_enabled(),
        Some(false)
    );
    window.set_access(FieldAccess::Editable);
    window.invoke_focus_name();
    window.invoke_select_name();
    text(&window, "Ada");
    assert_eq!(window.get_name_text().as_str(), "Ada");
}

#[test]
fn password_reveal_and_search_clear_retain_the_native_editor() {
    let window = fixture();
    window.invoke_focus_password();
    text(&window, "secret");
    let password = editor(&window, "Password");
    element(&window, "Show password").mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_secret_visibility(), SecretVisibility::Revealed);
    assert!(password.is_valid());
    assert!(window.get_password_focused());
    text(&window, "!");
    assert_eq!(window.get_password_text().as_str(), "secret!");
    window.set_search_text("Ada".into());
    window.invoke_focus_search();
    let search = editor(&window, "Find a person");
    element(&window, "Clear search").mock_single_click(PointerEventButton::Left);
    assert_eq!(window.get_search_text().as_str(), "");
    assert!(window.get_search_focused());
    assert!(search.is_valid());
    text(&window, "Grace");
    assert_eq!(window.get_search_text().as_str(), "Grace");
}

#[test]
fn comfortable_editor_actions_use_the_full_touch_height_and_preserve_editor_focus() {
    let window = fixture();
    window.set_control_size(ControlSize::Comfortable);
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        window.set_search_text("Ada".into());
        window.set_secret_visibility(SecretVisibility::Hidden);
        window.invoke_focus_password();
        let search_bounds = editor(&window, "Find a person");
        click(
            &window,
            LogicalPosition::new(
                search_bounds.absolute_position().x + search_bounds.size().width / 2.,
                search_bounds.absolute_position().y - 5.,
            ),
        );
        assert!(
            window.get_search_focused(),
            "the visible surface padding belongs to the focus target"
        );
        window.invoke_focus_search();
        let native_editor = editor(&window, "Find a person");
        let clear = element(&window, "Clear search");
        let reveal = element(&window, "Show password");
        for action in [&clear, &reveal] {
            assert!(
                action.size().width >= window.get_theme().spacing.comfortable.height
                    && action.size().height >= window.get_theme().spacing.comfortable.height,
                "scale {scale}, action {:?}: {:?}",
                action.accessible_label(),
                action.size()
            );
        }
        assert!(
            clear.absolute_position().y
                >= native_editor.absolute_position().y
                    - window.get_theme().spacing.comfortable.padding_block
        );
        assert!(
            clear.absolute_position().y + clear.size().height
                <= native_editor.absolute_position().y
                    + native_editor.size().height
                    + window.get_theme().spacing.comfortable.padding_block
        );
        click(
            &window,
            LogicalPosition::new(
                clear.absolute_position().x + clear.size().width / 2.,
                clear.absolute_position().y + 1.,
            ),
        );
        assert!(window.get_search_text().is_empty());
        assert!(native_editor.is_valid());
        assert!(window.get_search_focused());
        text(&window, "Grace");
        assert_eq!(window.get_search_text(), "Grace");
    }
}

#[test]
fn textarea_newline_theme_and_undo_preserve_one_native_editor() {
    let window = fixture();
    let commits = Rc::new(Cell::new(0));
    let observed = commits.clone();
    window.on_area_committed(move |_| observed.set(observed.get() + 1));
    window.invoke_focus_area();
    text(&window, "First");
    key(&window, Key::Return);
    text(&window, "Second 👩‍💻");
    assert_eq!(window.get_area_text().as_str(), "First\nSecond 👩‍💻");
    assert_eq!(commits.get(), 0);
    let native_editor = editor(&window, "Notes");
    let mut theme = window.get_theme();
    theme.text_scale = 2.;
    window.set_theme(theme);
    assert!(window.get_area_focused());
    assert!(native_editor.is_valid());
    text(&window, "!");
    assert!(window.get_area_text().ends_with('!'));
    // Public synthetic events use Slint's normalized shortcut modifier.
    let modifier = Key::Control;
    down(&window, modifier);
    text(&window, "z");
    up(&window, modifier);
    let undone = window.get_area_text();
    assert!(
        "First\nSecond 👩‍💻".starts_with(undone.as_str()),
        "native undo restores an earlier prefix, not a typed shortcut character: {undone}"
    );
    assert!(!undone.ends_with('!'));
    assert_eq!(kit_core::grapheme_count("👩‍💻e\u{301}"), 2);
}

#[test]
fn choice_activation_and_roving_navigation_use_typed_intents() {
    let window = fixture();
    let states = Rc::new(RefCell::new(Vec::new()));
    let observed = states.clone();
    window.on_checkbox_toggled(move |state| observed.borrow_mut().push(state));
    element(&window, "Project notifications").invoke_accessible_default_action();
    assert_eq!(&*states.borrow(), &[CheckState::Checked]);
    window.set_access(FieldAccess::ReadOnly);
    element(&window, "Project notifications").invoke_accessible_default_action();
    assert_eq!(states.borrow().len(), 1);
    window.set_access(FieldAccess::Editable);
    window.set_choices(ModelRc::new(VecModel::from(vec![
        ChoiceRow {
            token: 1,
            label: "Monthly".into(),
            disabled: false,
            selected: false,
        },
        ChoiceRow {
            token: 2,
            label: "Unavailable".into(),
            disabled: true,
            selected: false,
        },
        ChoiceRow {
            token: 3,
            label: "Yearly".into(),
            disabled: false,
            selected: false,
        },
    ])));
    window.set_active_token(1);
    let controller = Rc::new(RefCell::new(
        kit_core::MultiSelectController::new(
            vec![
                kit_core::SelectOption::new(1, "Monthly"),
                {
                    let mut option = kit_core::SelectOption::new(2, "Unavailable");
                    option.disabled = true;
                    option
                },
                kit_core::SelectOption::new(3, "Yearly"),
            ],
            vec![],
            kit_core::Ownership::Local,
            None,
        )
        .unwrap(),
    ));
    controller
        .borrow_mut()
        .navigate(kit_core::NavigationStep::First, 5);
    let owner = controller.clone();
    let weak = window.as_weak();
    window.on_radio_navigate(move |step| {
        let step = match step {
            NavigationStep::First => kit_core::NavigationStep::First,
            NavigationStep::Last => kit_core::NavigationStep::Last,
            NavigationStep::Next => kit_core::NavigationStep::Next,
            NavigationStep::Previous => kit_core::NavigationStep::Previous,
            NavigationStep::PageNext => kit_core::NavigationStep::PageNext,
            NavigationStep::PagePrevious => kit_core::NavigationStep::PagePrevious,
        };
        let next = owner.borrow_mut().navigate(step, 5).copied();
        if let Some(next) = next {
            weak.upgrade().unwrap().set_active_token(next);
        }
    });
    let activated = Rc::new(Cell::new(None));
    let observed = activated.clone();
    window.on_radio_activate(move |token, generation| observed.set(Some((token, generation))));
    window.invoke_focus_radio();
    assert!(window.get_radio_focused());
    key(&window, Key::DownArrow);
    assert_eq!(window.get_active_token(), 3);
    key(&window, Key::Return);
    assert_eq!(activated.get(), Some((3, 7)));
}

fn calendar_days() -> Vec<CalendarDay> {
    (1..=31)
        .map(|day| CalendarDay {
            year: 2026,
            month: 10,
            day,
            label: day.to_string().into(),
            accessible_label: format!("October {day}, 2026").into(),
            enabled: day != 2,
            selected: day == 1,
            in_range: false,
            today: false,
        })
        .collect()
}
#[test]
fn calendar_keyboard_and_native_popup_keep_provisional_selection_explicit() {
    let window = fixture();
    window.set_calendar_days(ModelRc::new(VecModel::from(calendar_days())));
    window.set_calendar_active(0);
    let moves = Rc::new(RefCell::new(Vec::new()));
    let observed = moves.clone();
    window.on_calendar_navigate(move |step| observed.borrow_mut().push(step));
    let chosen = Rc::new(Cell::new(0));
    let observed = chosen.clone();
    window.on_calendar_choose(move |day| observed.set(day.day));
    window.invoke_focus_calendar();
    assert!(window.get_calendar_focused());
    key(&window, Key::RightArrow);
    key(&window, Key::PageDown);
    assert_eq!(
        &*moves.borrow(),
        &[CalendarNavigation::NextDay, CalendarNavigation::NextMonth]
    );
    key(&window, Key::Return);
    assert_eq!(chosen.get(), 1);
    window.set_calendar_active(1);
    key(&window, Key::Return);
    assert_eq!(chosen.get(), 1, "disabled day cannot be chosen");
    let selected = Rc::new(Cell::new(0));
    let observed = selected.clone();
    window.on_picker_choose(move |day| observed.set(day.day));
    element(&window, "Ouvrir le calendrier de réservation")
        .mock_single_click(PointerEventButton::Left);
    assert!(window.get_picker_open());
    assert!(element(&window, "Mois précédent").size().width > 0.);
    assert!(element(&window, "Mois suivant").size().width > 0.);
    key(&window, Key::Escape);
    assert!(!window.get_picker_open());
    window.set_picker_open(true);
    window.set_calendar_active(2);
    key(&window, Key::Return);
    assert_eq!(selected.get(), 3);
    assert!(!window.get_picker_open());
}

#[test]
fn control_geometry_is_intrinsic_across_window_heights_and_scales() {
    let window = fixture();
    for (size, expected) in [
        (ControlSize::Compact, 32.),
        (ControlSize::Medium, 36.),
        (ControlSize::Comfortable, 40.),
    ] {
        window.set_control_size(size);
        let before = (
            window.get_simple_height(),
            window.get_button_height(),
            window.get_select_height(),
        );
        assert_eq!(before, (expected, expected, expected));
        window.window().dispatch_event(WindowEvent::Resized {
            size: LogicalSize::new(1100., 1300.),
        });
        assert_eq!(
            (
                window.get_simple_height(),
                window.get_button_height(),
                window.get_select_height()
            ),
            before
        );
    }
    let before = window.get_area_height();
    let mut theme = window.get_theme();
    theme.text_scale = 2.;
    window.set_theme(theme);
    assert!(window.get_simple_height() >= 52.);
    assert!(window.get_button_height() >= 52.);
    assert!(window.get_select_height() >= 52.);
    assert!(window.get_area_height() > before);
}

#[test]
fn slider_cancel_is_distinct_from_commit_and_read_only_blocks_edits() {
    let window = fixture();
    let cancelled = Rc::new(Cell::new(0));
    let observed = cancelled.clone();
    window.on_slider_cancelled(move || observed.set(observed.get() + 1));
    let committed = Rc::new(Cell::new(0));
    let observed = committed.clone();
    window.on_slider_committed(move || observed.set(observed.get() + 1));
    let started = Rc::new(Cell::new(0));
    let observed = started.clone();
    window.on_slider_started(move || observed.set(observed.get() + 1));
    window.invoke_focus_slider();
    key(&window, Key::Escape);
    assert_eq!(cancelled.get(), 1);
    assert_eq!(committed.get(), 0);
    let slider = element(&window, "Volume");
    let origin = slider.absolute_position();
    click(
        &window,
        LogicalPosition::new(
            origin.x + slider.size().width / 2.,
            origin.y + slider.size().height / 2.,
        ),
    );
    assert_eq!(started.get(), 1);
    assert_eq!(committed.get(), 1);
    window.set_access(FieldAccess::ReadOnly);
    click(
        &window,
        LogicalPosition::new(origin.x + 20., origin.y + 10.),
    );
    assert_eq!(started.get(), 1);
    assert_eq!(committed.get(), 1);
    window.set_access(FieldAccess::Editable);
    let position = LogicalPosition::new(origin.x + 40., origin.y + 10.);
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    assert_eq!(started.get(), 2);
    window.set_access(FieldAccess::ReadOnly);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert_eq!(
        cancelled.get(),
        2,
        "an access change cancels the active gesture"
    );
    assert_eq!(
        committed.get(),
        1,
        "a cancelled drag must not commit at release"
    );
}

#[test]
fn choice_access_changes_cancel_held_activation_without_disabling_read_only_focus() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    let activated = Rc::new(Cell::new(0));
    let observed = activated.clone();
    window.on_checkbox_activated(move || observed.set(observed.get() + 1));
    let observed = activated.clone();
    window.on_switch_activated(move || observed.set(observed.get() + 1));
    for switch in [false, true] {
        if switch {
            window.invoke_focus_switch();
        } else {
            window.invoke_focus_check();
        }
        down(&window, Key::Space);
        window.set_access(FieldAccess::ReadOnly);
        key(&window, Key::RightArrow);
        if !switch {
            assert!(window.get_checkbox_focused());
        }
        window.set_access(FieldAccess::Editable);
        up(&window, Key::Space);
        assert_eq!(activated.get(), 0);
        key(&window, Key::Space);
        assert_eq!(activated.get(), 1);
        activated.set(0);
    }
}

#[test]
fn read_only_combobox_value_is_native_selectable_and_copyable() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    window.invoke_focus_combo();
    window.set_access(FieldAccess::ReadOnly);
    key(&window, Key::RightArrow);
    assert!(
        window.get_combobox_focused(),
        "changing to read-only preserves focus on the value"
    );
    let opened = Rc::new(Cell::new(0));
    let observed = opened.clone();
    window.on_combo_opened(move || observed.set(observed.get() + 1));
    key(&window, Key::Return);
    assert_eq!(opened.get(), 0);
    // Public synthetic events use Slint's normalized shortcut modifier.
    let modifier = Key::Control;
    down(&window, modifier);
    text(&window, "a");
    text(&window, "c");
    up(&window, modifier);
    window.invoke_focus_paste();
    down(&window, modifier);
    text(&window, "v");
    up(&window, modifier);
    assert_eq!(window.get_pasted().as_str(), "Grace Hopper");
    let multi_opened = Rc::new(Cell::new(0));
    let observed = multi_opened.clone();
    window.on_multi_opened(move || observed.set(observed.get() + 1));
    window.invoke_focus_multi();
    key(&window, Key::Return);
    assert_eq!(multi_opened.get(), 0);
    let chip = ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Ada Lovelace"))
        .find_first()
        .unwrap();
    assert_eq!(chip.accessible_enabled(), Some(true));
}

#[test]
fn long_choice_labels_increase_every_family_height_at_200_percent() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    let mut theme = window.get_theme();
    theme.text_scale = 2.;
    window.set_theme(theme);
    let label = "Notifications pour toutes les personnes qui participent au projet et aux prochaines étapes de sa réalisation";
    window.set_long_label(label.into());
    window.set_choices(ModelRc::new(VecModel::from(vec![
        ChoiceRow {
            token: 1,
            label: label.into(),
            disabled: false,
            selected: false,
        },
        ChoiceRow {
            token: 2,
            label: "Un autre choix long qui doit rester lisible avec un texte agrandi".into(),
            disabled: false,
            selected: true,
        },
    ])));
    // Visiting mounts model delegates, as a native frame does before measurement.
    let options = ElementQuery::from_root(&window)
        .match_inherits("ChoiceOption")
        .find_all();
    assert!(window.get_checkbox_height() > 52.);
    assert!(window.get_switch_height() > 52.);
    assert!(
        window.get_radio_height() > 104.,
        "heights check={} switch={} radio={} segmented={} toggle={}",
        window.get_checkbox_height(),
        window.get_switch_height(),
        window.get_radio_height(),
        window.get_segmented_height(),
        window.get_toggle_height()
    );
    assert!(window.get_segmented_height() > 58.);
    assert!(window.get_toggle_height() > 52.);
    assert_eq!(options.len(), 6);
    for option in options {
        assert!(
            option.size().height > 52.,
            "wrapped option retains sufficient height"
        );
    }
}

#[test]
fn choice_and_calendar_options_cancel_pointer_press_when_access_changes() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    window.set_choices(ModelRc::new(VecModel::from(vec![ChoiceRow {
        token: 1,
        label: "Stable choice".into(),
        disabled: false,
        selected: false,
    }])));
    window.set_days(ModelRc::new(VecModel::from(calendar_days())));
    let activated = Rc::new(Cell::new(0));
    let observed = activated.clone();
    window.on_option_activated(move || observed.set(observed.get() + 1));
    let observed = activated.clone();
    window.on_calendar_activated(move || observed.set(observed.get() + 1));
    for kind in ["ChoiceOption", "CalendarOption"] {
        let option = ElementQuery::from_root(&window)
            .match_inherits(kind)
            .find_first()
            .unwrap();
        let origin = option.absolute_position();
        let position = LogicalPosition::new(
            origin.x + option.size().width / 2.,
            origin.y + option.size().height / 2.,
        );
        window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        window.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        window.set_access(FieldAccess::ReadOnly);
        window
            .window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        window.set_access(FieldAccess::Editable);
        window
            .window()
            .dispatch_event(WindowEvent::PointerReleased {
                position,
                button: PointerEventButton::Left,
            });
        assert_eq!(
            activated.get(),
            0,
            "{kind} cancels press across an access transition"
        );
        option.mock_single_click(PointerEventButton::Left);
        assert_eq!(
            activated.get(),
            1,
            "{kind} remains usable after cancellation"
        );
        activated.set(0);
    }
}

#[test]
fn multiselect_read_only_projection_does_not_reserve_hidden_editor_rows() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    ElementQuery::from_root(&window)
        .match_inherits("KitChip")
        .find_all();
    assert!(
        window.get_multi_height() <= 100.,
        "one chip plus trigger should occupy two rows, got {}",
        window.get_multi_height()
    );
    window.set_access(FieldAccess::ReadOnly);
    ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .find_all();
    assert!(
        window.get_multi_height() <= 100.,
        "read-only copy target replaces rather than duplicates the chip row"
    );
}

fn quality_fixture() -> FormQualityFixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = FormQualityFixture::new().unwrap();
    kit_slint::init_form_validation!(ui, window);
    window.show().unwrap();
    window
}

fn quality_element(window: &FormQualityFixture, label: &str) -> ElementHandle {
    ElementHandle::find_by_accessible_label(window, label)
        .next()
        .unwrap_or_else(|| panic!("missing quality control {label}"))
}

#[test]
fn search_clear_reports_the_empty_semantic_value_once() {
    let window = quality_fixture();
    let values = Rc::new(RefCell::new(Vec::new()));
    let observed = values.clone();
    window.on_query_value_edited(move |value| observed.borrow_mut().push(value));
    quality_element(&window, "Clear search").invoke_accessible_default_action();
    assert_eq!(window.get_query(), "");
    assert_eq!(&*values.borrow(), &[slint::SharedString::from("")]);
    let native_editor = ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Find people"))
        .find_first()
        .unwrap();
    text(&window, "Grace");
    assert!(native_editor.is_valid());
    assert_eq!(window.get_query(), "Grace");
    assert_eq!(values.borrow().last().unwrap(), "Grace");
}

#[test]
fn segmented_rtl_reverses_visual_order_without_changing_tokens_or_held_presses() {
    let window = quality_fixture();
    let actions = Rc::new(RefCell::new(Vec::new()));
    let observed = actions.clone();
    window.on_segment_activated(move |token, generation| {
        observed.borrow_mut().push((token, generation));
    });
    let first = quality_element(&window, "First");
    let last = quality_element(&window, "Last");
    assert!(first.absolute_position().x < last.absolute_position().x);
    let position = LogicalPosition::new(
        first.absolute_position().x + first.size().width / 2.,
        first.absolute_position().y + first.size().height / 2.,
    );
    window
        .window()
        .dispatch_event(WindowEvent::PointerMoved { position });
    window.window().dispatch_event(WindowEvent::PointerPressed {
        position,
        button: PointerEventButton::Left,
    });
    window.set_flow(ChoiceFlow::RightToLeft);
    window
        .window()
        .dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
    assert!(
        actions.borrow().is_empty(),
        "a reordered held press is cancelled"
    );
    let first = quality_element(&window, "First");
    let last = quality_element(&window, "Last");
    assert!(last.absolute_position().x < first.absolute_position().x);
    last.mock_single_click(PointerEventButton::Left);
    assert_eq!(&*actions.borrow(), &[(99, 17)]);
    quality_element(&window, "Middle").invoke_accessible_default_action();
    assert_eq!(
        &*actions.borrow(),
        &[(99, 17)],
        "disabled identity stays disabled"
    );
    let moves = Rc::new(RefCell::new(Vec::new()));
    let observed = moves.clone();
    window.on_segment_navigated(move |step| observed.borrow_mut().push(step));
    window.invoke_focus_segments();
    key(&window, Key::RightArrow);
    assert_eq!(&*moves.borrow(), &[NavigationStep::Previous]);
}

#[test]
fn sliders_and_nested_color_and_ambiguity_controls_follow_density_at_200_percent() {
    let window = quality_fixture();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme.clone());
        for size in [
            ControlSize::Compact,
            ControlSize::Medium,
            ControlSize::Comfortable,
        ] {
            window.set_control_size(size);
            let density = match size {
                ControlSize::Compact => &theme.spacing.compact,
                ControlSize::Medium => &theme.spacing.medium,
                ControlSize::Comfortable => &theme.spacing.comfortable,
            };
            let control_height = density
                .height
                .max(theme.typography.line_height_control * scale + 2. * density.padding_block);
            let sliders = ElementQuery::from_root(&window)
                .match_inherits("KitSlider")
                .find_all();
            assert_eq!(sliders.len(), 5, "single value and four byte channels");
            for slider in sliders {
                assert!(
                    slider.size().height >= control_height,
                    "{size:?} at {scale}: slider height {} below {control_height}",
                    slider.size().height
                );
            }
            assert!(quality_element(&window, "Value range").size().height >= control_height);
            let button_height = quality_element(&window, "Density reference").size().height;
            for label in ["Use the earlier occurrence", "Use the later occurrence"] {
                assert!(
                    (quality_element(&window, label).size().height - button_height).abs() < 0.01,
                    "{label} follows {size:?} at {scale}: actual {}, reference {button_height}",
                    quality_element(&window, label).size().height
                );
            }
        }
    }
}

#[test]
fn color_byte_editor_filters_letters_and_blocks_out_of_range_or_fractional_commit() {
    let window = quality_fixture();
    let commits = Rc::new(RefCell::new(Vec::new()));
    let observed = commits.clone();
    window.on_channel_committed(move |channel, value| observed.borrow_mut().push((channel, value)));
    let red = ElementQuery::from_root(&window)
        .match_type_name("TextInput")
        .match_predicate(|element| element.accessible_label().as_deref() == Some("Red"))
        .find_first()
        .unwrap();
    red.mock_single_click(PointerEventButton::Left);
    text(&window, "Ada");
    assert_eq!(
        window.get_red_text(),
        "79",
        "letters do not enter a channel draft"
    );
    for invalid in ["256", "-1", "12.5"] {
        window.set_red_text(invalid.into());
        key(&window, Key::Return);
        assert!(
            commits.borrow().is_empty(),
            "invalid byte {invalid} cannot commit"
        );
        assert!(red.is_valid(), "the native editor is never remounted");
    }
    window.set_red_text("128".into());
    key(&window, Key::Return);
    assert_eq!(
        &*commits.borrow(),
        &[(ColorChannel::Red, slint::SharedString::from("128"))]
    );
}

#[test]
fn switch_track_and_radio_marks_stay_centered_beside_wrapped_or_enlarged_labels() {
    i_slint_backend_testing::init_no_event_loop();
    let window = ChoiceRegressionFixture::new().unwrap();
    window.show().unwrap();
    for scale in [1., 2.] {
        let mut theme = window.get_theme();
        theme.text_scale = scale;
        window.set_theme(theme);
        for (size, box_side) in [
            (ControlSize::Compact, 16.),
            (ControlSize::Medium, 20.),
            (ControlSize::Comfortable, 24.),
        ] {
            window.set_control_size(size);
            for label in [
                "Updates",
                "Notify everyone participating in this project and the following stages",
            ] {
                window.set_long_label(label.into());
                window.set_choices(ModelRc::new(VecModel::from(vec![ChoiceRow {
                    token: 41,
                    label: label.into(),
                    disabled: false,
                    selected: true,
                }])));
                let switch = ElementQuery::from_root(&window)
                    .match_inherits("KitSwitch")
                    .find_first()
                    .unwrap();
                let track = switch
                    .query_descendants()
                    .match_type_name("Rectangle")
                    .match_predicate(move |element| {
                        (element.size().width - box_side * 1.9).abs() < 0.01
                            && (element.size().height - box_side - 2.).abs() < 0.01
                    })
                    .find_first()
                    .expect("switch track");
                assert!(
                    ((track.absolute_position().y + track.size().height / 2.)
                        - (switch.absolute_position().y + switch.size().height / 2.))
                        .abs()
                        < 0.01,
                    "switch {size:?}, scale {scale}, label {label}"
                );
                let option = ElementQuery::from_root(&window)
                    .match_inherits("KitRadioGroup")
                    .find_first()
                    .unwrap()
                    .query_descendants()
                    .match_inherits("ChoiceOption")
                    .find_first()
                    .unwrap();
                let mark = option
                    .query_descendants()
                    .match_type_name("Rectangle")
                    .match_predicate(move |element| {
                        (element.size().width - box_side).abs() < 0.01
                            && (element.size().height - box_side).abs() < 0.01
                    })
                    .find_first()
                    .expect("radio indicator");
                assert!(
                    ((mark.absolute_position().y + mark.size().height / 2.)
                        - (option.absolute_position().y + option.size().height / 2.))
                        .abs()
                        < 0.01,
                    "radio {size:?}, scale {scale}, label {label}"
                );
            }
        }
    }
}
