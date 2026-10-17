// SPDX-License-Identifier: GPL-3.0-or-later
//! Typed profile and billing examples; Slint owns native text editing.
use super::ui::{
    self, FormExamples, LocaleExample, NumericExample, Showcase, SubmissionPresentation,
    TextExample,
};
use kit::{
    AbsencePolicy, CommitPolicy, CommitTrigger, CurrencyMetadata, DecimalRules, DecimalScale,
    DecimalValue, DraftValue, DurationUnit, DurationValue, FieldAccess, FieldBinding, FieldError,
    FieldFailure, FieldIntent, FieldProjection, FormCoordinator, FormIntent, GraphemeLimit,
    MaskPattern, MaskToken, MaskedValue, Money, NumberLocale, NumericBounds, OtpAlphabet,
    OtpLength, OtpValue, Ownership, ParseIssue, Percent, Quantity, RoundingPolicy, SignPolicy,
    StepDirection, ValidationPresentation, format_decimal, grapheme_count, parse_integer,
};
use slint::ComponentHandle;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

#[derive(Clone, PartialEq, Eq)]
struct DisplayName(String);
#[derive(Clone, PartialEq, Eq)]
struct EmailAddress(String);
#[derive(Clone, PartialEq, Eq)]
struct Website(url::Url);
#[derive(Clone, PartialEq, Eq)]
struct PhoneNumber(String);
// Deliberately no Debug implementation or diagnostic projection for secrets.
#[derive(Clone, PartialEq, Eq)]
struct SecretText(String);
#[derive(Clone, PartialEq, Eq)]
struct Biography(String);
#[derive(Clone, PartialEq, Eq)]
struct SearchQuery(String);
#[derive(Clone, Copy, PartialEq, Eq)]
enum DistanceUnit {
    Kilometres,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProfileField {
    Name,
    Email,
    Website,
    Phone,
    Password,
    Biography,
    Otp,
    Reference,
    Consent,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FormIssue {
    InvalidValue,
    EmailUnavailable,
    ConsentRequired,
    OutOfBounds,
}
impl FormIssue {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidValue => "Check this value before saving.",
            Self::EmailUnavailable => "This demo address is already in use. Try another address.",
            Self::ConsentRequired => "Accept the project terms before saving.",
            Self::OutOfBounds => "Choose between 1 and 100 seats.",
        }
    }
}
#[derive(Clone)]
struct ProfilePayload {
    name: DisplayName,
    email: EmailAddress,
    website: Option<Website>,
    phone: PhoneNumber,
    password: SecretText,
    biography: Option<Biography>,
    otp: Option<OtpValue>,
    reference: Option<MaskedValue>,
    consent: bool,
    notifications: bool,
}
#[derive(Clone)]
struct BillingPayload {
    people: i64,
    decimal: DecimalValue,
    budget: Money,
    allocation: Percent,
    distance: Quantity<DistanceUnit>,
    duration: DurationValue,
    seats: i64,
}
#[derive(Clone, Copy)]
enum FieldView {
    Text(TextExample),
    Numeric(NumericExample),
}

trait FieldCommands {
    fn snapshot(&self) -> FieldProjection;
    fn edit(&self, text: &str);
    fn commit(&self, trigger: CommitTrigger) -> Result<(), FieldFailure>;
    fn cancel(&self);
    fn access(&self, value: FieldAccess);
}
impl<T: Clone + PartialEq + 'static> FieldCommands for FieldBinding<T, FormIssue> {
    fn snapshot(&self) -> FieldProjection {
        self.snapshot()
    }
    fn edit(&self, text: &str) {
        let _ = self.edited(text);
        self.refresh();
    }
    fn commit(&self, trigger: CommitTrigger) -> Result<(), FieldFailure> {
        self.commit(trigger).map(|_| ())
    }
    fn cancel(&self) {
        self.cancel();
    }
    fn access(&self, value: FieldAccess) {
        if self.snapshot().access != value {
            self.set_access(value);
        }
    }
}
struct TextFields {
    name: FieldBinding<DisplayName, FormIssue>,
    email: FieldBinding<EmailAddress, FormIssue>,
    website: FieldBinding<Website, FormIssue>,
    phone: FieldBinding<PhoneNumber, FormIssue>,
    password: FieldBinding<SecretText, FormIssue>,
    search: FieldBinding<SearchQuery, FormIssue>,
    biography: FieldBinding<Biography, FormIssue>,
    otp: FieldBinding<OtpValue, FormIssue>,
    reference: FieldBinding<MaskedValue, FormIssue>,
}
impl TextFields {
    fn get(&self, field: TextExample) -> &dyn FieldCommands {
        match field {
            TextExample::Name => &self.name,
            TextExample::Email => &self.email,
            TextExample::Website => &self.website,
            TextExample::Phone => &self.phone,
            TextExample::Password => &self.password,
            TextExample::Search => &self.search,
            TextExample::Biography => &self.biography,
            TextExample::Otp => &self.otp,
            TextExample::Reference => &self.reference,
        }
    }
    fn restore(&self, saved: &ProfilePayload) {
        self.name.reset(Some(saved.name.clone()));
        self.email.reset(Some(saved.email.clone()));
        self.website.reset(saved.website.clone());
        self.phone.reset(Some(saved.phone.clone()));
        self.password.reset(Some(saved.password.clone()));
        self.search.reset(None);
        self.biography.reset(saved.biography.clone());
        self.otp.reset(saved.otp.clone());
        self.reference.reset(saved.reference.clone());
    }
}
struct NumericFields {
    integer: FieldBinding<i64, FormIssue>,
    decimal: FieldBinding<DecimalValue, FormIssue>,
    money: FieldBinding<Money, FormIssue>,
    percent: FieldBinding<Percent, FormIssue>,
    quantity: FieldBinding<Quantity<DistanceUnit>, FormIssue>,
    duration: FieldBinding<DurationValue, FormIssue>,
    stepper: FieldBinding<i64, FormIssue>,
    locale: Cell<NumberLocale>,
}
impl NumericFields {
    fn get(&self, field: NumericExample) -> &dyn FieldCommands {
        match field {
            NumericExample::Integer => &self.integer,
            NumericExample::Decimal => &self.decimal,
            NumericExample::Money => &self.money,
            NumericExample::Percent => &self.percent,
            NumericExample::Quantity => &self.quantity,
            NumericExample::Duration => &self.duration,
            NumericExample::Stepper => &self.stepper,
        }
    }
    fn restore(&self, saved: &BillingPayload) {
        self.integer.reset(Some(saved.people));
        self.decimal.reset(Some(saved.decimal));
        self.money.reset(Some(saved.budget.clone()));
        self.percent.reset(Some(saved.allocation));
        self.quantity.reset(Some(saved.distance.clone()));
        self.duration.reset(Some(saved.duration));
        self.stepper.reset(Some(saved.seats));
    }
    fn change_locale(&self, locale: NumberLocale) {
        self.locale.set(locale);
        self.integer.reconfigure(
            move |s| parse_integer(s, locale, Some(people_bounds())),
            move |n| integer_text(*n, locale),
        );
        self.decimal.reconfigure(
            move |s| decimal_rules(locale, 4, SignPolicy::Signed, None).parse(s),
            move |n| decimal_text(*n, locale, 4),
        );
        self.money.reconfigure(
            move |s| parse_money(s, locale),
            move |n| decimal_text(n.amount, locale, 2),
        );
        self.percent.reconfigure(
            move |s| parse_percent(s, locale),
            move |n| decimal_text(n.0, locale, 2),
        );
        self.quantity.reconfigure(
            move |s| parse_quantity(s, locale),
            move |n| decimal_text(n.value, locale, 3),
        );
        self.duration.reconfigure(
            move |s| parse_duration(s, locale),
            move |n| duration_text(*n, locale),
        );
        self.stepper.reconfigure(
            move |s| parse_integer(s, locale, Some(seat_bounds())),
            move |n| integer_text(*n, locale),
        );
    }
}
struct Examples {
    text: TextFields,
    numbers: NumericFields,
    profile_form: RefCell<FormCoordinator<ProfileField, FormIssue>>,
    billing_form: RefCell<FormCoordinator<NumericExample, FormIssue>>,
    saved_profile: RefCell<ProfilePayload>,
    saved_billing: RefCell<BillingPayload>,
    profile_error_generation: Cell<i32>,
    billing_error_generation: Cell<i32>,
    profile_error_fields: RefCell<Vec<ProfileField>>,
    billing_error_fields: RefCell<Vec<NumericExample>>,
}

