// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::ui;
use kit_core::{
    FieldError, FormCoordinator, FormIntent, HtmlFormValidation, HtmlValidity, SubmissionState,
    SubmissionTicket,
};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

/// Snapshot of one registered stable field. Validation stays in its own control.
#[derive(Clone)]
pub struct FormFieldSnapshot {
    pub will_validate: bool,
    pub validity: HtmlValidity,
    pub dirty: bool,
    pub message: SharedString,
    pub target: ui::FocusRequest,
}
#[derive(Clone)]
pub struct FormProjection {
    pub valid: bool,
    pub dirty: bool,
    pub submitting: bool,
    pub errors: ModelRc<ui::FormErrorRow>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormRegistrationError {
    DuplicateKey,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormSubmitError<K> {
    InvalidFields { first: K, remaining: Vec<K> },
    Busy,
    WindowClosed,
    InvalidValue { first: K, remaining: Vec<K> },
}

/// An admitted typed payload and its response ticket.
pub struct FormSubmission<P> {
    pub ticket: SubmissionTicket,
    pub payload: P,
}

/// Nonempty typed field errors returned at the payload parsing boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormErrors<K> {
    pub first: FieldError<K, String>,
    pub remaining: Vec<FieldError<K, String>>,
}
impl<K> FormErrors<K> {
    pub fn one(field: K, error: impl Into<String>) -> Self {
        Self {
            first: FieldError {
                field,
                error: error.into(),
            },
            remaining: Vec::new(),
        }
    }
    fn into_vec(self) -> Vec<FieldError<K, String>> {
        std::iter::once(self.first).chain(self.remaining).collect()
    }
}
impl From<ui::FormFieldView> for FormFieldSnapshot {
    fn from(view: ui::FormFieldView) -> Self {
        Self {
            will_validate: view.will_validate,
            validity: view.validity.into(),
            dirty: view.dirty,
            message: view.message,
            target: view.target,
        }
    }
}

type Snapshot<W> = Rc<dyn Fn(&W) -> FormFieldSnapshot>;
type FieldAction<W> = Rc<dyn Fn(&W)>;
type Focus<W> = Rc<dyn Fn(&W) -> bool>;
type SetError<W> = Rc<dyn Fn(&W, SharedString)>;
type Report<W> = Rc<dyn Fn(&W) -> bool>;
struct Registration<W, K> {
    key: K,
    snapshot: Snapshot<W>,
    report: Report<W>,
    focus: Focus<W>,
    set_error: SetError<W>,
    reset: FieldAction<W>,
}
impl<W, K: Clone> Clone for Registration<W, K> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            snapshot: self.snapshot.clone(),
            report: self.report.clone(),
            focus: self.focus.clone(),
            set_error: self.set_error.clone(),
            reset: self.reset.clone(),
        }
    }
}
type FormPublisher<W> = Rc<dyn Fn(&W, FormProjection)>;
struct FormState<W: ComponentHandle, K> {
    window: slint::Weak<W>,
    fields: RefCell<Vec<Registration<W, K>>>,
    coordinator: RefCell<FormCoordinator<K, String>>,
    reported: Cell<bool>,
    publish: FormPublisher<W>,
    focus_fallback: RefCell<Option<FieldAction<W>>>,
}
/// Window-local form coordination, with stable registration order and weak ownership.
/// No field callback or publisher runs while an internal borrow is held.
pub struct FormBinding<W: ComponentHandle, K> {
    state: Rc<FormState<W, K>>,
}
impl<W: ComponentHandle, K> Clone for FormBinding<W, K> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<W: ComponentHandle + 'static, K: Clone + PartialEq + 'static> FormBinding<W, K> {
    pub fn new(window: &W, publish: impl Fn(&W, FormProjection) + 'static) -> Self {
        Self {
            state: Rc::new(FormState {
                window: window.as_weak(),
                fields: RefCell::new(Vec::new()),
                coordinator: RefCell::new(FormCoordinator::default()),
                reported: Cell::new(false),
                publish: Rc::new(publish),
                focus_fallback: RefCell::new(None),
            }),
        }
    }
    /// Register once, in the order in which invalid fields should receive focus.
    /// The report callback must leave focus unchanged; use the separate focus callback.
    pub fn register(
        &self,
        key: K,
        snapshot: impl Fn(&W) -> FormFieldSnapshot + 'static,
        report: impl Fn(&W) -> bool + 'static,
        focus: impl Fn(&W) + 'static,
        reset: impl Fn(&W) + 'static,
    ) -> Result<(), FormRegistrationError> {
        self.register_focusable(
            key,
            snapshot,
            report,
            move |window| {
                focus(window);
                true
            },
            reset,
            |_, _| {},
        )
    }
    /// Register a composition view, explicit focus admission, reset, and field error presentation.
    pub fn register_view(
        &self,
        key: K,
        snapshot: impl Fn(&W) -> ui::FormFieldView + 'static,
        report: impl Fn(&W) -> bool + 'static,
        focus: impl Fn(&W) -> bool + 'static,
        reset: impl Fn(&W) + 'static,
        set_error: impl Fn(&W, SharedString) + 'static,
    ) -> Result<(), FormRegistrationError> {
        self.register_focusable(
            key,
            move |window| snapshot(window).into(),
            report,
            focus,
            reset,
            set_error,
        )
    }
    pub fn register_focusable(
        &self,
        key: K,
        snapshot: impl Fn(&W) -> FormFieldSnapshot + 'static,
        report: impl Fn(&W) -> bool + 'static,
        focus: impl Fn(&W) -> bool + 'static,
        reset: impl Fn(&W) + 'static,
        set_error: impl Fn(&W, SharedString) + 'static,
    ) -> Result<(), FormRegistrationError> {
        {
            let mut fields = self.state.fields.borrow_mut();
            if fields.iter().any(|field| field.key == key) {
                return Err(FormRegistrationError::DuplicateKey);
            }
            fields.push(Registration {
                key,
                snapshot: Rc::new(snapshot),
                report: Rc::new(report),
                focus: Rc::new(focus),
                set_error: Rc::new(set_error),
                reset: Rc::new(reset),
            });
        }
        self.refresh();
        Ok(())
    }
    /// Focus the form or its summary when no invalid field admits a focus request.
    pub fn set_focus_fallback(&self, focus: impl Fn(&W) + 'static) {
        *self.state.focus_fallback.borrow_mut() = Some(Rc::new(focus));
    }
    pub fn focus_target(&self, target: &ui::FocusRequest) -> bool {
        if !target.present {
            return false;
        }
        let Some(window) = self.state.window.upgrade() else {
            return false;
        };
        for field in self.fields() {
            if (field.snapshot)(&window).target == *target {
                return (field.focus)(&window);
            }
        }
        false
    }
    /// Clear a server error only for the stable field that the user edited.
    pub fn field_edited(&self, key: &K) {
        self.state.coordinator.borrow_mut().clear_error(key);
        if let Some(window) = self.state.window.upgrade() {
            for field in self.fields() {
                if &field.key == key {
                    (field.set_error)(&window, SharedString::default());
                }
            }
        }
        self.refresh();
    }
    pub fn unregister(&self, key: &K) {
        self.state
            .fields
            .borrow_mut()
            .retain(|field| &field.key != key);
        self.refresh();
    }
    fn fields(&self) -> Vec<Registration<W, K>> {
        self.state.fields.borrow().clone()
    }
    pub fn refresh(&self) {
        let Some(window) = self.state.window.upgrade() else {
            return;
        };
        let fields = self.fields();
        // Error ports are form-owned projections. Read intrinsic constraints
        // without feeding the previous client/server message back into them.
        for field in &fields {
            (field.set_error)(&window, SharedString::default());
        }
        let mut validation = HtmlFormValidation::default();
        let mut dirty = false;
        let mut errors = Vec::new();
        let mut field_messages = Vec::new();
        for field in &fields {
            let snapshot = (field.snapshot)(&window);
            validation.update(field.key.clone(), snapshot.will_validate, snapshot.validity);
            dirty |= snapshot.dirty;
            let visible =
                self.state.reported.get() && snapshot.will_validate && !snapshot.validity.valid();
            field_messages.push((
                field.key.clone(),
                if visible {
                    snapshot.message.to_string()
                } else {
                    String::new()
                },
            ));
            if visible {
                errors.push(ui::FormErrorRow {
                    target: snapshot.target,
                    message: snapshot.message,
                });
            }
        }
        let (submitting, server_errors) = match self.state.coordinator.borrow().state() {
            SubmissionState::Submitting(_) => (true, Vec::new()),
            SubmissionState::Rejected(errors) => (false, errors.clone()),
            SubmissionState::Idle | SubmissionState::Succeeded => (false, Vec::new()),
        };
        let server_valid = server_errors.is_empty();
        for error in server_errors {
            if let Some(field) = fields.iter().find(|field| field.key == error.field) {
                if let Some((_, message)) = field_messages
                    .iter_mut()
                    .find(|(key, _)| *key == error.field)
                {
                    if !message.is_empty() {
                        message.push('\n');
                    }
                    message.push_str(&error.error);
                }
                errors.push(ui::FormErrorRow {
                    target: (field.snapshot)(&window).target,
                    message: error.error.into(),
                });
            } else {
                errors.push(ui::FormErrorRow {
                    target: Default::default(),
                    message: error.error.into(),
                });
            }
        }
        for (field, (_, message)) in fields.iter().zip(field_messages) {
            (field.set_error)(&window, message.into());
        }
        (self.state.publish)(
            &window,
            FormProjection {
                valid: validation.can_submit(false) && server_valid,
                dirty,
                submitting,
                errors: ModelRc::new(VecModel::from(errors)),
            },
        );
    }
    fn report_fields(&self, window: &W) -> Option<(K, Vec<K>)> {
        self.state.reported.set(true);
        let mut invalid = Vec::new();
        for field in self.fields() {
            let reported = (field.report)(window);
            let snapshot = (field.snapshot)(window);
            if !reported || (snapshot.will_validate && !snapshot.validity.valid()) {
                invalid.push(field);
            }
        }
        if !invalid.is_empty() && !invalid.iter().any(|field| (field.focus)(window)) {
            let fallback = self.state.focus_fallback.borrow().clone();
            if let Some(focus) = fallback {
                focus(window);
            }
        }
        let mut invalid = invalid.into_iter();
        let first = invalid.next()?;
        Some((first.key, invalid.map(|field| field.key).collect()))
    }
    /// Report every field, then focus the first invalid field exactly once.
    pub fn report_validity(&self) -> bool {
        let Some(window) = self.state.window.upgrade() else {
            return false;
        };
        let valid = self.report_fields(&window).is_none();
        self.refresh();
        valid
    }
    /// Guard every Rust submission path, including commands outside the UI.
    pub fn submit<P>(&self, payload: P) -> Result<FormIntent<P>, FormSubmitError<K>> {
        self.submit_with_validation(payload, true)
    }
    /// Explicitly opt out of field validation, retaining window and busy guards.
    /// Pair with KitForm.form-no-validate; domain parsing remains the caller's duty.
    pub fn submit_without_validation<P>(
        &self,
        payload: P,
    ) -> Result<FormIntent<P>, FormSubmitError<K>> {
        self.submit_with_validation(payload, false)
    }
    fn submit_with_validation<P>(
        &self,
        payload: P,
        validate: bool,
    ) -> Result<FormIntent<P>, FormSubmitError<K>> {
        if matches!(
            self.state.coordinator.borrow().state(),
            SubmissionState::Submitting(_)
        ) {
            return Err(FormSubmitError::Busy);
        }
        let window = self
            .state
            .window
            .upgrade()
            .ok_or(FormSubmitError::WindowClosed)?;
        if validate && let Some((first, remaining)) = self.report_fields(&window) {
            self.refresh();
            return Err(FormSubmitError::InvalidFields { first, remaining });
        }
        let intent = self
            .state
            .coordinator
            .borrow_mut()
            .submit(payload)
            .ok_or(FormSubmitError::Busy)?;
        self.refresh();
        Ok(intent)
    }
    /// Validate the registered fields, then parse the payload once at this boundary.
    pub fn submit_typed<P>(
        &self,
        parse: impl FnOnce(&W) -> Result<P, FormErrors<K>>,
    ) -> Result<FormSubmission<P>, FormSubmitError<K>> {
        if matches!(
            self.state.coordinator.borrow().state(),
            SubmissionState::Submitting(_)
        ) {
            return Err(FormSubmitError::Busy);
        }
        let window = self
            .state
            .window
            .upgrade()
            .ok_or(FormSubmitError::WindowClosed)?;
        if let Some((first, remaining)) = self.report_fields(&window) {
            self.refresh();
            return Err(FormSubmitError::InvalidFields { first, remaining });
        }
        match parse(&window) {
            Ok(payload) => {
                let intent = self
                    .state
                    .coordinator
                    .borrow_mut()
                    .submit(payload)
                    .ok_or(FormSubmitError::Busy)?;
                self.refresh();
                match intent {
                    FormIntent::Submit { ticket, payload } => {
                        Ok(FormSubmission { ticket, payload })
                    }
                    FormIntent::Reset => unreachable!("submit produces a submission intent"),
                }
            }
            Err(errors) => {
                let errors = errors.into_vec();
                let first = errors[0].field.clone();
                let remaining = errors
                    .iter()
                    .skip(1)
                    .map(|error| error.field.clone())
                    .collect();
                let intent = self
                    .state
                    .coordinator
                    .borrow_mut()
                    .submit(())
                    .ok_or(FormSubmitError::Busy)?;
                match intent {
                    FormIntent::Submit {
                        ticket,
                        payload: (),
                    } => {
                        let keys = errors
                            .iter()
                            .map(|error| error.field.clone())
                            .collect::<Vec<_>>();
                        self.finish(ticket, errors);
                        let focused = self
                            .fields()
                            .iter()
                            .filter(|field| keys.contains(&field.key))
                            .any(|field| (field.focus)(&window));
                        if !focused {
                            let fallback = self.state.focus_fallback.borrow().clone();
                            if let Some(focus) = fallback {
                                focus(&window);
                            }
                        }
                    }
                    FormIntent::Reset => unreachable!("submit produces a submission intent"),
                }
                Err(FormSubmitError::InvalidValue { first, remaining })
            }
        }
    }
    pub fn finish(&self, ticket: SubmissionTicket, errors: Vec<FieldError<K, String>>) -> bool {
        let accepted = self.state.coordinator.borrow_mut().finish(ticket, errors);
        if accepted {
            self.refresh();
        }
        accepted
    }
    pub fn reset(&self) {
        self.state.coordinator.borrow_mut().reset();
        self.state.reported.set(false);
        if let Some(window) = self.state.window.upgrade() {
            for field in self.fields() {
                (field.reset)(&window);
            }
        }
        self.refresh();
    }
}

