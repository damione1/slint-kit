// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use kit_core::{
    HtmlConstraints, HtmlInputState, HtmlPattern, HtmlValidity, MaskPattern, ParseIssue,
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

/// Window-local, bounded pattern cache. Patterns are authored by the application.
#[derive(Default)]
pub struct HtmlValidationEngine {
    patterns: RefCell<VecDeque<(String, Result<HtmlPattern, String>)>>,
    masks: RefCell<VecDeque<CachedMask>>,
}
struct CachedMask {
    source: String,
    compiled: Result<Rc<MaskPattern>, ParseIssue>,
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
    pub fn mask(&self, source: &str) -> Result<Rc<MaskPattern>, ParseIssue> {
        // Never retain an arbitrarily large invalid source as a cache key.
        if source.len() > MaskPattern::MAX_SOURCE_BYTES {
            return Err(ParseIssue::InvalidSyntax);
        }
        let mut cache = self.masks.borrow_mut();
        if let Some(entry) = cache.iter().find(|entry| entry.source == source) {
            return entry.compiled.clone();
        }
        let result = MaskPattern::compile(source).map(Rc::new);
        if cache.len() == 64 {
            cache.pop_front();
        }
        cache.push_back(CachedMask {
            source: source.into(),
            compiled: result.clone(),
        });
        result
    }