fn view_access(access: ui::FieldAccess) -> FieldAccess {
    match access {
        ui::FieldAccess::Editable => FieldAccess::Editable,
        ui::FieldAccess::ReadOnly => FieldAccess::ReadOnly,
        ui::FieldAccess::Disabled => FieldAccess::Disabled,
    }
}
fn view_validation(value: ValidationPresentation) -> ui::ValidationPresentation {
    match value {
        ValidationPresentation::Pristine => ui::ValidationPresentation::Pristine,
        ValidationPresentation::Checking => ui::ValidationPresentation::Checking,
        ValidationPresentation::Valid => ui::ValidationPresentation::Valid,
        ValidationPresentation::Invalid => ui::ValidationPresentation::Invalid,
    }
}
fn locale(value: LocaleExample) -> NumberLocale {
    match value {
        LocaleExample::FrenchCanada => NumberLocale::FrCa,
        LocaleExample::FrenchFrance => NumberLocale::FrFr,
        LocaleExample::EnglishCanada => NumberLocale::EnCa,
        LocaleExample::EnglishUs => NumberLocale::EnUs,
    }
}
fn publisher(window: &Showcase, field: FieldView) -> impl Fn(FieldProjection) + 'static {
    let weak = window.as_weak();
    move |snapshot| {
        let Some(window) = weak.upgrade() else { return };
        let view = window.global::<FormExamples>();
        // Setting an unchanged draft would be an unnecessary external editor update.
        macro_rules! project {
            ($get:ident,$text:ident,$validation:ident,$error:ident,$count:ident) => {{
                if view.$get().as_str() != snapshot.draft {
                    view.$text(snapshot.draft.as_str().into());
                }
                view.$validation(view_validation(snapshot.validation));
                view.$error(snapshot.error.as_str().into());
                view.$count(i32::try_from(snapshot.character_count).unwrap_or(i32::MAX));
            }};
        }
        match field {
            FieldView::Text(TextExample::Name) => project!(
                get_name_text,
                set_name_text,
                set_name_validation,
                set_name_error,
                set_name_count
            ),
            FieldView::Text(TextExample::Email) => project!(
                get_email_text,
                set_email_text,
                set_email_validation,
                set_email_error,
                set_email_count
            ),
            FieldView::Text(TextExample::Website) => project!(
                get_website_text,
                set_website_text,
                set_website_validation,
                set_website_error,
                set_website_count
            ),
            FieldView::Text(TextExample::Phone) => project!(
                get_phone_text,
                set_phone_text,
                set_phone_validation,
                set_phone_error,
                set_phone_count
            ),
            FieldView::Text(TextExample::Password) => project!(
                get_password_text,
                set_password_text,
                set_password_validation,
                set_password_error,
                set_password_count
            ),
            FieldView::Text(TextExample::Search) => project!(
                get_search_text,
                set_search_text,
                set_search_validation,
                set_search_error,
                set_search_count
            ),
            FieldView::Text(TextExample::Biography) => project!(
                get_biography_text,
                set_biography_text,
                set_biography_validation,
                set_biography_error,
                set_biography_count
            ),
            FieldView::Text(TextExample::Otp) => project!(
                get_otp_text,
                set_otp_text,
                set_otp_validation,
                set_otp_error,
                set_otp_count
            ),
            FieldView::Text(TextExample::Reference) => project!(
                get_reference_text,
                set_reference_text,
                set_reference_validation,
                set_reference_error,
                set_reference_count
            ),
            FieldView::Numeric(NumericExample::Integer) => project!(
                get_integer_text,
                set_integer_text,
                set_integer_validation,
                set_integer_error,
                set_integer_count
            ),
            FieldView::Numeric(NumericExample::Decimal) => project!(
                get_decimal_text,
                set_decimal_text,
                set_decimal_validation,
                set_decimal_error,
                set_decimal_count
            ),
            FieldView::Numeric(NumericExample::Money) => project!(
                get_money_text,
                set_money_text,
                set_money_validation,
                set_money_error,
                set_money_count
            ),
            FieldView::Numeric(NumericExample::Percent) => project!(
                get_percent_text,
                set_percent_text,
                set_percent_validation,
                set_percent_error,
                set_percent_count
            ),
            FieldView::Numeric(NumericExample::Quantity) => project!(
                get_quantity_text,
                set_quantity_text,
                set_quantity_validation,
                set_quantity_error,
                set_quantity_count
            ),
            FieldView::Numeric(NumericExample::Duration) => project!(
                get_duration_text,
                set_duration_text,
                set_duration_validation,
                set_duration_error,
                set_duration_count
            ),
            FieldView::Numeric(NumericExample::Stepper) => project!(
                get_stepper_text,
                set_stepper_text,
                set_stepper_validation,
                set_stepper_error,
                set_stepper_count
            ),
        }
    }
}
fn field<T: Clone + PartialEq + 'static>(
    window: &Showcase,
    target: FieldView,
    accepted: Option<T>,
    absence: AbsencePolicy,
    codec: (
        impl Fn(&str) -> DraftValue<T> + 'static,
        impl Fn(&T) -> String + 'static,
    ),
) -> FieldBinding<T, FormIssue> {
    FieldBinding::new(
        accepted,
        Ownership::Local,
        CommitPolicy::OnBlur,
        absence,
        codec.0,
        codec.1,
        |issue: &FormIssue| issue.message().into(),
        publisher(window, target),
    )
}
fn map_draft<A, B>(
    value: DraftValue<A>,
    map: impl FnOnce(A) -> Result<B, ParseIssue>,
) -> DraftValue<B> {
    match value {
        DraftValue::Empty => DraftValue::Empty,
        DraftValue::Incomplete => DraftValue::Incomplete,
        DraftValue::Invalid(issue) => DraftValue::Invalid(issue),
        DraftValue::Parsed(value) => {
            map(value).map_or_else(DraftValue::Invalid, DraftValue::Parsed)
        }
    }
}
fn plain<T>(text: &str, limit: usize, make: impl FnOnce(String) -> T) -> DraftValue<T> {
    if text.trim().is_empty() {
        return DraftValue::Empty;
    }
    if let Err(issue) = GraphemeLimit::new(limit).validate(text) {
        return DraftValue::Invalid(issue);
    }
    DraftValue::Parsed(make(text.into()))
}
fn parse_email(text: &str) -> DraftValue<EmailAddress> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    if !text.contains('@') || text.ends_with('@') {
        return DraftValue::Incomplete;
    }
    let mut parts = text.split('@');
    let local = parts.next().unwrap_or_default();
    let domain = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || local.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || text.chars().any(char::is_whitespace)
        || text.chars().any(char::is_control)
    {
        DraftValue::Invalid(ParseIssue::InvalidSyntax)
    } else {
        plain(text, 254, EmailAddress)
    }
}
fn parse_website(text: &str) -> DraftValue<Website> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    if matches!(
        text,
        "http:" | "http:/" | "http://" | "https:" | "https:/" | "https://"
    ) {
        return DraftValue::Incomplete;
    }
    match url::Url::parse(text) {
        Ok(url) if matches!(url.scheme(), "http" | "https") && url.host_str().is_some() => {
            DraftValue::Parsed(Website(url))
        }
        Ok(_) | Err(_) => DraftValue::Invalid(ParseIssue::InvalidSyntax),
    }
}
fn parse_phone(text: &str) -> DraftValue<PhoneNumber> {
    if text.is_empty() {
        return DraftValue::Empty;
    }
    // This example's application policy accepts international numbers. It keeps
    // punctuation and leading zeros; it does not claim regional deliverability.
    if !text.starts_with('+')
        || text
            .chars()
            .skip(1)
            .any(|c| !(c.is_ascii_digit() || matches!(c, ' ' | '(' | ')' | '-' | '.')))
    {
        return DraftValue::Invalid(ParseIssue::InvalidSyntax);
    }
    let digits = text.bytes().filter(u8::is_ascii_digit).count();
    if digits < 7 {
        DraftValue::Incomplete
    } else if digits > 15 {
        DraftValue::Invalid(ParseIssue::InvalidSyntax)
    } else {
        DraftValue::Parsed(PhoneNumber(text.into()))
    }
}
fn parse_password(text: &str) -> DraftValue<SecretText> {
    if text.is_empty() {
        DraftValue::Empty
    } else if grapheme_count(text) < 8 {
        DraftValue::Incomplete
    } else {
        plain(text, 128, SecretText)
    }
}
fn reference_mask() -> MaskPattern {
    MaskPattern::new(vec![
        MaskToken::Letter,
        MaskToken::Letter,
        MaskToken::Literal('-'),
        MaskToken::Digit,
        MaskToken::Digit,
        MaskToken::Digit,
        MaskToken::Digit,
    ])
    .expect("the reference mask is statically valid")
}
fn decimal(value: &str) -> DecimalValue {
    DecimalValue::parse_canonical(value).expect("valid sample decimal")
}
fn scale(value: u32) -> DecimalScale {
    DecimalScale::new(value).expect("sample scale is at most four")
}
fn decimal_rules(
    locale: NumberLocale,
    precision: u32,
    sign: SignPolicy,
    bounds: Option<NumericBounds<DecimalValue>>,
) -> DecimalRules {
    DecimalRules {
        locale,
        scale: scale(precision),
        sign,
        bounds,
    }
}
fn people_bounds() -> NumericBounds<i64> {
    NumericBounds::new(1, 100_000).expect("valid people bounds")
}
fn seat_bounds() -> NumericBounds<i64> {
    NumericBounds::new(1, 100).expect("valid seat bounds")
}
fn bounded_decimal(maximum: i64) -> NumericBounds<DecimalValue> {
    NumericBounds::new(DecimalValue::ZERO, DecimalValue::from_integer(maximum))
        .expect("ordered positive bounds")
}
fn decimal_text(value: DecimalValue, locale: NumberLocale, precision: u32) -> String {
    format_decimal(value, locale, scale(precision), RoundingPolicy::Reject)
        .expect("validated samples respect their rendering precision")
}
fn integer_text(value: i64, locale: NumberLocale) -> String {
    decimal_text(DecimalValue::from_integer(value), locale, 0)
}
fn parse_money(text: &str, locale: NumberLocale) -> DraftValue<Money> {
    map_draft(
        decimal_rules(
            locale,
            2,
            SignPolicy::NonNegative,
            Some(bounded_decimal(1_000_000)),
        )
        .parse(text),
        |amount| {
            Ok(Money {
                amount,
                currency: CurrencyMetadata::cad().code,
            })
        },
    )
}
fn parse_percent(text: &str, locale: NumberLocale) -> DraftValue<Percent> {
    map_draft(
        decimal_rules(
            locale,
            2,
            SignPolicy::NonNegative,
            Some(bounded_decimal(100)),
        )
        .parse(text),
        |value| Ok(Percent(value)),
    )
}
fn parse_quantity(text: &str, locale: NumberLocale) -> DraftValue<Quantity<DistanceUnit>> {
    map_draft(
        decimal_rules(
            locale,
            3,
            SignPolicy::NonNegative,
            Some(bounded_decimal(100_000)),
        )
        .parse(text),
        |value| {
            Ok(Quantity {
                value,
                unit: DistanceUnit::Kilometres,
            })
        },
    )
}
fn parse_duration(text: &str, locale: NumberLocale) -> DraftValue<DurationValue> {
    map_draft(
        decimal_rules(
            locale,
            2,
            SignPolicy::NonNegative,
            Some(bounded_decimal(1_440)),
        )
        .parse(text),
        |value| DurationValue::from_quantity(value, DurationUnit::Minutes),
    )
}
fn duration_text(value: DurationValue, locale: NumberLocale) -> String {
    let milliseconds =
        i64::try_from(value.milliseconds()).expect("example durations are bounded to one day");
    let minutes = DecimalValue::from_integer(milliseconds)
        .checked_div(DecimalValue::from_integer(60_000))
        .expect("minute drafts have at most two decimal places");
    decimal_text(minutes, locale, 2)
}
fn initial_profile() -> ProfilePayload {
    ProfilePayload {
        name: DisplayName("Alex Morgan".into()),
        email: EmailAddress("alex@example.com".into()),
        website: Some(Website(
            url::Url::parse("https://example.com").expect("sample URL"),
        )),
        phone: PhoneNumber("+1 514 555 0100".into()),
        password: SecretText("native-editing".into()),
        biography: Some(Biography(
            "I build thoughtful tools for people.\nNative by design, open by choice.".into(),
        )),
        otp: None,
        reference: match reference_mask().parse("SK-0042") {
            DraftValue::Parsed(value) => Some(value),
            DraftValue::Empty | DraftValue::Incomplete | DraftValue::Invalid(_) => {
                unreachable!("the sample reference matches its static mask")
            }
        },
        consent: false,
        notifications: true,
    }
}
fn initial_billing() -> BillingPayload {
    BillingPayload {
        people: 12,
        decimal: decimal("0.30"),
        budget: Money {
            amount: decimal("1234.50"),
            currency: CurrencyMetadata::cad().code,
        },
        allocation: Percent(decimal("12.50")),
        distance: Quantity {
            value: decimal("42.195"),
            unit: DistanceUnit::Kilometres,
        },
        duration: DurationValue::from_milliseconds(5_400_000),
        seats: 3,
    }
}