// Native editors own Enter. A zero-delay UI task only submits after an editor
// actually accepted the key, and a generation fences nested/stale dispatches.
enum ImplicitSubmitState {
    Idle,
    Captured(ui::ImplicitSubmitTicket),
    Handled(ui::ImplicitSubmitTicket),
}
pub(crate) struct ImplicitSubmitDispatcher {
    next_sequence: Cell<i32>,
    state: RefCell<ImplicitSubmitState>,
}
impl Default for ImplicitSubmitDispatcher {
    fn default() -> Self {
        Self {
            next_sequence: Cell::new(0),
            state: RefCell::new(ImplicitSubmitState::Idle),
        }
    }
}
impl ImplicitSubmitDispatcher {
    pub(crate) fn begin(&self, focus_generation: i32) -> ui::ImplicitSubmitTicket {
        let sequence = self.next_sequence.get();
        self.next_sequence.set(
            sequence
                .checked_add(1)
                .expect("implicit form submission capacity exhausted"),
        );
        let ticket = ui::ImplicitSubmitTicket {
            sequence,
            focus_generation,
        };
        *self.state.borrow_mut() = ImplicitSubmitState::Captured(ticket.clone());
        ticket
    }
    pub(crate) fn active(&self, focus_generation: i32) -> bool {
        match &*self.state.borrow() {
            ImplicitSubmitState::Idle | ImplicitSubmitState::Handled(_) => false,
            ImplicitSubmitState::Captured(ticket) => ticket.focus_generation == focus_generation,
        }
    }
    pub(crate) fn note_native(&self, focus_generation: i32) {
        let next = match self.state.replace(ImplicitSubmitState::Idle) {
            ImplicitSubmitState::Idle => ImplicitSubmitState::Idle,
            ImplicitSubmitState::Captured(ticket) => {
                if ticket.focus_generation == focus_generation {
                    ImplicitSubmitState::Handled(ticket)
                } else {
                    ImplicitSubmitState::Idle
                }
            }
            ImplicitSubmitState::Handled(ticket) => ImplicitSubmitState::Handled(ticket),
        };
        *self.state.borrow_mut() = next;
    }
    pub(crate) fn take(&self, ticket: ui::ImplicitSubmitTicket, focus_generation: i32) -> bool {
        match self.state.replace(ImplicitSubmitState::Idle) {
            ImplicitSubmitState::Idle => false,
            ImplicitSubmitState::Captured(active) => {
                if active != ticket {
                    *self.state.borrow_mut() = ImplicitSubmitState::Captured(active);
                }
                false
            }
            ImplicitSubmitState::Handled(active) => {
                if active == ticket {
                    active.focus_generation == focus_generation
                } else {
                    *self.state.borrow_mut() = ImplicitSubmitState::Handled(active);
                    false
                }
            }
        }
    }
}
#[cfg(test)]
mod implicit_submit_tests {
    use super::*;
    #[test]
    fn native_acceptance_is_required_and_newer_dispatch_survives_an_old_timer() {
        let dispatcher = ImplicitSubmitDispatcher::default();
        let rejected = dispatcher.begin(4);
        assert!(!dispatcher.take(rejected, 4));
        let outer = dispatcher.begin(4);
        let inner = dispatcher.begin(4);
        dispatcher.note_native(4);
        assert!(!dispatcher.active(4));
        assert!(!dispatcher.take(outer, 4));
        assert!(dispatcher.take(inner.clone(), 4));
        assert!(!dispatcher.take(inner, 4));
    }
    #[test]
    fn moving_focus_invalidates_a_pending_form_gesture() {
        let dispatcher = ImplicitSubmitDispatcher::default();
        let pending = dispatcher.begin(4);
        assert!(!dispatcher.active(5));
        dispatcher.note_native(5);
        assert!(!dispatcher.take(pending, 5));
    }
}
