// SPDX-License-Identifier: GPL-3.0-or-later
use slint::{ComponentHandle, ModelRc, VecModel};
use slint_kit::{
    CloseResult, DismissReason, FocusDirection, FocusHandle, FocusId, FocusRegistry, FocusRequest,
    FocusScopeId, OverlayError, OverlayId, OverlaySpec, OverlayStack,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, time::Duration};

pub mod ui {
    pub use slint_kit::ui::*;
    slint::include_modules!();
}

/// The focus scope of the window content, outside every layer.
const WINDOW: FocusScopeId = FocusScopeId(0);

/// A modal layer inside the window: one overlay on the stack and one focus
/// scope that traps Tab while the layer is topmost. It is a surface composed
/// in this window, not a system window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layer {
    Archive,
    Confirm,
}

impl Layer {
    const ALL: [Self; 2] = [Self::Archive, Self::Confirm];
    fn overlay(self) -> OverlayId {
        match self {
            Self::Archive => OverlayId(1),
            Self::Confirm => OverlayId(2),
        }
    }
    fn scope(self) -> FocusScopeId {
        match self {
            Self::Archive => FocusScopeId(1),
            Self::Confirm => FocusScopeId(2),
        }
    }
    fn parent(self) -> Option<Self> {
        match self {
            Self::Archive => None,
            Self::Confirm => Some(Self::Archive),
        }
    }
    /// The control focused when the layer opens.
    fn first(self) -> Target {
        match self {
            Self::Archive => Target::Name,
            Self::Confirm => Target::Keep,
        }
    }
    fn from_overlay(id: OverlayId) -> Self {
        Self::ALL
            .into_iter()
            .find(|layer| layer.overlay() == id)
            .expect("every overlay on the stack is a layer")
    }
    fn from_view(layer: ui::DialogLayer) -> Self {
        match layer {
            ui::DialogLayer::Archive => Self::Archive,
            ui::DialogLayer::Confirm => Self::Confirm,
        }
    }
}

/// Every control the host can focus. The token is what the Slint side sees;
/// the registry handle adds the generation of the registration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Target {
    OpenFromToolbar,
    OpenFromCard,
    Name,
    Cancel,
    Archive,
    Keep,
    Confirm,
}

impl Target {
    /// Declaration order; it is also the Tab order inside each scope.
    const ALL: [Self; 7] = [
        Self::OpenFromToolbar,
        Self::OpenFromCard,
        Self::Name,
        Self::Cancel,
        Self::Archive,
        Self::Keep,
        Self::Confirm,
    ];
    fn token(self) -> u8 {
        match self {
            Self::OpenFromToolbar => 1,
            Self::OpenFromCard => 2,
            Self::Name => 3,
            Self::Cancel => 4,
            Self::Archive => 5,
            Self::Keep => 6,
            Self::Confirm => 7,
        }
    }
    fn from_token(token: i32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|target| i32::from(target.token()) == token)
    }
    /// The layer whose panel holds the control; `None` is the window content.
    fn layer(self) -> Option<Layer> {
        match self {
            Self::OpenFromToolbar | Self::OpenFromCard => None,
            Self::Name | Self::Cancel | Self::Archive => Some(Layer::Archive),
            Self::Keep | Self::Confirm => Some(Layer::Confirm),
        }
    }
    fn scope(self) -> FocusScopeId {
        self.layer().map_or(WINDOW, Layer::scope)
    }
}

/// The host side of the dialogs. The overlay stack decides which layer is
/// topmost, busy or closable; the focus registry decides which control may
/// take focus, how Tab cycles and where focus returns.
struct Session {
    overlays: OverlayStack,
    focus: FocusRegistry,
    handles: BTreeMap<Target, FocusHandle>,
    current: Option<FocusHandle>,
}

