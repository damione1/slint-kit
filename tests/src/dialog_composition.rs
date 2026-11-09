// SPDX-License-Identifier: GPL-3.0-or-later
//! The documentation's dialog composition on the real window: several openers
//! with exact focus return, nested layers, Tab and Shift+Tab, Escape and
//! system Back, a busy layer and inert background actions. The host plumbing
//! mirrors docs/dialog-consumer/src/main.rs on `OverlayStack` and `FocusRegistry`.
use super::*;
use i_slint_backend_testing::{ElementHandle, ElementQuery};
use kit_core::{
    CloseResult, DismissReason, FocusDirection, FocusHandle, FocusId, FocusRegistry, FocusScopeId,
    OverlayId, OverlaySpec, OverlayStack,
};
use slint::{
    ComponentHandle, LogicalPosition, Model, ModelRc, VecModel,
    platform::{Key, PointerEventButton, WindowEvent, WindowEventDispatchResult},
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

const WINDOW: FocusScopeId = FocusScopeId(0);
/// Tokens declared in docs/dialog-consumer/ui/dialog.slint.
const TOOLBAR: i32 = 1;
const CARD: i32 = 2;
const NAME: i32 = 3;
const CANCEL: i32 = 4;
const ARCHIVE: i32 = 5;
const KEEP: i32 = 6;
const CONFIRM: i32 = 7;

fn overlay(layer: DialogLayer) -> OverlayId {
    match layer {
        DialogLayer::Archive => OverlayId(1),
        DialogLayer::Confirm => OverlayId(2),
    }
}
fn scope(layer: DialogLayer) -> FocusScopeId {
    match layer {
        DialogLayer::Archive => FocusScopeId(1),
        DialogLayer::Confirm => FocusScopeId(2),
    }
}
fn layer_of(id: OverlayId) -> DialogLayer {
    [DialogLayer::Archive, DialogLayer::Confirm]
        .into_iter()
        .find(|layer| overlay(*layer) == id)
        .expect("every overlay is a layer")
}
fn layer_of_token(token: i32) -> Option<DialogLayer> {
    match token {
        TOOLBAR | CARD => None,
        NAME | CANCEL | ARCHIVE => Some(DialogLayer::Archive),
        KEEP | CONFIRM => Some(DialogLayer::Confirm),
        other => panic!("unknown token {other}"),
    }
}
fn first(layer: DialogLayer) -> i32 {
    match layer {
        DialogLayer::Archive => NAME,
        DialogLayer::Confirm => KEEP,
    }
}

struct Host {
    overlays: OverlayStack,
    focus: FocusRegistry,
    handles: BTreeMap<i32, FocusHandle>,
    current: Option<FocusHandle>,
    /// The token the window last reported as focused, while it stays focused.
    focused: Option<i32>,
    dismissals: Vec<(DialogLayer, CloseReason)>,
    opens: u32,
}

impl Host {
    fn new() -> Self {
        let mut focus = FocusRegistry::new();
        focus
            .register_scope(scope(DialogLayer::Archive), WINDOW)
            .unwrap();
        focus
            .register_scope(scope(DialogLayer::Confirm), scope(DialogLayer::Archive))
            .unwrap();
        let handles = (TOOLBAR..=CONFIRM)
            .map(|token| {
                let layer = layer_of_token(token);
                let id = FocusId(u64::try_from(token).unwrap());
                let handle = focus.register(
                    id,
                    layer.map_or(WINDOW, scope),
                    i64::from(token),
                    layer.is_none(),
                );
                (token, handle)
            })
            .collect();
        Self {
            overlays: OverlayStack::new(),
            focus,
            handles,
            current: None,
            focused: None,
            dismissals: vec![],
            opens: 0,
        }
    }
    fn eligible(&self, token: i32) -> bool {
        match layer_of_token(token) {
            None => self.overlays.is_empty(),
            Some(layer) => {
                let id = overlay(layer);
                self.overlays.topmost().is_some_and(|top| top.id == id)
                    && (!self.overlays.is_busy(id) || token == CONFIRM)
            }
        }
    }
    fn sync(&mut self) {
        for token in TOOLBAR..=CONFIRM {
            let eligible = self.eligible(token);
            self.focus.set_eligible(self.handles[&token], eligible);
        }
    }
    fn request(&mut self, handle: FocusHandle) -> Option<kit_core::FocusRequest> {
        let request = self.focus.request(handle)?;
        self.current = Some(request.target);
        Some(request)
    }
    fn open(&mut self, layer: DialogLayer, opener: i32) -> Option<kit_core::FocusRequest> {
        self.opens += 1;
        let parent = match layer {
            DialogLayer::Archive => None,
            DialogLayer::Confirm => Some(overlay(DialogLayer::Archive)),
        };
        let mut spec = OverlaySpec::new(overlay(layer), parent, true);
        spec.restore_focus = Some(self.handles[&opener]);
        self.overlays.push(spec).ok()?;
        self.focus.push_trap(scope(layer)).unwrap();
        self.sync();
        self.request(self.handles[&first(layer)])
    }
    fn close(
        &mut self,
        layer: DialogLayer,
        reason: DismissReason,
    ) -> Option<kit_core::FocusRequest> {
        match self.overlays.close(overlay(layer), reason) {
            CloseResult::Closed(closed) => {
                for spec in &closed {
                    self.focus.pop_trap(scope(layer_of(spec.id)));
                }
                self.sync();
                let preferred: Vec<_> = closed
                    .last()
                    .and_then(|spec| spec.restore_focus)
                    .into_iter()
                    .collect();
                let fallback = self
                    .overlays
                    .topmost()
                    .map_or(WINDOW, |top| scope(layer_of(top.id)));
                let request = self.focus.restore(&preferred, fallback)?;
                self.current = Some(request.target);
                Some(request)
            }
            CloseResult::Blocked | CloseResult::Empty => None,
        }
    }
    fn cycle(&mut self, backward: bool) -> Option<kit_core::FocusRequest> {
        let direction = if backward {
            FocusDirection::Backward
        } else {
            FocusDirection::Forward
        };
        let request = self.focus.cycle(WINDOW, self.current, direction)?;
        self.current = Some(request.target);
        Some(request)
    }
    fn state(&self, layer: DialogLayer) -> DialogState {
        let id = overlay(layer);
        DialogState {
            open: self.overlays.contains(id),
            topmost: self.overlays.topmost().is_some_and(|top| top.id == id),
            busy: self.overlays.is_busy(id),
        }
    }
    fn slots(&self) -> Vec<FocusSlot> {
        let mut slots = vec![FocusSlot::default(); usize::try_from(CONFIRM).unwrap() + 1];
        for (token, handle) in &self.handles {
            slots[usize::try_from(*token).unwrap()] = FocusSlot {
                generation: i32::try_from(handle.generation).unwrap(),
                eligible: self.eligible(*token),
            };
        }
        slots
    }
}

struct Harness {
    window: DialogApp,
    host: Rc<RefCell<Host>>,
}

/// Publishes after the borrow ends: applying a focus request calls back into
/// `focus-changed`, which borrows the host again.
fn publish(host: &Rc<RefCell<Host>>, window: &DialogApp, request: Option<kit_core::FocusRequest>) {
    let (slots, archive, confirm) = {
        let host = host.borrow();
        (
            host.slots(),
            host.state(DialogLayer::Archive),
            host.state(DialogLayer::Confirm),
        )
    };
    window.set_slots(ModelRc::new(VecModel::from(slots)));
    window.set_archive(archive);
    window.set_confirm(confirm);
    if let Some(request) = request {
        window.set_focus_request(FocusRequest {
            present: true,
            token: i32::try_from(request.target.id.0).unwrap(),
            generation: i32::try_from(request.target.generation).unwrap(),
            request: i32::try_from(request.serial).unwrap(),
        });
    }
    slint::platform::update_timers_and_animations();
}

fn harness() -> Harness {
    i_slint_backend_testing::init_no_event_loop();
    let window = DialogApp::new().unwrap();
    window.show().unwrap();
    let host = Rc::new(RefCell::new(Host::new()));
    publish(&host, &window, None);
    window.on_open_requested({
        let (weak, host) = (window.as_weak(), host.clone());
        move |layer, token| {
            let window = weak.upgrade().unwrap();
            let request = host.borrow_mut().open(layer, token);
            publish(&host, &window, request);
        }
    });
    window.on_dismiss({
        let (weak, host) = (window.as_weak(), host.clone());
        move |layer, reason| {
            let window = weak.upgrade().unwrap();
            let request = {
                let mut host = host.borrow_mut();
                host.dismissals.push((layer, reason));
                host.close(layer, DismissReason::from(reason))
            };
            publish(&host, &window, request);
        }
    });
    window.on_confirm_requested({
        let (weak, host) = (window.as_weak(), host.clone());
        move || {
            let window = weak.upgrade().unwrap();
            {
                let mut host = host.borrow_mut();
                host.overlays.set_busy(overlay(DialogLayer::Confirm), true);
                host.sync();
            }
            publish(&host, &window, None);
        }
    });
    window.on_focus_next({
        let (weak, host) = (window.as_weak(), host.clone());
        move |backward| {
            let window = weak.upgrade().unwrap();
            let request = host.borrow_mut().cycle(backward);
            publish(&host, &window, request);
        }
    });
    window.on_focus_changed({
        let host = host.clone();
        move |token, focused| {
            let mut host = host.borrow_mut();
            if focused {
                host.focused = Some(token);
                host.current = host.handles.get(&token).copied();
            } else if host.focused == Some(token) {
                host.focused = None;
            }
        }
    });
    Harness { window, host }
}

impl Harness {
    fn focused(&self) -> Option<i32> {
        self.host.borrow().focused
    }
    fn dismissals(&self) -> Vec<(DialogLayer, CloseReason)> {
        self.host.borrow().dismissals.clone()
    }
    fn element(&self, label: &str) -> ElementHandle {
        let requested = label.to_owned();
        ElementQuery::from_root(&self.window)
            .match_predicate(move |element| {
                element.accessible_label().as_deref() == Some(requested.as_str())
            })
            .find_first()
            .unwrap_or_else(|| panic!("missing element {label}"))
    }
    /// Semantic activation, as an assistive technology performs it.
    fn activate(&self, label: &str) {
        self.element(label).invoke_accessible_default_action();
        slint::platform::update_timers_and_animations();
    }
    fn press(&self, key: Key) -> bool {
        let result = self
            .window
            .window()
            .dispatch_event_with_result(WindowEvent::KeyPressed { text: key.into() })
            .unwrap();
        self.window
            .window()
            .dispatch_event(WindowEvent::KeyReleased { text: key.into() });
        slint::platform::update_timers_and_animations();
        matches!(result, WindowEventDispatchResult::Accepted)
    }
    fn click(&self, x: f32, y: f32) {
        let position = LogicalPosition::new(x, y);
        let window = self.window.window();
        window.dispatch_event(WindowEvent::PointerMoved { position });
        window.dispatch_event(WindowEvent::PointerPressed {
            position,
            button: PointerEventButton::Left,
        });
        window.dispatch_event(WindowEvent::PointerReleased {
            position,
            button: PointerEventButton::Left,
        });
        slint::platform::update_timers_and_animations();
    }
}

#[test]
fn each_opener_gets_focus_back_and_tab_cycles_inside_the_layer() {
    let harness = harness();
    let window = &harness.window;
    for (label, opener) in [
        ("Archive from the project card", CARD),
        ("Archive from the toolbar", TOOLBAR),
    ] {
        harness.activate(label);
        assert!(window.get_archive().open && window.get_archive().topmost);
        assert_eq!(
            harness.focused(),
            Some(NAME),
            "the first control of the layer takes focus"
        );
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(CANCEL));
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(ARCHIVE));
        assert!(harness.press(Key::Tab));
        assert_eq!(harness.focused(), Some(NAME), "Tab wraps inside the layer");
        assert!(harness.press(Key::Backtab));
        assert_eq!(
            harness.focused(),
            Some(ARCHIVE),
            "Shift+Tab wraps backwards"
        );
        assert!(harness.press(Key::Escape));
        assert_eq!(
            harness.dismissals().last(),
            Some(&(DialogLayer::Archive, CloseReason::Escape))
        );
        assert!(!window.get_archive().open);
        assert_eq!(
            harness.focused(),
            Some(opener),
            "focus returns to the exact opener"
        );
    }
}

