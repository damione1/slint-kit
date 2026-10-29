// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask lint-tokens`: theme tokens are the only source of geometry, color and
//! font weight in the kit's `.slint` files. Slint cannot enforce this through
//! types, so this check rejects hard-coded length, duration, color and
//! font-weight literals outside the token source and an explicit allowlist.
//! Every allowlist entry carries a reason and must still match a literal.
use std::fmt;
use std::path::{Path, PathBuf};

/// Files that define tokens or are generated, relative to the UI directory.
const EXEMPT_FILES: [(&str, &str); 2] = [
    (
        "theme.slint",
        "the token source: themes, metrics and recipes",
    ),
    (
        "shared-contracts.slint",
        "generated contracts, never hand-edited",
    ),
];

/// Literals allowed everywhere, with the reason.
const ALLOWED_LITERALS: [(&str, &str); 1] = [("100000px", "Slint's unbounded maximum size")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralKind {
    Length,
    Duration,
    Color,
    FontWeight,
}

impl fmt::Display for LiteralKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Length => "length",
            Self::Duration => "duration",
            Self::Color => "color",
            Self::FontWeight => "font-weight",
        })
    }
}

/// A literal found in a source file. `column` is the 1-based byte column.
#[derive(Debug, PartialEq, Eq)]
pub struct Literal {
    pub kind: LiteralKind,
    pub text: String,
    pub line: usize,
    pub column: usize,
}

/// One allowlist line: `file | literal | context | reason`. The literal is
/// allowed where it lies inside an occurrence of `context` on its source line.
#[derive(Debug, PartialEq, Eq)]
pub struct AllowEntry {
    pub file: String,
    pub literal: String,
    pub context: String,
    pub reason: String,
    pub line: usize,
}

/// Replace comments and string contents with spaces, keeping every byte
/// offset and newline, so positions in the mask are positions in the source.
fn mask(source: &str) -> Vec<u8> {
    let bytes = source.as_bytes();
    let mut out = bytes.to_vec();
    let mut index = 0;
    while index < bytes.len() {
        match (bytes[index], bytes.get(index + 1).copied()) {
            (b'/', Some(b'/')) => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    out[index] = b' ';
                    index += 1;
                }
            }
            (b'/', Some(b'*')) => {
                let mut previous = 0u8;
                while index < bytes.len() {
                    let current = bytes[index];
                    if current != b'\n' {
                        out[index] = b' ';
                    }
                    index += 1;
                    if previous == b'*' && current == b'/' {
                        break;
                    }
                    previous = current;
                }
            }
            (b'"', _) => {
                index += 1;
                while index < bytes.len() && bytes[index] != b'"' {
                    if bytes[index] == b'\\' && index + 1 < bytes.len() {
                        out[index] = b' ';
                        index += 1;
                    }
                    if bytes[index] != b'\n' {
                        out[index] = b' ';
                    }
                    index += 1;
                }
                index += 1;
            }
            _ => index += 1,
        }
    }
    out
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// Byte ranges of every `font-weight:` binding expression, up to `;` or `}`.
fn font_weight_spans(masked: &[u8]) -> Vec<(usize, usize)> {
    const KEY: &[u8] = b"font-weight";
    let mut spans = Vec::new();
    let mut index = 0;
    while index + KEY.len() <= masked.len() {
        let starts_word = index == 0 || !is_identifier_byte(masked[index - 1]);
        if starts_word && &masked[index..index + KEY.len()] == KEY {
            let mut cursor = index + KEY.len();
            while cursor < masked.len() && masked[cursor].is_ascii_whitespace() {
                cursor += 1;
            }
            if masked.get(cursor) == Some(&b':') {
                let start = cursor + 1;
                let mut end = start;
                while end < masked.len() && masked[end] != b';' && masked[end] != b'}' {
                    end += 1;
                }
                spans.push((start, end));
                index = end;
                continue;
            }
        }
        index += 1;
    }
    spans
}

/// A number may start a literal only where it cannot continue an identifier:
/// `h1` and `weight-2` are names; `-2px` and `(2px` are literals.
fn starts_number(masked: &[u8], index: usize) -> bool {
    match index.checked_sub(1).map(|previous| masked[previous]) {
        None => true,
        Some(b'-') => {
            index < 2 || !(masked[index - 2].is_ascii_alphanumeric() || masked[index - 2] == b'_')
        }
        Some(previous) => {
            !(previous.is_ascii_alphanumeric() || previous == b'_' || previous == b'.')
        }
    }
}