impl Session {
    fn new() -> Self {
        let mut focus = FocusRegistry::new();
        for layer in Layer::ALL {
            focus
                .register_scope(layer.scope(), layer.parent().map_or(WINDOW, Layer::scope))
                .expect("fresh scope");
        }
        let handles = Target::ALL
            .into_iter()
            .enumerate()
            .map(|(order, target)| {
                let order = i64::try_from(order).expect("seven targets");
                let id = FocusId(u64::from(target.token()));
                let eligible = target.layer().is_none();
                (target, focus.register(id, target.scope(), order, eligible))
            })
            .collect();
        Self {
            overlays: OverlayStack::new(),
            focus,
            handles,
            current: None,
        }
    }
    /// A control may act when its layer is topmost; a window control may act
    /// when no layer is open. The rule covers pointer, keyboard and
    /// accessibility activation alike, since Slint disables the control from
    /// the same flag. While a layer is busy only the control that started the
    /// work stays eligible: it shows the busy state and keeps focus inside the
    /// layer, so Escape and system Back still reach the surface, which
    /// consumes them.
    fn eligible(&self, target: Target) -> bool {
        match target.layer() {
            None => self.overlays.is_empty(),
            Some(layer) => {
                let id = layer.overlay();
                self.overlays.topmost().is_some_and(|top| top.id == id)
                    && (!self.overlays.is_busy(id) || target == Target::Confirm)
            }
        }
    }
    fn sync_eligibility(&mut self) {
        for target in Target::ALL {
            let eligible = self.eligible(target);
            self.focus.set_eligible(self.handles[&target], eligible);
        }
    }
    fn request(&mut self, handle: FocusHandle) -> Option<FocusRequest> {
        let request = self.focus.request(handle)?;
        self.current = Some(request.target);
        Some(request)
    }
    /// Opens a layer above its parent and remembers which control opened it.
    fn open(&mut self, layer: Layer, opener: Target) -> Option<FocusRequest> {
        let mut spec = OverlaySpec::new(layer.overlay(), layer.parent().map(Layer::overlay), true);
        spec.restore_focus = Some(self.handles[&opener]);
        match self.overlays.push(spec) {
            Ok(()) => {}
            // The layer is already open, or a nested layer was requested
            // before its parent: nothing changes.
            Err(OverlayError::DuplicateId | OverlayError::MissingParent) => return None,
        }
        self.focus
            .push_trap(layer.scope())
            .expect("registered scope");
        self.sync_eligibility();
        self.request(self.handles[&layer.first()])
    }
    /// Closes a layer and its descendants when the reason is allowed. Focus
    /// returns to the opener of the requested layer, or to the first eligible
    /// control of the layer that is now topmost.
    fn close(&mut self, layer: Layer, reason: DismissReason) -> Option<FocusRequest> {
        match self.overlays.close(layer.overlay(), reason) {
            CloseResult::Closed(closed) => {
                for spec in &closed {
                    self.focus.pop_trap(Layer::from_overlay(spec.id).scope());
                }
                self.sync_eligibility();
                let preferred: Vec<_> = closed
                    .last()
                    .and_then(|spec| spec.restore_focus)
                    .into_iter()
                    .collect();
                let fallback = self
                    .overlays
                    .topmost()
                    .map_or(WINDOW, |top| Layer::from_overlay(top.id).scope());
                let request = self.focus.restore(&preferred, fallback)?;
                self.current = Some(request.target);
                Some(request)
            }
            CloseResult::Blocked | CloseResult::Empty => None,
        }
    }
    /// Tab and Shift+Tab inside the topmost layer; the active trap bounds the cycle.
    fn cycle(&mut self, backward: bool) -> Option<FocusRequest> {
        let direction = if backward {
            FocusDirection::Backward
        } else {
            FocusDirection::Forward
        };
        let request = self.focus.cycle(WINDOW, self.current, direction)?;
        self.current = Some(request.target);
        Some(request)
    }
    /// A busy layer keeps Escape, system Back and the backdrop from closing
    /// it, and its controls stop taking focus until the work completes.
    fn set_busy(&mut self, layer: Layer, busy: bool) {
        // A layer closed in the meantime has nothing to update.
        self.overlays.set_busy(layer.overlay(), busy);
        self.sync_eligibility();
    }
    /// Pointer focus changes reported by Slint keep the Tab cycle anchored.
    fn focused(&mut self, token: i32, focused: bool) {
        if focused {
            self.current = Target::from_token(token).map(|target| self.handles[&target]);
        }
    }
    fn state(&self, layer: Layer) -> ui::DialogState {
        let id = layer.overlay();
        ui::DialogState {
            open: self.overlays.contains(id),
            topmost: self.overlays.topmost().is_some_and(|top| top.id == id),
            busy: self.overlays.is_busy(id),
        }
    }
    /// One slot per token, so Slint reads `slots[token]`; index 0 is unused.
    fn slots(&self) -> Vec<ui::FocusSlot> {
        let mut slots = vec![ui::FocusSlot::default(); Target::ALL.len() + 1];
        for target in Target::ALL {
            slots[usize::from(target.token())] = ui::FocusSlot {
                generation: i32::try_from(self.handles[&target].generation)
                    .expect("generations fit the Slint int"),
                eligible: self.eligible(target),
            };
        }
        slots
    }
}

