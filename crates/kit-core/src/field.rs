//! Semantic editing state. Native editors retain ownership of caret, IME and undo.
use crate::{CommitPolicy, FieldAccess, Ownership, ValidationPresentation};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
pub(crate) fn next_owner() -> u64 {
    NEXT_OWNER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseIssue {
    InvalidSyntax,
    InvalidGrouping,
    Overflow,
    DivisionByZero,
    ScaleExceeded,
    BelowMinimum,
    AboveMaximum,
    InvalidDate,
    InvalidTime,
    InvalidCurrency,
    InvalidColor,
    InvalidUnit,
    InvalidRange,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftValue<T> {
    Empty,
    Incomplete,
    Parsed(T),
    Invalid(ParseIssue),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationGeneration {
    owner: u64,
    revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidationState<E> {
    Unchecked,
    Pending { generation: ValidationGeneration },
    Valid,
    Invalid(Vec<E>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldIntent<T> {
    Commit(T),
    Clear,
    Cancel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbsencePolicy {
    Required,
    Optional,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvalidCommitPolicy {
    PreserveDraft,
    RevertToAccepted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitTrigger {
    Change,
    Blur,
    Submit,
    Explicit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldFailure {
    AccessDenied,
    MissingValue,
    Incomplete,
    Parse(ParseIssue),
    ValidationPending,
    ValidationFailed,
}
#[derive(Debug)]
pub struct FieldController<T, E> {
    accepted: Option<T>,
    draft: String,
    parsed: DraftValue<T>,
    validation: ValidationState<E>,
    ownership: Ownership,
    access: FieldAccess,
    policy: CommitPolicy,
    absence: AbsencePolicy,
    invalid_commit: InvalidCommitPolicy,
    owner: u64,
    revision: u64,
    dirty: bool,
    touched: bool,
}
impl<T: Clone + PartialEq, E> FieldController<T, E> {
    pub fn new(
        accepted: Option<T>,
        text: String,
        ownership: Ownership,
        policy: CommitPolicy,
        absence: AbsencePolicy,
    ) -> Self {
        let parsed = accepted
            .clone()
            .map_or(DraftValue::Empty, DraftValue::Parsed);
        Self {
            accepted,
            draft: text,
            parsed,
            validation: ValidationState::Unchecked,
            ownership,
            access: FieldAccess::Editable,
            policy,
            absence,
            invalid_commit: InvalidCommitPolicy::PreserveDraft,
            owner: next_owner(),
            revision: 0,
            dirty: false,
            touched: false,
        }
    }
    pub fn accepted(&self) -> Option<&T> {
        self.accepted.as_ref()
    }
    pub fn draft(&self) -> &str {
        &self.draft
    }
    pub fn parsed(&self) -> &DraftValue<T> {
        &self.parsed
    }
    pub fn validation(&self) -> &ValidationState<E> {
        &self.validation
    }
    pub fn dirty(&self) -> bool {
        self.dirty
    }
    pub fn mark_touched(&mut self) {
        self.touched = true;
    }
    pub fn touched(&self) -> bool {
        self.touched
    }
    pub fn access(&self) -> FieldAccess {
        self.access
    }
    pub fn commit_policy(&self) -> CommitPolicy {
        self.policy
    }
    pub fn set_commit_policy(&mut self, policy: CommitPolicy) {
        self.policy = policy;
    }
    pub fn set_access(&mut self, access: FieldAccess) {
        if self.access != access {
            self.invalidate_validation();
            self.access = access;
        }
    }
    pub fn set_invalid_commit_policy(&mut self, policy: InvalidCommitPolicy) {
        self.invalid_commit = policy;
    }
    fn invalidate_validation(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("field revision exhausted");
        self.validation = ValidationState::Unchecked;
    }
    /// The parser observes the editor's text; it never rewrites it.
    pub fn edit(&mut self, text: String, parsed: DraftValue<T>) -> Result<(), FieldFailure> {
        if self.access != FieldAccess::Editable {
            return Err(FieldFailure::AccessDenied);
        }
        self.invalidate_validation();
        self.draft = text;
        self.parsed = parsed;
        self.dirty = true;
        Ok(())
    }
    /// Recheck changed rules without treating application configuration as typing.
    pub fn reparse(&mut self, parsed: DraftValue<T>) {
        self.invalidate_validation();
        self.parsed = parsed;
    }
    pub fn begin_validation(&mut self) -> ValidationGeneration {
        self.invalidate_validation();
        let generation = ValidationGeneration {
            owner: self.owner,
            revision: self.revision,
        };
        self.validation = ValidationState::Pending { generation };
        generation
    }
    pub fn finish_validation(&mut self, ticket: ValidationGeneration, errors: Vec<E>) -> bool {
        if !matches!(self.validation, ValidationState::Pending { generation } if generation == ticket)
        {
            return false;
        }
        self.validation = if errors.is_empty() {
            match &self.parsed {
                DraftValue::Parsed(_) => ValidationState::Valid,
                DraftValue::Empty => match self.absence {
                    AbsencePolicy::Optional => ValidationState::Valid,
                    AbsencePolicy::Required => ValidationState::Unchecked,
                },
                DraftValue::Incomplete | DraftValue::Invalid(_) => ValidationState::Unchecked,
            }
        } else {
            ValidationState::Invalid(errors)
        };
        true
    }
    pub fn presentation(&self) -> ValidationPresentation {
        if matches!(self.parsed, DraftValue::Invalid(_)) {
            return ValidationPresentation::Invalid;
        }
        match &self.validation {
            ValidationState::Unchecked => ValidationPresentation::Pristine,
            ValidationState::Pending { .. } => ValidationPresentation::Checking,
            ValidationState::Valid => ValidationPresentation::Valid,
            ValidationState::Invalid(_) => ValidationPresentation::Invalid,
        }
    }
    /// Whether this editing policy requests a commit for the given trigger.
    pub fn permits_commit(&self, trigger: CommitTrigger) -> bool {
        match (self.policy, trigger) {
            (
                CommitPolicy::OnBlur | CommitPolicy::OnSubmit | CommitPolicy::OnChangeWhenValid,
                CommitTrigger::Explicit | CommitTrigger::Submit,
            ) => true,
            (CommitPolicy::OnBlur, CommitTrigger::Blur)
            | (CommitPolicy::OnChangeWhenValid, CommitTrigger::Change) => true,
            (CommitPolicy::OnSubmit, CommitTrigger::Change | CommitTrigger::Blur)
            | (CommitPolicy::OnBlur, CommitTrigger::Change)
            | (CommitPolicy::OnChangeWhenValid, CommitTrigger::Blur) => false,
        }
    }
    pub fn request_commit(
        &mut self,
        trigger: CommitTrigger,
        format: impl Fn(&T) -> String,
    ) -> Result<Option<FieldIntent<T>>, FieldFailure> {
        if self.access != FieldAccess::Editable {
            return Err(FieldFailure::AccessDenied);
        }
        if trigger == CommitTrigger::Blur {
            self.touched = true;
        }
        let permitted = self.permits_commit(trigger);
        if !permitted {
            return Ok(None);
        }
        let result = match &self.validation {
            ValidationState::Pending { .. } => Err(FieldFailure::ValidationPending),
            ValidationState::Invalid(_) => Err(FieldFailure::ValidationFailed),
            ValidationState::Unchecked | ValidationState::Valid => match &self.parsed {
                DraftValue::Empty => match self.absence {
                    AbsencePolicy::Required => Err(FieldFailure::MissingValue),
                    AbsencePolicy::Optional => Ok(FieldIntent::Clear),
                },
                DraftValue::Incomplete => Err(FieldFailure::Incomplete),
                DraftValue::Invalid(issue) => Err(FieldFailure::Parse(*issue)),
                DraftValue::Parsed(value) => Ok(FieldIntent::Commit(value.clone())),
            },
        };
        match result {
            Ok(intent) => {
                if self.ownership == Ownership::Local {
                    match &intent {
                        FieldIntent::Commit(value) => {
                            self.accepted = Some(value.clone());
                            if trigger != CommitTrigger::Change {
                                self.draft = format(value);
                            }
                        }
                        FieldIntent::Clear => {
                            self.accepted = None;
                            if trigger != CommitTrigger::Change {
                                self.draft.clear();
                            }
                        }
                        FieldIntent::Cancel => {}
                    }
                    self.dirty = false;
                }
                Ok(Some(intent))
            }
            Err(error) => {
                if self.invalid_commit == InvalidCommitPolicy::RevertToAccepted {
                    self.cancel(format);
                }
                Err(error)
            }
        }
    }
    /// A controlled application acknowledges a value without rewriting an active draft.
    pub fn acknowledge(&mut self, value: Option<T>) {
        self.accepted = value;
    }
    pub fn cancel(&mut self, format: impl Fn(&T) -> String) -> FieldIntent<T> {
        self.invalidate_validation();
        self.draft = self.accepted.as_ref().map_or_else(String::new, format);
        self.parsed = self
            .accepted
            .clone()
            .map_or(DraftValue::Empty, DraftValue::Parsed);
        self.dirty = false;
        FieldIntent::Cancel
    }
    pub fn reset(&mut self, snapshot: Option<T>, format: impl Fn(&T) -> String) {
        self.accepted = snapshot;
        self.cancel(format);
        self.touched = false;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SubmissionTicket {
    owner: u64,
    revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubmissionState<K, E> {
    Idle,
    Submitting(SubmissionTicket),
    Succeeded,
    Rejected(Vec<FieldError<K, E>>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError<K, E> {
    pub field: K,
    pub error: E,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormIntent<P> {
    Submit {
        ticket: SubmissionTicket,
        payload: P,
    },
    Reset,
}
#[derive(Debug)]
pub struct FormCoordinator<K, E> {
    owner: u64,
    revision: u64,
    state: SubmissionState<K, E>,
}
impl<K, E> Default for FormCoordinator<K, E> {
    fn default() -> Self {
        Self {
            owner: next_owner(),
            revision: 0,
            state: SubmissionState::Idle,
        }
    }
}
impl<K, E> FormCoordinator<K, E> {
    pub fn state(&self) -> &SubmissionState<K, E> {
        &self.state
    }
    pub fn submit<P>(&mut self, payload: P) -> Option<FormIntent<P>> {
        if matches!(self.state, SubmissionState::Submitting(_)) {
            return None;
        }
        self.revision = self
            .revision
            .checked_add(1)
            .expect("submission revision exhausted");
        let ticket = SubmissionTicket {
            owner: self.owner,
            revision: self.revision,
        };
        self.state = SubmissionState::Submitting(ticket);
        Some(FormIntent::Submit { ticket, payload })
    }
    pub fn finish(&mut self, ticket: SubmissionTicket, errors: Vec<FieldError<K, E>>) -> bool {
        if !matches!(self.state, SubmissionState::Submitting(active) if active == ticket) {
            return false;
        }
        self.state = if errors.is_empty() {
            SubmissionState::Succeeded
        } else {
            SubmissionState::Rejected(errors)
        };
        true
    }
    pub fn first_error(&self) -> Option<&K> {
        match &self.state {
            SubmissionState::Rejected(errors) => errors.first().map(|error| &error.field),
            SubmissionState::Idle | SubmissionState::Submitting(_) | SubmissionState::Succeeded => {
                None
            }
        }
    }
    pub fn reset(&mut self) {
        self.revision = self
            .revision
            .checked_add(1)
            .expect("submission revision exhausted");
        self.state = SubmissionState::Idle;
    }
}

pub fn grapheme_count(text: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation;
    text.graphemes(true).count()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_and_stale_validation() {
        let mut field = FieldController::<i64, &str>::new(
            Some(2),
            "2".into(),
            Ownership::Controlled,
            CommitPolicy::OnBlur,
            AbsencePolicy::Required,
        );
        field.edit("3".into(), DraftValue::Parsed(3)).unwrap();
        let stale = field.begin_validation();
        field.edit("-".into(), DraftValue::Incomplete).unwrap();
        assert!(!field.finish_validation(stale, vec![]));
        assert_eq!(field.accepted(), Some(&2));
        assert_eq!(
            field.request_commit(CommitTrigger::Blur, ToString::to_string),
            Err(FieldFailure::Incomplete)
        );
        field.edit("4".into(), DraftValue::Parsed(4)).unwrap();
        assert_eq!(
            field.request_commit(CommitTrigger::Blur, ToString::to_string),
            Ok(Some(FieldIntent::Commit(4)))
        );
        assert_eq!(field.accepted(), Some(&2));
        field.acknowledge(Some(4));
        field.cancel(ToString::to_string);
        assert_eq!(field.draft(), "4");
    }
    #[test]
    fn valid_change_commit_preserves_native_draft() {
        let mut field = FieldController::<i64, ()>::new(
            Some(1),
            "1".into(),
            Ownership::Local,
            CommitPolicy::OnChangeWhenValid,
            AbsencePolicy::Required,
        );
        field.edit("002".into(), DraftValue::Parsed(2)).unwrap();
        field
            .request_commit(CommitTrigger::Change, ToString::to_string)
            .unwrap();
        assert_eq!(field.draft(), "002");
        assert_eq!(field.accepted(), Some(&2));
        field
            .edit("x".into(), DraftValue::Invalid(ParseIssue::InvalidSyntax))
            .unwrap();
        let ticket = field.begin_validation();
        field.finish_validation(ticket, vec![]);
        assert_eq!(field.presentation(), ValidationPresentation::Invalid);
        assert_eq!(field.validation(), &ValidationState::Unchecked);
    }
    #[test]
    fn tickets_are_owned_and_reset_invalidates() {
        let mut a = FieldController::<i64, ()>::new(
            None,
            String::new(),
            Ownership::Local,
            CommitPolicy::OnSubmit,
            AbsencePolicy::Optional,
        );
        let mut b = FieldController::<i64, ()>::new(
            None,
            String::new(),
            Ownership::Local,
            CommitPolicy::OnSubmit,
            AbsencePolicy::Optional,
        );
        let ticket = a.begin_validation();
        b.begin_validation();
        assert!(!b.finish_validation(ticket, vec![]));
        a.reset(Some(7), ToString::to_string);
        assert!(!a.finish_validation(ticket, vec![]));
        assert_eq!(grapheme_count("👨‍👩‍👦e\u{301}"), 2);
    }
    #[test]
    fn form_blocks_repeated_submit_and_discards_old_response() {
        let mut form = FormCoordinator::<u8, &str>::default();
        let FormIntent::Submit { ticket, .. } = form.submit(42).unwrap() else {
            panic!()
        };
        assert!(form.submit(3).is_none());
        form.reset();
        assert!(!form.finish(ticket, vec![]));
        let FormIntent::Submit { ticket, .. } = form.submit(43).unwrap() else {
            panic!()
        };
        assert!(form.finish(
            ticket,
            vec![FieldError {
                field: 2,
                error: "required"
            }]
        ));
        assert_eq!(form.first_error(), Some(&2));
    }
}
