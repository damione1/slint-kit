// SPDX-License-Identifier: GPL-3.0-or-later
#![forbid(unsafe_code)]
use kit_core::{
    HtmlConstraints, HtmlInputState, HtmlPattern, HtmlValidity, MaskPattern, ParseIssue,
};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

type CachedTextFilter = (
    (kit_core::TextFilterKind, String),
    Result<Rc<kit_core::TextFilter>, kit_core::ParseIssue>,
);

/// Window-local, bounded pattern cache. Patterns are authored by the application.
#[derive(Default)]
pub struct HtmlValidationEngine {
    patterns: RefCell<VecDeque<(String, Result<HtmlPattern, String>)>>,
    masks: RefCell<VecDeque<CachedMask>>,
    files: RefCell<VecDeque<CachedFileRules>>,
    text_filters: RefCell<VecDeque<CachedTextFilter>>,
}
struct CachedMask {
    source: String,
    compiled: Result<Rc<MaskPattern>, ParseIssue>,
}
struct CachedFileRules {
    accept: String,
    max_files: i32,
    max_size: String,
    rules: Result<Rc<kit_core::FileConstraints>, kit_core::HostError>,
}
fn view_validity(v: HtmlValidity) -> crate::ui::KitValidityState {
    crate::ui::KitValidityState {
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
    }
}
impl HtmlValidationEngine {
    fn file_constraints(
        &self,
        accept: &str,
        max_files: i32,
        max_size: &str,
    ) -> Result<Rc<kit_core::FileConstraints>, kit_core::HostError> {
        if accept.len() > 4096 || max_size.len() > 20 {
            return Err(kit_core::HostError::InvalidFilter);
        }
        let mut cache = self.files.borrow_mut();
        if let Some(entry) = cache.iter().find(|entry| {
            entry.accept == accept && entry.max_files == max_files && entry.max_size == max_size
        }) {
            return entry.rules.clone();
        }
        let parse = || {
            let max_files = if max_files == -1 {
                None
            } else {
                Some(usize::try_from(max_files).map_err(|_| kit_core::HostError::InvalidResource)?)
            };
            let max_size = if max_size.is_empty() {
                None
            } else {
                if !max_size.chars().all(|value| value.is_ascii_digit()) {
                    return Err(kit_core::HostError::InvalidResource);
                }
                Some(
                    max_size
                        .parse::<u64>()
                        .map_err(|_| kit_core::HostError::InvalidResource)?,
                )
            };
            Ok(Rc::new(kit_core::FileConstraints {
                accept: kit_core::FileAccept::parse(accept)?,
                max_files,
                max_size,
            }))
        };
        let rules = parse();
        if cache.len() == 64 {
            cache.pop_front();
        }
        cache.push_back(CachedFileRules {
            accept: accept.into(),
            max_files,
            max_size: max_size.into(),
            rules: rules.clone(),
        });
        rules
    }
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

