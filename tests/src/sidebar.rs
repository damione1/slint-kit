// SPDX-License-Identifier: GPL-3.0-or-later
//! `KitSidebar`: sections, nested items, the current item, keyboard navigation
//! and the collapsed icon rail.
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use slint::platform::{Key, WindowEvent};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, rc::Rc};

const WORKSPACE: i32 = 1;
const DASHBOARD: i32 = 2;
const PROJECTS: i32 = 3;
const ACTIVE: i32 = 4;
const ARCHIVED: i32 = 5;
const SETTINGS: i32 = 6;
const BILLING: i32 = 7;

fn row(
    token: i32,
    label: &str,
    kind: SidebarRowKind,
    parent: Option<i32>,
    disabled: bool,
) -> SidebarRow {
    SidebarRow {
        item: OptionRow {
            token,
            generation: 1,
            label: label.into(),
            description: "".into(),
            disabled,
        },
        kind,
        parent: SelectionRef {
            present: parent.is_some(),
            token: parent.unwrap_or_default(),
        },
        leading_icon: Default::default(),
        badge: "".into(),
    }
}

fn rows() -> ModelRc<SidebarRow> {
    use SidebarRowKind::{Item, Section};
    ModelRc::new(VecModel::from(vec![
        row(WORKSPACE, "Workspace", Section, None, false),
        row(DASHBOARD, "Dashboard", Item, None, false),
        row(PROJECTS, "Projects", Item, None, false),
        row(ACTIVE, "Active", Item, Some(PROJECTS), false),
        row(ARCHIVED, "Archived", Item, Some(PROJECTS), false),
        row(SETTINGS, "Settings", Item, None, false),
        row(BILLING, "Billing", Item, None, true),
    ]))
}

struct Fixture {
    window: SidebarFixture,
    chosen: Rc<RefCell<Vec<i32>>>,
    expansions: Rc<RefCell<Vec<(i32, bool)>>>,
    rail_parents: Rc<RefCell<Vec<i32>>>,
}

fn fixture() -> Fixture {
    i_slint_backend_testing::init_no_event_loop();
    let window = SidebarFixture::new().unwrap();
    slint_kit::install(&window);
    window.set_rows(rows());
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let sink = chosen.clone();
    window.on_chosen(move |item| sink.borrow_mut().push(item.token));
    let expansions = Rc::new(RefCell::new(Vec::new()));
    let sink = expansions.clone();
    window.on_expansion_requested(move |item, open| sink.borrow_mut().push((item.token, open)));
    let rail_parents = Rc::new(RefCell::new(Vec::new()));
    let sink = rail_parents.clone();
    window.on_rail_parent_activated(move |item| sink.borrow_mut().push(item.token));
    window.show().unwrap();
    Fixture {
        window,
        chosen,
        expansions,
        rail_parents,
    }
}

fn press(window: &SidebarFixture, key: Key) {
    window
        .window()
        .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
    window
        .window()
        .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    slint::platform::update_timers_and_animations();
}

fn item(window: &SidebarFixture, label: &'static str) -> ElementHandle {
    ElementQuery::from_root(window)
        .match_inherits("KitOption")
        .match_predicate(move |e| e.accessible_label().as_deref() == Some(label))
        .find_first()
        .unwrap_or_else(|| panic!("missing sidebar item {label}"))
}

fn shown(window: &SidebarFixture) -> Vec<i32> {
    window
        .get_shown()
        .iter()
        .map(|entry| entry.row.item.token)
        .collect()
}

#[test]
fn sections_and_top_level_items_show_and_nested_items_wait_for_their_parent() {
    let Fixture { window, .. } = fixture();
    assert_eq!(
        shown(&window),
        [WORKSPACE, DASHBOARD, PROJECTS, SETTINGS, BILLING]
    );
    let projects = window.get_shown().row_data(2).unwrap();
    assert!(projects.expandable && !projects.expanded);
    assert_eq!(item(&window, "Projects").accessible_expanded(), Some(false));
    assert_eq!(
        item(&window, "Dashboard").accessible_expandable(),
        Some(false)
    );
    assert_eq!(
        item(&window, "Projects").accessible_expandable(),
        Some(true)
    );
}

#[test]
fn activating_an_item_makes_it_current_and_marks_it_for_assistive_technology() {
    let Fixture { window, chosen, .. } = fixture();
    item(&window, "Dashboard").invoke_accessible_default_action();
    assert_eq!(*chosen.borrow(), [DASHBOARD]);
    assert_eq!(
        window.get_current(),
        SelectionRef {
            present: true,
            token: DASHBOARD
        }
    );
    assert_eq!(
        item(&window, "Dashboard").accessible_item_selected(),
        Some(true)
    );
    assert_eq!(
        item(&window, "Settings").accessible_item_selected(),
        Some(false)
    );
    item(&window, "Settings").invoke_accessible_default_action();
    assert_eq!(
        item(&window, "Dashboard").accessible_item_selected(),
        Some(false)
    );
    assert_eq!(
        item(&window, "Settings").accessible_item_selected(),
        Some(true)
    );
}