fn focus_view(request: FocusRequest) -> ui::FocusRequest {
    ui::FocusRequest {
        present: true,
        token: i32::try_from(request.target.id.0).expect("tokens fit the Slint int"),
        generation: i32::try_from(request.target.generation)
            .expect("generations fit the Slint int"),
        request: i32::try_from(request.serial).expect("request serials fit the Slint int"),
    }
}

/// Applies a change to the session, then publishes the result. The borrow
/// ends before the window is updated, because applying a focus request calls
/// back into `focus-changed`.
fn apply(
    session: &Rc<RefCell<Session>>,
    app: &ui::DialogApp,
    change: impl FnOnce(&mut Session) -> Option<FocusRequest>,
) {
    let (request, slots, archive, confirm) = {
        let mut session = session.borrow_mut();
        let request = change(&mut session);
        (
            request,
            session.slots(),
            session.state(Layer::Archive),
            session.state(Layer::Confirm),
        )
    };
    app.set_slots(ModelRc::new(VecModel::from(slots)));
    app.set_archive(archive);
    app.set_confirm(confirm);
    if let Some(request) = request {
        app.set_focus_request(focus_view(request));
    }
}

fn status(language: ui::DialogLanguage, project: &str, archived: bool) -> String {
    match (language, archived) {
        (ui::DialogLanguage::English, false) => format!("{project} is active."),
        (ui::DialogLanguage::English, true) => format!("{project} is archived."),
        (ui::DialogLanguage::French, false) => format!("{project} est actif."),
        (ui::DialogLanguage::French, true) => format!("{project} est archivé."),
    }
}

pub fn create_app(language: ui::DialogLanguage) -> Result<ui::DialogApp, slint::PlatformError> {
    let app = ui::DialogApp::new()?;
    slint_kit::install(&app);
    app.set_language(language);
    app.set_status(status(language, &app.get_project_name(), false).into());
    let session = Rc::new(RefCell::new(Session::new()));
    apply(&session, &app, |_| None);
    app.on_open_requested({
        let (weak, session) = (app.as_weak(), session.clone());
        move |layer, token| {
            let Some(opener) = Target::from_token(token) else {
                return;
            };
            if let Some(app) = weak.upgrade() {
                apply(&session, &app, |s| s.open(Layer::from_view(layer), opener));
            }
        }
    });
    app.on_dismiss({
        let (weak, session) = (app.as_weak(), session.clone());
        move |layer, reason| {
            if let Some(app) = weak.upgrade() {
                let reason = DismissReason::from(reason);
                apply(&session, &app, |s| s.close(Layer::from_view(layer), reason));
            }
        }
    });
    app.on_confirm_requested({
        let (weak, session) = (app.as_weak(), session.clone());
        move || {
            let Some(app) = weak.upgrade() else {
                return;
            };
            apply(&session, &app, |s| {
                s.set_busy(Layer::Confirm, true);
                None
            });
            // Stands in for asynchronous work. When it completes, both layers
            // close and focus returns to the control that opened the first one.
            let (weak, session) = (weak.clone(), session.clone());
            slint::Timer::single_shot(Duration::from_millis(1200), move || {
                let Some(app) = weak.upgrade() else {
                    return;
                };
                apply(&session, &app, |s| {
                    s.close(Layer::Archive, DismissReason::Programmatic)
                });
                app.set_status(status(language, &app.get_project_name(), true).into());
            });
        }
    });
    app.on_focus_next({
        let (weak, session) = (app.as_weak(), session.clone());
        move |backward| {
            if let Some(app) = weak.upgrade() {
                apply(&session, &app, |s| s.cycle(backward));
            }
        }
    });
    app.on_focus_changed({
        let session = session.clone();
        move |token, focused| session.borrow_mut().focused(token, focused)
    });
    Ok(app)
}

