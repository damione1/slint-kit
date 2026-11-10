// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
//! Weak-window friendly field bindings. Native TextInput remains the editor.
use kit_core::{
    AbsencePolicy, CommitPolicy, CommitTrigger, DraftValue, FieldAccess, FieldController,
    FieldFailure, FieldIntent, HtmlConstraints, HtmlInputState, HtmlValidity, Ownership,
    ParseIssue, ValidationGeneration, ValidationPresentation, ValidationState, grapheme_count,
};
use std::{cell::RefCell, rc::Rc};

impl From<FieldAccess> for crate::ui::FieldAccess {
    fn from(value: FieldAccess) -> Self {
        match value {
            FieldAccess::Editable => Self::Editable,
            FieldAccess::ReadOnly => Self::ReadOnly,
            FieldAccess::Disabled => Self::Disabled,
        }
    }
}

impl From<ValidationPresentation> for crate::ui::ValidationPresentation {
    fn from(value: ValidationPresentation) -> Self {
        match value {
            ValidationPresentation::Pristine => Self::Pristine,
            ValidationPresentation::Checking => Self::Checking,
            ValidationPresentation::Valid => Self::Valid,
            ValidationPresentation::Invalid => Self::Invalid,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldProjection {
    pub draft: String,
    pub formatted: String,
    pub validation: ValidationPresentation,
    pub access: FieldAccess,
    pub error: String,
    pub dirty: bool,
    pub touched: bool,
    pub character_count: usize,
    pub html_validity: Option<HtmlValidity>,
    pub will_validate: bool,
}

type DraftNormalizer = Rc<dyn Fn(&str) -> Result<String, ParseIssue>>;
type Parser<T> = Rc<dyn Fn(&str) -> DraftValue<T>>;
type Formatter<T> = Rc<dyn Fn(&T) -> String>;
type Validator<T, E> = Rc<dyn Fn(&T) -> Vec<E>>;
type FieldIntentHandler<T> = Rc<dyn Fn(FieldIntent<T>)>;
struct Codec<T> {
    parse: Parser<T>,
    format: Formatter<T>,
}
struct BoundField<T, E> {
    controller: FieldController<T, E>,
    codec: Codec<T>,
    pending_codec: Option<Codec<T>>,
    failure: Option<FieldFailure>,
    ownership: Ownership,
    awaiting_ack: bool,
    html: Option<HtmlConstraints>,
    normalizer: Option<DraftNormalizer>,
    html_user_edited: bool,
    html_reported: bool,
    messages: Option<crate::ui::KitFieldMessages>,
}
/// A local controller with an explicit parsing/formatting boundary. The publisher
/// should capture a weak window. No thread, task runtime or global state is used.
pub struct FieldBinding<T, E> {
    inner: Rc<RefCell<BoundField<T, E>>>,
    publish: Rc<dyn Fn(FieldProjection)>,
    intent: Rc<RefCell<Option<FieldIntentHandler<T>>>>,
    validator: Rc<RefCell<Option<Validator<T, E>>>>,
    error_text: Rc<dyn Fn(&E) -> String>,
}
impl<T, E> Clone for FieldBinding<T, E> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            publish: self.publish.clone(),
            intent: self.intent.clone(),
            validator: self.validator.clone(),
            error_text: self.error_text.clone(),
        }
    }
}
impl<T: Clone + PartialEq + 'static, E: 'static> FieldBinding<T, E> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        accepted: Option<T>,
        ownership: Ownership,
        policy: CommitPolicy,
        absence: AbsencePolicy,
        parse: impl Fn(&str) -> DraftValue<T> + 'static,
        format: impl Fn(&T) -> String + 'static,
        error_text: impl Fn(&E) -> String + 'static,
        publish: impl Fn(FieldProjection) + 'static,
    ) -> Self {
        let format: Formatter<T> = Rc::new(format);
        let text = accepted
            .as_ref()
            .map_or_else(String::new, |value| format(value));
        let binding = Self {
            inner: Rc::new(RefCell::new(BoundField {
                controller: FieldController::new(accepted, text, ownership, policy, absence),
                codec: Codec {
                    parse: Rc::new(parse),
                    format,
                },
                pending_codec: None,
                failure: None,
                ownership,
                awaiting_ack: false,
                html: None,
                normalizer: None,
                html_user_edited: false,
                html_reported: false,
                messages: None,
            })),
            publish: Rc::new(publish),
            intent: Rc::new(RefCell::new(None)),
            validator: Rc::new(RefCell::new(None)),
            error_text: Rc::new(error_text),
        };
        binding.refresh();
        binding
    }
    /// Intrinsic HTML constraints run before the application's domain parser.
    /// This also guards commits triggered by blur or a form submit in Rust.
    pub fn set_messages(&self, messages: crate::ui::KitFieldMessages) {
        self.inner.borrow_mut().messages = Some(messages);
        self.refresh();
    }
    pub fn set_html_constraints(&self, rules: HtmlConstraints) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.html = Some(rules);
            let draft = inner.controller.draft().to_string();
            let parsed = Self::parse_draft(&inner, &draft, inner.html_user_edited);
            inner.controller.reparse(parsed);
            inner.failure = None;
        }
        self.refresh();
    }
    /// Normalize semantic validation/parser input without assigning to the native draft.
    /// For phone fields, use normalize_phone and map its error to InvalidSyntax.
    pub fn set_draft_normalizer(
        &self,
        normalizer: impl Fn(&str) -> Result<String, ParseIssue> + 'static,
    ) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.normalizer = Some(Rc::new(normalizer));
            let draft = inner.controller.draft().to_string();
            let parsed = Self::parse_draft(&inner, &draft, inner.html_user_edited);
            inner.controller.reparse(parsed);
            inner.failure = None;
        }
        self.refresh();
    }
    fn parse_draft(inner: &BoundField<T, E>, text: &str, user_edited: bool) -> DraftValue<T> {
        let normalized = match inner.normalizer.as_ref().map(|f| f(text)).transpose() {
            Ok(value) => value,
            Err(issue) => return DraftValue::Invalid(issue),
        };
        let text = normalized.as_deref().unwrap_or(text);
        if let Some(rules) = &inner.html {
            let validity = rules.validate(
                text,
                HtmlInputState {
                    user_edited,
                    ..Default::default()
                },
            );
            if !validity.valid() && !text.is_empty() {
                return DraftValue::Invalid(ParseIssue::InvalidSyntax);
            }
            (inner.codec.parse)(&rules.sanitize(text))
        } else {
            (inner.codec.parse)(text)
        }
    }
    pub fn on_intent(&self, callback: impl Fn(FieldIntent<T>) + 'static) {
        *self.intent.borrow_mut() = Some(Rc::new(callback));
    }
    pub fn set_validator(&self, validator: impl Fn(&T) -> Vec<E> + 'static) {
        *self.validator.borrow_mut() = Some(Rc::new(validator));
    }
    pub fn accepted(&self) -> Option<T> {
        self.inner.borrow().controller.accepted().cloned()
    }
    pub fn snapshot(&self) -> FieldProjection {
        let inner = self.inner.borrow();
        let c = &inner.controller;
        let mut validation = if inner.failure.is_some() {
            ValidationPresentation::Invalid
        } else {
            c.presentation()
        };
        let mut error = match c.validation() {
            ValidationState::Invalid(errors) => errors
                .iter()
                .map(|e| (self.error_text)(e))
                .collect::<Vec<_>>()
                .join("\n"),
            ValidationState::Unchecked
            | ValidationState::Pending { .. }
            | ValidationState::Valid => match inner.failure {
                Some(failure) => inner.messages.as_ref().map_or_else(
                    || failure_message(failure).into(),
                    |messages| localized_failure(messages, failure),
                ),
                None => match c.parsed() {
                    DraftValue::Invalid(issue) => inner.messages.as_ref().map_or_else(
                        || parse_message(*issue).into(),
                        |messages| localized_failure(messages, FieldFailure::Parse(*issue)),
                    ),
                    DraftValue::Empty | DraftValue::Incomplete | DraftValue::Parsed(_) => {
                        String::new()
                    }
                },
            },
        };
        let normalized = inner.normalizer.as_ref().map(|f| f(c.draft())).transpose();
        let html_validity = inner.html.as_ref().map(|rules| {
            let mut validity = rules.validate(
                normalized
                    .as_ref()
                    .ok()
                    .and_then(|v| v.as_deref())
                    .unwrap_or(c.draft()),
                HtmlInputState {
                    user_edited: inner.html_user_edited,
                    ..Default::default()
                },
            );
            if normalized.is_err() {
                validity.bad_input = true;
            }
            validity
        });
        let will_validate = c.access() == FieldAccess::Editable
            && inner
                .html
                .as_ref()
                .is_none_or(HtmlConstraints::will_validate);
        if let (Some(rules), Some(validity)) = (&inner.html, html_validity) {
            if !will_validate {
                validation = ValidationPresentation::Pristine;
                error.clear();
            } else if !validity.valid() {
                let show = c.touched()
                    || inner.html_reported
                    || c.commit_policy() == CommitPolicy::OnChangeWhenValid
                    || (inner.html_user_edited
                        && rules.kind == kit_core::HtmlInputKind::Email
                        && kit_core::html_email_hard_error(c.draft(), rules.multiple));
                validation = if show {
                    ValidationPresentation::Invalid
                } else {
                    ValidationPresentation::Pristine
                };
                error = if show {
                    inner.messages.as_ref().map_or_else(
                        || validity.message(rules),
                        |messages| localized_constraint(messages, validity, rules),
                    )
                } else {
                    String::new()
                };
            }
        }
        FieldProjection {
            draft: c.draft().into(),
            formatted: c
                .accepted()
                .map_or_else(String::new, |value| (inner.codec.format)(value)),
            validation,
            access: c.access(),
            error,
            dirty: c.dirty(),
            touched: c.touched(),
            character_count: grapheme_count(c.draft()),
            html_validity,
            will_validate,
        }
    }
    pub fn refresh(&self) {
        (self.publish)(self.snapshot());
    }
    fn emit(&self, intent: FieldIntent<T>) {
        let callback = self.intent.borrow().clone();
        if let Some(callback) = callback {
            callback(intent);
        }
    }
    pub fn edited(&self, text: &str) -> Result<(), FieldFailure> {
        if self
            .inner
            .borrow()
            .html
            .as_ref()
            .is_some_and(|rules| rules.disabled || rules.read_only)
        {
            return Err(FieldFailure::AccessDenied);
        }
        {
            let mut inner = self.inner.borrow_mut();
            let parsed = Self::parse_draft(&inner, text, true);
            inner.controller.edit(text.into(), parsed)?;
            inner.html_user_edited = true;
            inner.failure = None;
            inner.awaiting_ack = false;
        }
        self.refresh();
        self.commit(CommitTrigger::Change).map(|_| ())
    }
    pub fn commit(&self, trigger: CommitTrigger) -> Result<Option<FieldIntent<T>>, FieldFailure> {
        if self
            .inner
            .borrow()
            .html
            .as_ref()
            .is_some_and(|rules| rules.disabled || rules.read_only)
        {
            return Err(FieldFailure::AccessDenied);
        }
        if self.inner.borrow().controller.access() != FieldAccess::Editable {
            return Err(FieldFailure::AccessDenied);
        }
        if trigger == CommitTrigger::Blur {
            self.inner.borrow_mut().controller.mark_touched();
        }
        if matches!(trigger, CommitTrigger::Submit | CommitTrigger::Explicit) {
            self.inner.borrow_mut().html_reported = true;
        }
        let html_failure = {
            let inner = self.inner.borrow();
            let normalized = inner
                .normalizer
                .as_ref()
                .map(|f| f(inner.controller.draft()))
                .transpose();
            inner.html.as_ref().is_some_and(|rules| {
                inner.controller.permits_commit(trigger)
                    && rules.will_validate()
                    && (normalized.is_err()
                        || !rules
                            .validate(
                                normalized
                                    .as_ref()
                                    .ok()
                                    .and_then(|v| v.as_deref())
                                    .unwrap_or(inner.controller.draft()),
                                HtmlInputState {
                                    user_edited: inner.html_user_edited,
                                    ..Default::default()
                                },
                            )
                            .valid())
            })
        };
        if html_failure {
            self.inner.borrow_mut().failure = Some(FieldFailure::ValidationFailed);
            self.refresh();
            return Err(FieldFailure::ValidationFailed);
        }
        let validator = self.validator.borrow().clone();
        let (value, should_validate) = {
            let inner = self.inner.borrow();
            (
                match inner.controller.parsed() {
                    DraftValue::Parsed(value) => Some(value.clone()),
                    DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => None,
                },
                matches!(inner.controller.validation(), ValidationState::Unchecked)
                    && inner.controller.permits_commit(trigger)
                    && inner.controller.access() == FieldAccess::Editable,
            )
        };
        if should_validate && let (Some(validator), Some(value)) = (validator, value) {
            let errors = validator(&value);
            let mut inner = self.inner.borrow_mut();
            let ticket = inner.controller.begin_validation();
            inner.controller.finish_validation(ticket, errors);
        }
        let result = {
            let mut inner = self.inner.borrow_mut();
            let format = inner.codec.format.clone();
            let result = inner.controller.request_commit(trigger, |v| format(v));
            inner.failure = result.as_ref().err().copied().filter(|failure| {
                !matches!(
                    failure,
                    FieldFailure::AccessDenied | FieldFailure::ValidationPending
                )
            });
            if result.as_ref().is_ok_and(|intent| intent.is_some())
                && trigger != CommitTrigger::Change
            {
                match inner.ownership {
                    Ownership::Local => Self::apply_pending(&mut inner),
                    Ownership::Controlled => inner.awaiting_ack = true,
                }
            }
            result
        };
        self.refresh();
        if let Ok(Some(intent)) = &result {
            self.emit(intent.clone());
        }
        result
    }
    pub fn cancel(&self) {
        let intent = {
            let mut inner = self.inner.borrow_mut();
            let format = inner.codec.format.clone();
            let intent = inner.controller.cancel(|v| format(v));
            inner.failure = None;
            inner.awaiting_ack = false;
            inner.html_user_edited = false;
            inner.html_reported = false;
            Self::apply_pending(&mut inner);
            intent
        };
        self.refresh();
        self.emit(intent);
    }
    pub fn reset(&self, value: Option<T>) {
        {
            let mut inner = self.inner.borrow_mut();
            let format = inner.codec.format.clone();
            inner.controller.reset(value, |v| format(v));
            inner.failure = None;
            inner.awaiting_ack = false;
            inner.html_user_edited = false;
            inner.html_reported = false;
            Self::apply_pending(&mut inner);
        }
        self.refresh();
    }
    pub fn acknowledge(&self, value: Option<T>) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.controller.acknowledge(value);
            if inner.awaiting_ack {
                inner.awaiting_ack = false;
                let format = inner.codec.format.clone();
                inner.controller.cancel(|v| format(v));
                inner.html_user_edited = false;
                inner.html_reported = false;
                Self::apply_pending(&mut inner);
            }
        }
        self.refresh();
    }
    pub fn set_access(&self, access: FieldAccess) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.controller.set_access(access);
            inner.failure = None;
        }
        self.refresh();
    }
    pub fn begin_validation(&self) -> ValidationGeneration {
        let ticket = {
            let mut inner = self.inner.borrow_mut();
            inner.failure = None;
            inner.controller.begin_validation()
        };
        self.refresh();
        ticket
    }
    pub fn finish_validation(&self, ticket: ValidationGeneration, errors: Vec<E>) -> bool {
        let accepted = self
            .inner
            .borrow_mut()
            .controller
            .finish_validation(ticket, errors);
        if accepted {
            self.refresh();
        }
        accepted
    }
    /// Reconfigure locale/format without reinterpreting an active draft. The new
    /// codec becomes active after explicit commit, blur, submit, or cancellation.
    pub fn reconfigure(
        &self,
        parse: impl Fn(&str) -> DraftValue<T> + 'static,
        format: impl Fn(&T) -> String + 'static,
    ) {
        {
            let mut inner = self.inner.borrow_mut();
            inner.pending_codec = Some(Codec {
                parse: Rc::new(parse),
                format: Rc::new(format),
            });
            if !inner.controller.dirty() {
                Self::apply_pending(&mut inner);
            }
        }
        self.refresh();
    }
    fn apply_pending(inner: &mut BoundField<T, E>) {
        if let Some(codec) = inner.pending_codec.take() {
            inner.codec = codec;
            let format = inner.codec.format.clone();
            inner.controller.cancel(|v| format(v));
        }
    }
}
pub(crate) fn failure_message(failure: FieldFailure) -> &'static str {
    match failure {
        FieldFailure::AccessDenied => "This field cannot be edited.",
        FieldFailure::MissingValue => "Enter a value.",
        FieldFailure::Incomplete => "Finish entering this value.",
        FieldFailure::Parse(issue) => parse_message(issue),
        FieldFailure::ValidationPending => "Validation is still running.",
        FieldFailure::ValidationFailed => "Check this value.",
    }
}
fn parse_message(issue: ParseIssue) -> &'static str {
    match issue {
        ParseIssue::DivisionByZero => "Cannot divide by zero.",
        ParseIssue::InvalidSyntax => "Enter a value in the expected format.",
        ParseIssue::InvalidGrouping => "Check the digit grouping for this locale.",
        ParseIssue::Overflow => "This value is too large.",
        ParseIssue::ScaleExceeded => "This value has too many decimal places.",
        ParseIssue::BelowMinimum => "This value is below the minimum.",
        ParseIssue::AboveMaximum => "This value is above the maximum.",
        ParseIssue::InvalidDate => "Enter a valid date.",
        ParseIssue::InvalidTime => "Enter a valid time.",
        ParseIssue::InvalidCurrency => "Choose a valid currency.",
        ParseIssue::InvalidColor => "Enter a valid color.",
        ParseIssue::InvalidUnit => "Choose a valid unit.",
        ParseIssue::InvalidRange => "Enter an ordered range.",
    }
}

