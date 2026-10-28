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
    notice_focus: Option<NoticeId>,
    notice_focus_request: i32,
    page: PageIndex,
    page_count: NonZeroUsize,
    status: String,
    toolbar_result: String,
    carousel_detail: String,
    carousel_body: String,
    notice_detail: String,
    notice_body: String,
    breadcrumb_content: String,
    saved_settings: (String, bool, bool),
    drawer_result: String,
    drawer_error: String,
    drawer_launcher: ui::DrawerFocus,
    clock: Instant,
    focus: FocusRegistry,
    handles: [FocusHandle; 8],
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
            focus.register(FocusId(73), DRAWER_SCOPE, 4, true),
            focus.register(FocusId(74), DRAWER_SCOPE, 5, true),
            focus.register(FocusId(75), FocusScopeId(0), 1, true),
            focus.register(FocusId(76), DRAWER_SCOPE, 1, true),
            focus.register(FocusId(77), DRAWER_SCOPE, 2, true),
            focus.register(FocusId(78), DRAWER_SCOPE, 3, true),
        ];
        Self {
            menu: controller(vec![
                option(ItemKey::Workspace, "Workspace", "All your recent work"),
                option(
                    ItemKey::Collections,
                    "Collections",
                    "Incident reviews and access audits",
                ),
                option(
                    ItemKey::Favorites,
                    "Favorites",
                    "Pinned incident notes and audit reports",
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
                    "Allsafe has three incident notes ready for review.",
                ),
                option(
                    ItemKey::Activity,
                    "Activity",
                    "Darlene reviewed three findings. Elliot added the response checklist.",
                ),
                option(
                    ItemKey::People,
                    "People",
                    "Elliot Alderson and Darlene share this Allsafe workspace.",
                ),
            ]),
            carousel: controller(vec![
                option(
                    ItemKey::QuietPlaces,
                    "Incident notes",
                    "Review three incident findings and assign the next response with Elliot.",
                ),
                option(
                    ItemKey::SmallDetails,
                    "Access audit",
                    "Darlene checks team permissions and records the access changes.",
                ),
                option(
                    ItemKey::NewPerspectives,
                    "Server inventory",
                    "Track the Allsafe servers, scan results and maintenance owners.",
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
            mode: ui::ExpansionMode::Multiple,
            notices: vec![
                Notice {
                    id: NoticeId(101),
                    title: "Darlene reviewed three findings",
                    body: "Two access changes and one network finding need your review.",
                    read: false,
                },
                Notice {
                    id: NoticeId(102),
                    title: "Allsafe scan complete",
                    body: "The server inventory is ready for the Allsafe team.",
                    read: false,
                },
                Notice {
                    id: NoticeId(103),
                    title: "Allsafe workspace created",
                    body: "Incident notes, access reviews and the server inventory are ready.",
                    read: true,
                },
            ],
            notice_generation: 1,
            notices_open: false,
            notice_focus: None,
            notice_focus_request: 0,
            page: PageIndex(0),
            page_count: NonZeroUsize::new(8).expect("eight pages"),
            status: "Choose a workspace view.".into(),
            toolbar_result: "Choose an action to try it on Allsafe notes.".into(),
            carousel_detail: String::new(),
            carousel_body: String::new(),
            notice_detail: String::new(),
            notice_body: String::new(),
            breadcrumb_content: "Allsafe / Collections / Incident notes".into(),
            saved_settings: ("Allsafe notes".into(), false, true),
            drawer_result: "No settings saved yet.".into(),
            drawer_error: String::new(),
            drawer_launcher: ui::DrawerFocus::Trigger,
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
        match kind {
            ui::NavigationDemo::Toolbar => {
                self.toolbar_result = match key {
                    ItemKey::Share => {
                        "Demo share link: allsafe.example/collections/notes · Team access enabled"
                            .into()
                    }
                    ItemKey::Duplicate => {
                        "Allsafe notes — Copy · New draft added to this demo collection".into()
                    }
                    ItemKey::Export => {
                        "Export ready: allsafe-notes.txt · 3 notes · Demo preview".into()
                    }
                    ItemKey::Workspace
                    | ItemKey::Collections
                    | ItemKey::Favorites
                    | ItemKey::Archive
                    | ItemKey::Overview
                    | ItemKey::Activity
                    | ItemKey::People
                    | ItemKey::QuietPlaces
                    | ItemKey::SmallDetails
                    | ItemKey::NewPerspectives
                    | ItemKey::Access
                    | ItemKey::Updates
                    | ItemKey::Privacy => unreachable!("toolbar key"),
                };
            }
            ui::NavigationDemo::Carousel => {
                self.carousel_detail = label.clone();
                self.carousel_body = self
                    .carousel
                    .options()
                    .iter()
                    .find(|r| r.key == key)
                    .unwrap()
                    .description
                    .clone();
            }
            ui::NavigationDemo::Menu | ui::NavigationDemo::Tabs | ui::NavigationDemo::Accordion => {
            }
        }
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
    fn open_drawer(&mut self, launcher: ui::DrawerFocus) {
        if self.overlays.contains(DRAWER_ID) {
            return;
        }
        let mut spec = OverlaySpec::new(DRAWER_ID, None, true);
        self.drawer_error.clear();
        self.drawer_launcher = launcher;
        spec.restore_focus = Some(self.handles[focus_index(launcher)]);
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
        ui::DrawerFocus::CollapseTrigger => 4,
        ui::DrawerFocus::Name => 5,
        ui::DrawerFocus::Access => 6,
        ui::DrawerFocus::Updates => 7,
        ui::DrawerFocus::Close => 1,
        ui::DrawerFocus::Save => 2,
        ui::DrawerFocus::Cancel => 3,
    }
}
fn focus_view(handles: &[FocusHandle; 8], handle: FocusHandle) -> ui::DrawerFocus {
    [
        ui::DrawerFocus::Trigger,
        ui::DrawerFocus::Close,
        ui::DrawerFocus::Save,
        ui::DrawerFocus::Cancel,
        ui::DrawerFocus::CollapseTrigger,
        ui::DrawerFocus::Name,
        ui::DrawerFocus::Access,
        ui::DrawerFocus::Updates,
    ][handles
        .iter()
        .position(|h| *h == handle)
        .expect("registered drawer target")]
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
    {
        let s = shared.borrow();
        let menu = s
            .menu
            .options()
            .iter()
            .find(|r| Some(r.key) == s.menu.state().selected)
            .unwrap();
        g.set_current_title(format!("{} / {}", menu.label, title).into());
        g.set_current_description(format!("{} · {}", menu.description, description).into());
        g.set_toolbar_result(s.toolbar_result.as_str().into());
        g.set_carousel_detail(s.carousel_detail.as_str().into());
        g.set_carousel_body(s.carousel_body.as_str().into());
        g.set_notice_detail(s.notice_detail.as_str().into());
        g.set_notice_body(s.notice_body.as_str().into());
        g.set_notice_focus_token(
            s.notice_focus
                .map_or(-1, |id| i32::try_from(id.0).expect("bounded notice id")),
        );
        g.set_notice_focus_request(s.notice_focus_request);
        g.set_breadcrumb_content(s.breadcrumb_content.as_str().into());
        g.set_drawer_result(s.drawer_result.as_str().into());
        g.set_drawer_error(s.drawer_error.as_str().into());
        if !drawer_state.open {
            g.set_drawer_name(s.saved_settings.0.as_str().into());
            g.set_drawer_team(s.saved_settings.1);
            g.set_drawer_updates(s.saved_settings.2);
        }
        let names = [
            "Incident notes",
            "Network audit",
            "Access review",
            "Deployment plan",
            "Team handbook",
            "Recovery plan",
            "Security training",
            "Service inventory",
            "Release notes",
            "Threat model",
            "Device policy",
            "Archive index",
            "On call guide",
            "Audit report",
            "Patch schedule",
            "Test results",
            "Code review",
            "Build checklist",
            "Design notes",
            "Meeting notes",
            "Runbook",
            "Status report",
            "Backup plan",
            "Contact list",
        ];
        g.set_page_items(ModelRc::new(VecModel::from(
            names[s.page.0 * 3..s.page.0 * 3 + 3]
                .iter()
                .map(|n| slint::SharedString::from(*n))
                .collect::<Vec<_>>(),
        )));
    }
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

    g.set_status(status.into());
    g.set_drawer_state(drawer_state);
    g.set_drawer_focus(requested);
    g.set_drawer_focus_request(serial);
}

pub fn install(window: &ui::Showcase) {
    install_examples(window, Examples::new());
}
fn install_examples(window: &ui::Showcase, examples: Examples) {
    let shared = Rc::new(RefCell::new(examples));
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
            label: "Incident notes".into(),
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
            s.accordion.activate(&key);
            s.accordion.open();
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
                .state()
                .active
                .filter(|key| s.expanded.contains(key))
                .or_else(|| {
                    s.accordion
                        .options()
                        .iter()
                        .map(|o| o.key)
                        .find(|key| s.expanded.contains(key))
                });
            s.expanded.clear();
            if let Some(key) = keep {
                s.expanded.insert(key);
            }
        }
    });
    bind!(on_notice_reset, |s| {
        let initial = Examples::new();
        s.notices = initial.notices;
        s.notice_generation = s
            .notice_generation
            .checked_add(1)
            .expect("bounded generation");
        s.notice_detail.clear();
    });
    bind!(on_notice_open, |s| {
        s.notices_open = true;
    });
    bind!(on_notice_close, |s| {
        s.notices_open = false;
    });
    bind!(on_notice_back, |s| {
        s.notice_detail.clear();
        s.notice_body.clear();
    });
    bind!(on_carousel_preview_close, |s| {
        s.carousel_detail.clear();
        s.carousel_body.clear();
    });
    bind!(on_notice_read, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices[i].read = true;
            s.notice_focus = Some(s.notices[i].id);
            s.notice_focus_request = s
                .notice_focus_request
                .checked_add(1)
                .expect("bounded focus request");
        }
    });
    bind!(on_notice_dismiss, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices.remove(i);
            s.notice_focus = s
                .notices
                .get(i.min(s.notices.len().saturating_sub(1)))
                .map(|n| n.id);
            s.notice_focus_request = s
                .notice_focus_request
                .checked_add(1)
                .expect("bounded focus request");
            s.notice_generation = s
                .notice_generation
                .checked_add(1)
                .expect("bounded generation");
        }
    });
    bind!(on_notice_activate, |s, token, generation| {
        if let Some(i) = s.notice_index(token, generation) {
            s.notices[i].read = true;
            s.notice_detail = s.notices[i].title.into();
            s.notice_body = s.notices[i].body.into();
        }
    });
    bind!(on_breadcrumb_activate, |s, token, generation| {
        if generation == 1 {
            match token {
                0 => s.breadcrumb_content = "Allsafe · Collections, Favorites, Team".into(),
                1 => {
                    s.breadcrumb_content =
                        "Collections · Incident notes, Network audit, Access review".into()
                }
                2 => {
                    s.breadcrumb_content =
                        "Incident notes · Three notes shared with the Allsafe team".into()
                }
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
    bind!(on_drawer_open, |s, launcher| {
        s.open_drawer(launcher);
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
    bind!(on_drawer_save, |s, name, team, updates| {
        if name.trim().is_empty() {
            s.drawer_error = "Enter a collection name.".into();
            let name_target = s.handles[focus_index(ui::DrawerFocus::Name)];
            s.request(name_target);
        } else {
            s.saved_settings = (name.trim().to_owned(), team, updates);
            s.drawer_result = format!(
                "Saved: {} · {} · Updates {}",
                name.trim(),
                if team { "Team access" } else { "Private" },
                if updates { "on" } else { "off" }
            );
            s.close_drawer(DismissReason::Programmatic);
        }
    });
    publish(&shared, window);
}

#[cfg(test)]
mod tests {
    use super::*;
    use i_slint_backend_testing::{ElementHandle, ElementQuery};
    use slint::platform::{Key, PointerEventButton, WindowEvent};
    use std::time::Duration;

    fn settle(milliseconds: u64) {
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(milliseconds));
        slint::platform::update_timers_and_animations();
    }
    fn button(window: &ui::Showcase, label: &str) -> ElementHandle {
        let expected = label.to_owned();
        ElementQuery::from_root(window)
            .match_inherits("KitPressable")
            .match_predicate(move |element| {
                element.accessible_label().as_deref() == Some(expected.as_str())
            })
            .find_first()
            .unwrap_or_else(|| panic!("visible button {label}"))
    }
    fn reveal(window: &ui::Showcase, target: ui::PreviewTarget) {
        let weak = window.as_weak();
        let preview = window.global::<ui::PreviewExamples>();
        preview.on_reveal(move |top, _height| {
            if let Some(window) = weak.upgrade() {
                window.invoke_position_content(top);
            }
        });
        let _ = ElementQuery::from_root(window)
            .match_inherits("KitPressable")
            .find_first()
            .unwrap()
            .size();
        preview.set_target(target);
        preview.set_request(preview.get_request() + 1);
        settle(1);
    }

    #[test]
    fn closed_previews_stay_closed_across_unrelated_controller_publications() {
        let window = fixture();
        let g = window.global::<ui::NavigationExamples>();
        let carousel = g.get_carousel_state();
        g.invoke_activate(
            ui::NavigationDemo::Carousel,
            carousel.active.token,
            carousel.generation,
        );
        assert!(!g.get_carousel_detail().is_empty());
        g.invoke_carousel_preview_close();
        assert!(g.get_carousel_detail().is_empty());
        assert!(g.get_carousel_body().is_empty());
        g.invoke_notice_open();
        g.invoke_notice_close();
        let toolbar = g.get_toolbar_state();
        g.invoke_activate(
            ui::NavigationDemo::Toolbar,
            toolbar.active.token,
            toolbar.generation,
        );
        assert!(g.get_carousel_detail().is_empty());
        let first = g.get_notifications().row_data(0).unwrap();
        g.invoke_notice_activate(first.item.token, first.item.generation);
        assert!(!g.get_notice_detail().is_empty());
        g.invoke_notice_back();
        g.invoke_notice_read(first.item.token, first.item.generation);
        g.invoke_notice_dismiss(first.item.token, first.item.generation);
        g.invoke_notice_close();
        g.invoke_notice_open();
        assert!(g.get_notice_detail().is_empty());
        assert!(g.get_notice_body().is_empty());
    }

    #[test]
    fn notification_session_keeps_frame_actions_and_page_context_stable() {
        i_slint_backend_testing::init_no_event_loop();
        for (width, height, enlarged, trigger_y) in [
            (1180., 900., false, 180.),
            (1180., 900., false, 618.),
            (960., 740., true, 450.),
        ] {
            let window = ui::Showcase::new().unwrap();
            window.set_page(ui::ShowcasePage::Navigation);
            let mut examples = Examples::new();
            examples.notices[0].title = "Darlene reviewed the Allsafe access changes and network findings before the next incident response";
            install_examples(&window, examples);
            window.show().unwrap();
            window
                .window()
                .set_size(slint::LogicalSize::new(width, height));
            window.set_enlarged(enlarged);
            reveal(&window, ui::PreviewTarget::Notifications);
            let trigger = ElementQuery::from_root(&window)
                .match_inherits("KitNotificationButton")
                .find_first()
                .unwrap();
            let section = ElementQuery::from_root(&window)
                .match_inherits("ExampleSection")
                .match_predicate(|element| {
                    element
                        .query_descendants()
                        .match_inherits("KitNotificationButton")
                        .find_first()
                        .is_some()
                })
                .find_first()
                .unwrap();
            window.invoke_position_content(
                section.absolute_position().y - (trigger_y - trigger.absolute_position().y),
            );
            let trigger_before = trigger.absolute_position();
            let section_before = section.absolute_position();
            let g = window.global::<ui::NavigationExamples>();
            trigger.mock_single_click(PointerEventButton::Left);
            settle(1);
            assert!(g.get_notifications_open());
            let frame = g.get_notice_popup_bounds();
            assert!(frame.height > 100.);
            assert!(frame.x >= 15. && frame.y >= 15.);
            assert!(frame.x + frame.width <= width + 1. && frame.y + frame.height <= height + 1.);
            let close = button(&window, "Close").absolute_position();
            let inbox = ElementQuery::from_root(&window)
                .match_inherits("KitNotificationCenter")
                .find_first()
                .unwrap();
            let items = inbox
                .query_descendants()
                .match_inherits("NotificationItem")
                .find_all();
            assert!(!items.is_empty(), "visible notifications expose list items");
            for item in items {
                assert_eq!(
                    item.accessible_role(),
                    Some(i_slint_backend_testing::AccessibleRole::ListItem)
                );
                assert!(
                    item.accessible_label()
                        .is_some_and(|label| !label.is_empty())
                );
            }
            inbox.scroll(0., -10000.);
            settle(1);
            let last_action = button(&window, "Dismiss");
            assert!(
                last_action.absolute_position().y + last_action.size().height <= frame.height - 15.,
                "last row action is scrollable into the finite viewport"
            );
            inbox.scroll(0., 10000.);
            settle(1);
            let dismiss = button(&window, "Dismiss").absolute_position();
            button(&window, "Mark read").mock_single_click(PointerEventButton::Left);
            settle(1);
            let after_read = button(&window, "Dismiss").absolute_position();
            assert!(
                (after_read.x - dismiss.x).abs() <= 1. && (after_read.y - dismiss.y).abs() <= 1.
            );
            assert_eq!(g.get_notice_popup_bounds(), frame);
            let dismiss_button = button(&window, "Dismiss");
            dismiss_button.mock_single_click(PointerEventButton::Left);
            assert_eq!(
                g.get_notifications().row_count(),
                3,
                "row stays mounted while fading"
            );
            assert_eq!(dismiss_button.accessible_enabled(), Some(false));
            settle(75);
            assert_eq!(g.get_notifications().row_count(), 3);
            settle(100);
            assert_eq!(g.get_notifications().row_count(), 2);
            while let Some(row) = g.get_notifications().row_data(0) {
                assert_eq!(g.get_notice_popup_bounds(), frame);
                g.invoke_notice_dismiss(row.item.token, row.item.generation);
                settle(1);
                assert_eq!(button(&window, "Close").absolute_position(), close);
                assert_eq!(trigger.absolute_position(), trigger_before);
                assert_eq!(section.absolute_position(), section_before);
            }
            assert_eq!(g.get_notice_popup_bounds(), frame);
            assert_eq!(g.get_notice_focus_token(), -1);
            key(&window, Key::Escape);
            assert!(!g.get_notifications_open());
            g.invoke_notice_reset();
            trigger.mock_single_click(PointerEventButton::Left);
            settle(1);
            let reopened_frame = g.get_notice_popup_bounds();
            button(&window, "Open preview").mock_single_click(PointerEventButton::Left);
            settle(160);
            assert!(!g.get_notice_detail().is_empty());
            assert_eq!(g.get_notice_popup_bounds(), reopened_frame);
            button(&window, "Back to messages").mock_single_click(PointerEventButton::Left);
            settle(160);
            assert!(g.get_notice_detail().is_empty());
            assert_eq!(g.get_notice_popup_bounds(), reopened_frame);
            button(&window, "Close").mock_single_click(PointerEventButton::Left);
            assert!(!g.get_notifications_open());
            assert_eq!(trigger.absolute_position(), trigger_before);
            assert_eq!(section.absolute_position(), section_before);
            trigger.mock_single_click(PointerEventButton::Left);
            settle(1);
            window
                .window()
                .set_size(slint::LogicalSize::new(960., 740.));
            let _ = button(&window, "Close").size();
            settle(1);
            let resized = g.get_notice_popup_bounds();
            assert!(resized.x >= 15. && resized.y >= 15.);
            assert!(resized.x + resized.width <= 961. && resized.y + resized.height <= 741.);
            let outside = slint::LogicalPosition::new(5., 735.);
            window.window().dispatch_event(WindowEvent::PointerPressed {
                position: outside,
                button: PointerEventButton::Left,
            });
            window
                .window()
                .dispatch_event(WindowEvent::PointerReleased {
                    position: outside,
                    button: PointerEventButton::Left,
                });
            settle(1);
            assert!(!g.get_notifications_open());
        }
    }

    #[test]
    fn notification_exit_respects_reduced_motion_and_cancels_recycled_identity() {
        let window = fixture();
        reveal(&window, ui::PreviewTarget::Notifications);
        let g = window.global::<ui::NavigationExamples>();
        window.set_reduced_motion(true);
        g.invoke_notice_open();
        settle(1);
        button(&window, "Dismiss").mock_single_click(PointerEventButton::Left);
        settle(1);
        assert_eq!(g.get_notifications().row_count(), 2);
        g.invoke_notice_reset();
        window.set_reduced_motion(false);
        let old_generation = g.get_notifications().row_data(0).unwrap().item.generation;
        button(&window, "Dismiss").mock_single_click(PointerEventButton::Left);
        g.invoke_notice_reset();
        settle(200);
        assert_eq!(g.get_notifications().row_count(), 3);
        assert_ne!(
            g.get_notifications().row_data(0).unwrap().item.generation,
            old_generation
        );
        assert_eq!(button(&window, "Dismiss").accessible_enabled(), Some(true));
    }

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
        navigation_window()
    }
    fn navigation_window() -> ui::Showcase {
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
        global.invoke_drawer_open(ui::DrawerFocus::Trigger);
        assert!(global.get_drawer_state().open);
        for target in [
            ui::DrawerFocus::Name,
            ui::DrawerFocus::Access,
            ui::DrawerFocus::Updates,
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
    #[test]
    fn drawer_covers_window_and_restores_the_exact_secondary_launcher() {
        use i_slint_backend_testing::ElementQuery;
        let window = fixture();
        let global = window.global::<ui::NavigationExamples>();
        global.set_collapse_open(true);
        global.invoke_drawer_open(ui::DrawerFocus::CollapseTrigger);
        let drawer = ElementQuery::from_root(&window)
            .match_inherits("KitDrawer")
            .find_first()
            .unwrap();
        let panel = drawer
            .query_descendants()
            .match_inherits("KitDialogPanel")
            .find_first()
            .unwrap();
        let size = window.window().size();
        assert_eq!(
            drawer.absolute_position(),
            slint::LogicalPosition::new(0., 0.)
        );
        assert!((drawer.size().width - size.width as f32).abs() <= 1.);
        assert!((drawer.size().height - size.height as f32).abs() <= 1.);
        assert!(
            (panel.absolute_position().x + panel.size().width - drawer.size().width).abs() <= 1.
        );
        global.invoke_drawer_save("Revised notes".into(), true, false);
        assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::CollapseTrigger);
        assert!(global.get_drawer_result().contains("Revised notes"));
        assert!(global.get_drawer_team());
        global.invoke_drawer_open(ui::DrawerFocus::Trigger);
        global.set_drawer_name("Discard this draft".into());
        key(&window, Key::Escape);
        assert_eq!(global.get_drawer_name(), "Revised notes");
        assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::Trigger);
    }
    #[test]
    fn drawer_empty_and_whitespace_names_keep_save_reachable_and_focus_the_error() {
        let window = fixture();
        let global = window.global::<ui::NavigationExamples>();
        global.invoke_drawer_open(ui::DrawerFocus::Trigger);
        let field = i_slint_backend_testing::ElementQuery::from_root(&window)
            .match_type_name("TextInput")
            .match_predicate(|e| e.accessible_label().as_deref() == Some("Collection name"))
            .find_first()
            .expect("visible native drawer editor");
        let _ = field.size();
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(1));
        slint::platform::update_timers_and_animations();
        key(&window, Key::Tab);
        assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::Name);
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(1));
        slint::platform::update_timers_and_animations();
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: "X".into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: "X".into() });
        assert!(
            global.get_drawer_name().contains('X'),
            "native input must really receive keyboard focus"
        );
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: Key::Control.into(),
        });
        window
            .window()
            .dispatch_event(WindowEvent::KeyPressed { text: "a".into() });
        window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: "a".into() });
        window.window().dispatch_event(WindowEvent::KeyReleased {
            text: Key::Control.into(),
        });
        key(&window, Key::Backspace);
        assert!(
            global.get_drawer_name().is_empty(),
            "native drawer draft after Backspace: {:?}",
            global.get_drawer_name()
        );
        for spaces in [false, true] {
            if spaces {
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyPressed { text: "   ".into() });
                window
                    .window()
                    .dispatch_event(WindowEvent::KeyReleased { text: "   ".into() });
                assert_eq!(global.get_drawer_name(), "   ");
            }
            for target in [
                ui::DrawerFocus::Access,
                ui::DrawerFocus::Updates,
                ui::DrawerFocus::Save,
            ] {
                key(&window, Key::Tab);
                assert_eq!(global.get_drawer_focus(), target);
            }
            key(&window, Key::Return);
            assert!(global.get_drawer_state().open);
            assert_eq!(global.get_drawer_focus(), ui::DrawerFocus::Name);
            assert!(!global.get_drawer_error().is_empty());
        }
    }
    #[test]
    fn carousel_controls_fit_inside_its_surface_at_double_text() {
        use i_slint_backend_testing::ElementQuery;
        let window = fixture();
        window.set_enlarged(true);
        let weak = window.as_weak();
        let preview = window.global::<ui::PreviewExamples>();
        preview.on_reveal(move |top, _height| {
            if let Some(window) = weak.upgrade() {
                window.invoke_position_content(top);
            }
        });
        let _ = ElementQuery::from_root(&window)
            .match_inherits("KitPressable")
            .find_first()
            .unwrap()
            .size();
        preview.set_target(ui::PreviewTarget::Carousel);
        preview.set_request(1);
        i_slint_backend_testing::mock_elapsed_time(std::time::Duration::from_millis(1));
        slint::platform::update_timers_and_animations();
        let carousel = ElementQuery::from_root(&window)
            .match_inherits("KitCarousel")
            .find_first()
            .unwrap();
        let controls = carousel
            .query_descendants()
            .match_inherits("KitButton")
            .find_all();
        assert_eq!(
            controls.len(),
            2,
            "both visible carousel navigation buttons"
        );
        for control in controls {
            assert!(
                control.absolute_position().y + control.size().height
                    <= carousel.absolute_position().y + carousel.size().height - 15.,
                "carousel footer must retain its inset"
            );
        }
    }
}