pub fn main() -> Result<(), slint::PlatformError> {
    let language = if std::env::args().any(|arg| arg == "--fr") {
        ui::DialogLanguage::French
    } else {
        ui::DialogLanguage::English
    };
    create_app(language)?.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn handle(session: &Session, target: Target) -> FocusHandle {
        session.handles[&target]
    }

    #[test]
    fn each_opener_gets_focus_back_and_window_controls_are_inert_while_a_layer_is_open() {
        let mut session = Session::new();
        for opener in [Target::OpenFromCard, Target::OpenFromToolbar] {
            let first = session.open(Layer::Archive, opener).expect("layer opens");
            assert_eq!(first.target, handle(&session, Target::Name));
            assert!(session.state(Layer::Archive).open);
            assert!(session.state(Layer::Archive).topmost);
            assert!(!session.eligible(Target::OpenFromToolbar));
            assert!(!session.slots()[1].eligible);
            let background = handle(&session, Target::OpenFromToolbar);
            assert!(session.focus.request(background).is_none());
            assert!(
                session
                    .open(Layer::Archive, Target::OpenFromToolbar)
                    .is_none()
            );
            let restored = session
                .close(Layer::Archive, DismissReason::Escape)
                .expect("focus returns");
            assert_eq!(restored.target, handle(&session, opener));
            assert!(session.overlays.is_empty());
            assert!(session.eligible(Target::OpenFromToolbar));
        }
    }

    #[test]
    fn nested_layer_closes_one_level_at_a_time_and_each_level_restores_its_own_opener() {
        let mut session = Session::new();
        assert!(session.open(Layer::Confirm, Target::Archive).is_none());
        session
            .open(Layer::Archive, Target::OpenFromToolbar)
            .unwrap();
        session.focused(i32::from(Target::Archive.token()), true);
        let first = session.open(Layer::Confirm, Target::Archive).unwrap();
        assert_eq!(first.target, handle(&session, Target::Keep));
        assert!(!session.state(Layer::Archive).topmost);
        assert!(!session.eligible(Target::Name));
        assert_eq!(
            session.cycle(false).unwrap().target,
            handle(&session, Target::Confirm)
        );
        assert_eq!(
            session.cycle(false).unwrap().target,
            handle(&session, Target::Keep)
        );
        assert_eq!(
            session.cycle(true).unwrap().target,
            handle(&session, Target::Confirm)
        );
        let restored = session
            .close(Layer::Confirm, DismissReason::SystemBack)
            .unwrap();
        assert_eq!(restored.target, handle(&session, Target::Archive));
        assert!(session.state(Layer::Archive).topmost);
        assert!(!session.state(Layer::Confirm).open);
        let restored = session
            .close(Layer::Archive, DismissReason::Escape)
            .unwrap();
        assert_eq!(restored.target, handle(&session, Target::OpenFromToolbar));
        assert!(session.overlays.is_empty());
    }

    #[test]
    fn busy_layer_refuses_dismissal_and_completion_returns_focus_to_the_first_opener() {
        let mut session = Session::new();
        session.open(Layer::Archive, Target::OpenFromCard).unwrap();
        session.open(Layer::Confirm, Target::Archive).unwrap();
        session.set_busy(Layer::Confirm, true);
        assert!(session.state(Layer::Confirm).busy);
        assert!(!session.eligible(Target::Keep));
        assert!(
            session.eligible(Target::Confirm),
            "the busy action keeps focus"
        );
        assert_eq!(
            session.cycle(false).unwrap().target,
            handle(&session, Target::Confirm)
        );
        for reason in [
            DismissReason::Escape,
            DismissReason::SystemBack,
            DismissReason::OutsidePress,
        ] {
            assert!(session.close(Layer::Confirm, reason).is_none());
            assert!(session.state(Layer::Confirm).open);
        }
        let restored = session
            .close(Layer::Archive, DismissReason::Programmatic)
            .unwrap();
        assert_eq!(restored.target, handle(&session, Target::OpenFromCard));
        assert!(session.overlays.is_empty());
        assert!(session.slots()[1..=2].iter().all(|slot| slot.eligible));
        assert!(session.slots()[3..].iter().all(|slot| !slot.eligible));
    }
}
