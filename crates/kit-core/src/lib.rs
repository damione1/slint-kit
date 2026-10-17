//! Deterministic interaction state, independent of rendering and platform APIs.
#![forbid(unsafe_code)]
#![deny(clippy::wildcard_enum_match_arm)]

mod choices;
mod color;
mod field;
mod focus;
mod geometry;
mod host;
mod numeric;
mod overlay;
mod select;
mod temporal;
mod text;

pub use choices::*;
pub use color::*;
pub use field::*;
pub use focus::*;
pub use geometry::*;
pub use host::*;
pub use numeric::*;
pub use overlay::*;
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
    DismissReason => "CloseReason" {
        Escape => "escape", OutsidePress => "outside-press", SystemBack => "system-back",
        TriggerToggle => "trigger-toggle", Programmatic => "programmatic", HostClosed => "host-closed",
    }
}
