// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use crate::{HtmlValidationEngine, ui};
use kit_core::*;
use std::{collections::HashMap, rc::Rc};

// HTML validates the native draft before the codec. Keep the grammar the
// control explicitly declares (decimal comma / twelve-hour clock); other
// sanitization still follows the shared HTML engine.
fn codec_input(rules: &HtmlConstraints, text: &str) -> String {
    if (rules.kind == HtmlInputKind::Number
        && rules.number_format == NumberInputFormat::DecimalComma)
        || (matches!(
            rules.kind,
            HtmlInputKind::Time | HtmlInputKind::DateTimeLocal
        ) && rules.twelve_hour)
    {
        text.to_owned()
    } else {
        rules.sanitize(text)
    }
}
fn decimal_draft(value: DecimalValue, syntax: NumberSyntax) -> String {
    value
        .to_string()
        .replace('.', &syntax.decimal_separator().to_string())
}

/// Slint has no sum types: the outcome splits into its status and the text
/// that only a `stepped` status carries.
fn step_result(outcome: NumberStepOutcome) -> ui::NumberStepResult {
    match outcome {
        NumberStepOutcome::Unavailable => ui::NumberStepResult {
            status: ui::NumberStepStatus::Unavailable,
            text: Default::default(),
        },
        NumberStepOutcome::Unchanged => ui::NumberStepResult {
            status: ui::NumberStepStatus::Unchanged,
            text: Default::default(),
        },
        NumberStepOutcome::Stepped(text) => ui::NumberStepResult {
            status: ui::NumberStepStatus::Stepped,
            text: text.into(),
        },
    }
}

type SpecParser<T> = Rc<dyn Fn(&str) -> DraftValue<T>>;
type SpecFormatter<T> = Rc<dyn Fn(&T) -> String>;
type SpecValidator<T, E> = Rc<dyn Fn(&T) -> Result<(), E>>;

