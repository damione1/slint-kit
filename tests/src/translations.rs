// SPDX-License-Identifier: GPL-3.0-or-later
use crate::TranslationsFixture;
use i_slint_backend_testing::{TestingBackend, TestingBackendOptions};
use std::{collections::BTreeMap, fs, path::PathBuf};

fn fixture() -> TranslationsFixture {
    slint::platform::set_platform(Box::new(TestingBackend::new(TestingBackendOptions {
        mock_time: true,
        renderer_name: Some("software".into()),
        ..Default::default()
    })))
    .ok();
    let app = TranslationsFixture::new().unwrap();
    slint_kit::install(&app);
    app
}

fn translate(app: &TranslationsFixture, code: &str) {
    slint::select_bundled_translation(code).unwrap();
    app.invoke_translate();
}

#[test]
fn switching_the_bundled_catalogue_retranslates_kit_strings_and_application_text() {
    let app = fixture();
    app.set_count(2);
    translate(&app, "fr");
    // `{{name}}` in the source and catalogue is the escape for the kit's literal `{name}` slot.
    assert_eq!(app.get_dismiss(), "Fermer {title}");
    assert_eq!(app.get_pagination(), "Page {current} sur {total}");
    assert_eq!(app.get_carousel(), "{current} / {total}");
    assert_eq!(app.get_summary(), "2 éléments");
    app.set_count(1);
    assert_eq!(app.get_summary(), "1 élément");
    translate(&app, "en");
    assert_eq!(app.get_dismiss(), "Dismiss {title}");
    assert_eq!(app.get_pagination(), "{current} / {total}");
    assert_eq!(app.get_summary(), "1 item");
    assert!(slint::select_bundled_translation("de").is_err());
}

fn example(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn slint_literal(text: &str) -> String {
    text.replace("\\\\", "\0")
        .replace("\\\"", "\"")
        .replace('\0', "\\")
}

fn po_literal(text: &str) -> String {
    text.replace("\\\"", "\"").replace("\\\\", "\\")
}

fn placeholders(text: &str) -> Vec<String> {
    let mut found: Vec<String> = text
        .split("{{")
        .skip(1)
        .filter_map(|rest| rest.split_once("}}").map(|(name, _)| name.to_owned()))
        .collect();
    found.sort();
    found
}

/// Kit defaults, by property name, from the kit's own `KitStrings`.
fn kit_defaults() -> BTreeMap<String, String> {
    example("crates/kit-slint/ui/strings.slint")
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("in-out property <string> ")?;
            let (name, value) = rest.split_once(": \"")?;
            let value = value.strip_suffix("\";")?;
            Some((name.to_owned(), slint_literal(value)))
        })
        .collect()
}

/// `(context, msgid)` of every `KitStrings` assignment in the shipped `apply()`, by property name.
fn catalogue_sources() -> BTreeMap<String, (Option<String>, String)> {
    example("docs/translations/ui/kit-translations.slint")
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("KitStrings.")?;
            let (name, call) = rest.split_once(" = @tr(")?;
            let call = call.strip_suffix(");")?;
            let (context, literal) = match call.split_once("\" => \"") {
                Some((context, literal)) => (Some(context.trim_start_matches('"')), literal),
                None => (None, call.trim_start_matches('"')),
            };
            let literal = literal.strip_suffix('"')?;
            Some((
                name.to_owned(),
                (context.map(str::to_owned), slint_literal(literal)),
            ))
        })
        .collect()
}

/// `(context, msgid) -> msgstr` for the singular entries of the shipped French catalogue.
fn french_entries() -> BTreeMap<(Option<String>, String), String> {
    let mut entries = BTreeMap::new();
    for block in
        example("docs/translations/lang/fr/LC_MESSAGES/kit-docs-translations.po").split("\n\n")
    {
        let field = |key: &str| {
            block.lines().find_map(|line| {
                let value = line.strip_prefix(key)?.strip_prefix(" \"")?;
                Some(po_literal(value.strip_suffix('"')?))
            })
        };
        if let (Some(id), Some(text)) = (field("msgid"), field("msgstr")) {
            assert!(
                entries
                    .insert((field("msgctxt"), id.clone()), text)
                    .is_none(),
                "duplicate catalogue entry {id}"
            );
        }
    }
    entries
}

#[test]
fn the_shipped_translation_global_covers_every_kit_string_with_the_kit_default() {
    let defaults = kit_defaults();
    let sources = catalogue_sources();
    assert_eq!(
        sources.keys().collect::<Vec<_>>(),
        defaults.keys().collect::<Vec<_>>(),
        "docs/translations/ui/kit-translations.slint must assign exactly the KitStrings text properties"
    );
    for (name, (_, source)) in &sources {
        assert_eq!(
            source.replace("{{", "{").replace("}}", "}"),
            defaults[name],
            "{name}: the @tr source text must equal the kit default"
        );
    }
}

#[test]
fn the_french_catalogue_translates_every_kit_string_and_keeps_its_placeholders() {
    let entries = french_entries();
    for (name, (context, source)) in catalogue_sources() {
        let translated = entries
            .get(&(context, source.clone()))
            .unwrap_or_else(|| panic!("{name}: no French entry for {source:?}"));
        assert!(!translated.is_empty(), "{name}: empty French translation");
        assert_eq!(
            placeholders(&source),
            placeholders(translated),
            "{name}: French text must keep the same placeholders"
        );
    }
}
