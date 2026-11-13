// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed window-local modal stack and focus publication.
use crate::ui;
use kit_core::{
    CloseResult, DismissReason, FocusDirection, FocusHandle, FocusId, FocusRegistry, FocusScopeId,
    OverlayId, OverlaySpec, OverlayStack,
};
use slint::{ComponentHandle, ModelRc, VecModel};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

const WINDOW: FocusScopeId = FocusScopeId(0);

/// Declare parents before descendants. Keys are application identities.
#[derive(Clone, Debug)]
pub struct DialogLayerSpec<L, T> {
    pub key: L,
    pub parent: Option<L>,
    pub initial_focus: T,
}
/// A stable Slint token and its typed identity. Declaration order is Tab order.
#[derive(Clone, Debug)]
pub struct DialogFocusSpec<L, T> {
    pub key: T,
    pub layer: Option<L>,
    pub token: u16,
    pub enabled: bool,
}
/// Busy work keeps one control eligible so Escape and Tab still reach the modal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogActivity<T> {
    Ready,
    Busy(T),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogHostError {
    DuplicateLayer,
    MissingParent,
    DuplicateTarget,
    DuplicateToken,
    MissingLayer,
    InvalidInitialFocus,
}
impl std::fmt::Display for DialogHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::DuplicateLayer => "dialog layer keys must be unique",
            Self::MissingParent => "declare a dialog parent before its descendants",
            Self::DuplicateTarget => "dialog focus keys must be unique",
            Self::DuplicateToken => "dialog focus tokens must be unique",
            Self::MissingLayer => "dialog focus target refers to an unknown layer",
            Self::InvalidInitialFocus => "initial focus must belong to its dialog layer",
        })
    }
}
impl std::error::Error for DialogHostError {}

/// A focus request names either one registered control or the modal layer's
/// own keyboard scope when it has no eligible controls.
#[derive(Clone, Debug)]
pub enum DialogFocusRequest<L> {
    Control(ui::FocusRequest),
    Layer(L),
}
fn control_focus(request: kit_core::FocusRequest) -> ui::FocusRequest {
    ui::FocusRequest {
        present: true,
        token: i32::try_from(request.target.id.0).expect("u16 target token"),
        generation: i32::try_from(request.target.generation)
            .expect("focus generation exhausted Slint int"),
        request: i32::try_from(request.serial).expect("focus serial exhausted Slint int"),
    }
}