pub fn scan(source: &str) -> Vec<Literal> {
    let masked = mask(source);
    let weights = font_weight_spans(&masked);
    let mut literals = Vec::new();
    let mut line = 1;
    let mut line_start = 0;
    let mut index = 0;
    let mut push = |kind, start: usize, end: usize, line: usize, line_start: usize| {
        literals.push(Literal {
            kind,
            text: source[start..end].to_owned(),
            line,
            column: start - line_start + 1,
        });
    };
    while index < masked.len() {
        let byte = masked[index];
        if byte == b'\n' {
            line += 1;
            line_start = index + 1;
            index += 1;
            continue;
        }
        if byte == b'#' {
            let mut end = index + 1;
            while end < masked.len() && masked[end].is_ascii_hexdigit() {
                end += 1;
            }
            let digits = end - index - 1;
            let bounded = end == masked.len() || !is_identifier_byte(masked[end]);
            if bounded && matches!(digits, 3 | 4 | 6 | 8) {
                push(LiteralKind::Color, index, end, line, line_start);
            }
            index = end.max(index + 1);
            continue;
        }
        if byte.is_ascii_digit() && starts_number(&masked, index) {
            let mut end = index;
            while end < masked.len() && masked[end].is_ascii_digit() {
                end += 1;
            }
            if masked.get(end) == Some(&b'.') && masked.get(end + 1).is_some_and(u8::is_ascii_digit)
            {
                end += 1;
                while end < masked.len() && masked[end].is_ascii_digit() {
                    end += 1;
                }
            }
            let number_end = end;
            while end < masked.len() && (masked[end].is_ascii_alphabetic() || masked[end] == b'%') {
                end += 1;
            }
            let zero = masked[index..number_end]
                .iter()
                .all(|b| *b == b'0' || *b == b'.');
            let kind = match &source[number_end..end] {
                "px" | "phx" | "pt" | "in" | "mm" | "cm" => Some(LiteralKind::Length),
                "ms" | "s" => Some(LiteralKind::Duration),
                "" if weights
                    .iter()
                    .any(|(start, stop)| (*start..*stop).contains(&index)) =>
                {
                    Some(LiteralKind::FontWeight)
                }
                _ => None,
            };
            if let Some(kind) = kind
                && !zero
            {
                push(kind, index, end, line, line_start);
            }
            index = end;
            continue;
        }
        index += 1;
    }
    literals
}

pub fn parse_allowlist(text: &str) -> Result<Vec<AllowEntry>, String> {
    let mut entries: Vec<AllowEntry> = Vec::new();
    for (offset, raw) in text.lines().enumerate() {
        let line = offset + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split(" | ").map(str::trim).collect();
        let [file, literal, context, reason] = fields.as_slice() else {
            return Err(format!(
                "line {line}: expected `file | literal | context | reason`"
            ));
        };
        if [file, literal, context, reason]
            .iter()
            .any(|field| field.is_empty())
        {
            return Err(format!("line {line}: every field needs a value"));
        }
        if !context.contains(literal) {
            return Err(format!(
                "line {line}: context `{context}` does not contain `{literal}`"
            ));
        }
        if let Some(previous) = entries
            .iter()
            .find(|e| e.file == *file && e.literal == *literal && e.context == *context)
        {
            return Err(format!("line {line}: duplicates line {}", previous.line));
        }
        entries.push(AllowEntry {
            file: (*file).to_owned(),
            literal: (*literal).to_owned(),
            context: (*context).to_owned(),
            reason: (*reason).to_owned(),
            line,
        });
    }
    Ok(entries)
}

fn covers(entry: &AllowEntry, file: &str, source_line: &str, literal: &Literal) -> bool {
    if entry.file != file || entry.literal != literal.text {
        return false;
    }
    let start = literal.column - 1;
    let end = start + literal.text.len();
    source_line
        .match_indices(entry.context.as_str())
        .any(|(offset, context)| offset <= start && end <= offset + context.len())
}

/// Literals of one file that no entry covers, marking the entries used.
pub fn unlisted(
    file: &str,
    source: &str,
    entries: &[AllowEntry],
    used: &mut [bool],
) -> Vec<Literal> {
    let lines: Vec<&str> = source.lines().collect();
    let mut missing = Vec::new();
    for literal in scan(source) {
        if ALLOWED_LITERALS
            .iter()
            .any(|(text, _)| *text == literal.text)
        {
            continue;
        }
        let source_line = lines.get(literal.line - 1).copied().unwrap_or_default();
        let mut covered = false;
        for (entry, mark) in entries.iter().zip(used.iter_mut()) {
            if covers(entry, file, source_line, &literal) {
                *mark = true;
                covered = true;
            }
        }
        if !covered {
            missing.push(literal);
        }
    }
    missing
}