#[test]
fn nested_layer_owns_escape_back_and_tab_then_returns_focus_level_by_level() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the toolbar");
    harness.activate("Archive…");
    assert!(window.get_confirm().topmost);
    assert!(window.get_archive().open && !window.get_archive().topmost);
    assert_eq!(harness.focused(), Some(KEEP));
    assert!(harness.press(Key::Tab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Tab));
    assert_eq!(
        harness.focused(),
        Some(KEEP),
        "Tab stays in the nested layer"
    );
    assert!(harness.press(Key::Backtab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Escape));
    assert_eq!(
        harness.dismissals(),
        [(DialogLayer::Confirm, CloseReason::Escape)]
    );
    assert!(!window.get_confirm().open);
    assert!(window.get_archive().open && window.get_archive().topmost);
    assert_eq!(
        harness.focused(),
        Some(ARCHIVE),
        "the nested layer returns focus to its own opener"
    );
    harness.activate("Archive…");
    assert_eq!(harness.focused(), Some(KEEP));
    assert!(harness.press(Key::Back));
    assert_eq!(
        harness.dismissals().last(),
        Some(&(DialogLayer::Confirm, CloseReason::SystemBack))
    );
    assert_eq!(harness.focused(), Some(ARCHIVE));
    assert!(harness.press(Key::Back));
    assert_eq!(
        harness.dismissals().last(),
        Some(&(DialogLayer::Archive, CloseReason::SystemBack))
    );
    assert!(!window.get_archive().open);
    assert_eq!(harness.focused(), Some(TOOLBAR));
    assert!(harness.host.borrow().overlays.is_empty());
}