/// Explicit setters; publication always sends eligibility, states, then focus.
pub struct DialogHostPorts<W, L> {
    pub slots: fn(&W, ModelRc<ui::FocusSlot>),
    pub state: fn(&W, L, ui::DialogState),
    pub focus: fn(&W, DialogFocusRequest<L>),
}
struct Layer<L, T> {
    spec: DialogLayerSpec<L, T>,
    id: OverlayId,
    scope: FocusScopeId,
    activity: DialogActivity<T>,
}
struct Target<L, T> {
    spec: DialogFocusSpec<L, T>,
    handle: FocusHandle,
}
struct Session<L, T> {
    layers: Vec<Layer<L, T>>,
    targets: BTreeMap<T, Target<L, T>>,
    tokens: BTreeMap<i32, T>,
    overlays: OverlayStack,
    focus: FocusRegistry,
    current: Option<FocusHandle>,
    observed: Option<T>,
}
impl<L: Copy + Eq, T: Copy + Ord> Session<L, T> {
    fn layer(&self, key: L) -> Option<&Layer<L, T>> {
        self.layers.iter().find(|layer| layer.spec.key == key)
    }
    fn layer_mut(&mut self, key: L) -> Option<&mut Layer<L, T>> {
        self.layers.iter_mut().find(|layer| layer.spec.key == key)
    }
    fn eligible(&self, key: T) -> bool {
        let target = &self.targets[&key];
        target.spec.enabled
            && match target.spec.layer {
                None => self.overlays.topmost_modal().is_none(),
                Some(key) => {
                    let layer = self.layer(key).expect("registered modal layer");
                    self.overlays
                        .topmost_modal()
                        .is_some_and(|top| top.id == layer.id)
                        && match layer.activity {
                            DialogActivity::Ready => true,
                            DialogActivity::Busy(keep) => target.spec.key == keep,
                        }
                }
            }
    }
    fn sync(&mut self) {
        for (key, target) in &self.targets {
            let eligible = self.eligible(*key);
            self.focus.set_eligible(target.handle, eligible);
        }
    }
    fn state(&self, key: L) -> ui::DialogState {
        let layer = self.layer(key).expect("registered modal layer");
        ui::DialogState {
            open: self.overlays.contains(layer.id),
            topmost: self
                .overlays
                .topmost()
                .is_some_and(|top| top.id == layer.id),
            busy: self.overlays.is_busy(layer.id),
        }
    }
    fn request(&mut self, handle: FocusHandle) -> Option<kit_core::FocusRequest> {
        let request = self.focus.request(handle)?;
        self.current = Some(request.target);
        Some(request)
    }
    fn recover_focus(&mut self) -> Option<DialogFocusRequest<L>> {
        let top = self.overlays.topmost_modal()?;
        let top_id = top.id;
        let scope = top.focus_scope.expect("registered modal scope");
        let key = self
            .layers
            .iter()
            .find(|layer| layer.id == top_id)
            .map(|layer| layer.spec.key)
            .expect("registered modal layer");
        if self
            .targets
            .iter()
            .any(|(key, target)| Some(target.handle) == self.current && self.eligible(*key))
        {
            return None;
        }
        match self.focus.restore(&[], scope) {
            Some(request) => {
                self.current = Some(request.target);
                Some(DialogFocusRequest::Control(control_focus(request)))
            }
            None => {
                self.current = None;
                self.observed = None;
                Some(DialogFocusRequest::Layer(key))
            }
        }
    }
    fn slots(&self) -> Vec<ui::FocusSlot> {
        let length = self.tokens.last_key_value().map_or(0, |(token, _)| {
            usize::try_from(*token).expect("u16 token") + 1
        });
        let mut slots = vec![ui::FocusSlot::default(); length];
        for (key, target) in &self.targets {
            slots[usize::from(target.spec.token)] = ui::FocusSlot {
                generation: i32::try_from(target.handle.generation)
                    .expect("focus generation exhausted Slint int"),
                eligible: self.eligible(*key),
            };
        }
        slots
    }
}

