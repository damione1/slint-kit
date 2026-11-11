// SPDX-License-Identifier: GPL-3.0-or-later
use crate::{FieldAccess, OptionRow, Ownership, SelectionRef, SelectionSearchSessionFixture};
use i_slint_backend_testing::{ElementHandle, ElementQuery, TestingBackend, TestingBackendOptions};
use slint::{
    ComponentHandle, Model, ModelRc, SharedString, VecModel,
    platform::{Key, PointerEventButton, WindowEvent},
};
use slint_kit::{Choices, SelectBinding, SelectOption};
use std::{
    cell::{Cell, OnceCell, RefCell},
    rc::Rc,
};

thread_local! { static BACKEND: OnceCell<()> = const { OnceCell::new() }; }
fn options() -> Vec<SelectOption<u32>> {
    let mut result = vec![
        SelectOption::new(0, "Amber"),
        SelectOption::new(1, "Apricot"),
        SelectOption::new(2, "Blocked"),
        SelectOption::new(3, "Banana"),
    ];
    result[2].disabled = true;
    result
}
fn fixture() -> (SelectionSearchSessionFixture, Choices<u32>) {
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
    let app = SelectionSearchSessionFixture::new().unwrap();
    slint_kit::install(&app);
    let choices = Choices::from_options(options()).unwrap();
    app.set_options(choices.model());
    app.show().unwrap();
    settle(&app);
    (app, choices)
}
fn settle(app: &SelectionSearchSessionFixture) {
    slint::platform::update_timers_and_animations();
    app.window().take_snapshot().unwrap();
    slint::platform::update_timers_and_animations();
}
fn key(app: &SelectionSearchSessionFixture, text: impl Into<SharedString>) {
    let text = text.into();
    app.window()
        .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
    app.window()
        .dispatch_event(WindowEvent::KeyReleased { text });
    settle(app);
}
fn control(app: &SelectionSearchSessionFixture, text: &str) {
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Control.into(),
    });
    key(app, text);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Control.into(),
    });
    settle(app);
}
fn element(app: &SelectionSearchSessionFixture, name: &str) -> ElementHandle {
    let name = name.to_owned();
    ElementQuery::from_root(app)
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(name.as_str()))
        .find_first()
        .unwrap()
}
fn search_editor(app: &SelectionSearchSessionFixture) -> ElementHandle {
    ElementQuery::from_root(app)
        .match_type_name("TextInput")
        .match_predicate(|e| e.accessible_label().as_deref() == Some("Search options"))
        .find_first()
        .unwrap()
}
#[derive(Clone, Copy)]
enum Kind {
    Single,
    Multiple,
}
impl Kind {
    fn focus(self, app: &SelectionSearchSessionFixture) {
        match self {
            Self::Single => app.invoke_focus_single(),
            Self::Multiple => app.invoke_focus_multi(),
        }
    }
    fn query(self, app: &SelectionSearchSessionFixture) -> SharedString {
        match self {
            Self::Single => app.get_single_query(),
            Self::Multiple => app.get_multi_query(),
        }
    }
    fn open(self, app: &SelectionSearchSessionFixture) -> bool {
        match self {
            Self::Single => app.get_single_state().open,
            Self::Multiple => app.get_multi_state().open,
        }
    }
    fn rows(self, app: &SelectionSearchSessionFixture) -> i32 {
        match self {
            Self::Single => app.get_single_row_count(),
            Self::Multiple => app.get_multi_row_count(),
        }
    }
    fn name(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multiple => "multi",
        }
    }
    fn component(self) -> &'static str {
        match self {
            Self::Single => "KitSelect",
            Self::Multiple => "KitMultiSelect",
        }
    }
}
fn search(app: &SelectionSearchSessionFixture, kind: Kind) {
    kind.focus(app);
    key(app, Key::DownArrow);
    assert!(kind.open(app));
    search_editor(app).mock_single_click(PointerEventButton::Left);
    key(app, "Ap");
    assert_eq!(kind.query(app), "Ap");
    assert_eq!(kind.rows(app), 1);
    assert_eq!(search_editor(app).accessible_value().as_deref(), Some("Ap"));
}
fn capture_reopened(app: &SelectionSearchSessionFixture, kind: Kind) {
    let Some(_) = std::env::var_os("KIT_CONTROL_CAPTURE_DIR") else {
        return;
    };
    for (name, base) in [("light", app.get_theme()), ("dark", app.get_dark_theme())] {
        for scale in [1., 2.] {
            let mut theme = base.clone();
            theme.text_scale = scale;
            app.set_theme(theme);
            settle(app);
            assert!(kind.open(app));
            assert_eq!(kind.rows(app), 4);
            assert_eq!(search_editor(app).accessible_value().as_deref(), Some(""));
            crate::choice_presentation::save_capture(
                &app.window().take_snapshot().unwrap(),
                &format!("search-session-reopen-{}-{name}-{scale}", kind.name()),
            );
        }
    }
}