#[test]
fn busy_layer_consumes_escape_back_and_backdrop_until_the_host_completes() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the project card");
    harness.activate("Archive…");
    assert!(harness.press(Key::Tab));
    assert_eq!(harness.focused(), Some(CONFIRM));
    assert!(harness.press(Key::Return), "Enter starts the work");
    assert!(window.get_confirm().busy);
    assert_eq!(
        harness.focused(),
        Some(CONFIRM),
        "the busy action keeps focus inside the layer"
    );
    assert!(harness.press(Key::Escape), "Escape is consumed");
    assert!(harness.press(Key::Back), "system Back is consumed");
    harness.click(12., 12.);
    assert!(harness.dismissals().is_empty());
    assert!(window.get_confirm().open && window.get_archive().open);
    assert!(harness.press(Key::Tab));
    assert_eq!(
        harness.focused(),
        Some(CONFIRM),
        "disabled controls are skipped"
    );
    harness.activate("Keep project");
    assert!(
        harness.dismissals().is_empty(),
        "a disabled action does nothing"
    );
    // The host completes the work: both layers close at once.
    let request = harness
        .host
        .borrow_mut()
        .close(DialogLayer::Archive, DismissReason::Programmatic);
    publish(&harness.host, window, request);
    assert!(!window.get_confirm().open && !window.get_archive().open);
    assert_eq!(
        harness.focused(),
        Some(CARD),
        "focus returns to the first opener"
    );
}

#[test]
fn background_actions_stay_inert_while_a_layer_is_open() {
    let harness = harness();
    let window = &harness.window;
    harness.activate("Archive from the toolbar");
    assert_eq!(harness.host.borrow().opens, 1);
    harness.activate("Archive from the project card");
    assert_eq!(
        harness.host.borrow().opens,
        1,
        "semantic activation of a covered control does nothing"
    );
    assert_eq!(harness.focused(), Some(NAME));
    let covered = harness.host.borrow().handles[&CARD];
    assert!(
        harness.host.borrow_mut().focus.request(covered).is_none(),
        "a covered control cannot take focus"
    );
    assert!(
        !window
            .get_slots()
            .row_data(usize::try_from(CARD).unwrap())
            .unwrap()
            .eligible
    );
    harness.click(12., 12.);
    assert_eq!(
        harness.dismissals(),
        [(DialogLayer::Archive, CloseReason::OutsidePress)],
        "the backdrop dismisses; it is not an action"
    );
    assert_eq!(harness.focused(), Some(TOOLBAR));
    assert!(
        window
            .get_slots()
            .row_data(usize::try_from(CARD).unwrap())
            .unwrap()
            .eligible
    );
}
