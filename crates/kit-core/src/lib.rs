//! Domain values and shared interaction rules for Slint Kit, independent of
//! rendering and platform APIs: exact decimals and money, HTML form constraints,
//! field state, stable selection identity and generation guards.
//!
//! Applications normally use the `slint-kit` facade, which re-exports these types.
//! See [why Slint Kit](https://slint-kit.dev/guides/why-slint-kit/),
//! [forms](https://slint-kit.dev/guides/forms/) and
//! [typed models](https://slint-kit.dev/guides/controllers/).
#![forbid(unsafe_code)]
#![deny(clippy::wildcard_enum_match_arm)]

mod choices;
mod color;
mod field;
mod file;
mod focus;
mod geometry;
mod host;
mod html;
mod notification;
mod numeric;
mod overlay;
mod phone;
mod select;
mod temporal;
mod text;

pub use choices::*;
pub use color::*;
pub use field::*;
pub use file::*;
pub use focus::*;
pub use geometry::*;
pub use host::*;
pub use html::*;
pub use notification::*;
pub use numeric::*;
pub use overlay::*;
pub use phone::*;
pub use select::*;
pub use temporal::*;
pub use text::*;

macro_rules! contract_enums {
    ($( $rust_name:ident => $slint_name:literal { $( $variant:ident => $slint_variant:literal ),+ $(,)? } )+) => {
        $(
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub enum $rust_name { $( $variant ),+ }
        )+
        pub const SLINT_ENUM_CONTRACTS: &[(&str, &[&str])] = &[
            $( ($slint_name, &[$($slint_variant),+]) ),+
        ];
    };
}

contract_enums! {
    FieldAction => "FieldAction" { Edit => "edit", Blur => "blur", Submit => "submit", Cancel => "cancel" }
    Ownership => "Ownership" { Local => "local", Controlled => "controlled" }
    WeekStart => "WeekStart" { Monday => "monday", Tuesday => "tuesday", Wednesday => "wednesday", Thursday => "thursday", Friday => "friday", Saturday => "saturday", Sunday => "sunday" }
    PhoneNumberMode => "PhoneNumberMode" { National => "national", International => "international", Either => "either" }
    NumberInputFormat => "NumberInputFormat" { Canonical => "canonical", DecimalComma => "decimal-comma" }
    HtmlInputKind => "HtmlInputKind" {
        Text => "text", Search => "search", Telephone => "telephone", Email => "email", Url => "url",
        Password => "password", TextArea => "text-area", Number => "number", Date => "date", Month => "month",
        Week => "week", Time => "time", DateTimeLocal => "date-time-local", Checkbox => "checkbox",
        Radio => "radio", Select => "select", File => "file", Range => "range", Color => "color",
    }
    CheckState => "CheckState" { Unchecked => "unchecked", Checked => "checked", Mixed => "mixed" }
    RangeThumb => "RangeThumb" { Start => "lower", End => "upper" }
    FileSelectionMode => "FileSelectionMode" { Single => "single", Multiple => "multiple" }
    CalendarMove => "CalendarNavigation" {
        PreviousDay => "previous-day", NextDay => "next-day", PreviousWeek => "previous-week", NextWeek => "next-week",
        WeekStart => "week-start", WeekEnd => "week-end", PreviousMonth => "previous-month", NextMonth => "next-month",
        PreviousYear => "previous-year", NextYear => "next-year",
    }
    TimePrecision => "TimePrecision" { Minute => "minute", Second => "second" }
    ClockFormat => "ClockFormat" { TwentyFourHour => "twenty-four-hour", TwelveHour => "twelve-hour" }
    AlphaPolicy => "ColorAlphaMode" { Opaque => "opaque", Editable => "editable" }
    TextPurpose => "TextPurpose" {
        Plain => "plain", Email => "email", Url => "url", Telephone => "telephone", Password => "password", Search => "search",
    }
    SecretVisibility => "SecretVisibility" { Hidden => "hidden", Revealed => "revealed" }
    StepDirection => "StepDirection" { Increment => "increment", Decrement => "decrement" }
    NumberStepStatus => "NumberStepStatus" { Unavailable => "unavailable", Unchanged => "unchanged", Stepped => "stepped" }
    FieldAccess => "FieldAccess" {
        Editable => "editable", ReadOnly => "read-only", Disabled => "disabled",
    }
    CommitPolicy => "CommitPolicy" {
        OnBlur => "on-blur", OnSubmit => "on-submit", OnChangeWhenValid => "on-change-when-valid",
    }
    ValidationPresentation => "ValidationPresentation" {
        Pristine => "pristine", Checking => "checking", Valid => "valid", Invalid => "invalid",
    }
    NavigationStep => "NavigationStep" {
        Next => "next", Previous => "previous", First => "first", Last => "last",
        PageNext => "page-next", PagePrevious => "page-previous",
    }
    SelectionAvailability => "SelectionAvailability" {
        Absent => "absent", Available => "available", Unavailable => "unavailable",
    }
    SelectionRetention => "SelectionRetention" { BoundToOptions => "bound-to-options", Retain => "retain" }
    DismissReason => "CloseReason" {
        Escape => "escape", OutsidePress => "outside-press", SystemBack => "system-back",
        TriggerToggle => "trigger-toggle", Programmatic => "programmatic", HostClosed => "host-closed",
    }
}