pub(crate) fn localized_failure(
    messages: &crate::ui::KitFieldMessages,
    failure: FieldFailure,
) -> String {
    let message = match failure {
        FieldFailure::AccessDenied => &messages.access_denied,
        FieldFailure::MissingValue => &messages.missing_value,
        FieldFailure::Incomplete => &messages.incomplete,
        FieldFailure::ValidationPending => &messages.validation_pending,
        FieldFailure::ValidationFailed => &messages.validation_failed,
        FieldFailure::Parse(issue) => match issue {
            ParseIssue::InvalidSyntax => &messages.invalid_syntax,
            ParseIssue::InvalidGrouping => &messages.invalid_grouping,
            ParseIssue::Overflow => &messages.overflow,
            ParseIssue::DivisionByZero => &messages.division_by_zero,
            ParseIssue::ScaleExceeded => &messages.scale_exceeded,
            ParseIssue::BelowMinimum => &messages.below_minimum,
            ParseIssue::AboveMaximum => &messages.above_maximum,
            ParseIssue::InvalidDate => &messages.invalid_date,
            ParseIssue::InvalidTime => &messages.invalid_time,
            ParseIssue::InvalidCurrency => &messages.invalid_currency,
            ParseIssue::InvalidColor => &messages.invalid_color,
            ParseIssue::InvalidUnit => &messages.invalid_unit,
            ParseIssue::InvalidRange => &messages.invalid_range,
        },
    };
    message.to_string()
}
fn localized_constraint(
    messages: &crate::ui::KitFieldMessages,
    validity: HtmlValidity,
    rules: &HtmlConstraints,
) -> String {
    if !rules.will_validate() || validity.valid() {
        return String::new();
    }
    if validity.custom_error {
        return rules.custom_error.clone();
    }
    if validity.value_missing {
        return messages.required_message.to_string();
    }
    if validity.type_mismatch {
        return if rules.kind == kit_core::HtmlInputKind::Email {
            &messages.email_message
        } else {
            &messages.url_message
        }
        .to_string();
    }
    if validity.pattern_mismatch {
        return messages.pattern_message.to_string();
    }
    if validity.too_short {
        return messages
            .too_short_template
            .replace("{count}", &rules.min_length.unwrap_or(0).to_string());
    }
    if validity.too_long {
        return messages
            .too_long_template
            .replace("{count}", &rules.max_length.unwrap_or(0).to_string());
    }
    if validity.range_underflow {
        return messages
            .underflow_template
            .replace("{value}", rules.minimum.as_deref().unwrap_or(""));
    }
    if validity.range_overflow {
        return messages
            .overflow_template
            .replace("{value}", rules.maximum.as_deref().unwrap_or(""));
    }
    if validity.step_mismatch {
        return messages.step_message.to_string();
    }
    messages.bad_input_message.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(text: &str) -> DraftValue<i64> {
        match text {
            "" => DraftValue::Empty,
            "-" => DraftValue::Incomplete,
            value => value.parse().map_or(
                DraftValue::Invalid(ParseIssue::InvalidSyntax),
                DraftValue::Parsed,
            ),
        }
    }
    fn binding(ownership: Ownership) -> FieldBinding<i64, ()> {
        FieldBinding::new(
            Some(2),
            ownership,
            CommitPolicy::OnBlur,
            AbsencePolicy::Required,
            parse,
            ToString::to_string,
            |_| String::new(),
            |_| {},
        )
    }
    #[test]
    fn incomplete_draft_cannot_replace_value_and_locale_change_is_deferred() {
        let b = binding(Ownership::Local);
        b.edited("-").unwrap();
        assert_eq!(b.accepted(), Some(2));
        b.reconfigure(parse, |value| format!("{value} units"));
        assert_eq!(b.snapshot().draft, "-");
        assert_eq!(b.commit(CommitTrigger::Blur), Err(FieldFailure::Incomplete));
        assert_eq!(b.snapshot().draft, "-");
        b.cancel();
        assert_eq!(b.snapshot().draft, "2 units");
        assert_eq!(b.accepted(), Some(2));
    }
    #[test]
    fn controlled_commit_waits_for_ack_and_preserves_newer_edit() {
        let b = binding(Ownership::Controlled);
        b.edited("3").unwrap();
        b.reconfigure(parse, |v| format!("{v} accepted"));
        b.commit(CommitTrigger::Explicit).unwrap();
        assert_eq!(b.accepted(), Some(2));
        assert_eq!(b.snapshot().draft, "3");
        b.acknowledge(Some(3));
        assert_eq!(b.snapshot().draft, "3 accepted");
        b.edited("4").unwrap();
        b.acknowledge(Some(7));
        assert_eq!(b.snapshot().draft, "4");
    }
    #[test]
    fn stale_validation_and_access_cannot_publish_an_old_value() {
        let b = binding(Ownership::Local);
        let old = b.begin_validation();
        b.edited("3").unwrap();
        assert!(!b.finish_validation(old, vec![]));
        b.set_access(FieldAccess::ReadOnly);
        assert_eq!(b.edited("9"), Err(FieldFailure::AccessDenied));
        assert_eq!(b.accepted(), Some(2));
        b.set_access(FieldAccess::Disabled);
        assert_eq!(
            b.commit(CommitTrigger::Explicit),
            Err(FieldFailure::AccessDenied)
        );
    }
    #[test]
    fn pending_validation_remains_checking_when_commit_is_requested() {
        let b = binding(Ownership::Local);
        b.edited("3").unwrap();
        let ticket = b.begin_validation();
        assert_eq!(
            b.commit(CommitTrigger::Submit),
            Err(FieldFailure::ValidationPending)
        );
        assert_eq!(b.snapshot().validation, ValidationPresentation::Checking);
        assert_eq!(b.accepted(), Some(2));
        assert!(b.finish_validation(ticket, vec![]));
        b.commit(CommitTrigger::Submit).unwrap();
        assert_eq!(b.accepted(), Some(3));
    }
    #[test]
    fn on_change_runs_business_validation_before_accepting_a_value() {
        let b = FieldBinding::<i64, &'static str>::new(
            Some(2),
            Ownership::Local,
            CommitPolicy::OnChangeWhenValid,
            AbsencePolicy::Required,
            parse,
            ToString::to_string,
            |error| error.to_string(),
            |_| {},
        );
        b.set_validator(|value| {
            if *value > 5 {
                vec!["Maximum 5"]
            } else {
                vec![]
            }
        });
        assert_eq!(b.edited("9"), Err(FieldFailure::ValidationFailed));
        assert_eq!(b.accepted(), Some(2));
        assert_eq!(b.snapshot().draft, "9");
        assert_eq!(b.snapshot().error, "Maximum 5");
        b.edited("4").unwrap();
        assert_eq!(b.accepted(), Some(4));
    }
    #[test]
    fn on_change_does_not_format_native_editor_text() {
        let b = FieldBinding::<i64, ()>::new(
            Some(2),
            Ownership::Local,
            CommitPolicy::OnChangeWhenValid,
            AbsencePolicy::Required,
            parse,
            |v| format!("{v} units"),
            |_| String::new(),
            |_| {},
        );
        b.edited("003").unwrap();
        assert_eq!(b.accepted(), Some(3));
        assert_eq!(b.snapshot().draft, "003");
    }
}