#[test]
fn select_and_multi_share_visible_query_and_reopen_unfiltered_with_controlled_acknowledgment() {
    for kind in [Kind::Single, Kind::Multiple] {
        for ownership in [Ownership::Local, Ownership::Controlled] {
            let (app, choices) = fixture();
            app.set_ownership(ownership);
            let proposed = Rc::new(RefCell::new(None));
            let events = Rc::new(RefCell::new(Vec::new()));
            match kind {
                Kind::Single => {
                    let output = proposed.clone();
                    let source = choices.clone();
                    let order = events.clone();
                    app.on_single_chosen(move |row| {
                        order.borrow_mut().push("chosen".to_owned());
                        *output.borrow_mut() = source.accept(row);
                    });
                    let order = events.clone();
                    app.on_single_query_edited(move |query| {
                        if query.is_empty() {
                            order.borrow_mut().push("reset".to_owned());
                        }
                    });
                }
                Kind::Multiple => {
                    let output = proposed.clone();
                    let source = choices.clone();
                    let order = events.clone();
                    app.on_multi_changed(move |row, added| {
                        assert!(added);
                        order.borrow_mut().push("chosen".to_owned());
                        *output.borrow_mut() = source.accept(row);
                    });
                    let order = events.clone();
                    app.on_multi_query_edited(move |query| {
                        if query.is_empty() {
                            order.borrow_mut().push("reset".to_owned());
                        }
                    });
                }
            }
            search(&app, kind);
            match kind {
                Kind::Single => key(&app, Key::Return),
                Kind::Multiple => ElementQuery::from_root(&app)
                    .match_inherits("KitSelectSurface")
                    .find_first()
                    .unwrap()
                    .query_descendants()
                    .match_inherits("KitOption")
                    .match_predicate(|row| row.accessible_label().as_deref() == Some("Apricot"))
                    .find_first()
                    .unwrap()
                    .mock_single_click(PointerEventButton::Left),
            };
            settle(&app);
            assert_eq!(*proposed.borrow(), Some(1));
            assert_eq!(&*events.borrow(), &["chosen", "reset"]);
            assert!(!kind.open(&app));
            assert_eq!(kind.query(&app), "");
            assert_eq!(kind.rows(&app), 4);
            let selected = choices.selection(&1).unwrap();
            match (kind, ownership) {
                (Kind::Single, Ownership::Local) => assert_eq!(app.get_single_selected(), selected),
                (Kind::Single, Ownership::Controlled) => {
                    assert!(!app.get_single_selected().present);
                    app.set_single_selected(selected);
                }
                (Kind::Multiple, Ownership::Local) => {
                    assert_eq!(app.get_multi_selected().row_count(), 1)
                }
                (Kind::Multiple, Ownership::Controlled) => {
                    assert_eq!(app.get_multi_selected().row_count(), 0);
                    app.set_multi_selected(choices.selections(&[1]).unwrap());
                }
            }
            kind.focus(&app);
            key(&app, Key::DownArrow);
            assert_eq!(kind.rows(&app), 4);
            assert_eq!(search_editor(&app).accessible_value().as_deref(), Some(""));
            if ownership == Ownership::Local {
                capture_reopened(&app, kind);
            }
            key(&app, Key::Escape);
        }
    }
}

