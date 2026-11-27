// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask lint-style`: the style completeness report. A component draws what its
//! style describes, so a component file should neither read theme fields nor
//! derive geometry from the spacing unit. The report counts both and never fails
//! on violations; it is not part of `xtask check`.
//!
//! A theme passed on whole (to a child or to a recipe function) is not a read.
//! The check is textual: values a helper derives from a passed theme are listed
//! separately, and a theme held under a name that does not end in `theme` is
//! not seen.
use std::collections::BTreeMap;

use crate::lint_tokens::{self, is_identifier_byte, mask};

/// Components that must stay free of violations.
const PILOTS: [(&str, &str); 1] = [("feedback.slint", "KitAlert")];

const FILE_SCOPE: &str = "(file scope)";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViolationKind {
    ThemeRead,
    SpacingArithmetic,
}

impl ViolationKind {
    fn label(self) -> &'static str {
        match self {
            Self::ThemeRead => "theme read",
            Self::SpacingArithmetic => "spacing.unit arithmetic",
        }
    }
}

/// One site in a component file. `column` is the 1-based byte column.
#[derive(Debug, PartialEq, Eq)]
pub struct Violation {
    pub kind: ViolationKind,
    pub text: String,
    pub component: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Default)]
pub struct Scan {
    pub violations: Vec<Violation>,
    /// Names of the top-level components and globals of the file.
    pub components: Vec<String>,
    /// Components that call a metrics helper, which derives drawn values from a
    /// theme without a field read, with the number of calls.
    pub helper_calls: BTreeMap<String, usize>,
}

/// A maximal `a.b-c.d` chain of the masked source, as a byte range.
fn chains(masked: &[u8]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut index = 0;
    while index < masked.len() {
        // A name cannot start with a hyphen: a leading one is a minus sign.
        if !is_identifier_byte(masked[index]) || masked[index] == b'-' {
            index += 1;
            continue;
        }
        let start = index;
        while index < masked.len()
            && (is_identifier_byte(masked[index])
                || (masked[index] == b'.'
                    && masked.get(index + 1).is_some_and(u8::is_ascii_alphabetic)))
        {
            index += 1;
        }
        // Colors and numbers are not names.
        let color = start > 0 && masked[start - 1] == b'#';
        if !color && !masked[start].is_ascii_digit() {
            found.push((start, index));
        }
    }
    found
}

/// Whether the chain reads a field of a theme value.
fn reads_theme(segments: &[&str]) -> bool {
    segments.iter().enumerate().any(|(position, segment)| {
        let rest = segments.len() - position - 1;
        ((*segment == "theme" || segment.ends_with("-theme")) && rest >= 1)
            || (*segment == "KitThemeContext"
                && segments.get(position + 1) == Some(&"current")
                && rest >= 2)
    })
}

fn is_spacing_unit(segments: &[&str]) -> bool {
    segments.windows(2).any(|pair| pair == ["spacing", "unit"])
}

/// Whether an arithmetic operator touches the expression at `start..end`. A
/// leading minus counts: a negated unit is derived geometry too.
fn in_arithmetic(masked: &[u8], start: usize, end: usize) -> bool {
    let operator = |byte: &u8| matches!(byte, b'+' | b'-' | b'*' | b'/');
    let before = masked[..start]
        .iter()
        .rev()
        .find(|byte| !byte.is_ascii_whitespace());
    let after = masked[end..]
        .iter()
        .find(|byte| !byte.is_ascii_whitespace());
    before.is_some_and(operator) || after.is_some_and(operator)
}