#[test]
fn activating_a_parent_opens_and_closes_it_without_changing_the_current_item() {
    let Fixture {
        window,
        chosen,
        expansions,
        ..
    } = fixture();
    item(&window, "Dashboard").invoke_accessible_default_action();
    item(&window, "Projects").invoke_accessible_default_action();
    assert_eq!(
        shown(&window),
        [
            WORKSPACE, DASHBOARD, PROJECTS, ACTIVE, ARCHIVED, SETTINGS, BILLING
        ]
    );
    assert_eq!(item(&window, "Projects").accessible_expanded(), Some(true));
    assert_eq!(window.get_shown().row_data(3).unwrap().depth, 1);
    item(&window, "Projects").invoke_accessible_default_action();
    assert_eq!(
        shown(&window),
        [WORKSPACE, DASHBOARD, PROJECTS, SETTINGS, BILLING]
    );
    assert_eq!(*expansions.borrow(), [(PROJECTS, true), (PROJECTS, false)]);
    assert_eq!(*chosen.borrow(), [DASHBOARD]);
    assert_eq!(window.get_current().token, DASHBOARD);
}

#[test]
fn a_disabled_item_cannot_be_chosen() {
    let Fixture { window, chosen, .. } = fixture();
    item(&window, "Billing").invoke_accessible_default_action();
    assert!(chosen.borrow().is_empty());
    assert!(!window.get_current().present);
}

#[test]
fn arrows_move_between_rows_skipping_headings_and_enter_activates() {
    let Fixture { window, chosen, .. } = fixture();
    window.invoke_focus_sidebar();
    assert!(window.get_sidebar_focused());
    assert_eq!(
        window.get_state().active.token,
        DASHBOARD,
        "the heading is skipped"
    );
    press(&window, Key::Return);
    assert_eq!(*chosen.borrow(), [DASHBOARD]);
    press(&window, Key::DownArrow);
    assert_eq!(window.get_state().active.token, PROJECTS);
    press(&window, Key::UpArrow);
    press(&window, Key::End);
    press(&window, Key::Return);
    assert_eq!(
        *chosen.borrow(),
        [DASHBOARD, SETTINGS],
        "the disabled last row is skipped"
    );
    press(&window, Key::Home);
    press(&window, Key::Return);
    assert_eq!(*chosen.borrow(), [DASHBOARD, SETTINGS, DASHBOARD]);
}

#[test]
fn right_opens_a_parent_then_enters_it_and_left_closes_it() {
    let Fixture {
        window,
        chosen,
        expansions,
        ..
    } = fixture();
    window.invoke_focus_sidebar();
    press(&window, Key::DownArrow);
    assert_eq!(window.get_state().active.token, PROJECTS);
    press(&window, Key::RightArrow);
    assert_eq!(*expansions.borrow(), [(PROJECTS, true)]);
    assert_eq!(shown(&window).len(), 7);
    press(&window, Key::RightArrow);
    assert_eq!(window.get_state().active.token, ACTIVE);
    press(&window, Key::Return);
    assert_eq!(*chosen.borrow(), [ACTIVE]);
    press(&window, Key::UpArrow);
    press(&window, Key::LeftArrow);
    assert_eq!(*expansions.borrow(), [(PROJECTS, true), (PROJECTS, false)]);
    assert_eq!(shown(&window).len(), 5);
    assert_eq!(window.get_current().token, ACTIVE);
}

#[test]
fn controlled_ownership_waits_for_the_host() {
    let Fixture {
        window,
        chosen,
        expansions,
        ..
    } = fixture();
    window.set_ownership(Ownership::Controlled);
    item(&window, "Dashboard").invoke_accessible_default_action();
    assert_eq!(*chosen.borrow(), [DASHBOARD]);
    assert!(!window.get_current().present);
    item(&window, "Projects").invoke_accessible_default_action();
    assert_eq!(*expansions.borrow(), [(PROJECTS, true)]);
    assert_eq!(shown(&window).len(), 5);
    window.set_expanded(ModelRc::new(VecModel::from(vec![SelectionRef {
        present: true,
        token: PROJECTS,
    }])));
    assert_eq!(shown(&window).len(), 7);
}

#[test]
fn the_rail_keeps_top_level_items_and_asks_the_host_to_expand_a_parent() {
    let Fixture {
        window,
        chosen,
        expansions,
        rail_parents,
    } = fixture();
    let expanded_width = window.get_sidebar_width();
    item(&window, "Projects").invoke_accessible_default_action();
    window.set_collapsed(true);
    assert_eq!(shown(&window), [DASHBOARD, PROJECTS, SETTINGS, BILLING]);
    assert!(window.get_sidebar_width() < expanded_width);
    let projects = window.get_shown().row_data(1).unwrap();
    assert!(projects.expandable && !projects.expanded);
    item(&window, "Projects").invoke_accessible_default_action();
    assert_eq!(*rail_parents.borrow(), [PROJECTS]);
    assert_eq!(
        *expansions.borrow(),
        [(PROJECTS, true)],
        "the rail does not toggle"
    );
    item(&window, "Settings").invoke_accessible_default_action();
    assert_eq!(*chosen.borrow(), [SETTINGS]);
    window.set_collapsed(false);
    assert_eq!(window.get_sidebar_width(), expanded_width);
    assert_eq!(shown(&window).len(), 7, "the open parent is restored");
}

#[test]
fn an_inert_sidebar_ignores_activation() {
    let Fixture { window, chosen, .. } = fixture();
    window.set_interactive(false);
    item(&window, "Dashboard").invoke_accessible_default_action();
    assert!(chosen.borrow().is_empty());
}