pub fn install(window: &Showcase) {
    let profile = initial_profile();
    let billing = initial_billing();
    let initial_locale = NumberLocale::FrCa;
    let text = TextFields {
        name: field(
            window,
            FieldView::Text(TextExample::Name),
            Some(profile.name.clone()),
            AbsencePolicy::Required,
            (|s| plain(s, 80, DisplayName), |v: &DisplayName| v.0.clone()),
        ),
        email: field(
            window,
            FieldView::Text(TextExample::Email),
            Some(profile.email.clone()),
            AbsencePolicy::Required,
            (parse_email, |v: &EmailAddress| v.0.clone()),
        ),
        website: field(
            window,
            FieldView::Text(TextExample::Website),
            profile.website.clone(),
            AbsencePolicy::Optional,
            (parse_website, |v: &Website| v.0.as_str().into()),
        ),
        phone: field(
            window,
            FieldView::Text(TextExample::Phone),
            Some(profile.phone.clone()),
            AbsencePolicy::Required,
            (parse_phone, |v: &PhoneNumber| v.0.clone()),
        ),
        password: field(
            window,
            FieldView::Text(TextExample::Password),
            Some(profile.password.clone()),
            AbsencePolicy::Required,
            (parse_password, |v: &SecretText| v.0.clone()),
        ),
        search: field(
            window,
            FieldView::Text(TextExample::Search),
            None,
            AbsencePolicy::Optional,
            (
                |s| plain(s, 100, SearchQuery),
                |v: &SearchQuery| v.0.clone(),
            ),
        ),
        biography: field(
            window,
            FieldView::Text(TextExample::Biography),
            profile.biography.clone(),
            AbsencePolicy::Optional,
            (|s| plain(s, 280, Biography), |v: &Biography| v.0.clone()),
        ),
        otp: field(
            window,
            FieldView::Text(TextExample::Otp),
            profile.otp.clone(),
            AbsencePolicy::Optional,
            (
                |s| {
                    OtpValue::parse(
                        s,
                        OtpLength::new(6).expect("six symbols is bounded"),
                        OtpAlphabet::AsciiDigits,
                    )
                },
                |v: &OtpValue| v.as_str().into(),
            ),
        ),
        reference: field(
            window,
            FieldView::Text(TextExample::Reference),
            profile.reference.clone(),
            AbsencePolicy::Optional,
            (
                |s| reference_mask().parse(s),
                |v: &MaskedValue| v.as_str().into(),
            ),
        ),
    };
    let numbers = NumericFields {
        integer: field(
            window,
            FieldView::Numeric(NumericExample::Integer),
            Some(billing.people),
            AbsencePolicy::Required,
            (
                move |s| parse_integer(s, initial_locale, Some(people_bounds())),
                move |v: &i64| integer_text(*v, initial_locale),
            ),
        ),
        decimal: field(
            window,
            FieldView::Numeric(NumericExample::Decimal),
            Some(billing.decimal),
            AbsencePolicy::Required,
            (
                move |s| decimal_rules(initial_locale, 4, SignPolicy::Signed, None).parse(s),
                move |v: &DecimalValue| decimal_text(*v, initial_locale, 4),
            ),
        ),
        money: field(
            window,
            FieldView::Numeric(NumericExample::Money),
            Some(billing.budget.clone()),
            AbsencePolicy::Required,
            (
                move |s| parse_money(s, initial_locale),
                move |v: &Money| decimal_text(v.amount, initial_locale, 2),
            ),
        ),
        percent: field(
            window,
            FieldView::Numeric(NumericExample::Percent),
            Some(billing.allocation),
            AbsencePolicy::Required,
            (
                move |s| parse_percent(s, initial_locale),
                move |v: &Percent| decimal_text(v.0, initial_locale, 2),
            ),
        ),
        quantity: field(
            window,
            FieldView::Numeric(NumericExample::Quantity),
            Some(billing.distance.clone()),
            AbsencePolicy::Required,
            (
                move |s| parse_quantity(s, initial_locale),
                move |v: &Quantity<DistanceUnit>| decimal_text(v.value, initial_locale, 3),
            ),
        ),
        duration: field(
            window,
            FieldView::Numeric(NumericExample::Duration),
            Some(billing.duration),
            AbsencePolicy::Required,
            (
                move |s| parse_duration(s, initial_locale),
                move |v: &DurationValue| duration_text(*v, initial_locale),
            ),
        ),
        stepper: field(
            window,
            FieldView::Numeric(NumericExample::Stepper),
            Some(billing.seats),
            AbsencePolicy::Required,
            (
                move |s| parse_integer(s, initial_locale, Some(seat_bounds())),
                move |v: &i64| integer_text(*v, initial_locale),
            ),
        ),
        locale: Cell::new(initial_locale),
    };
    let state = Rc::new(Examples {
        text,
        numbers,
        profile_form: RefCell::default(),
        billing_form: RefCell::default(),
        saved_profile: RefCell::new(profile),
        saved_billing: RefCell::new(billing),
        profile_error_generation: Cell::new(0),
        billing_error_generation: Cell::new(0),
        profile_error_fields: RefCell::new(vec![]),
        billing_error_fields: RefCell::new(vec![]),
    });
    install_text(window, &state);
    install_numbers(window, &state);
    install_submission(window, &state);
}