    /// Parse a view-authored filter once and retain its typed rule or typed error.
    /// HTML acceptance and insertion filtering reuse the same compiled pattern.
    pub fn text_filter(
        &self,
        kind: kit_core::TextFilterKind,
        source: &str,
    ) -> Result<Rc<kit_core::TextFilter>, kit_core::ParseIssue> {
        use kit_core::{ParseIssue, TextFilter, TextFilterKind};
        let mut cache = self.text_filters.borrow_mut();
        if let Some((_, result)) = cache.iter().find(|((cached_kind, cached_source), _)| {
            *cached_kind == kind && cached_source == source
        }) {
            return result.clone();
        }
        let result = match kind {
            TextFilterKind::Unfiltered => Ok(TextFilter::Unfiltered),
            TextFilterKind::Pattern => self
                .pattern(source)
                .map(TextFilter::Pattern)
                .map_err(|_| ParseIssue::InvalidSyntax),
        }
        .map(Rc::new);
        if cache.len() == 64 {
            cache.pop_front();
        }
        cache.push_back(((kind, source.into()), result.clone()));
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

// Native offsets can still describe the previous text after a host replacement.
// Apply the native editor's bounded UTF-8 insertion projection before checking
// the mask; do not change the native selection or rewrite the draft.
fn native_mask_offsets(text: &str, anchor: i32, cursor: i32) -> (usize, usize) {
    let project = |offset: i32| {
        let offset = usize::try_from(offset.max(0)).expect("nonnegative native byte offset");
        text.ceil_char_boundary(offset)
    };
    (project(anchor), project(cursor))
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
    let dispatch = std::rc::Rc::new(crate::form::ImplicitSubmitDispatcher::default());
    let begin = dispatch.clone();
    global.on_begin_implicit_submit(move |generation| begin.begin(generation));
    let active = dispatch.clone();
    global.on_implicit_submit_active(move |generation| active.active(generation));
    let native = dispatch.clone();
    global.on_note_native_submit(move |generation| native.note_native(generation));
    global.on_take_implicit_submit(move |ticket, generation| dispatch.take(ticket, generation));
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
        let (anchor, cursor) = native_mask_offsets(current.as_str(), anchor, cursor);
        compiled.accepts_key_insertion(current.as_str(), anchor, cursor, inserted.as_str())
    });
    let mask_paste_engine = Rc::clone(&engine);
    global.on_mask_paste(move |mask, current, anchor, cursor, inserted| {
        let Ok(compiled) = mask_paste_engine.mask(mask.as_str()) else {
            return false;
        };
        let (anchor, cursor) = native_mask_offsets(current.as_str(), anchor, cursor);
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
    let file_engine = Rc::clone(&engine);
    global.on_file_validation(move |accept, max_files, max_size, entries, messages| {
        use slint::{Model as _, ModelExt as _};
        let invalid = || ui::KitFileValidation {
            validity: view_validity(HtmlValidity {
                bad_input: true,
                ..Default::default()
            }),
            message: messages.invalid.clone(),
        };
        let Ok(rules) = file_engine.file_constraints(accept.as_str(), max_files, max_size.as_str())
        else {
            return invalid();
        };
        entries.model_tracker().track_row_count_changes();
        let entries = (0..entries.row_count())
            .filter_map(|index| entries.row_data_tracked(index))
            .collect::<Vec<_>>();
        let mut metadata = Vec::with_capacity(entries.len());
        for entry in &entries {
            let byte_length = if entry.byte_length.is_empty() {
                None
            } else {
                if !entry
                    .byte_length
                    .chars()
                    .all(|value| value.is_ascii_digit())
                {
                    return invalid();
                }
                let Ok(bytes) = entry.byte_length.parse::<u64>() else {
                    return invalid();
                };
                Some(bytes)
            };
            metadata.push(kit_core::FileMetadata {
                name: entry.name.as_str(),
                byte_length,
            });
        }
        let html = HtmlConstraints {
            kind: kit_core::HtmlInputKind::File,
            ..Default::default()
        };
        let result = html.validate_file_selection(&rules, &metadata);
        let message = match result.violation {
            None => String::new(),
            Some(kit_core::FileViolation::TypeMismatch { name }) => {
                messages.type_mismatch.replace("{name}", &name)
            }
            Some(kit_core::FileViolation::TooMany { maximum }) => {
                messages.too_many.replace("{count}", &maximum.to_string())
            }
            Some(kit_core::FileViolation::TooLarge { name, maximum }) => messages
                .too_large
                .replace("{name}", &name)
                .replace("{bytes}", &maximum.to_string()),
            Some(kit_core::FileViolation::UnknownSize { name }) => {
                messages.size_unknown.replace("{name}", &name)
            }
        };
        ui::KitFileValidation {
            validity: view_validity(result.validity),
            message: message.into(),
        }
    });
    global.on_otp_pattern(|alphabet| {
        let alphabet = match alphabet {
            ui::OtpAlphabet::AsciiDigits => crate::__core::OtpAlphabet::AsciiDigits,
            ui::OtpAlphabet::AsciiAlphanumeric => crate::__core::OtpAlphabet::AsciiAlphanumeric,
        };
        alphabet.html_pattern_source().into()
    });
    fn filter_kind(kind: ui::TextFilterKind) -> crate::__core::TextFilterKind {
        match kind {
            ui::TextFilterKind::Unfiltered => crate::__core::TextFilterKind::Unfiltered,
            ui::TextFilterKind::Pattern => crate::__core::TextFilterKind::Pattern,
        }
    }
    fn allows_insertion(
        engine: &crate::HtmlValidationEngine,
        spec: ui::TextFilterSpec,
        text: &str,
        insertion: &str,
        anchor: i32,
        cursor: i32,
        maximum: i32,
    ) -> bool {
        let (anchor, cursor) = native_mask_offsets(text, anchor, cursor);
        let selection = crate::__core::TextEditSelection::new(text, anchor, cursor).ok();
        selection.is_some_and(|selection| {
            engine
                .text_filter(filter_kind(spec.kind), spec.source.as_str())
                .is_ok_and(|filter| {
                    let candidate = selection.replace(insertion);
                    filter.allows_draft(&candidate)
                        && usize::try_from(maximum)
                            .ok()
                            .is_none_or(|maximum| candidate.encode_utf16().count() <= maximum)
                })
        })
    }
    let insertion_engine = engine.clone();
    global.on_filter_insertion(move |spec, text, insertion, anchor, cursor, maximum| {
        allows_insertion(
            &insertion_engine,
            spec,
            text.as_str(),
            insertion.as_str(),
            anchor,
            cursor,
            maximum,
        )
    });
    let key_engine = engine.clone();
    global.on_filter_key(move |spec, text, key, anchor, cursor, maximum| {
        crate::__core::text_input_is_command(key.as_str())
            || allows_insertion(
                &key_engine,
                spec,
                text.as_str(),
                key.as_str(),
                anchor,
                cursor,
                maximum,
            )
    });
    let filter_engine = engine.clone();
    global.on_filter_error(move |spec, message| {
        filter_engine
            .text_filter(filter_kind(spec.kind), spec.source.as_str())
            .map_or_else(|_issue| message, |_rule| slint::SharedString::default())
    });
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
            validity: view_validity(v),
        }
    });
    global.set_ready(true);
}