/// One modal host per window. Clones share state and retain only a weak window.
pub struct DialogHost<W: ComponentHandle + 'static, L, T> {
    window: slint::Weak<W>,
    ports: Rc<DialogHostPorts<W, L>>,
    session: Rc<RefCell<Session<L, T>>>,
}
impl<W: ComponentHandle + 'static, L, T> Clone for DialogHost<W, L, T> {
    fn clone(&self) -> Self {
        Self {
            window: self.window.clone(),
            ports: self.ports.clone(),
            session: self.session.clone(),
        }
    }
}
impl<W: ComponentHandle + 'static, L: Copy + Eq + 'static, T: Copy + Ord + 'static>
    DialogHost<W, L, T>
{
    pub fn new(
        window: &W,
        layers: impl IntoIterator<Item = DialogLayerSpec<L, T>>,
        targets: impl IntoIterator<Item = DialogFocusSpec<L, T>>,
        ports: DialogHostPorts<W, L>,
    ) -> Result<Self, DialogHostError> {
        let mut session = Session {
            layers: Vec::new(),
            targets: BTreeMap::new(),
            tokens: BTreeMap::new(),
            overlays: OverlayStack::new(),
            focus: FocusRegistry::new(),
            current: None,
            observed: None,
        };
        for spec in layers {
            if session.layer(spec.key).is_some() {
                return Err(DialogHostError::DuplicateLayer);
            }
            let parent = match spec.parent {
                None => WINDOW,
                Some(key) => {
                    session
                        .layer(key)
                        .ok_or(DialogHostError::MissingParent)?
                        .scope
                }
            };
            let number = u64::try_from(session.layers.len()).expect("layer count fits u64") + 1;
            let scope = FocusScopeId(number);
            session
                .focus
                .register_scope(scope, parent)
                .expect("validated scope topology");
            session.layers.push(Layer {
                spec,
                id: OverlayId(number),
                scope,
                activity: DialogActivity::Ready,
            });
        }
        for (order, spec) in targets.into_iter().enumerate() {
            if session.targets.contains_key(&spec.key) {
                return Err(DialogHostError::DuplicateTarget);
            }
            if session.tokens.contains_key(&i32::from(spec.token)) {
                return Err(DialogHostError::DuplicateToken);
            }
            let scope = match spec.layer {
                None => WINDOW,
                Some(key) => {
                    session
                        .layer(key)
                        .ok_or(DialogHostError::MissingLayer)?
                        .scope
                }
            };
            let handle = session.focus.register(
                FocusId(u64::from(spec.token)),
                scope,
                i64::try_from(order).expect("target order fits i64"),
                spec.enabled && spec.layer.is_none(),
            );
            session.tokens.insert(i32::from(spec.token), spec.key);
            session.targets.insert(spec.key, Target { spec, handle });
        }
        for layer in &session.layers {
            if session
                .targets
                .get(&layer.spec.initial_focus)
                .is_none_or(|target| target.spec.layer != Some(layer.spec.key))
            {
                return Err(DialogHostError::InvalidInitialFocus);
            }
        }
        let host = Self {
            window: window.as_weak(),
            ports: Rc::new(ports),
            session: Rc::new(RefCell::new(session)),
        };
        host.apply(|_| None);
        Ok(host)
    }
    fn apply(&self, change: impl FnOnce(&mut Session<L, T>) -> Option<kit_core::FocusRequest>) {
        let (slots, states, focus) = {
            let mut session = self.session.borrow_mut();
            let focus = match change(&mut session) {
                Some(request) => Some(DialogFocusRequest::Control(control_focus(request))),
                None => session.recover_focus(),
            };
            (
                session.slots(),
                session
                    .layers
                    .iter()
                    .map(|layer| (layer.spec.key, session.state(layer.spec.key)))
                    .collect::<Vec<_>>(),
                focus,
            )
        };
        if let Some(window) = self.window.upgrade() {
            (self.ports.slots)(&window, ModelRc::new(VecModel::from(slots)));
            for (key, state) in states {
                (self.ports.state)(&window, key, state);
            }
            if let Some(focus) = focus {
                (self.ports.focus)(&window, focus);
            }
        }
    }
    /// Parse a token once at the Slint callback boundary.
    pub fn target(&self, token: i32) -> Option<T> {
        self.session.borrow().tokens.get(&token).copied()
    }
    pub fn token(&self, target: T) -> Option<i32> {
        self.session
            .borrow()
            .targets
            .get(&target)
            .map(|target| i32::from(target.spec.token))
    }
    pub fn focused(&self) -> Option<T> {
        self.session.borrow().observed
    }
    pub fn eligible(&self, target: T) -> bool {
        let s = self.session.borrow();
        s.targets.contains_key(&target) && s.eligible(target)
    }
    pub fn state(&self, layer: L) -> Option<ui::DialogState> {
        let s = self.session.borrow();
        s.layer(layer).is_some().then(|| s.state(layer))
    }
    pub fn open(&self, key: L, opener: T) {
        self.apply(|s| {
            let (id, parent, scope, first) = {
                let layer = s.layer(key)?;
                (
                    layer.id,
                    layer.spec.parent,
                    layer.scope,
                    layer.spec.initial_focus,
                )
            };
            if !s.targets.contains_key(&opener) || !s.eligible(opener) {
                return None;
            }
            let mut spec = OverlaySpec::new(
                id,
                parent.map(|key| s.layer(key).expect("registered modal parent").id),
                true,
            );
            spec.focus_scope = Some(scope);
            spec.restore_focus = Some(s.targets[&opener].handle);
            s.overlays.push(spec).ok()?;
            s.focus.push_trap(scope).expect("registered modal scope");
            s.sync();
            s.request(s.targets[&first].handle).or_else(|| {
                let request = s.focus.restore(&[], scope)?;
                s.current = Some(request.target);
                Some(request)
            })
        });
    }
    pub fn close(&self, key: L, reason: DismissReason) {
        self.apply(|s| {
            let id = s.layer(key)?.id;
            match reason {
                DismissReason::Programmatic | DismissReason::HostClosed => {}
                DismissReason::Escape
                | DismissReason::OutsidePress
                | DismissReason::SystemBack
                | DismissReason::TriggerToggle => {
                    if s.overlays.topmost().is_none_or(|top| top.id != id) {
                        return None;
                    }
                }
            }
            match s.overlays.close(id, reason) {
                CloseResult::Closed(closed) => {
                    for spec in &closed {
                        if let Some(scope) = spec.focus_scope {
                            s.focus.pop_trap(scope);
                        }
                        let layer = s
                            .layers
                            .iter_mut()
                            .find(|layer| layer.id == spec.id)
                            .expect("registered modal layer");
                        layer.activity = DialogActivity::Ready;
                    }
                    s.sync();
                    let preferred = closed
                        .last()
                        .and_then(|spec| spec.restore_focus)
                        .into_iter()
                        .collect::<Vec<_>>();
                    let fallback = s
                        .overlays
                        .topmost_modal()
                        .and_then(|top| top.focus_scope)
                        .unwrap_or(WINDOW);
                    let request = s.focus.restore(&preferred, fallback)?;
                    s.current = Some(request.target);
                    Some(request)
                }
                CloseResult::Blocked | CloseResult::Empty => None,
            }
        });
    }
    pub fn cycle(&self, direction: FocusDirection) {
        self.apply(|s| {
            let request = s.focus.cycle(WINDOW, s.current, direction)?;
            s.current = Some(request.target);
            Some(request)
        });
    }
    pub fn set_activity(&self, key: L, activity: DialogActivity<T>) {
        self.apply(|s| {
            let layer = s.layer(key)?;
            if !s.overlays.contains(layer.id) {
                return None;
            }
            let busy = match activity {
                DialogActivity::Ready => false,
                DialogActivity::Busy(target) => {
                    if s.targets
                        .get(&target)
                        .is_none_or(|target| target.spec.layer != Some(key) || !target.spec.enabled)
                    {
                        return None;
                    }
                    true
                }
            };
            let id = layer.id;
            s.layer_mut(key).expect("checked layer").activity = activity;
            s.overlays.set_busy(id, busy);
            s.sync();
            match activity {
                DialogActivity::Ready => None,
                DialogActivity::Busy(target) => s.request(s.targets[&target].handle),
            }
        });
    }
    pub fn set_enabled(&self, key: T, enabled: bool) {
        self.apply(|s| {
            let target = s.targets.get_mut(&key)?;
            target.spec.enabled = enabled;
            let displaced = !enabled && s.current == Some(target.handle);
            s.sync();
            if displaced {
                let scope = s
                    .overlays
                    .topmost_modal()
                    .and_then(|top| top.focus_scope)
                    .unwrap_or(WINDOW);
                let request = s.focus.restore(&[], scope)?;
                s.current = Some(request.target);
                return Some(request);
            }
            None
        });
    }
    /// Forward `focus-observed(token, generation, focused)` from `KitFocusTarget`.
    /// A report for a generation other than the registered one is stale and ignored.
    pub fn observe_focus(&self, token: i32, generation: i32, focused: bool) {
        let mut s = self.session.borrow_mut();
        let Some(key) = s.tokens.get(&token).copied() else {
            return;
        };
        if u64::try_from(generation).ok() != Some(s.targets[&key].handle.generation) {
            return;
        }
        if focused && s.eligible(key) {
            s.observed = Some(key);
            s.current = Some(s.targets[&key].handle);
        } else if s.observed == Some(key) {
            s.observed = None;
        }
    }
}
