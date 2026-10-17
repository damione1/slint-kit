// SPDX-License-Identifier: GPL-3.0-or-later
//! Native navigation examples with stable application keys and controlled views.
use super::ui;
use kit::{
    DismissReason, FocusDirection, FocusHandle, FocusId, FocusRegistry, FocusScopeId,
    NavigationStep, OverlayId, OverlaySpec, OverlayStack, Ownership, SelectController,
    SelectOption,
};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::{cell::RefCell, collections::HashSet, num::NonZeroUsize, rc::Rc, time::Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ItemKey {
    Workspace,
    Collections,
    Favorites,
    Archive,
    Overview,
    Activity,
    People,
    Share,
    Duplicate,
    Export,
    QuietPlaces,
    SmallDetails,
    NewPerspectives,
    Access,
    Updates,
    Privacy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct NoticeId(u64);
#[derive(Clone, Copy)]
struct PageIndex(usize);
struct Notice {
    id: NoticeId,
    title: &'static str,
    body: &'static str,
    read: bool,
}
const DRAWER_SCOPE: FocusScopeId = FocusScopeId(71);
const DRAWER_ID: OverlayId = OverlayId(71);

struct Examples {
    menu: SelectController<ItemKey>,
    toolbar: SelectController<ItemKey>,
    tabs: SelectController<ItemKey>,
    carousel: SelectController<ItemKey>,
    accordion: SelectController<ItemKey>,
    expanded: HashSet<ItemKey>,
    mode: ui::ExpansionMode,
    notices: Vec<Notice>,
    notice_generation: i32,
    notices_open: bool,
    page: PageIndex,
    page_count: NonZeroUsize,
    status: String,
    clock: Instant,
    focus: FocusRegistry,
    handles: [FocusHandle; 4],
    current_focus: Option<FocusHandle>,
    overlays: OverlayStack,
    requested_focus: ui::DrawerFocus,
    focus_serial: i32,
    models: [Rc<VecModel<ui::OptionRow>>; 4],
    disclosure_model: Rc<VecModel<ui::DisclosureRow>>,
    notices_model: Rc<VecModel<ui::NotificationRow>>,
}
fn option(key: ItemKey, title: &str, description: &str) -> SelectOption<ItemKey> {
    let mut row = SelectOption::new(key, title);
    row.description = description.into();
    row
}
fn controller(rows: Vec<SelectOption<ItemKey>>) -> SelectController<ItemKey> {
    let initial = rows.first().map(|row| row.key);
    let mut value = SelectController::new(rows, initial, Ownership::Local);
    value.open();
    value
}
impl Examples {
    fn new() -> Self {
        let mut archive = option(ItemKey::Archive, "Archive", "No archived collections yet");
        archive.disabled = true;
        let mut focus = FocusRegistry::new();
        focus
            .register_scope(DRAWER_SCOPE, FocusScopeId(0))
            .expect("demo drawer scope");
        let handles = [
            focus.register(FocusId(71), FocusScopeId(0), 0, true),
            focus.register(FocusId(72), DRAWER_SCOPE, 0, true),
            focus.register(FocusId(73), DRAWER_SCOPE, 1, true),
            focus.register(FocusId(74), DRAWER_SCOPE, 2, true),
        ];
        Self {
            menu: controller(vec![
                option(ItemKey::Workspace, "Workspace", "All your recent work"),
                option(
                    ItemKey::Collections,
                    "Collections",
                    "Stories grouped around an idea",
                ),
                option(
                    ItemKey::Favorites,
                    "Favorites",
                    "A few things worth returning to",
                ),
                archive,
            ]),
            toolbar: controller(vec![
                option(ItemKey::Share, "Share", "Invite someone to your collection"),
                option(
                    ItemKey::Duplicate,
                    "Duplicate",
                    "Start a new draft from this collection",
                ),
                option(ItemKey::Export, "Export", "Prepare a local copy"),
            ]),
            tabs: controller(vec![
                option(
                    ItemKey::Overview,
                    "Overview",
                    "A quiet place to collect ideas and turn them into something worth sharing.",
                ),
                option(
                    ItemKey::Activity,
                    "Activity",
                    "Maya reviewed the latest collection. Your next draft is ready whenever you are.",
                ),
                option(
                    ItemKey::People,
                    "People",
                    "You and Maya are shaping this workspace together.",
                ),
            ]),
            carousel: controller(vec![
                option(
                    ItemKey::QuietPlaces,
                    "Quiet places",
                    "A collection of small escapes and spaces that leave room to think.",
                ),
                option(
                    ItemKey::SmallDetails,
                    "Small details",
                    "Notice the shapes, colors, and everyday moments that make a story yours.",
                ),
                option(
                    ItemKey::NewPerspectives,
                    "New perspectives",
                    "Ideas from the people who see the same world a little differently.",
                ),
            ]),
            accordion: controller(vec![
                option(
                    ItemKey::Access,
                    "Who can see my collection?",
                    "A collection starts private. Share it with specific people when you are ready.",
                ),
                option(
                    ItemKey::Updates,
                    "How do updates work?",
                    "Save a draft as often as you like. Publishing makes the latest version available to your readers.",
                ),
                option(
                    ItemKey::Privacy,
                    "Can I change my mind?",
                    "You can stop sharing a collection at any time. Your original drafts stay in your workspace.",
                ),
            ]),
            expanded: HashSet::from([ItemKey::Access]),
            mode: ui::ExpansionMode::Single,
            notices: vec![
                Notice {
                    id: NoticeId(101),
                    title: "Maya reviewed your collection",
                    body: "Three thoughtful suggestions are waiting for you.",
                    read: false,
                },
                Notice {
                    id: NoticeId(102),
                    title: "Your collection is ready",
                    body: "The latest version is available to your team.",
                    read: false,
                },
                Notice {
                    id: NoticeId(103),
                    title: "Welcome to your workspace",
                    body: "A place for ideas, drafts, and the things that matter.",
                    read: true,
                },
            ],
            notice_generation: 1,
            notices_open: false,
            page: PageIndex(0),
            page_count: NonZeroUsize::new(8).expect("eight pages"),
            status: "Choose a workspace view.".into(),
            clock: Instant::now(),
            focus,
            handles,
            current_focus: None,
            overlays: OverlayStack::new(),
            requested_focus: ui::DrawerFocus::Close,
            focus_serial: 0,
            models: std::array::from_fn(|_| Rc::new(VecModel::default())),
            disclosure_model: Rc::new(VecModel::default()),
            notices_model: Rc::new(VecModel::default()),
        }
    }
    fn controller(&self, kind: ui::NavigationDemo) -> &SelectController<ItemKey> {
        match kind {
            ui::NavigationDemo::Menu => &self.menu,
            ui::NavigationDemo::Toolbar => &self.toolbar,
            ui::NavigationDemo::Tabs => &self.tabs,
            ui::NavigationDemo::Carousel => &self.carousel,
            ui::NavigationDemo::Accordion => &self.accordion,
        }
    }
    fn controller_mut(&mut self, kind: ui::NavigationDemo) -> &mut SelectController<ItemKey> {
        match kind {
            ui::NavigationDemo::Menu => &mut self.menu,
            ui::NavigationDemo::Toolbar => &mut self.toolbar,
            ui::NavigationDemo::Tabs => &mut self.tabs,
            ui::NavigationDemo::Carousel => &mut self.carousel,
            ui::NavigationDemo::Accordion => &mut self.accordion,
        }
    }
    fn key(&self, kind: ui::NavigationDemo, token: i32, generation: i32) -> Option<ItemKey> {
        let c = self.controller(kind);
        if u64::try_from(generation).ok() != Some(c.generation()) {
            return None;
        }
        c.options()
            .get(usize::try_from(token).ok()?)
            .filter(|row| !row.disabled)
            .map(|row| row.key)
    }
    fn navigate(&mut self, kind: ui::NavigationDemo, step: NavigationStep) {
        let c = self.controller_mut(kind);
        c.navigate(step, 5);
        match kind {
            ui::NavigationDemo::Carousel | ui::NavigationDemo::Tabs => {
                c.set_selected(c.state().active);
            }
            ui::NavigationDemo::Menu
            | ui::NavigationDemo::Toolbar
            | ui::NavigationDemo::Accordion => {}
        }
    }
    fn activate(&mut self, kind: ui::NavigationDemo, token: i32, generation: i32) {
        let Some(key) = self.key(kind, token, generation) else {
            return;
        };
        let c = self.controller_mut(kind);
        c.activate(&key);
        c.open();
        let label = c
            .options()
            .iter()
            .find(|row| row.key == key)
            .map(|row| row.label.clone())
            .unwrap_or_default();
        self.status = match kind {
            ui::NavigationDemo::Menu => format!("Viewing {label}."),
            ui::NavigationDemo::Toolbar => format!("{label} requested for this collection."),
            ui::NavigationDemo::Tabs => format!("Showing {label}."),
            ui::NavigationDemo::Carousel => format!("Opened {label}."),
            ui::NavigationDemo::Accordion => format!("{label} selected."),
        };
    }
    fn notice_index(&self, token: i32, generation: i32) -> Option<usize> {
        if generation != self.notice_generation {
            return None;
        }
        let id = NoticeId(u64::try_from(token).ok()?);
        self.notices.iter().position(|row| row.id == id)
    }
    fn open_drawer(&mut self) {
        if self.overlays.contains(DRAWER_ID) {
            return;
        }
        let mut spec = OverlaySpec::new(DRAWER_ID, None, true);
        spec.restore_focus = Some(self.handles[0]);
        self.overlays.push(spec).expect("unique demo drawer");
        self.focus
            .push_trap(DRAWER_SCOPE)
            .expect("registered scope");
        self.request(self.handles[1]);
    }
    fn close_drawer(&mut self, reason: DismissReason) {
        let result = self.overlays.close(DRAWER_ID, reason);
        if !self.overlays.contains(DRAWER_ID) {
            self.focus.pop_trap(DRAWER_SCOPE);
            if let Some(handle) = result.restore_focus() {
                self.request(handle);
            }
        }
    }
    fn request(&mut self, handle: FocusHandle) {
        if let Some(request) = self.focus.request(handle) {
            self.current_focus = Some(handle);
            self.requested_focus = focus_view(&self.handles, request.target);
            self.focus_serial = i32::try_from(request.serial).expect("bounded demo focus serial");
        }
    }
}
fn focus_index(value: ui::DrawerFocus) -> usize {
    match value {
        ui::DrawerFocus::Trigger => 0,
        ui::DrawerFocus::Close => 1,
        ui::DrawerFocus::Save => 2,
        ui::DrawerFocus::Cancel => 3,
    }
}
fn focus_view(handles: &[FocusHandle; 4], handle: FocusHandle) -> ui::DrawerFocus {
    if handle == handles[0] {
        ui::DrawerFocus::Trigger
    } else if handle == handles[1] {
        ui::DrawerFocus::Close
    } else if handle == handles[2] {
        ui::DrawerFocus::Save
    } else {
        ui::DrawerFocus::Cancel
    }
}
fn step(value: ui::NavigationStep) -> NavigationStep {
    match value {
        ui::NavigationStep::Next => NavigationStep::Next,
        ui::NavigationStep::Previous => NavigationStep::Previous,
        ui::NavigationStep::First => NavigationStep::First,
        ui::NavigationStep::Last => NavigationStep::Last,
        ui::NavigationStep::PageNext => NavigationStep::PageNext,
        ui::NavigationStep::PagePrevious => NavigationStep::PagePrevious,
    }
}
fn reason(value: ui::CloseReason) -> DismissReason {
    match value {
        ui::CloseReason::Escape => DismissReason::Escape,
        ui::CloseReason::OutsidePress => DismissReason::OutsidePress,
        ui::CloseReason::SystemBack => DismissReason::SystemBack,
        ui::CloseReason::TriggerToggle => DismissReason::TriggerToggle,
        ui::CloseReason::Programmatic => DismissReason::Programmatic,
        ui::CloseReason::HostClosed => DismissReason::HostClosed,
    }
}
fn rows(c: &SelectController<ItemKey>) -> Vec<ui::OptionRow> {
    c.options()
        .iter()
        .enumerate()
        .map(|(index, row)| ui::OptionRow {
            token: i32::try_from(index).expect("small demo"),
            generation: i32::try_from(c.generation()).expect("bounded generation"),
            label: row.label.as_str().into(),
            description: row.description.as_str().into(),
            disabled: row.disabled,
        })
        .collect()
}
fn view(c: &SelectController<ItemKey>) -> ui::SelectState {
    let token = |key: Option<ItemKey>| {
        key.and_then(|key| c.options().iter().position(|row| row.key == key))
            .and_then(|i| i32::try_from(i).ok())
    };
    let selection = |value: Option<i32>| ui::SelectionRef {
        present: value.is_some(),
        token: value.unwrap_or(-1),
    };
    let state = c.state();
    let active = token(state.active);
    let selected = token(state.selected);
    ui::SelectState {
        active: selection(active),
        selected: selection(selected),
        active_index: active.unwrap_or(-1),
        generation: i32::try_from(c.generation()).expect("bounded generation"),
        open: true,
        selection_availability: if selected.is_some() {
            ui::SelectionAvailability::Available
        } else if state.selected.is_some() {
            ui::SelectionAvailability::Unavailable
        } else {
            ui::SelectionAvailability::Absent
        },
    }
}
fn sync<T: Clone + PartialEq + 'static>(model: &Rc<VecModel<T>>, rows: Vec<T>) {
    while model.row_count() > rows.len() {
        model.remove(model.row_count() - 1);
    }
    for (index, row) in rows.into_iter().enumerate() {
        if index < model.row_count() {
            if model.row_data(index).as_ref() != Some(&row) {
                model.set_row_data(index, row);
            }
        } else {
            model.push(row);
        }
    }
}

fn publish(shared: &Rc<RefCell<Examples>>, window: &ui::Showcase) {
    // Drop the state borrow before any setter: focus can re-enter callbacks.
    let (
        models,
        values,
        states,
        disclosure_model,
        disclosures,
        notice_model,
        notices,
        mode,
        unread,
        notices_open,
        page,
        page_count,
        title,
        description,
        status,
        drawer_state,
        requested,
        serial,
    ) = {
        let s = shared.borrow();
        let kinds = [
            ui::NavigationDemo::Menu,
            ui::NavigationDemo::Toolbar,
            ui::NavigationDemo::Tabs,
            ui::NavigationDemo::Carousel,
        ];
        let values = kinds.map(|kind| rows(s.controller(kind)));
        let states = [
            view(&s.menu),
            view(&s.toolbar),
            view(&s.tabs),
            view(&s.carousel),
            view(&s.accordion),
        ];
        let disclosures = rows(&s.accordion)
            .into_iter()
            .enumerate()
            .map(|(i, item)| ui::DisclosureRow {
                item,
                expanded: s.expanded.contains(&s.accordion.options()[i].key),
            })
            .collect();
        let notices = s
            .notices
            .iter()
            .map(|row| ui::NotificationRow {
                item: ui::OptionRow {
                    token: i32::try_from(row.id.0).expect("bounded notice id"),
                    generation: s.notice_generation,
                    label: row.title.into(),
                    description: row.body.into(),
                    disabled: false,
                },
                read: row.read,
            })
            .collect();
        let current = s
            .tabs
            .options()
            .iter()
            .find(|row| Some(row.key) == s.tabs.state().selected)
            .expect("stable tab selection");
        (
            s.models.clone(),
            values,
            states,
            s.disclosure_model.clone(),
            disclosures,
            s.notices_model.clone(),
            notices,
            s.mode,
            i32::try_from(s.notices.iter().filter(|n| !n.read).count())
                .expect("bounded notice count"),
            s.notices_open,
            s.page.0,
            s.page_count.get(),
            current.label.clone(),
            current.description.clone(),
            s.status.clone(),
            ui::DialogState {
                open: s.overlays.contains(DRAWER_ID),
                topmost: s.overlays.topmost().is_some_and(|o| o.id == DRAWER_ID),
                busy: false,
            },
            s.requested_focus,
            s.focus_serial,
        )
    };
    for (index, value) in values.into_iter().enumerate() {
        sync(&models[index], value);
    }
    sync(&disclosure_model, disclosures);
    sync(&notice_model, notices);
    let g = window.global::<ui::NavigationExamples>();
    g.set_menu_rows(ModelRc::from(models[0].clone()));
    g.set_toolbar_rows(ModelRc::from(models[1].clone()));
    g.set_tabs_rows(ModelRc::from(models[2].clone()));
    g.set_carousel_rows(ModelRc::from(models[3].clone()));
    let [menu, toolbar, tabs, carousel, accordion] = states;
    g.set_menu_state(menu);
    g.set_toolbar_state(toolbar);
    g.set_tabs_state(tabs);
    g.set_carousel_state(carousel);
    g.set_accordion_state(accordion);
    g.set_accordion_rows(ModelRc::from(disclosure_model));
    g.set_notifications(ModelRc::from(notice_model));
    g.set_expansion_mode(mode);
    g.set_unread_count(unread);
    g.set_notifications_open(notices_open);
    g.set_page_index(i32::try_from(page).expect("bounded page"));
    g.set_page_count(i32::try_from(page_count).expect("bounded page count"));
    g.set_current_title(title.into());
    g.set_current_description(description.into());
    g.set_status(status.into());
    g.set_drawer_state(drawer_state);
    g.set_drawer_focus(requested);
    g.set_drawer_focus_request(serial);
}

pub fn install(window: &ui::Showcase) {
    let shared = Rc::new(RefCell::new(Examples::new()));
    let g = window.global::<ui::NavigationExamples>();
    g.set_breadcrumbs(ModelRc::new(VecModel::from(vec![
        ui::OptionRow {
            token: 0,
            generation: 1,
            label: "Workspace".into(),
            description: "".into(),
            disabled: false,
        },
        ui::OptionRow {
            token: 1,
            generation: 1,
            label: "Collections".into(),
            description: "".into(),
            disabled: false,
        },
        ui::OptionRow {
            token: 2,
            generation: 1,
            label: "Quiet places".into(),
            description: "".into(),
            disabled: false,
        },
    ])));
    macro_rules! bind {($method:ident,|$s:ident $(,$arg:ident)*| $body:block)=>{{let shared=shared.clone();let weak=window.as_weak();g.$method(move|$($arg),*|{{let mut $s=shared.borrow_mut();$body}if let Some(window)=weak.upgrade(){publish(&shared,&window);}});}};}
    bind!(on_navigate, |s, kind, value| {
        s.navigate(kind, step(value));
    });
    bind!(on_activate, |s, kind, token, generation| {
        s.activate(kind, token, generation);
    });
    bind!(on_typeahead, |s, kind, text| {
        let now = u64::try_from(s.clock.elapsed().as_millis()).unwrap_or(u64::MAX);
        s.controller_mut(kind).typeahead(text.as_str(), now);
    });
    bind!(on_expand, |s, token, generation, expanded, mode| {
        if let Some(key) = s.key(ui::NavigationDemo::Accordion, token, generation) {
            s.mode = mode;
            if expanded {
                if mode == ui::ExpansionMode::Single {
                    s.expanded.clear();
                }
                s.expanded.insert(key);
            } else {
                s.expanded.remove(&key);
            }
        }
    });
    bind!(on_mode_changed, |s, mode| {
        s.mode = mode;
        if mode == ui::ExpansionMode::Single && s.expanded.len() > 1 {
            let keep = s
                .accordion
                .options()
                .iter()
                .map(|o| o.key)
                .find(|k| s.expanded.contains(k));
            s.expanded.clear();
            if let Some(key) = keep {
                s.expanded.insert(key);
            }
        }
    });
    bind!(on_notice_open, |s| {
        s.notices_open = true;
    });
    bind!(on_notice_close, |s| {
        s.notices_open = false;
    });
    bind!(on_notice_read, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices[i].read = true;
        }
    });
    bind!(on_notice_dismiss, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices.remove(i);
            s.notice_generation = s
                .notice_generation
                .checked_add(1)
                .expect("bounded generation");
        }
    });
    bind!(on_notice_activate, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices[i].read = true;
            s.status = format!("Opened notification: {}.", s.notices[i].title);
            s.notices_open = false;
        }
    });
    bind!(on_breadcrumb_activate, |s, token, generation| {
        if generation == 1 {
            match token {
                0 => s.status = "Returned to your workspace.".into(),
                1 => s.status = "Showing all collections.".into(),
                _ => {}
            }
        }
    });
    bind!(on_page_navigate, |s, value| {
        let last = s.page_count.get() - 1;
        s.page = PageIndex(match step(value) {
            NavigationStep::Next => s.page.0.saturating_add(1).min(last),
            NavigationStep::Previous => s.page.0.saturating_sub(1),
            NavigationStep::First => 0,
            NavigationStep::Last => last,
            NavigationStep::PageNext => s.page.0.saturating_add(5).min(last),
            NavigationStep::PagePrevious => s.page.0.saturating_sub(5),
        });
    });
    bind!(on_drawer_open, |s| {
        s.open_drawer();
    });
    bind!(on_drawer_dismiss, |s, value| {
        s.close_drawer(reason(value));
    });
    bind!(on_drawer_next, |s, reverse| {
        let current = s.current_focus;
        if let Some(request) = s.focus.cycle(
            DRAWER_SCOPE,
            current,
            if reverse {
                FocusDirection::Backward
            } else {
                FocusDirection::Forward
            },
        ) {
            s.current_focus = Some(request.target);
            s.requested_focus = focus_view(&s.handles, request.target);
            s.focus_serial = i32::try_from(request.serial).expect("bounded focus serial");
        }
    });
    bind!(on_drawer_observed, |s, target| {
        s.current_focus = Some(s.handles[focus_index(target)]);
    });
    bind!(on_drawer_save, |s| {
        s.status = "Collection settings saved.".into();
        s.close_drawer(DismissReason::Programmatic);
    });
    publish(&shared, window);
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::platform::{Key, WindowEvent};

    #[test]
    fn navigation_retains_application_keys_and_rejects_stale_or_disabled_rows() {
        let mut examples = Examples::new();
        examples.navigate(ui::NavigationDemo::Menu, NavigationStep::Last);
        assert_eq!(examples.menu.state().active, Some(ItemKey::Favorites));
        let before = view(&examples.menu);
        examples.activate(
            ui::NavigationDemo::Menu,
            before.active.token,
            before.generation,
        );
        assert_eq!(examples.menu.state().selected, Some(ItemKey::Favorites));

        let mut reordered = examples.menu.options().to_vec();
        reordered.reverse();
        examples.menu.reconcile(reordered).unwrap();
        assert_eq!(examples.menu.state().selected, Some(ItemKey::Favorites));
        let current = view(&examples.menu);
        assert_ne!(before.generation, current.generation);
        // Token 2 now describes Collections, but belongs to an obsolete gesture.
        examples.activate(ui::NavigationDemo::Menu, 2, before.generation);
        assert_eq!(examples.menu.state().selected, Some(ItemKey::Favorites));
        // Archive is now row zero and remains unavailable even with a fresh token.
        examples.activate(ui::NavigationDemo::Menu, 0, current.generation);
        assert_eq!(examples.menu.state().selected, Some(ItemKey::Favorites));
        examples.activate(ui::NavigationDemo::Menu, 3, current.generation);
        assert_eq!(examples.menu.state().selected, Some(ItemKey::Workspace));
    }

    fn fixture() -> ui::Showcase {
        i_slint_backend_testing::init_no_event_loop();
        let window = ui::Showcase::new().unwrap();
        window.set_page(ui::ShowcasePage::Navigation);
        install(&window);
        window.show().unwrap();
        window
    }

    #[test]
    fn notification_callbacks_reject_previous_generation_after_removal() {
        let window = fixture();
        let global = window.global::<ui::NavigationExamples>();
        let first = global.get_notifications().row_data(0).unwrap();
        let second = global.get_notifications().row_data(1).unwrap();
        assert_eq!(global.get_unread_count(), 2);
        global.invoke_notice_dismiss(first.item.token, first.item.generation);
        assert_eq!(global.get_notifications().row_count(), 2);
        assert_eq!(global.get_unread_count(), 1);

        let current = global.get_notifications().row_data(0).unwrap();
        assert_eq!(current.item.token, second.item.token);
        assert_ne!(current.item.generation, second.item.generation);
        global.invoke_notice_read(second.item.token, second.item.generation);
        global.invoke_notice_dismiss(second.item.token, second.item.generation);
        global.invoke_notice_activate(first.item.token, first.item.generation);
        assert_eq!(global.get_unread_count(), 1);
        assert_eq!(global.get_notifications().row_count(), 2);

        global.invoke_notice_read(current.item.token, current.item.generation);
        assert_eq!(global.get_unread_count(), 0);
        assert!(global.get_notifications().row_data(0).unwrap().read);
    }

    fn key(window: &ui::Showcase, key: Key) {
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: key.into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
    }

    #[test]
    fn drawer_popup_traps_both_tab_directions_and_restores_after_escape() {
        let window = fixture();
        let global = window.global::<ui::NavigationExamples>();
        global.invoke_drawer_open();
        assert!(global.get_drawer_state().open);
        for target in [
            ui::DrawerFocus::Save,
            ui::DrawerFocus::Cancel,
            ui::DrawerFocus::Close,
        ] {
            key(&window, Key::Tab);
            assert_eq!(global.get_drawer_focus(), target);
        }
        key(&window, Key::Backtab);
        assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::Cancel);
        key(&window, Key::Escape);
        assert!(!global.get_drawer_state().open);
        assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::Trigger);
        // Restoration reached the real trigger: Enter can open the drawer again.
        key(&window, Key::Return);
        assert!(global.get_drawer_state().open);
    }
}