#[test]
fn dismissals_clear_only_the_closed_search_and_reopening_keeps_all_rows() {
    for kind in [Kind::Single, Kind::Multiple] {
        let (app, _) = fixture();
        for dismissal in [Key::Escape, Key::Tab] {
            search(&app, kind);
            key(&app, dismissal);
            assert!(!kind.open(&app));
            assert_eq!(kind.query(&app), "");
            assert_eq!(kind.rows(&app), 4);
        }
        search(&app, kind);
        element(&app, "Outside editor").mock_single_click(PointerEventButton::Left);
        settle(&app);
        assert!(!kind.open(&app));
        assert_eq!(kind.query(&app), "");
        search(&app, kind);
        let trigger = ElementQuery::from_root(&app)
            .match_inherits(kind.component())
            .find_first()
            .unwrap()
            .query_descendants()
            .match_inherits("KitSelectTrigger")
            .find_first()
            .unwrap();
        trigger.invoke_accessible_default_action();
        settle(&app);
        assert!(!kind.open(&app));
        assert_eq!(kind.query(&app), "");
        for access in [FieldAccess::ReadOnly, FieldAccess::Disabled] {
            search(&app, kind);
            app.set_access(access);
            settle(&app);
            assert!(!kind.open(&app));
            assert_eq!(kind.query(&app), "");
            app.set_access(FieldAccess::Editable);
            settle(&app);
        }
    }
}

#[test]
fn stale_and_disabled_activation_do_not_clear_an_open_search() {
    let (app, choices) = fixture();
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    app.on_single_chosen(move |_| output.set(output.get() + 1));
    search(&app, Kind::Single);
    for disable in [false, true] {
        let row = ElementQuery::from_root(&app)
            .match_inherits("KitSelectSurface")
            .find_first()
            .unwrap()
            .query_descendants()
            .match_inherits("KitOption")
            .match_predicate(|row| row.accessible_label().as_deref() == Some("Apricot"))
            .find_first()
            .unwrap();
        // absolute_position is popup-tree local. Pointer dispatch needs the
        // actual popup anchor in the window, not a point in the main form.
        let anchor = ElementQuery::from_root(&app)
            .match_inherits("KitSelect")
            .find_first()
            .unwrap();
        let position = slint::LogicalPosition::new(
            anchor.absolute_position().x + row.absolute_position().x + row.size().width / 2.,
            anchor.absolute_position().y
                + anchor.size().height
                + app.get_theme().spacing.unit
                + row.absolute_position().y
                + row.size().height / 2.,
        );
        app.window()
            .dispatch_event(WindowEvent::PointerMoved { position });
        app.window().dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        assert!(
            app.get_single_state().open,
            "the press must stay inside the popup"
        );
        assert_eq!(
            app.get_single_query(),
            "Ap",
            "the press must not dismiss the search"
        );
        let mut replacement = options();
        replacement[1].disabled = disable;
        choices.replace(replacement).unwrap();
        settle(&app);
        app.window().dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        settle(&app);
        assert_eq!(app.get_single_query(), "Ap");
        assert!(app.get_single_state().open);
        assert_eq!(count.get(), 0);
    }
}