fn invalidate_profile(window: &Showcase, state: &Examples) {
    state.profile_form.borrow_mut().reset();
    profile_errors(window, state, &[]);
    let view = window.global::<FormExamples>();
    view.set_profile_submission(SubmissionPresentation::Idle);
    view.set_profile_status("Your changes stay here until you save.".into());
}
fn invalidate_billing(window: &Showcase, state: &Examples) {
    state.billing_form.borrow_mut().reset();
    billing_errors(window, state, &[]);
    let view = window.global::<FormExamples>();
    view.set_billing_submission(SubmissionPresentation::Idle);
    view.set_billing_status("Your invoice values have changed.".into());
}
fn install_text(window: &Showcase, state: &Rc<Examples>) {
    let view = window.global::<FormExamples>();
    let weak = window.as_weak();
    let state_edit = state.clone();
    view.on_text_edited(move |key, text| {
        if let Some(window) = weak.upgrade() {
            invalidate_profile(&window, &state_edit);
            let binding = state_edit.text.get(key);
            binding.access(view_access(window.global::<FormExamples>().get_access()));
            binding.edit(text.as_str());
        }
    });
    let weak = window.as_weak();
    let state_commit = state.clone();
    view.on_text_commit(move |key, text| {
        if let Some(window) = weak.upgrade() {
            let binding = state_commit.text.get(key);
            binding.access(view_access(window.global::<FormExamples>().get_access()));
            if binding.snapshot().draft != text.as_str() {
                invalidate_profile(&window, &state_commit);
                binding.edit(text.as_str());
            }
            let _ = binding.commit(CommitTrigger::Explicit);
        }
    });
    let weak = window.as_weak();
    let state_blur = state.clone();
    view.on_text_blur(move |key| {
        if let Some(window) = weak.upgrade() {
            let binding = state_blur.text.get(key);
            binding.access(view_access(window.global::<FormExamples>().get_access()));
            let _ = binding.commit(CommitTrigger::Blur);
        }
    });
    let state_cancel = state.clone();
    let weak = window.as_weak();
    view.on_text_cancel(move |key| {
        if let Some(window) = weak.upgrade() {
            invalidate_profile(&window, &state_cancel);
        }
        state_cancel.text.get(key).cancel();
    });
}
fn install_numbers(window: &Showcase, state: &Rc<Examples>) {
    let view = window.global::<FormExamples>();
    let weak = window.as_weak();
    let target = state.clone();
    view.on_numeric_edited(move |key, text| {
        if let Some(window) = weak.upgrade() {
            invalidate_billing(&window, &target);
            let binding = target.numbers.get(key);
            binding.access(FieldAccess::Editable);
            binding.edit(text.as_str());
        }
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_numeric_commit(move |key, text| {
        if let Some(window) = weak.upgrade() {
            let binding = target.numbers.get(key);
            if binding.snapshot().draft != text.as_str() {
                invalidate_billing(&window, &target);
                binding.edit(text.as_str());
            }
            let _ = binding.commit(CommitTrigger::Explicit);
        }
    });
    let target = state.clone();
    view.on_numeric_blur(move |key| {
        let _ = target.numbers.get(key).commit(CommitTrigger::Blur);
    });
    let target = state.clone();
    let weak = window.as_weak();
    view.on_numeric_cancel(move |key| {
        if let Some(window) = weak.upgrade() {
            invalidate_billing(&window, &target);
        }
        target.numbers.get(key).cancel();
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_locale_changed(move |selected| {
        if let Some(window) = weak.upgrade() {
            window.global::<FormExamples>().set_locale(selected);
            target.numbers.change_locale(locale(selected));
        }
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_step_requested(move |direction| {
        let Some(window) = weak.upgrade() else { return };
        invalidate_billing(&window, &target);
        if target
            .numbers
            .stepper
            .commit(CommitTrigger::Explicit)
            .is_err()
        {
            window.invoke_focus_form_numeric(NumericExample::Stepper);
            return;
        }
        let current = target.numbers.stepper.accepted().unwrap_or(1);
        let direction = match direction {
            ui::StepDirection::Increment => StepDirection::Increment,
            ui::StepDirection::Decrement => StepDirection::Decrement,
        };
        let next = match direction {
            StepDirection::Increment => current.checked_add(1),
            StepDirection::Decrement => current.checked_sub(1),
        }
        .and_then(|value| seat_bounds().validate(value).ok());
        if let Some(next) = next {
            let _ = target
                .numbers
                .stepper
                .edited(&integer_text(next, target.numbers.locale.get()));
            let _ = target.numbers.stepper.commit(CommitTrigger::Explicit);
        } else {
            let ticket = target.numbers.stepper.begin_validation();
            target
                .numbers
                .stepper
                .finish_validation(ticket, vec![FormIssue::OutOfBounds]);
        }
    });
}
fn submitted<T: Clone + PartialEq + 'static, K: Copy>(
    binding: &FieldBinding<T, FormIssue>,
    key: K,
    errors: &mut Vec<FieldError<K, FormIssue>>,
) -> Option<T> {
    match binding.commit(CommitTrigger::Submit) {
        Ok(Some(FieldIntent::Commit(value))) => Some(value),
        Ok(Some(FieldIntent::Clear)) => None,
        Ok(Some(FieldIntent::Cancel)) | Ok(None) => binding.accepted(),
        Err(_) => {
            errors.push(FieldError {
                field: key,
                error: FormIssue::InvalidValue,
            });
            None
        }
    }
}
fn profile_focus(window: &Showcase, field: ProfileField) {
    if field == ProfileField::Consent {
        window.invoke_focus_form_consent();
        return;
    }
    let target = match field {
        ProfileField::Name => TextExample::Name,
        ProfileField::Email => TextExample::Email,
        ProfileField::Website => TextExample::Website,
        ProfileField::Phone => TextExample::Phone,
        ProfileField::Password => TextExample::Password,
        ProfileField::Biography => TextExample::Biography,
        ProfileField::Otp => TextExample::Otp,
        ProfileField::Reference => TextExample::Reference,
        ProfileField::Consent => TextExample::Name,
    };
    window.invoke_focus_form_text(target);
}
fn profile_payload(
    window: &Showcase,
    fields: &TextFields,
) -> Result<ProfilePayload, Vec<FieldError<ProfileField, FormIssue>>> {
    let mut errors = vec![];
    let name = submitted(&fields.name, ProfileField::Name, &mut errors);
    let email = submitted(&fields.email, ProfileField::Email, &mut errors);
    let website = submitted(&fields.website, ProfileField::Website, &mut errors);
    let phone = submitted(&fields.phone, ProfileField::Phone, &mut errors);
    let password = submitted(&fields.password, ProfileField::Password, &mut errors);
    let biography = submitted(&fields.biography, ProfileField::Biography, &mut errors);
    let otp = submitted(&fields.otp, ProfileField::Otp, &mut errors);
    let reference = submitted(&fields.reference, ProfileField::Reference, &mut errors);
    let view = window.global::<FormExamples>();
    let consent = match view.get_consent() {
        ui::CheckState::Checked => true,
        ui::CheckState::Unchecked | ui::CheckState::Mixed => false,
    };
    if !consent {
        errors.push(FieldError {
            field: ProfileField::Consent,
            error: FormIssue::ConsentRequired,
        });
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let (Some(name), Some(email), Some(phone), Some(password)) = (name, email, phone, password)
    else {
        return Err(vec![FieldError {
            field: ProfileField::Name,
            error: FormIssue::InvalidValue,
        }]);
    };
    Ok(ProfilePayload {
        name,
        email,
        website,
        phone,
        password,
        biography,
        otp,
        reference,
        consent,
        notifications: view.get_notifications(),
    })
}
fn billing_payload(
    fields: &NumericFields,
) -> Result<BillingPayload, Vec<FieldError<NumericExample, FormIssue>>> {
    let mut errors = vec![];
    let people = submitted(&fields.integer, NumericExample::Integer, &mut errors);
    let decimal = submitted(&fields.decimal, NumericExample::Decimal, &mut errors);
    let budget = submitted(&fields.money, NumericExample::Money, &mut errors);
    let allocation = submitted(&fields.percent, NumericExample::Percent, &mut errors);
    let distance = submitted(&fields.quantity, NumericExample::Quantity, &mut errors);
    let duration = submitted(&fields.duration, NumericExample::Duration, &mut errors);
    let seats = submitted(&fields.stepper, NumericExample::Stepper, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    let (
        Some(people),
        Some(decimal),
        Some(budget),
        Some(allocation),
        Some(distance),
        Some(duration),
        Some(seats),
    ) = (
        people, decimal, budget, allocation, distance, duration, seats,
    )
    else {
        return Err(vec![FieldError {
            field: NumericExample::Integer,
            error: FormIssue::InvalidValue,
        }]);
    };
    Ok(BillingPayload {
        people,
        decimal,
        budget,
        allocation,
        distance,
        duration,
        seats,
    })
}
fn install_submission(window: &Showcase, state: &Rc<Examples>) {
    let view = window.global::<FormExamples>();
    let weak = window.as_weak();
    let target = state.clone();
    view.on_profile_error_activated(move |request| {
        if !request.present
            || request.generation != target.profile_error_generation.get()
            || request.request != request.generation
        {
            return;
        }
        let Some(field) = profile_from_token(request.token) else {
            return;
        };
        if !target.profile_error_fields.borrow().contains(&field) {
            return;
        }
        if let Some(window) = weak.upgrade() {
            profile_focus(&window, field);
        }
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_billing_error_activated(move |request| {
        if !request.present
            || request.generation != target.billing_error_generation.get()
            || request.request != request.generation
        {
            return;
        }
        let Some(field) = numeric_from_token(request.token) else {
            return;
        };
        if !target.billing_error_fields.borrow().contains(&field) {
            return;
        }
        if let Some(window) = weak.upgrade() {
            window.invoke_focus_form_numeric(field);
        }
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_profile_submit(move || {
        let Some(window) = weak.upgrade() else { return };
        let view = window.global::<FormExamples>();
        if view.get_access() != ui::FieldAccess::Editable
            || view.get_profile_submission() == SubmissionPresentation::Pending
        {
            return;
        }
        // Synchronize app access even when its UI toggle never generated an edit.
        for key in [
            TextExample::Name,
            TextExample::Email,
            TextExample::Website,
            TextExample::Phone,
            TextExample::Password,
            TextExample::Search,
            TextExample::Biography,
            TextExample::Otp,
            TextExample::Reference,
        ] {
            target.text.get(key).access(FieldAccess::Editable);
        }
        let payload = match profile_payload(&window, &target.text) {
            Ok(payload) => payload,
            Err(errors) => {
                profile_errors(&window, &target, &errors);
                view.set_profile_submission(SubmissionPresentation::Invalid);
                view.set_profile_status(errors[0].error.message().into());
                profile_focus(&window, errors[0].field);
                return;
            }
        };
        let Some(FormIntent::Submit { ticket, payload }) =
            target.profile_form.borrow_mut().submit(payload)
        else {
            return;
        };
        let validation = target.text.email.begin_validation();
        view.set_profile_submission(SubmissionPresentation::Pending);
        view.set_profile_status("Checking your email before saving…".into());
        let weak = window.as_weak();
        let target = target.clone();
        slint::Timer::single_shot(Duration::from_millis(350), move || {
            let Some(window) = weak.upgrade() else { return };
            let errors = if payload.email.0.eq_ignore_ascii_case("taken@example.com") {
                vec![FormIssue::EmailUnavailable]
            } else {
                vec![]
            };
            if !target
                .text
                .email
                .finish_validation(validation, errors.clone())
            {
                return;
            }
            let view = window.global::<FormExamples>();
            let selection_changed = (view.get_consent() == ui::CheckState::Checked)
                != payload.consent
                || view.get_notifications() != payload.notifications;
            if selection_changed {
                target.profile_form.borrow_mut().reset();
                view.set_profile_submission(SubmissionPresentation::Idle);
                view.set_profile_status(
                    "Your choices changed. Save again when you are ready.".into(),
                );
                return;
            }
            let errors = errors
                .into_iter()
                .map(|error| FieldError {
                    field: ProfileField::Email,
                    error,
                })
                .collect::<Vec<_>>();
            if !target
                .profile_form
                .borrow_mut()
                .finish(ticket, errors.clone())
            {
                return;
            }
            profile_errors(&window, &target, &errors);
            if let Some(error) = errors.first() {
                view.set_profile_submission(SubmissionPresentation::Invalid);
                view.set_profile_status(error.error.message().into());
                profile_focus(&window, error.field);
            } else {
                *target.saved_profile.borrow_mut() = payload;
                view.set_profile_submission(SubmissionPresentation::Succeeded);
                view.set_profile_status("Profile saved. Your changes are ready.".into());
            }
        });
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_profile_reset(move || {
        if let Some(window) = weak.upgrade() {
            target.profile_form.borrow_mut().reset();
            profile_errors(&window, &target, &[]);
            let saved = target.saved_profile.borrow().clone();
            target.text.restore(&saved);
            let view = window.global::<FormExamples>();
            view.set_consent(if saved.consent {
                ui::CheckState::Checked
            } else {
                ui::CheckState::Unchecked
            });
            view.set_notifications(saved.notifications);
            view.set_profile_submission(SubmissionPresentation::Idle);
            view.set_profile_status("Restored the last saved profile.".into());
        }
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_billing_submit(move || {
        let Some(window) = weak.upgrade() else { return };
        let view = window.global::<FormExamples>();
        if view.get_billing_submission() == SubmissionPresentation::Pending {
            return;
        }
        let payload = match billing_payload(&target.numbers) {
            Ok(payload) => payload,
            Err(errors) => {
                billing_errors(&window, &target, &errors);
                view.set_billing_submission(SubmissionPresentation::Invalid);
                view.set_billing_status(errors[0].error.message().into());
                window.invoke_focus_form_numeric(errors[0].field);
                return;
            }
        };
        let Some(FormIntent::Submit { ticket, payload }) =
            target.billing_form.borrow_mut().submit(payload)
        else {
            return;
        };
        view.set_billing_submission(SubmissionPresentation::Pending);
        view.set_billing_status("Creating your invoice…".into());
        let weak = window.as_weak();
        let target = target.clone();
        slint::Timer::single_shot(Duration::from_millis(250), move || {
            if let Some(window) = weak.upgrade() {
                if !target.billing_form.borrow_mut().finish(ticket, vec![]) {
                    return;
                }
                let view = window.global::<FormExamples>();
                let budget = payload
                    .budget
                    .format(
                        &CurrencyMetadata::cad(),
                        target.numbers.locale.get(),
                        RoundingPolicy::Reject,
                    )
                    .expect("validated CAD amount retains two decimal places");
                view.set_billing_status(
                    format!("Invoice created: {budget} CAD · {} seats.", payload.seats).into(),
                );
                view.set_billing_submission(SubmissionPresentation::Succeeded);
                billing_errors(&window, &target, &[]);
                *target.saved_billing.borrow_mut() = payload;
            }
        });
    });
    let weak = window.as_weak();
    let target = state.clone();
    view.on_billing_reset(move || {
        if let Some(window) = weak.upgrade() {
            target.billing_form.borrow_mut().reset();
            billing_errors(&window, &target, &[]);
            target.numbers.restore(&target.saved_billing.borrow());
            let view = window.global::<FormExamples>();
            view.set_billing_submission(SubmissionPresentation::Idle);
            view.set_billing_status("Restored the last accepted invoice values.".into());
        }
    });
}

fn profile_token(field: ProfileField) -> i32 {
    match field {
        ProfileField::Name => 1,
        ProfileField::Email => 2,
        ProfileField::Website => 3,
        ProfileField::Phone => 4,
        ProfileField::Password => 5,
        ProfileField::Biography => 7,
        ProfileField::Otp => 8,
        ProfileField::Reference => 9,
        ProfileField::Consent => 100,
    }
}
fn profile_from_token(token: i32) -> Option<ProfileField> {
    match token {
        1 => Some(ProfileField::Name),
        2 => Some(ProfileField::Email),
        3 => Some(ProfileField::Website),
        4 => Some(ProfileField::Phone),
        5 => Some(ProfileField::Password),
        7 => Some(ProfileField::Biography),
        8 => Some(ProfileField::Otp),
        9 => Some(ProfileField::Reference),
        100 => Some(ProfileField::Consent),
        _ => None,
    }
}
fn numeric_token(field: NumericExample) -> i32 {
    match field {
        NumericExample::Integer => 1,
        NumericExample::Decimal => 2,
        NumericExample::Money => 3,
        NumericExample::Percent => 4,
        NumericExample::Quantity => 5,
        NumericExample::Duration => 6,
        NumericExample::Stepper => 7,
    }
}
fn numeric_from_token(token: i32) -> Option<NumericExample> {
    match token {
        1 => Some(NumericExample::Integer),
        2 => Some(NumericExample::Decimal),
        3 => Some(NumericExample::Money),
        4 => Some(NumericExample::Percent),
        5 => Some(NumericExample::Quantity),
        6 => Some(NumericExample::Duration),
        7 => Some(NumericExample::Stepper),
        _ => None,
    }
}
fn profile_label(field: ProfileField) -> &'static str {
    match field {
        ProfileField::Name => "Display name",
        ProfileField::Email => "Email",
        ProfileField::Website => "Website",
        ProfileField::Phone => "Phone",
        ProfileField::Password => "Password",
        ProfileField::Biography => "About you",
        ProfileField::Otp => "Verification code",
        ProfileField::Reference => "Project reference",
        ProfileField::Consent => "Project terms",
    }
}
fn numeric_label(field: NumericExample) -> &'static str {
    match field {
        NumericExample::Integer => "Team size",
        NumericExample::Decimal => "Exact decimal",
        NumericExample::Money => "Monthly budget",
        NumericExample::Percent => "Allocation",
        NumericExample::Quantity => "Distance",
        NumericExample::Duration => "Focus session",
        NumericExample::Stepper => "Seats",
    }
}
fn summary_generation(counter: &Cell<i32>) -> i32 {
    let value = counter
        .get()
        .checked_add(1)
        .expect("summary generation exhausted");
    counter.set(value);
    value
}
fn profile_errors(
    window: &Showcase,
    state: &Examples,
    errors: &[FieldError<ProfileField, FormIssue>],
) {
    let generation = summary_generation(&state.profile_error_generation);
    *state.profile_error_fields.borrow_mut() = errors.iter().map(|error| error.field).collect();
    let rows = errors
        .iter()
        .map(|error| {
            let snapshot = match error.field {
                ProfileField::Name => Some(state.text.name.snapshot()),
                ProfileField::Email => Some(state.text.email.snapshot()),
                ProfileField::Website => Some(state.text.website.snapshot()),
                ProfileField::Phone => Some(state.text.phone.snapshot()),
                ProfileField::Password => Some(state.text.password.snapshot()),
                ProfileField::Biography => Some(state.text.biography.snapshot()),
                ProfileField::Otp => Some(state.text.otp.snapshot()),
                ProfileField::Reference => Some(state.text.reference.snapshot()),
                ProfileField::Consent => None,
            };
            let detail = snapshot
                .filter(|snapshot| !snapshot.error.is_empty())
                .map_or_else(
                    || error.error.message().to_string(),
                    |snapshot| snapshot.error,
                );
            ui::FormErrorRow {
                target: ui::FocusRequest {
                    present: true,
                    token: profile_token(error.field),
                    generation,
                    request: generation,
                },
                message: format!("{}: {detail}", profile_label(error.field)).into(),
            }
        })
        .collect::<Vec<_>>();
    window
        .global::<FormExamples>()
        .set_profile_errors(slint::ModelRc::new(slint::VecModel::from(rows)));
}
fn billing_errors(
    window: &Showcase,
    state: &Examples,
    errors: &[FieldError<NumericExample, FormIssue>],
) {
    let generation = summary_generation(&state.billing_error_generation);
    *state.billing_error_fields.borrow_mut() = errors.iter().map(|error| error.field).collect();
    let rows = errors
        .iter()
        .map(|error| {
            let snapshot = state.numbers.get(error.field).snapshot();
            let detail = if snapshot.error.is_empty() {
                error.error.message().to_string()
            } else {
                snapshot.error
            };
            ui::FormErrorRow {
                target: ui::FocusRequest {
                    present: true,
                    token: numeric_token(error.field),
                    generation,
                    request: generation,
                },
                message: format!("{}: {detail}", numeric_label(error.field)).into(),
            }
        })
        .collect::<Vec<_>>();
    window
        .global::<FormExamples>()
        .set_billing_errors(slint::ModelRc::new(slint::VecModel::from(rows)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::{Model, platform::WindowEvent};

    fn fixture(page: ui::ShowcasePage) -> Showcase {
        i_slint_backend_testing::init_no_event_loop();
        let window = Showcase::new().unwrap();
        window.set_page(page);
        install(&window);
        window.show().unwrap();
        window
    }
    fn settle(milliseconds: u64) {
        i_slint_backend_testing::mock_elapsed_time(Duration::from_millis(milliseconds));
        slint::platform::update_timers_and_animations();
    }
    #[test]
    fn invalid_profile_summary_focuses_native_editor_and_rejects_stale_links() {
        let window = fixture(ui::ShowcasePage::Profile);
        let view = window.global::<FormExamples>();
        view.invoke_text_edited(TextExample::Name, "".into());
        view.invoke_profile_submit();
        settle(1);
        assert_eq!(
            view.get_profile_submission(),
            SubmissionPresentation::Invalid
        );
        let errors = view.get_profile_errors();
        assert_eq!(
            errors.row_count(),
            2,
            "name and consent both need attention"
        );
        let old_request = errors.row_data(0).unwrap().target;
        assert!(
            errors
                .row_data(0)
                .unwrap()
                .message
                .starts_with("Display name:")
        );
        window.window().dispatch_event(WindowEvent::KeyPressed {
            text: "Jane".into(),
        });
        window.window().dispatch_event(WindowEvent::KeyReleased {
            text: "Jane".into(),
        });
        assert_eq!(
            view.get_name_text().as_str(),
            "Jane",
            "submit focused the real first invalid editor"
        );
        assert_eq!(view.get_profile_errors().row_count(), 0);
        let sequence = view.get_text_focus_sequence();
        view.invoke_profile_error_activated(old_request);
        assert_eq!(
            view.get_text_focus_sequence(),
            sequence,
            "an obsolete summary cannot move focus"
        );
    }
    #[test]
    fn profile_async_validation_ignores_old_reply_and_reset_restores_saved_snapshot() {
        let window = fixture(ui::ShowcasePage::Profile);
        let view = window.global::<FormExamples>();
        view.set_consent(ui::CheckState::Checked);
        view.invoke_text_edited(TextExample::Email, "taken@example.com".into());
        view.invoke_profile_submit();
        assert_eq!(
            view.get_profile_submission(),
            SubmissionPresentation::Pending
        );
        view.invoke_text_edited(TextExample::Email, "fresh@example.com".into());
        view.invoke_profile_submit();
        settle(400);
        assert_eq!(
            view.get_profile_submission(),
            SubmissionPresentation::Succeeded
        );
        assert_eq!(view.get_profile_errors().row_count(), 0);
        assert_eq!(
            view.get_email_validation(),
            ui::ValidationPresentation::Valid
        );
        view.invoke_text_edited(TextExample::Name, "Unsaved name".into());
        view.invoke_text_edited(TextExample::Email, "another@example.com".into());
        view.invoke_profile_reset();
        assert_eq!(view.get_name_text().as_str(), "Alex Morgan");
        assert_eq!(view.get_email_text().as_str(), "fresh@example.com");
        assert_eq!(view.get_consent(), ui::CheckState::Checked);
        assert_eq!(view.get_profile_submission(), SubmissionPresentation::Idle);
    }
    #[test]
    fn billing_partial_draft_survives_locale_change_then_exact_invoice_snapshot_resets() {
        let window = fixture(ui::ShowcasePage::Billing);
        let view = window.global::<FormExamples>();
        view.invoke_numeric_edited(NumericExample::Money, "1,".into());
        view.invoke_locale_changed(LocaleExample::EnglishUs);
        assert_eq!(view.get_money_text().as_str(), "1,");
        view.invoke_numeric_commit(NumericExample::Money, "1,".into());
        assert_eq!(
            view.get_money_validation(),
            ui::ValidationPresentation::Invalid
        );
        assert_eq!(view.get_money_text().as_str(), "1,");
        view.invoke_numeric_cancel(NumericExample::Money);
        assert_eq!(view.get_money_text().as_str(), "1,234.50");
        view.invoke_numeric_edited(NumericExample::Money, "0.10".into());
        view.invoke_billing_submit();
        assert_eq!(
            view.get_billing_submission(),
            SubmissionPresentation::Pending
        );
        settle(300);
        assert_eq!(
            view.get_billing_submission(),
            SubmissionPresentation::Succeeded
        );
        assert!(view.get_billing_status().contains("$0.10 CAD"));
        view.invoke_numeric_edited(NumericExample::Money, "0.20".into());
        view.invoke_billing_reset();
        assert_eq!(view.get_money_text().as_str(), "0.10");
    }
    #[test]
    fn readonly_profile_cannot_be_changed_through_application_edit_callback() {
        let window = fixture(ui::ShowcasePage::Profile);
        let view = window.global::<FormExamples>();
        view.set_access(ui::FieldAccess::ReadOnly);
        view.invoke_text_edited(TextExample::Name, "Unexpected replacement".into());
        assert_eq!(view.get_name_text().as_str(), "Alex Morgan");
        view.invoke_profile_submit();
        assert_eq!(view.get_profile_submission(), SubmissionPresentation::Idle);
    }
}