impl From<crate::ui::KitValidityState> for HtmlValidity {
    fn from(value: crate::ui::KitValidityState) -> Self {
        Self {
            value_missing: value.value_missing,
            type_mismatch: value.type_mismatch,
            pattern_mismatch: value.pattern_mismatch,
            too_long: value.too_long,
            too_short: value.too_short,
            range_underflow: value.range_underflow,
            range_overflow: value.range_overflow,
            step_mismatch: value.step_mismatch,
            bad_input: value.bad_input,
            custom_error: value.custom_error,
        }
    }
}

#[cfg(test)]
mod mask_cache_tests {
    use super::HtmlValidationEngine;
    use kit_core::{MaskPattern, ParseIssue};
    use std::rc::Rc;

    #[test]
    fn native_offsets_follow_host_text_replacement_before_mask_insertion() {
        let mask = MaskPattern::compile("AA-####").unwrap();
        let (anchor, cursor) = super::native_mask_offsets("", 0, 5);
        assert_eq!((anchor, cursor), (0, 0));
        assert!(mask.accepts_key_insertion("", anchor, cursor, "AB-1234"));
        assert_eq!(super::native_mask_offsets("éA", -1, 1), (0, 2));
        assert_eq!(super::native_mask_offsets("éA", 99, 2), (3, 2));
    }

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

#[cfg(test)]
mod file_constraint_cache_tests {
    use super::*;
    #[test]
    fn file_rules_parse_once_and_preserve_exact_renderer_boundaries() {
        let engine = HtmlValidationEngine::default();
        let first = engine
            .file_constraints(".pdf", 2, "9007199254740993")
            .unwrap();
        let again = engine
            .file_constraints(".pdf", 2, "9007199254740993")
            .unwrap();
        assert!(Rc::ptr_eq(&first, &again));
        assert_eq!(first.max_size, Some(9_007_199_254_740_993));
        for (count, size) in [
            (-2, ""),
            (-1, "-1"),
            (-1, "1.5"),
            (-1, "18446744073709551616"),
        ] {
            assert!(engine.file_constraints(".pdf", count, size).is_err());
        }
        assert!(engine.file_constraints("*.pdf", -1, "").is_err());
        let unbounded = engine.file_constraints("", -1, "").unwrap();
        assert_eq!(unbounded.max_files, None);
        assert_eq!(unbounded.max_size, None);
    }
}