fn projected(app: &SelectionSearchSessionFixture) -> SelectBinding<u32> {
    let weak = app.as_weak();
    let binding = SelectBinding::new(
        options(),
        None,
        kit_core::Ownership::Controlled,
        move |projection| {
            let Some(app) = weak.upgrade() else { return };
            let reference = |token: Option<i32>| SelectionRef {
                present: token.is_some(),
                token: token.unwrap_or(0),
            };
            app.set_projected_rows(ModelRc::new(VecModel::from(
                projection
                    .rows
                    .iter()
                    .map(|row| OptionRow {
                        token: row.token,
                        generation: row.generation,
                        label: row.label.clone().into(),
                        description: row.description.clone().into(),
                        disabled: row.disabled,
                    })
                    .collect::<Vec<_>>(),
            )));
            app.set_projected_state(crate::SelectState {
                open: projection.open,
                selected: reference(projection.selected),
                active: reference(projection.active),
                active_index: projection.active_index,
                generation: projection.generation,
                selection_availability: match projection.selection_availability {
                    kit_core::SelectionAvailability::Absent => crate::SelectionAvailability::Absent,
                    kit_core::SelectionAvailability::Available => {
                        crate::SelectionAvailability::Available
                    }
                    kit_core::SelectionAvailability::Unavailable => {
                        crate::SelectionAvailability::Unavailable
                    }
                },
            });
        },
    );
    let own = binding.clone();
    app.on_projected_open(move || own.open());
    let own = binding.clone();
    app.on_projected_navigate(move |step| own.navigate(step.into(), None));
    let own = binding.clone();
    app.on_projected_activate(move |token, generation| own.activate(token, generation));
    let own = binding.clone();
    app.on_projected_query_edited(move |query| own.query(query.as_str()));
    let own = binding.clone();
    app.on_projected_dismissed(move |reason| own.dismiss(reason.into()));
    binding
}
#[test]
fn projected_host_close_and_reentrant_choice_or_dismiss_preserve_the_new_session_query() {
    let (app, _) = fixture();
    let binding = projected(&app);
    app.invoke_focus_projected();
    key(&app, Key::DownArrow);
    search_editor(&app).mock_single_click(PointerEventButton::Left);
    key(&app, "Ap");
    binding.dismiss(kit_core::DismissReason::Programmatic);
    settle(&app);
    assert_eq!(app.get_projected_query(), "");
    assert_eq!(app.get_projected_rows().row_count(), 4);
    for by_choice in [true, false] {
        app.set_projected_query("Ap".into());
        binding.query("Ap");
        binding.open();
        settle(&app);
        let own = binding.clone();
        let weak = app.as_weak();
        let reopen = move || {
            let app = weak.upgrade().unwrap();
            app.set_projected_query("Ba".into());
            own.query("Ba");
            own.open();
        };
        if by_choice {
            binding.on_intent(move |intent| {
                assert!(matches!(intent, kit_core::SelectionIntent::Commit(1)));
                reopen();
            });
            let state = app.get_projected_state();
            binding.activate(state.active.token, state.generation);
        } else {
            let own = binding.clone();
            app.on_projected_dismissed(move |reason| {
                own.dismiss(reason.into());
                reopen();
            });
            app.invoke_focus_projected();
            key(&app, Key::Escape);
        }
        settle(&app);
        assert_eq!(app.get_projected_query(), "Ba");
        assert!(app.get_projected_state().open);
        assert_eq!(app.get_projected_rows().row_count(), 1);
        assert_eq!(
            search_editor(&app).accessible_value().as_deref(),
            Some("Ba")
        );
    }
}

#[test]
fn persistent_menu_filters_survive_activation_and_dismissal() {
    let (app, _) = fixture();
    app.set_persistent_query("Ap".into());
    settle(&app);
    app.invoke_cycle_persistent();
    key(&app, Key::End);
    key(&app, Key::Return);
    key(&app, Key::Escape);
    assert_eq!(app.get_persistent_choices(), 1);
    assert_eq!(app.get_persistent_query(), "Ap");
}

#[test]
fn combobox_close_cancels_old_debounce_without_replacing_native_selection_or_undo() {
    let (app, choices) = fixture();
    app.set_combo_selected(choices.selection(&1).unwrap());
    app.set_combo_debounce(50);
    settle(&app);
    let queries = Rc::new(RefCell::new(Vec::new()));
    let output = queries.clone();
    app.on_combo_query_edited(move |query| output.borrow_mut().push(query.to_string()));
    app.invoke_focus_combo();
    control(&app, "a");
    key(&app, "Ap");
    key(&app, Key::LeftArrow);
    app.window().dispatch_event(WindowEvent::KeyPressed {
        text: Key::Shift.into(),
    });
    key(&app, Key::RightArrow);
    app.window().dispatch_event(WindowEvent::KeyReleased {
        text: Key::Shift.into(),
    });
    let trigger = ElementQuery::from_root(&app)
        .match_inherits("KitCombobox")
        .find_first()
        .unwrap()
        .query_descendants()
        .match_inherits("KitSelectTrigger")
        .find_first()
        .unwrap();
    trigger.invoke_accessible_default_action();
    settle(&app);
    assert!(!app.get_combo_state().open);
    assert_eq!(app.get_combo_query(), "");
    assert_eq!(app.get_combo_text(), "Ap");
    assert!(app.get_combo_focused());
    i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(60));
    settle(&app);
    assert_eq!(&*queries.borrow(), &[""]);
    key(&app, "b");
    assert_eq!(app.get_combo_text(), "Ab");
    control(&app, "z");
    control(&app, "z");
    assert_eq!(app.get_combo_text(), "Ap");
    control(&app, "z");
    control(&app, "z");
    assert_eq!(app.get_combo_text(), "Apricot");
}