#[cfg(test)]
mod html_tests {
    use super::*;
    use kit_core::HtmlInputKind;
    #[test]
    fn intrinsic_rules_guard_blur_and_submit_even_with_an_unrestricted_domain_parser() {
        let field = FieldBinding::<String, ()>::new(
            Some("alex@example.com".into()),
            Ownership::Local,
            CommitPolicy::OnBlur,
            AbsencePolicy::Required,
            |s| {
                if s.is_empty() {
                    DraftValue::Empty
                } else {
                    DraftValue::Parsed(s.into())
                }
            },
            Clone::clone,
            |_| String::new(),
            |_| {},
        );
        field.set_html_constraints(HtmlConstraints {
            kind: HtmlInputKind::Email,
            required: true,
            ..Default::default()
        });
        field.edited("alex@@example.com").unwrap();
        assert!(field.snapshot().html_validity.unwrap().type_mismatch);
        assert_eq!(
            field.snapshot().error,
            "Please enter a valid email address."
        );
        assert_eq!(
            field.commit(CommitTrigger::Blur),
            Err(FieldFailure::ValidationFailed)
        );
        assert_eq!(
            field.commit(CommitTrigger::Submit),
            Err(FieldFailure::ValidationFailed)
        );
        assert_eq!(field.accepted().as_deref(), Some("alex@example.com"));
        field.edited(" other@example.net ").unwrap();
        field.commit(CommitTrigger::Blur).unwrap();
        assert_eq!(field.accepted().as_deref(), Some("other@example.net"));
    }
}