/// Byte ranges of the top-level `component` and `global` bodies, by name.
fn blocks(masked: &[u8], words: &[(usize, usize)]) -> Vec<(String, usize, usize)> {
    let text = |(start, end): (usize, usize)| String::from_utf8_lossy(&masked[start..end]);
    let mut found = Vec::new();
    let mut depth = 0usize;
    let mut cursor = 0;
    let mut pending: Option<String> = None;
    let mut open: Option<(String, usize)> = None;
    let mut next_word = 0;
    while cursor < masked.len() {
        while next_word < words.len() && words[next_word].0 < cursor {
            next_word += 1;
        }
        if depth == 0
            && words.get(next_word).is_some_and(|word| word.0 == cursor)
            && matches!(&*text(words[next_word]), "component" | "global")
            && let Some(name) = words.get(next_word + 1)
        {
            pending = Some(text(*name).into_owned());
        }
        match masked[cursor] {
            b'{' => {
                if depth == 0
                    && let Some(name) = pending.take()
                {
                    open = Some((name, cursor));
                }
                depth += 1;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0
                    && let Some((name, start)) = open.take()
                {
                    found.push((name, start, cursor + 1));
                }
            }
            _ => {}
        }
        cursor += 1;
    }
    found
}

pub fn scan(source: &str) -> Scan {
    let masked = mask(source);
    let words = chains(&masked);
    let blocks = blocks(&masked, &words);
    let owner = |offset: usize| {
        blocks
            .iter()
            .find(|(_, start, end)| (*start..*end).contains(&offset))
            .map_or(FILE_SCOPE, |(name, _, _)| name.as_str())
    };
    let mut scan = Scan {
        components: blocks.iter().map(|(name, _, _)| name.clone()).collect(),
        ..Scan::default()
    };
    for (start, end) in words {
        let chain = &source[start..end];
        let segments: Vec<&str> = chain.split('.').collect();
        if segments.first() == Some(&"KitMetrics") && segments.len() > 1 {
            *scan
                .helper_calls
                .entry(owner(start).to_owned())
                .or_default() += 1;
        }
        let kind = if is_spacing_unit(&segments) && in_arithmetic(&masked, start, end) {
            ViolationKind::SpacingArithmetic
        } else if reads_theme(&segments) {
            ViolationKind::ThemeRead
        } else {
            continue;
        };
        let line_start = source[..start].rfind('\n').map_or(0, |newline| newline + 1);
        scan.violations.push(Violation {
            kind,
            text: chain.to_owned(),
            component: owner(start).to_owned(),
            line: source[..start].matches('\n').count() + 1,
            column: start - line_start + 1,
        });
    }
    scan
}

fn counts<'a>(violations: impl Iterator<Item = &'a Violation>) -> (usize, usize) {
    violations.fold((0, 0), |(reads, arithmetic), violation| {
        match violation.kind {
            ViolationKind::ThemeRead => (reads + 1, arithmetic),
            ViolationKind::SpacingArithmetic => (reads, arithmetic + 1),
        }
    })
}