    pub(crate) fn constraints(
        &self,
        c: &crate::ui::KitInputConstraints,
    ) -> (HtmlConstraints, String) {
        use crate::__core::HtmlInputKind as Core;
        use crate::ui;
        use ui::HtmlInputKind as View;
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
            match self.pattern(c.pattern.as_str()) {
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
        let rules = crate::__core::HtmlConstraints {
            kind,
            number_format: match c.number_format {
                ui::NumberInputFormat::Canonical => crate::__core::NumberInputFormat::Canonical,
                ui::NumberInputFormat::DecimalComma => {
                    crate::__core::NumberInputFormat::DecimalComma
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
        (rules, pattern_error)
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
/// Deprecated compatibility spelling; use `install(&window)` with `@kit` imports.
#[deprecated(note = "use slint_kit::install(&window)")]
#[macro_export]
macro_rules! init_form_validation {
    ($ui:ident, $window:expr) => {{
        $crate::install(&$window);
    }};
}

/// Register the shared HTML constraint engine once for this window.
pub(crate) fn install_validation<W>(window: &W)
where
    W: slint::ComponentHandle + 'static,
    for<'a> crate::ui::KitFormValidation<'a>: slint::Global<'a, W>,
{
    use crate::ui;
    let global = window.global::<ui::KitFormValidation>();
    if global.get_ready() {
        return;
    }
    let engine = Rc::new(crate::HtmlValidationEngine::default());
    global.on_grapheme_count(|text| {
        i32::try_from(crate::__core::grapheme_count(text.as_str()))
            .expect("Slint string count fits i32")
    });
    global.on_utf16_count(|text| {
        i32::try_from(text.encode_utf16().count()).expect("Slint string count fits i32")
    });
    global.on_valid_time_zone(|text| crate::__core::TimeZoneId::new(text.as_str()).is_ok());
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
    global.on_choice_enabled(|rows, token| {
        use slint::Model as _;
        crate::choice_token_enabled(rows.iter().map(|row| (row.token, row.disabled)), token)
    });
    global.on_numeric_insertion(|text| crate::__core::html_numeric_insertion(text.as_str()));
    global.on_limit_text_edit(|previous, edited, maximum| {
        let maximum = usize::try_from(maximum).unwrap_or(usize::MAX);
        let limited =
            crate::__core::limit_html_text_edit(previous.as_str(), edited.as_str(), maximum);
        ui::KitTextEdit {
            text: limited.text.into(),
            cursor_byte_offset: i32::try_from(limited.cursor_byte_offset)
                .expect("Slint text offset fits i32"),
        }
    });
    let mask_compile_engine = Rc::clone(&engine);
    global.on_compile_mask(move |source, message| {
        if source.is_empty() {
            return ui::KitMaskRules {
                length: -1,
                placeholder: Default::default(),
                pattern: Default::default(),
                error: Default::default(),
            };
        }
        if let Ok(mask) = mask_compile_engine.mask(source.as_str()) {
            ui::KitMaskRules {
                length: i32::from(mask.utf16_length()),
                placeholder: mask.placeholder().into(),
                pattern: mask.html_pattern_source().into(),
                error: Default::default(),
            }
        } else {
            ui::KitMaskRules {
                length: -1,
                placeholder: Default::default(),
                pattern: Default::default(),
                error: message,
            }
        }
    });
    let mask_insert_engine = Rc::clone(&engine);
    global.on_mask_insertion(move |mask, current, anchor, cursor, inserted| {
        let Ok(compiled) = mask_insert_engine.mask(mask.as_str()) else {
            return false;
        };
        let Ok(anchor) = usize::try_from(anchor) else {
            return false;
        };
        let Ok(cursor) = usize::try_from(cursor) else {
            return false;
        };
        compiled.accepts_key_insertion(current.as_str(), anchor, cursor, inserted.as_str())
    });
    let mask_paste_engine = Rc::clone(&engine);
    global.on_mask_paste(move |mask, current, anchor, cursor, inserted| {
        let Ok(compiled) = mask_paste_engine.mask(mask.as_str()) else {
            return false;
        };
        let Ok(anchor) = usize::try_from(anchor) else {
            return false;
        };
        let Ok(cursor) = usize::try_from(cursor) else {
            return false;
        };
        compiled.accepts_edit(current.as_str(), anchor, cursor, inserted.as_str())
    });
    global.on_phone_insertion(|text| crate::__core::phone_insertion(text.as_str()));
    #[cfg(target_arch = "wasm32")]
    let phone_paste_guard = {
        let weak = window.as_weak();
        crate::guard_web_phone_paste(move || {
            weak.upgrade()
                .is_some_and(|view| view.global::<ui::KitFormValidation>().get_phone_focused())
        })
        .expect("install phone paste guard")
    };
    global.on_phone_paste(move |text| {
        #[cfg(target_arch = "wasm32")]
        let _keep_alive = &phone_paste_guard;
        crate::__core::normalize_phone(text.as_str()).is_ok()
    });
    global.on_phone_value(|text| {
        crate::__core::normalize_phone(text.as_str())
            .unwrap_or_default()
            .into()
    });
    global.on_phone_display(|text, source| {
        let Ok(canonical) = crate::__core::normalize_phone(text.as_str()) else {
            return text;
        };
        if source.is_empty() {
            return canonical.into();
        }
        crate::__core::PhoneDisplayMask::new(source.as_str())
            .map_or(canonical.clone(), |mask| mask.format(&canonical))
            .into()
    });
    global.on_phone_mask_error(|source, message| {
        if source.is_empty() || crate::__core::PhoneDisplayMask::new(source.as_str()).is_ok() {
            "".into()
        } else {
            message
        }
    });
    global.on_phone_error(
        |text, allow_plus, require_plus, min, max, syntax, length, bounds| {
            use crate::__core::{PhoneError, PhoneNumberMode, PhoneRules};
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
                return bounds;
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
        },
    );
    global.on_evaluate(move |c, text, user_edited, checked, selected_count| {
        let (rules, pattern_error) = engine.constraints(&c);
        let (v, value, will_validate) = engine.evaluate(
            &rules,
            text.as_str(),
            crate::__core::HtmlInputState {
                user_edited,
                checked,
                selected_count: usize::try_from(selected_count).unwrap_or(0),
            },
        );
        ui::KitValidationResult {
            value: value.into(),
            will_validate,
            pattern_error: pattern_error.into(),
            hard_error: rules.kind == crate::__core::HtmlInputKind::Email
                && crate::__core::html_email_hard_error(text.as_str(), c.multiple),
            validity: ui::KitValidityState {
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
}

#[cfg(test)]
mod mask_cache_tests {
    use super::HtmlValidationEngine;
    use kit_core::ParseIssue;
    use std::rc::Rc;

    #[test]
    fn caches_compiled_masks_and_evicts_oldest_past_sixty_four_entries() {
        let engine = HtmlValidationEngine::default();
        let first = engine.mask("AA-####").unwrap();
        let cached = engine.mask("AA-####").unwrap();
        assert!(Rc::ptr_eq(&first, &cached));
        assert_eq!(engine.mask("---"), Err(ParseIssue::InvalidSyntax));
        assert_eq!(engine.mask("---"), Err(ParseIssue::InvalidSyntax));

        let engine = HtmlValidationEngine::default();
        let first = engine.mask("A#0").unwrap();
        for index in 1..64 {
            assert!(engine.mask(&format!("A#{index}")).is_ok());
        }
        assert!(Rc::ptr_eq(&first, &engine.mask("A#0").unwrap()));
        assert!(engine.mask("A#64").is_ok());
        assert!(!Rc::ptr_eq(&first, &engine.mask("A#0").unwrap()));
        let entries = engine.masks.borrow().len();
        assert_eq!(
            engine.mask(&"A".repeat(kit_core::MaskPattern::MAX_SOURCE_BYTES + 1)),
            Err(ParseIssue::InvalidSyntax)
        );
        assert_eq!(
            engine.masks.borrow().len(),
            entries,
            "oversized sources are not retained"
        );
    }
}
