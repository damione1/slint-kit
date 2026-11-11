//! Compiled text-entry rules, separate from final form acceptance.
use crate::{HtmlPattern, ParseIssue, TextFilterKind};

/// A view-authored filter parsed once at the editing boundary.
#[derive(Clone, Debug)]
pub enum TextFilter {
    Unfiltered,
    Pattern(HtmlPattern),
}
impl TextFilter {
    pub fn compile(kind: TextFilterKind, source: &str) -> Result<Self, ParseIssue> {
        match kind {
            TextFilterKind::Unfiltered => Ok(Self::Unfiltered),
            TextFilterKind::Pattern => HtmlPattern::compile(source)
                .map(Self::Pattern)
                .map_err(|_| ParseIssue::InvalidSyntax),
        }
    }
    /// Empty drafts remain clearable; final required/length rules stay in HTML validation.
    pub fn allows_draft(&self, text: &str) -> bool {
        match self {
            Self::Unfiltered => true,
            Self::Pattern(pattern) => text.is_empty() || pattern.matches(text),
        }
    }
    pub fn allows_insertion(&self, selection: TextEditSelection<'_>, insertion: &str) -> bool {
        self.allows_draft(&selection.replace(insertion))
    }
}

/// The retained portions around a native selection, validated once against UTF-8.
/// Byte offsets may be reversed, but must be character boundaries within `text`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextEditSelection<'a> {
    before: &'a str,
    after: &'a str,
}
impl<'a> TextEditSelection<'a> {
    pub fn new(text: &'a str, anchor: usize, cursor: usize) -> Result<Self, ParseIssue> {
        let start = anchor.min(cursor);
        let end = anchor.max(cursor);
        match (text.get(..start), text.get(end..)) {
            (Some(before), Some(after)) => Ok(Self { before, after }),
            (Some(_), None) | (None, Some(_)) | (None, None) => Err(ParseIssue::InvalidRange),
        }
    }
    pub fn replace(self, insertion: &str) -> String {
        [self.before, insertion, self.after].concat()
    }
}

/// Slint's non-text keys must reach its editor instead of being matched as text.
/// Only a single control/special-key scalar is a command, not a mixed insertion.
pub fn text_input_is_command(text: &str) -> bool {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(value), None) => value.is_control() || ('\u{f700}'..='\u{f7ff}').contains(&value),
        (Some(_), Some(_)) | (None, Some(_)) | (None, None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_selection_is_checked_before_replacing_multibyte_text() {
        assert_eq!(
            TextEditSelection::new("aéz", 1, 3).unwrap().replace("β"),
            "aβz"
        );
        assert_eq!(
            TextEditSelection::new("aéz", 3, 1).unwrap().replace("β"),
            "aβz"
        );
        assert_eq!(
            TextEditSelection::new("aéz", 1, 2),
            Err(ParseIssue::InvalidRange)
        );
        assert_eq!(
            TextEditSelection::new("aéz", 8, 8),
            Err(ParseIssue::InvalidRange)
        );
    }
    #[test]
    fn filters_check_the_result_of_selection_replacement_and_keep_empty_drafts() {
        let filter = TextFilter::compile(TextFilterKind::Pattern, "[A-Z0-9]*").unwrap();
        assert!(filter.allows_insertion(TextEditSelection::new("AB12", 0, 4).unwrap(), "C"));
        assert!(!filter.allows_insertion(TextEditSelection::new("AB12", 2, 2).unwrap(), " "));
        assert!(filter.allows_draft(""));
        let unicode = TextFilter::compile(TextFilterKind::Pattern, "\\p{Letter}*").unwrap();
        assert!(unicode.allows_draft("éβ"));
        assert!(!unicode.allows_draft("é1"));
        assert!(TextFilter::compile(TextFilterKind::Pattern, "[").is_err());
    }
    #[test]
    fn special_keys_stay_native_while_mixed_and_private_use_text_is_filtered() {
        for key in ["\n", "\t", "\u{8}", "\u{f700}", "\u{f728}"] {
            assert!(text_input_is_command(key));
        }
        for insertion in ["", "A", "12", "a\n", "\u{e001}"] {
            assert!(!text_input_is_command(insertion));
        }
    }
}