/// Print the kit-wide totals and the pilot status; with a component name, also
/// print each of its sites.
pub fn run(detail: Option<&str>) -> Result<(), String> {
    let ui = kit_build::ui_directory();
    let mut scans = Vec::new();
    for (relative, path) in lint_tokens::component_files(&ui)? {
        let source =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        scans.push((relative, path, scan(&source)));
    }

    println!("lint-style: file, theme reads, spacing.unit arithmetic, total");
    for (relative, _, scan) in &scans {
        let (reads, arithmetic) = counts(scan.violations.iter());
        if reads + arithmetic > 0 {
            println!("  {relative} {reads} {arithmetic} {}", reads + arithmetic);
        }
    }
    let (reads, arithmetic) = counts(scans.iter().flat_map(|(_, _, s)| &s.violations));
    let mut violating: Vec<(&str, &str)> = scans
        .iter()
        .flat_map(|(relative, _, scan)| {
            scan.violations
                .iter()
                .map(move |v| (relative.as_str(), v.component.as_str()))
        })
        .collect();
    violating.sort_unstable();
    violating.dedup();
    println!(
        "lint-style: {} violations ({reads} theme reads, {arithmetic} spacing.unit arithmetic) in {} components and globals, {} component files checked",
        reads + arithmetic,
        violating.len(),
        scans.len()
    );
    let helper_calls: usize = scans
        .iter()
        .flat_map(|(_, _, s)| s.helper_calls.values())
        .sum();
    println!("lint-style: not counted: {helper_calls} KitMetrics calls in component files");

    for (file, name) in PILOTS {
        let (_, _, scan) = scans
            .iter()
            .find(|(relative, _, scan)| {
                relative == file && scan.components.iter().any(|c| c == name)
            })
            .ok_or_else(|| format!("pilot {name} not found in {file}"))?;
        let (reads, arithmetic) = counts(scan.violations.iter().filter(|v| v.component == name));
        println!(
            "lint-style: pilot {name}: {} violations, {} KitMetrics calls",
            reads + arithmetic,
            scan.helper_calls.get(name).copied().unwrap_or_default()
        );
    }

    if let Some(name) = detail {
        let mut known = false;
        for (_, path, scan) in &scans {
            known |= scan.components.iter().any(|c| c == name);
            for violation in scan.violations.iter().filter(|v| v.component == name) {
                println!(
                    "{}:{}:{}: {} `{}`",
                    path.display(),
                    violation.line,
                    violation.column,
                    violation.kind.label(),
                    violation.text
                );
            }
        }
        if !known {
            return Err(format!("no component or global named {name}"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(source: &str) -> Vec<(ViolationKind, String)> {
        scan(source)
            .violations
            .into_iter()
            .map(|v| (v.kind, v.text))
            .collect()
    }

    #[test]
    fn counts_each_theme_read_once() {
        let source = "component A { a: root.theme.text-primary; b: theme.typography.family; \
            c: KitThemeContext.current.accent; d: root.panel-theme.surface; }";
        assert_eq!(
            found(source),
            vec![
                (ViolationKind::ThemeRead, "root.theme.text-primary".into()),
                (ViolationKind::ThemeRead, "theme.typography.family".into()),
                (
                    ViolationKind::ThemeRead,
                    "KitThemeContext.current.accent".into()
                ),
                (ViolationKind::ThemeRead, "root.panel-theme.surface".into()),
            ]
        );
    }

    #[test]
    fn passing_a_theme_on_is_not_a_read() {
        let source = "import { KitTheme } from \"theme.slint\";\n\
            component A { in property <KitTheme> theme: KitThemeContext.current; \
            private property <Recipe> r: KitThemes.recipe(root.theme); \
            B { theme: root.theme; ink: root.r.theme-ink; } \
            // theme.accent\n c: \"theme.accent\"; /* theme.spacing.unit * 2 */ }";
        assert_eq!(found(source), vec![]);
    }

    #[test]
    fn spacing_arithmetic_is_one_violation_on_either_side_of_the_operator() {
        let source = "component A { a: root.theme.spacing.unit * 2; b: 3 * theme.spacing.unit; \
            c: -root.theme.spacing.unit; d: root.metrics.spacing.unit / 2; }";
        assert!(
            found(source)
                .iter()
                .all(|(kind, _)| *kind == ViolationKind::SpacingArithmetic)
        );
        assert_eq!(found(source).len(), 4);
    }

    #[test]
    fn a_bare_spacing_unit_is_a_theme_read() {
        assert_eq!(
            found("component A { spacing: root.theme.spacing.unit; }"),
            vec![(ViolationKind::ThemeRead, "root.theme.spacing.unit".into())]
        );
    }

    #[test]
    fn sites_belong_to_the_enclosing_top_level_block() {
        let source = "struct S { a: int }\n\
            export global G { public pure function f(theme: KitTheme) -> color { return theme.accent; } }\n\
            export component A inherits Rectangle {\n  Text { color: root.theme.accent; }\n}\n\
            component B { x: KitMetrics.font-size(root.theme, role); }";
        let scan = scan(source);
        assert_eq!(scan.components, vec!["G", "A", "B"]);
        let owners: Vec<(&str, usize, usize)> = scan
            .violations
            .iter()
            .map(|v| (v.component.as_str(), v.line, v.column))
            .collect();
        assert_eq!(owners[1], ("A", 4, 17));
        assert_eq!(owners[0].0, "G");
        assert_eq!(owners.len(), 2);
        assert_eq!(scan.helper_calls.get("B"), Some(&1));
    }
}
