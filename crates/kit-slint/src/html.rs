// SPDX-License-Identifier: GPL-3.0-or-later
use kit_core::{HtmlConstraints, HtmlInputState, HtmlPattern, HtmlValidity};
use std::{cell::RefCell, collections::VecDeque};

/// Window-local, bounded pattern cache. Patterns are authored by the application.
#[derive(Default)]
pub struct HtmlValidationEngine {
    patterns: RefCell<VecDeque<(String, Result<HtmlPattern, String>)>>,
}
impl HtmlValidationEngine {
    pub fn pattern(&self, source: &str) -> Result<HtmlPattern, String> {
        let mut cache = self.patterns.borrow_mut();
        if let Some((_, result)) = cache.iter().find(|(key, _)| key == source) {
            return result.clone();
        }
        let result = HtmlPattern::compile(source).map_err(|e| e.to_string());
        if cache.len() == 64 {
            cache.pop_front();
        }
        cache.push_back((source.into(), result.clone()));
        result
    }
    pub fn evaluate(
        &self,
        rules: &HtmlConstraints,
        draft: &str,
        state: HtmlInputState,
    ) -> (HtmlValidity, String, bool) {
        (
            rules.validate(draft, state),
            rules.sanitize(draft),
            rules.will_validate(),
        )
    }
}