/// A typed codec at the application boundary. Its projection remains text so
/// exact decimals and native drafts never pass through a renderer float.
pub struct FieldSpec<T, E = ParseIssue> {
    parse: SpecParser<T>,
    format: SpecFormatter<T>,
    display: SpecFormatter<T>,
    validate: SpecValidator<T, E>,
    error: Rc<dyn Fn(&E) -> String>,
}
impl<T, E> Clone for FieldSpec<T, E> {
    fn clone(&self) -> Self {
        Self {
            parse: self.parse.clone(),
            format: self.format.clone(),
            display: self.display.clone(),
            validate: self.validate.clone(),
            error: self.error.clone(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum FieldSpecError<E> {
    Field(FieldFailure),
    Domain(E),
}
/// A stable application-authored codec key, independent of field positions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldSpecKey(String);
impl FieldSpecKey {
    pub fn new(value: impl Into<String>) -> Result<Self, ParseIssue> {
        let value = value.into();
        if value.is_empty() {
            Err(ParseIssue::InvalidSyntax)
        } else {
            Ok(Self(value))
        }
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
type Canonicalizer = Rc<dyn Fn(&str) -> Result<DraftValue<String>, String>>;
pub struct NamedFieldSpec {
    key: FieldSpecKey,
    canonicalize: Canonicalizer,
    display: Rc<dyn Fn(&str) -> String>,
}
impl<T: 'static, E: 'static> FieldSpec<T, E> {
    pub fn new(
        parse: impl Fn(&str) -> DraftValue<T> + 'static,
        format: impl Fn(&T) -> String + 'static,
        validate: impl Fn(&T) -> Result<(), E> + 'static,
        error: impl Fn(&E) -> String + 'static,
    ) -> Self {
        let format: SpecFormatter<T> = Rc::new(format);
        Self {
            parse: Rc::new(parse),
            display: format.clone(),
            format,
            validate: Rc::new(validate),
            error: Rc::new(error),
        }
    }
    pub fn with_display(mut self, display: impl Fn(&T) -> String + 'static) -> Self {
        self.display = Rc::new(display);
        self
    }
    /// Display one separator after a Unicode scalar prefix without changing the
    /// accepted value or native draft. A prefix beyond the value adds nothing.
    pub fn with_grouping(mut self, prefix: usize, separator: impl Into<String>) -> Self {
        let format = self.format.clone();
        let separator = separator.into();
        self.display = Rc::new(move |value| {
            let text = format(value);
            match text.char_indices().nth(prefix) {
                Some((offset, _)) if offset > 0 => {
                    format!("{}{}{}", &text[..offset], separator, &text[offset..])
                }
                Some(_) | None => text,
            }
        });
        self
    }
    pub fn format(&self, value: &T) -> String {
        (self.format)(value)
    }
    /// Parse an already accepted canonical text, without field constraints.
    pub(crate) fn parse_text(&self, text: &str) -> DraftValue<T> {
        (self.parse)(text)
    }
    /// Guard Rust submission paths with the same constraints as the component.
    /// The parsed domain value is returned directly to business code.
    pub fn accept(&self, commit: ui::FieldCommit) -> Result<Option<T>, FieldSpecError<E>> {
        if !commit.value.present && !commit.value.text.is_empty() {
            return Err(FieldSpecError::Field(FieldFailure::Parse(
                ParseIssue::InvalidSyntax,
            )));
        }
        let (rules, _) = HtmlValidationEngine::default().constraints(&commit.constraints);
        if rules.disabled || rules.read_only {
            return Err(FieldSpecError::Field(FieldFailure::AccessDenied));
        }
        if !rules
            .validate(
                commit.value.text.as_str(),
                HtmlInputState {
                    user_edited: true,
                    ..Default::default()
                },
            )
            .valid()
        {
            return Err(FieldSpecError::Field(FieldFailure::Parse(
                ParseIssue::InvalidSyntax,
            )));
        }
        match (self.parse)(&codec_input(&rules, commit.value.text.as_str())) {
            DraftValue::Parsed(value) => {
                (self.validate)(&value).map_err(FieldSpecError::Domain)?;
                Ok(Some(value))
            }
            DraftValue::Empty if !rules.required => Ok(None),
            DraftValue::Empty => Err(FieldSpecError::Field(FieldFailure::MissingValue)),
            DraftValue::Incomplete => Err(FieldSpecError::Field(FieldFailure::Incomplete)),
            DraftValue::Invalid(issue) => Err(FieldSpecError::Field(FieldFailure::Parse(issue))),
        }
    }
    /// Register a pure codec shared by any number of independently owned fields.
    pub fn named(&self, key: FieldSpecKey) -> NamedFieldSpec {
        let spec = self.clone();
        let display_spec = self.clone();
        NamedFieldSpec {
            key,
            display: Rc::new(move |text| match (display_spec.parse)(text) {
                DraftValue::Parsed(value) => (display_spec.display)(&value),
                DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => text.into(),
            }),
            canonicalize: Rc::new(move |text| match (spec.parse)(text) {
                DraftValue::Parsed(value) => {
                    (spec.validate)(&value).map_err(|error| (spec.error)(&error))?;
                    Ok(DraftValue::Parsed((spec.format)(&value)))
                }
                DraftValue::Empty => Ok(DraftValue::Empty),
                DraftValue::Incomplete => Ok(DraftValue::Incomplete),
                DraftValue::Invalid(issue) => Ok(DraftValue::Invalid(issue)),
            }),
        }
    }
}
impl<T: 'static> FieldSpec<T> {
    pub fn codec(
        parse: impl Fn(&str) -> DraftValue<T> + 'static,
        format: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self::new(parse, format, |_| Ok(()), |issue| format!("{issue:?}"))
    }
}
impl FieldSpec<TextValue> {
    pub fn text() -> Self {
        Self::codec(
            |text| {
                if text.is_empty() {
                    DraftValue::Empty
                } else {
                    DraftValue::Parsed(TextValue::new(text))
                }
            },
            |value| value.as_str().into(),
        )
    }
}
impl FieldSpec<EmailAddress> {
    pub fn email() -> Self {
        Self::codec(EmailAddress::parse, |value| value.as_str().into())
    }
}
impl FieldSpec<WebUrl> {
    pub fn url() -> Self {
        Self::codec(WebUrl::parse, |value| value.as_str().into())
    }
}
impl FieldSpec<i64> {
    /// Grouped input follows `syntax`; the drafted text is ungrouped.
    pub fn integer(syntax: NumberSyntax, bounds: Option<NumericBounds<i64>>) -> Self {
        Self::codec(
            move |text| parse_integer(text, syntax, bounds),
            ToString::to_string,
        )
    }
}
impl FieldSpec<DecimalValue> {
    pub fn decimal(rules: DecimalRules) -> Self {
        Self::codec(
            move |text| rules.parse(text),
            move |value| decimal_draft(*value, rules.syntax),
        )
    }
}
impl FieldSpec<Money> {
    pub fn money(currency: CurrencyCode, rules: DecimalRules) -> Self {
        Self::codec(
            move |text| {
                map_draft(rules.parse(text), |amount| {
                    Ok(Money {
                        amount,
                        currency: currency.clone(),
                    })
                })
            },
            move |value| decimal_draft(value.amount, rules.syntax),
        )
    }
}
impl FieldSpec<Money> {
    /// Money whose scale comes from the currency: input beyond the currency's
    /// minor units is rejected with `ScaleExceeded`. The at-rest display adds
    /// the symbol and the digit grouping of `syntax`; the native draft stays a
    /// plain decimal. A stored value finer than the scale shows as its draft.
    pub fn money_in(
        metadata: CurrencyMetadata,
        syntax: NumberSyntax,
        sign: SignPolicy,
        bounds: Option<NumericBounds<DecimalValue>>,
    ) -> Self {
        let rules = DecimalRules {
            syntax,
            scale: metadata.scale,
            sign,
            bounds,
        };
        Self::money(metadata.code.clone(), rules).with_display(move |value| {
            value
                .format(&metadata, syntax, RoundingPolicy::Reject)
                .unwrap_or_else(|_| decimal_draft(value.amount, syntax))
        })
    }
}
impl FieldSpec<Percent> {
    pub fn percent(rules: DecimalRules) -> Self {
        Self::codec(
            move |text| map_draft(rules.parse(text), |amount| Ok(Percent(amount))),
            move |value| decimal_draft(value.0, rules.syntax),
        )
    }
}
impl<U: Clone + 'static> FieldSpec<Quantity<U>> {
    pub fn quantity(unit: U, rules: DecimalRules) -> Self {
        Self::codec(
            move |text| {
                map_draft(rules.parse(text), |amount| {
                    Ok(Quantity {
                        value: amount,
                        unit: unit.clone(),
                    })
                })
            },
            move |value| decimal_draft(value.value, rules.syntax),
        )
    }
}
impl FieldSpec<CivilDate> {
    pub fn date() -> Self {
        Self::codec(CivilDate::parse_iso, |value| value.iso_string())
    }
}
impl FieldSpec<LocalTime> {
    /// Twelve-hour text uses the default AM and PM markers.
    pub fn time(precision: TimePrecision, clock: ClockFormat) -> Self {
        Self::time_with_markers(precision, clock, DayPeriodMarkers::default())
    }
    /// Twelve-hour text prints and accepts the given day-period markers.
    pub fn time_with_markers(
        precision: TimePrecision,
        clock: ClockFormat,
        markers: DayPeriodMarkers,
    ) -> Self {
        Self::codec(
            {
                let markers = markers.clone();
                move |text| LocalTime::parse_with_markers(text, precision, clock, &markers)
            },
            move |value| value.format_with_markers(precision, clock, &markers),
        )
    }
}
impl FieldSpec<Year> {
    pub fn year() -> Self {
        Self::codec(
            |text| {
                map_draft(parse_integer(text, NumberSyntax::en_ca(), None), |year| {
                    Year::new(i32::try_from(year).map_err(|_| ParseIssue::InvalidDate)?)
                })
            },
            |value| format!("{:04}", value.get()),
        )
    }
}
impl FieldSpec<YearMonth> {
    pub fn month() -> Self {
        Self::codec(
            |text| parse_pair(text, "-", 7, YearMonth::new),
            |value| format!("{:04}-{:02}", value.year(), value.month()),
        )
    }
}
impl FieldSpec<IsoWeek> {
    pub fn iso_week() -> Self {
        Self::codec(
            |text| parse_pair(text, "-W", 8, IsoWeek::new),
            |value| format!("{:04}-W{:02}", value.year().get(), value.week()),
        )
    }
}
impl FieldSpec<DurationValue> {
    /// Configure the visible numeric unit explicitly. A suffix is presentation.
    /// Loaded values use the fallible DurationFieldSpec::format boundary.
    pub fn duration(unit: DurationUnit, rules: DecimalRules) -> DurationFieldSpec {
        DurationFieldSpec {
            unit,
            rules,
            codec: FieldSpec::codec(
                move |text| {
                    map_draft(rules.parse(text), |quantity| {
                        DurationValue::from_quantity(quantity, unit)
                            .map(|duration| DurationDraft { duration, quantity })
                    })
                },
                move |value| decimal_draft(value.quantity, rules.syntax),
            ),
        }
    }
}
// Only the parser constructs this pair: quantity is the exact accepted value
// in the configured unit, retained for native canonicalization and display.
#[derive(Clone)]
struct DurationDraft {
    duration: DurationValue,
    quantity: DecimalValue,
}
/// Duration codec using the shared field engine, with an explicit unit and a
/// fallible loaded-value formatting boundary. No conversion rounds silently.
#[derive(Clone)]
pub struct DurationFieldSpec {
    unit: DurationUnit,
    rules: DecimalRules,
    codec: FieldSpec<DurationDraft>,
}
impl DurationFieldSpec {
    /// Express a loaded duration in this unit and configured input scale/bounds.
    /// Recurring or too-precise quantities return ScaleExceeded. Overflow and
    /// out-of-bounds values retain their ordinary ParseIssue variants.
    pub fn format(&self, value: &DurationValue) -> Result<String, ParseIssue> {
        let quantity = value
            .quantity(self.unit)?
            .quantize(self.rules.scale, RoundingPolicy::Reject)?;
        let quantity = self
            .rules
            .bounds
            .map_or(Ok(quantity), |bounds| bounds.validate(quantity))?;
        Ok(decimal_draft(quantity, self.rules.syntax))
    }
    pub fn accept(
        &self,
        commit: ui::FieldCommit,
    ) -> Result<Option<DurationValue>, FieldSpecError<ParseIssue>> {
        self.codec
            .accept(commit)
            .map(|value| value.map(|draft| draft.duration))
    }
    pub fn named(&self, key: FieldSpecKey) -> NamedFieldSpec {
        self.codec.named(key)
    }
    pub fn with_display(mut self, display: impl Fn(&DurationValue) -> String + 'static) -> Self {
        self.codec = self
            .codec
            .with_display(move |value| display(&value.duration));
        self
    }
    pub fn with_grouping(mut self, prefix: usize, separator: impl Into<String>) -> Self {
        self.codec = self.codec.with_grouping(prefix, separator);
        self
    }
}
impl FieldSpec<PhoneNumber> {
    pub fn telephone(rules: PhoneRules) -> Self {
        Self::codec(move |text| rules.parse(text), |value| value.as_str().into())
    }
}
impl FieldSpec<OtpValue> {
    pub fn otp(rules: OtpRules) -> Self {
        Self::codec(move |text| rules.parse(text), |value| value.as_str().into())
    }
}
impl FieldSpec<MaskedValue> {
    pub fn masked(mask: MaskPattern) -> Self {
        Self::codec(move |text| mask.parse(text), |value| value.as_str().into())
    }
}
fn map_draft<T, U>(
    draft: DraftValue<T>,
    map: impl FnOnce(T) -> Result<U, ParseIssue>,
) -> DraftValue<U> {
    match draft {
        DraftValue::Empty => DraftValue::Empty,
        DraftValue::Incomplete => DraftValue::Incomplete,
        DraftValue::Invalid(issue) => DraftValue::Invalid(issue),
        DraftValue::Parsed(value) => match map(value) {
            Ok(value) => DraftValue::Parsed(value),
            Err(issue) => DraftValue::Invalid(issue),
        },
    }
}
fn parse_pair<T>(
    text: &str,
    separator: &str,
    length: usize,
    construct: impl FnOnce(i32, u32) -> Result<T, ParseIssue>,
) -> DraftValue<T> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    if text.len() < length {
        return DraftValue::Incomplete;
    }
    let parse = || {
        let (year, part) = text.split_once(separator).ok_or(ParseIssue::InvalidDate)?;
        if text.len() != length
            || year.len() != 4
            || part.len() != 2
            || !year
                .bytes()
                .chain(part.bytes())
                .all(|byte| byte.is_ascii_digit())
        {
            return Err(ParseIssue::InvalidDate);
        }
        construct(
            year.parse().map_err(|_| ParseIssue::InvalidDate)?,
            part.parse().map_err(|_| ParseIssue::InvalidDate)?,
        )
    };
    match parse() {
        Ok(value) => DraftValue::Parsed(value),
        Err(issue) => DraftValue::Invalid(issue),
    }
}

/// Install custom pure codecs before showing a window. This replaces the family
/// codec catalogue, never its component-owned state; duplicate keys are rejected.
pub fn install_field_specs<W>(
    window: &W,
    specs: impl IntoIterator<Item = NamedFieldSpec>,
) -> Result<(), ParseIssue>
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitFieldService<'a>: slint::Global<'a, W>,
{
    let mut codecs = HashMap::new();
    for spec in specs {
        if codecs.insert(spec.key.clone(), spec).is_some() {
            return Err(ParseIssue::InvalidSyntax);
        }
    }
    let codecs = Rc::new(codecs);
    let display_codecs = codecs.clone();
    let engine = HtmlValidationEngine::default();
    let service = window.global::<ui::KitFieldService>();
    crate::text_edit::install(window);
    crate::popup::install(window);
    let edit_window = window.as_weak();
    service.on_edit_selection(move |text| {
        if let Some(window) = edit_window.upgrade() {
            use slint::platform::{Key, WindowEvent};
            let text = if text.is_empty() {
                Key::Delete.into()
            } else {
                text
            };
            window
                .window()
                .dispatch_event(WindowEvent::KeyPressed { text: text.clone() });
            window
                .window()
                .dispatch_event(WindowEvent::KeyReleased { text });
        }
    });
    service.on_display(move |spec, text| {
        FieldSpecKey::new(spec.to_string())
            .ok()
            .and_then(|key| display_codecs.get(&key))
            .map_or(text.clone(), |codec| (codec.display)(text.as_str()).into())
    });
    service.on_step_number(|text, constraints, direction, count| {
        let (rules, _) = HtmlValidationEngine::default().constraints(&constraints);
        // The page size is an `int` in Slint; only a positive count is a step.
        let Some(count) = u32::try_from(count)
            .ok()
            .and_then(std::num::NonZeroU32::new)
        else {
            return step_result(NumberStepOutcome::Unavailable);
        };
        step_result(rules.step_number(
            text.as_str(),
            NumberStep::Steps {
                direction: direction.into(),
                count,
            },
        ))
    });
    service.on_step_number_to_bound(|text, constraints, direction| {
        let (rules, _) = HtmlValidationEngine::default().constraints(&constraints);
        step_result(rules.step_number(text.as_str(), NumberStep::Bound(direction.into())))
    });
    service.on_number_range(|constraints| {
        let (rules, _) = HtmlValidationEngine::default().constraints(&constraints);
        let projection = rules.number_range_projection();
        ui::NumberRangeView {
            minimum: projection.minimum.unwrap_or(f32::NEG_INFINITY),
            maximum: projection.maximum.unwrap_or(f32::INFINITY),
            step: projection.step.unwrap_or(0.0),
        }
    });
    service.on_reduce(
        move |state, action, text, constraints, policy, ownership, spec, user_edited, messages| {
            let (rules, _) = engine.constraints(&constraints);
            let valid = rules
                .validate(
                    text.as_str(),
                    HtmlInputState {
                        user_edited,
                        ..Default::default()
                    },
                )
                .valid();
            let canonical = codec_input(&rules, text.as_str());
            let mut codec_error = None;
            let parsed = if !valid {
                DraftValue::Invalid(ParseIssue::InvalidSyntax)
            } else if spec.is_empty() {
                if canonical.is_empty() {
                    DraftValue::Empty
                } else {
                    DraftValue::Parsed(canonical)
                }
            } else {
                match FieldSpecKey::new(spec.to_string())
                    .ok()
                    .and_then(|key| codecs.get(&key))
                {
                    Some(codec) => match (codec.canonicalize)(&canonical) {
                        Ok(value) => value,
                        Err(error) => {
                            codec_error = Some(error);
                            DraftValue::Invalid(ParseIssue::InvalidSyntax)
                        }
                    },
                    None => {
                        codec_error = Some(messages.missing_codec.to_string());
                        DraftValue::Invalid(ParseIssue::InvalidSyntax)
                    }
                }
            };
            let action = match action {
                ui::FieldAction::Edit => FieldAction::Edit,
                ui::FieldAction::Blur => FieldAction::Blur,
                ui::FieldAction::Submit => FieldAction::Submit,
                ui::FieldAction::Cancel => FieldAction::Cancel,
            };
            let config = FieldRules {
                ownership: ownership.into(),
                access: if rules.disabled {
                    FieldAccess::Disabled
                } else if rules.read_only {
                    FieldAccess::ReadOnly
                } else {
                    FieldAccess::Editable
                },
                policy: match policy {
                    ui::CommitPolicy::OnBlur => CommitPolicy::OnBlur,
                    ui::CommitPolicy::OnSubmit => CommitPolicy::OnSubmit,
                    ui::CommitPolicy::OnChangeWhenValid => CommitPolicy::OnChangeWhenValid,
                },
                absence: if rules.required {
                    AbsencePolicy::Required
                } else {
                    AbsencePolicy::Optional
                },
            };
            let (next, outcome) = FieldController::<String, ()>::reduce(
                FieldState {
                    accepted: state
                        .accepted
                        .present
                        .then(|| state.accepted.text.to_string()),
                    draft: text.to_string(),
                    dirty: state.dirty,
                    touched: state.touched,
                },
                config,
                action,
                parsed,
                Clone::clone,
            );
            let value_view = |value: Option<String>| {
                value.map_or_else(ui::FieldValue::default, |text| ui::FieldValue {
                    present: true,
                    text: text.into(),
                })
            };
            let (committed, value, error) = match outcome {
                Ok(Some(FieldIntent::Commit(value))) => {
                    (true, value_view(Some(value)), String::new())
                }
                Ok(Some(FieldIntent::Clear)) => (true, ui::FieldValue::default(), String::new()),
                Ok(Some(FieldIntent::Cancel)) | Ok(None) => {
                    (false, ui::FieldValue::default(), String::new())
                }
                Err(error) => (
                    false,
                    ui::FieldValue::default(),
                    if !valid || action == FieldAction::Edit {
                        String::new()
                    } else {
                        codec_error
                            .unwrap_or_else(|| crate::field::localized_failure(&messages, error))
                    },
                ),
            };
            ui::FieldState {
                accepted: value_view(next.accepted),
                draft: next.draft.into(),
                dirty: next.dirty,
                touched: next.touched,
                error: error.into(),
                committed,
                value,
            }
        },
    );
    crate::numeric_service::install_numeric(&service);
    crate::color_service::install_color(&service);
    service.set_ready(true);
    Ok(())
}
pub(crate) fn install_fields<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> ui::KitFieldService<'a>: slint::Global<'a, W>,
{
    if !window.global::<ui::KitFieldService>().get_ready() {
        install_field_specs(window, []).expect("empty codec catalogue");
    }
}