#[cfg(test)]
mod html_editing_origin_tests {
    use super::*;
    #[test]
    fn accepted_edits_remain_subject_to_length_rules_after_constraints_change() {
        let field = FieldBinding::<String, ()>::new(
            None,
            Ownership::Local,
            CommitPolicy::OnBlur,
            AbsencePolicy::Optional,
            |s| {
                if s.is_empty() {
                    DraftValue::Empty
                } else {
                    DraftValue::Parsed(s.into())
                }
            },
            Clone::clone,
            |_| String::new(),
            |_| {},
        );
        field.set_html_constraints(HtmlConstraints::default());
        field.edited("ABCD").unwrap();
        field.commit(CommitTrigger::Blur).unwrap();
        assert!(!field.snapshot().dirty);
        field.set_html_constraints(HtmlConstraints {
            min_length: Some(6),
            ..Default::default()
        });
        assert!(field.snapshot().html_validity.unwrap().too_short);
        assert_eq!(
            field.commit(CommitTrigger::Submit),
            Err(FieldFailure::ValidationFailed)
        );
        field.reset(Some("ABCD".into()));
        assert!(!field.snapshot().html_validity.unwrap().too_short);
    }
    #[test]
    fn incomplete_email_is_deferred_to_blur_but_never_accepted() {
        let field = FieldBinding::<String, ()>::new(
            None,
            Ownership::Local,
            CommitPolicy::OnBlur,
            AbsencePolicy::Optional,
            |s| DraftValue::Parsed(s.into()),
            Clone::clone,
            |_| String::new(),
            |_| {},
        );
        field.set_html_constraints(HtmlConstraints {
            kind: kit_core::HtmlInputKind::Email,
            ..Default::default()
        });
        field.edited("alex").unwrap();
        assert!(field.snapshot().html_validity.unwrap().type_mismatch);
        assert_eq!(
            field.snapshot().validation,
            ValidationPresentation::Pristine
        );
        assert_eq!(
            field.commit(CommitTrigger::Blur),
            Err(FieldFailure::ValidationFailed)
        );
        assert_eq!(field.snapshot().validation, ValidationPresentation::Invalid);
        assert!(field.accepted().is_none());
    }
}