/// Install HTML primitive validation once for a generated Slint window.
/// Export KitFormValidation from the root .slint file before calling this macro.
#[macro_export]
macro_rules! init_form_validation {
    ($ui:ident, $window:expr) => {{
        use slint::ComponentHandle as _;
        let window = &$window;
        let global = window.global::<$ui::KitFormValidation>();
        let engine = $crate::HtmlValidationEngine::default();
        global.on_valid_time_zone(|text| $crate::__core::TimeZoneId::new(text.as_str()).is_ok());
        global.on_count_selected(|rows| {
            use slint::Model as _;
            rows.model_tracker().track_row_count_changes();
            let mut count = 0_i32;
            for index in 0..rows.row_count() {
                rows.model_tracker().track_row_data_changes(index);
                if rows.row_data(index).is_some_and(|row| row.selected) {
                    count = count.saturating_add(1);
                }
            }
            count
        });
        global.on_numeric_insertion(|text| $crate::__core::html_numeric_insertion(text.as_str()));
        global.on_phone_insertion(|text| $crate::__core::phone_insertion(text.as_str()));
        #[cfg(target_arch = "wasm32")]
        let phone_paste_guard = {
            let weak = window.as_weak();
            $crate::guard_web_phone_paste(move || {
                weak.upgrade()
                    .is_some_and(|view| view.global::<$ui::KitFormValidation>().get_phone_focused())
            })
            .expect("install phone paste guard")
        };
        global.on_phone_paste(move |text| {
            #[cfg(target_arch = "wasm32")]
            let _keep_alive = &phone_paste_guard;
            $crate::__core::normalize_phone(text.as_str()).is_ok()
        });
        global.on_phone_value(|text| {
            $crate::__core::normalize_phone(text.as_str())
                .unwrap_or_default()
                .into()
        });
        global.on_phone_display(|text, source| {
            let Ok(canonical) = $crate::__core::normalize_phone(text.as_str()) else {
                return text;
            };
            if source.is_empty() {
                return canonical.into();
            }
            $crate::__core::PhoneDisplayMask::new(source.as_str())
                .map_or(canonical.clone(), |mask| mask.format(&canonical))
                .into()
        });
        global.on_phone_mask_error(|source| {
            if source.is_empty() || $crate::__core::PhoneDisplayMask::new(source.as_str()).is_ok() {
                "".into()
            } else {
                "Use 1–15 # digit slots, visual separators and at most one leading +.".into()
            }
        });
        global.on_phone_error(|text, allow_plus, require_plus, min, max, syntax, length| {
            use $crate::__core::{PhoneError, PhoneNumberMode, PhoneRules};
            let mode = if require_plus {
                PhoneNumberMode::International
            } else if allow_plus {
                PhoneNumberMode::Either
            } else {
                PhoneNumberMode::National
            };
            let rules = u8::try_from(min)
                .ok()
                .zip(u8::try_from(max).ok())
                .and_then(|(min, max)| PhoneRules::new(mode, min, max).ok());
            let Some(rules) = rules else {
                return "Phone digit bounds must be between 1 and 15, with min <= max.".into();
            };
            match rules.validate(text.as_str()) {
                Ok(_) => "".into(),
                Err(PhoneError::TooShort | PhoneError::TooLong) => length,
                Err(
                    PhoneError::UnsupportedCharacter
                    | PhoneError::InvalidPrefix
                    | PhoneError::InvalidConfiguration,
                ) => syntax,
            }
        });
        global.on_evaluate(move |c, text, user_edited, checked, selected_count| {
            use $crate::__core::HtmlInputKind as Core;
            use $ui::HtmlInputKind as View;
            let kind = match c.kind {
                View::Text => Core::Text,
                View::Search => Core::Search,
                View::Telephone => Core::Telephone,
                View::Email => Core::Email,
                View::Url => Core::Url,
                View::Password => Core::Password,
                View::TextArea => Core::TextArea,
                View::Number => Core::Number,
                View::Date => Core::Date,
                View::Month => Core::Month,
                View::Week => Core::Week,
                View::Time => Core::Time,
                View::DateTimeLocal => Core::DateTimeLocal,
                View::Checkbox => Core::Checkbox,
                View::Radio => Core::Radio,
                View::Select => Core::Select,
                View::File => Core::File,
                View::Range => Core::Range,
                View::Color => Core::Color,
            };
            let (pattern, pattern_error) = if c.pattern_enabled {
                match engine.pattern(c.pattern.as_str()) {
                    Ok(pattern) => (Some(pattern), String::new()),
                    // HTML ignores an invalid pattern; expose its diagnostic to authors.
                    Err(error) => (None, error),
                }
            } else {
                (None, String::new())
            };
            let attribute = |s: &slint::SharedString| {
                if s.is_empty() {
                    None
                } else {
                    Some(s.to_string())
                }
            };
            let rules = $crate::__core::HtmlConstraints {
                kind,
                number_format: match c.number_format {
                    $ui::NumberInputFormat::Canonical => {
                        $crate::__core::NumberInputFormat::Canonical
                    }
                    $ui::NumberInputFormat::DecimalComma => {
                        $crate::__core::NumberInputFormat::DecimalComma
                    }
                },
                twelve_hour: c.twelve_hour,
                required: c.required,
                multiple: c.multiple,
                min_length: usize::try_from(c.min_length).ok(),
                max_length: usize::try_from(c.max_length).ok(),
                pattern,
                minimum: attribute(&c.min),
                maximum: attribute(&c.max),
                step: attribute(&c.step),
                default_value: c.default_value.to_string(),
                custom_error: c.custom_error.to_string(),
                disabled: c.disabled,
                read_only: c.read_only,
            };
            let (v, value, will_validate) = engine.evaluate(
                &rules,
                text.as_str(),
                $crate::__core::HtmlInputState {
                    user_edited,
                    checked,
                    selected_count: usize::try_from(selected_count).unwrap_or(0),
                },
            );
            $ui::KitValidationResult {
                value: value.into(),
                will_validate,
                pattern_error: pattern_error.into(),
                hard_error: kind == Core::Email
                    && $crate::__core::html_email_hard_error(text.as_str(), c.multiple),
                validity: $ui::KitValidityState {
                    valid: v.valid(),
                    value_missing: v.value_missing,
                    type_mismatch: v.type_mismatch,
                    pattern_mismatch: v.pattern_mismatch,
                    too_long: v.too_long,
                    too_short: v.too_short,
                    range_underflow: v.range_underflow,
                    range_overflow: v.range_overflow,
                    step_mismatch: v.step_mismatch,
                    bad_input: v.bad_input,
                    custom_error: v.custom_error,
                },
            }
        });
        global.set_ready(true);
    }};
}