fn slint_files(
    directory: &Path,
    root: &Path,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), String> {
    let mut children: Vec<PathBuf> = std::fs::read_dir(directory)
        .map_err(|e| format!("{}: {e}", directory.display()))?
        .map(|entry| entry.map(|e| e.path()).map_err(|e| e.to_string()))
        .collect::<Result<_, _>>()?;
    children.sort();
    for path in children {
        if path.is_dir() {
            slint_files(&path, root, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "slint")
        {
            let relative = path
                .strip_prefix(root)
                .map_err(|e| e.to_string())?
                .components()
                .map(|part| part.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.push((relative, path));
        }
    }
    Ok(())
}

pub fn allowlist_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("token-allowlist.txt")
}

pub fn run() -> Result<(), String> {
    let ui = kit_build::ui_directory();
    let allowlist_file = allowlist_path();
    let allowlist_text = std::fs::read_to_string(&allowlist_file)
        .map_err(|e| format!("{}: {e}", allowlist_file.display()))?;
    let entries = parse_allowlist(&allowlist_text)
        .map_err(|e| format!("{}: {e}", allowlist_file.display()))?;
    let mut used = vec![false; entries.len()];
    let mut files = Vec::new();
    slint_files(&ui, &ui, &mut files)?;
    let mut problems = Vec::new();
    for (relative, path) in &files {
        if EXEMPT_FILES.iter().any(|(name, _)| name == relative) {
            continue;
        }
        let source =
            std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        for literal in unlisted(relative, &source, &entries, &mut used) {
            problems.push(format!(
                "{}:{}:{}: hard-coded {} literal `{}`; read a theme token or recipe, or allowlist it with a reason",
                path.display(),
                literal.line,
                literal.column,
                literal.kind,
                literal.text
            ));
        }
    }
    for (entry, used) in entries.iter().zip(&used) {
        if !used {
            problems.push(format!(
                "{}:{}: stale entry `{} | {} | {}` matches no literal; remove it",
                allowlist_file.display(),
                entry.line,
                entry.file,
                entry.literal,
                entry.context
            ));
        }
    }
    if problems.is_empty() {
        println!(
            "lint-tokens: {} files checked, {} allowlist entries in use",
            files.len(),
            entries.len()
        );
        Ok(())
    } else {
        Err(problems.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<(LiteralKind, String)> {
        scan(source).into_iter().map(|l| (l.kind, l.text)).collect()
    }

    #[test]
    fn finds_lengths_durations_colors_and_weights() {
        let source =
            "x: 12px; y: -4px; t: 150ms; b: #0f172a; font-weight: a ? 700 : 400; z: 1.5phx;";
        assert_eq!(
            kinds(source),
            vec![
                (LiteralKind::Length, "12px".into()),
                (LiteralKind::Length, "4px".into()),
                (LiteralKind::Duration, "150ms".into()),
                (LiteralKind::Color, "#0f172a".into()),
                (LiteralKind::FontWeight, "700".into()),
                (LiteralKind::FontWeight, "400".into()),
                (LiteralKind::Length, "1.5phx".into()),
            ]
        );
    }

    #[test]
    fn ignores_zero_tokens_names_comments_strings_and_plain_numbers() {
        let source = "a: 0px; b: 0ms; c: 0.0px; h1: theme.unit * 4; d: 45deg; e: 100%; \
            // 12px in a comment\n/* #fff */ f: \"8px\"; g: index + 8; w: weight-2;";
        assert_eq!(kinds(source), vec![]);
    }

    #[test]
    fn reports_one_based_positions() {
        let found = scan("a: 1;\n  b: 2px;");
        assert_eq!(
            found,
            vec![Literal {
                kind: LiteralKind::Length,
                text: "2px".into(),
                line: 2,
                column: 6
            }]
        );
    }

    #[test]
    fn font_weight_spans_end_at_the_binding() {
        assert_eq!(kinds("font-weight: w; x: 600;"), vec![]);
        assert_eq!(
            kinds("font-weight:600}"),
            vec![(LiteralKind::FontWeight, "600".into())]
        );
    }

    #[test]
    fn allowlist_requires_four_fields_a_containing_context_and_no_duplicates() {
        assert!(parse_allowlist("a.slint | 1px | 1px").is_err());
        assert!(parse_allowlist("a.slint | 1px | 2px | reason").is_err());
        assert!(parse_allowlist("a.slint | 1px | x: 1px | r\na.slint | 1px | x: 1px | r").is_err());
        let entries = parse_allowlist("# comment\n\na.slint | 1px | x: 1px | hairline").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].line, 3);
    }

    #[test]
    fn an_entry_covers_only_literals_inside_its_context() {
        let entries = parse_allowlist("a.slint | 8px | width: 8px | gutter").unwrap();
        let mut used = vec![false];
        let missing = unlisted("a.slint", "width: 8px; height: 8px;", &entries, &mut used);
        assert_eq!(missing.len(), 1);
        assert_eq!(missing[0].column, 21);
        assert!(used[0]);
        let mut unused = vec![false];
        assert_eq!(
            unlisted("b.slint", "width: 8px;", &entries, &mut unused).len(),
            1
        );
        assert!(!unused[0]);
    }
}