#[cfg(test)]
mod phone_normalization_tests {
    use super::*;
    use kit_core::{
        HtmlInputKind, HtmlPattern, PhoneNumber, PhoneNumberMode, PhoneRules, normalize_phone,
    };
    #[test]
    fn phone_normalization_precedes_regex_without_rewriting_active_edits() {
        let rules = PhoneRules::exact(PhoneNumberMode::National, 10).unwrap();
        let initial = rules.validate("0612345678").unwrap();
        let field = FieldBinding::<PhoneNumber, ()>::new(
            initial,
            Ownership::Local,
            CommitPolicy::OnBlur,
            AbsencePolicy::Required,
            move |text| rules.parse(text),
            |v| v.as_str().into(),
            |_| String::new(),
            |_| {},
        );
        field.set_draft_normalizer(|text| {
            normalize_phone(text).map_err(|_| ParseIssue::InvalidSyntax)
        });
        field.set_html_constraints(HtmlConstraints {
            kind: HtmlInputKind::Telephone,
            required: true,
            pattern: Some(HtmlPattern::compile("0[0-9]{9}").unwrap()),
            ..Default::default()
        });
        field.edited("06 98 76 54 32").unwrap();
        assert_eq!(field.snapshot().draft, "06 98 76 54 32");
        assert!(field.snapshot().html_validity.unwrap().valid());
        field.commit(CommitTrigger::Submit).unwrap();
        assert_eq!(field.accepted().unwrap().as_str(), "0698765432");
        assert!(field.snapshot().html_validity.unwrap().valid());
        for draft in ["Ada0612345678", "123", "061234567890", "+33612345678"] {
            field.edited(draft).unwrap();
            assert!(field.commit(CommitTrigger::Submit).is_err());
            assert_eq!(field.accepted().unwrap().as_str(), "0698765432");
        }
    }
}
