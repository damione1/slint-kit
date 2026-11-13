---
layout: base.njk
locale: en
title: "Translations"
translationKey: "guides/translations/"
permalink: "{{ 'guides/translations/' | localeUrl(locale) }}index.html"
---

# Translations

A multilingual application translates two kinds of text: its own text, and the text that the kit components display (button names, validation messages, month names). Slint Kit does not hard-code the second kind. Every built-in sentence is a property of the window-local `KitStrings` global, with an English default. You write each sentence once, in your own catalogue, and switch language at run time.

## Why `KitStrings` and not `@tr` inside the kit

Slint translates `@tr(...)` text with gettext catalogues that belong to the application. The kit is a library, so it does not ship a catalogue and does not call `@tr` itself: a library catalogue could not follow your language list, your plural rules or your word order. Instead, the kit exposes its text as properties and templates (`{label}`, `{count}`, `{title}`), and you assign them from `@tr`. A template names its slots, so each language chooses its own word order: `Dismiss {title}` becomes `Fermer {title}`.

Per-component properties such as `required-message` or `dismiss-label` still override the global default for one instance.

## Set up the build

Add a catalogue per language under `lang/<code>/LC_MESSAGES/<crate-name>.po` and ask the compiler to bundle them:

```rust
// build.rs
use slint_build::DefaultTranslationContext;

fn main() {
    let configuration = kit_build::config()
        .with_bundled_translations("lang")
        .with_default_translation_context(DefaultTranslationContext::None);
    kit_build::compile_with_config("ui/app.slint", configuration).expect("compile the interface");
}
```

Add `slint-build` next to `slint-kit-build` in `[build-dependencies]`. Bundled catalogues are compiled into the binary, so no file is read at run time and no gettext library is needed.

## Route the kit text through `@tr`

Create one global that assigns every `KitStrings` text property from `@tr`. The reference example ships it complete, with all 150 properties and a French catalogue:

```slint
import { KitStrings } from "@kit";

export global KitTranslations {
    public function apply() {
        KitStrings.required-message = @tr("Please fill out this field.");
        KitStrings.email-message = @tr("Please enter a valid email address.");
        // A kit template keeps its slots: write braces twice inside @tr.
        KitStrings.dismiss-label-template = @tr("Dismiss {{title}}");
        // Give a context when two sentences share the same English text.
        KitStrings.pagination-position-template = @tr("pagination" => "{{current}} / {{total}}");
    }
}
```

Two rules come from `@tr` itself. A single brace is a gettext slot, so a kit slot such as `{title}` is written `{{title}}` in the source and in the catalogue; the result is the literal `{title}` that the kit expects. Plural application text uses `@tr("{n} message" | "{n} messages" % count)`, with `msgid_plural` and a `Plural-Forms` header in the catalogue.

Then translate the application text in the same files:

```slint
title: @tr("Account settings");
KitTextField { theme: KitThemes.light; label: @tr("Full name"); required: true; }
```

## Switch language

`@tr` text follows the selected catalogue by itself. `KitStrings` receives a copy, so assign it again after every change. Keep the language as an enum in Rust and convert it once, where it meets Slint:

```rust
#[derive(Clone, Copy)]
enum Language {
    English,
    French,
}

impl Language {
    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::French => "fr",
        }
    }
}

fn translate(app: &App, language: Language) {
    slint::select_bundled_translation(language.code()).expect("bundled at build time");
    app.invoke_translate(); // calls KitTranslations.apply()
}
```

`"en"` selects the text written in the sources. Call `select_bundled_translation` after the first window exists, then call `translate` once at start-up and again whenever the user chooses another language. Open windows update at once, and field values are untouched.

If you use a `FieldBinding`, refresh its message snapshot after the switch with `binding.set_messages(window.global::<slint_kit::ui::KitStrings>().get_fields())`.

## Formats are explicit, not inferred

Translation does not change how values are parsed or shown. Choose formats yourself, and change them together with the language:

- Numbers and money: pass `NumberSyntax::fr_ca()` or `NumberSyntax::en_ca()` to the field spec and set `number-format` to match (see [Forms](/guides/forms/)).
- Dates: give calendars and date pickers `month-names`, `weekday-names` and the matching `title-pattern` and `accessible-date-pattern`, or use `DateNames::french()` in Rust.
- Right-to-left languages: the kit mirrors layout and keyboard navigation when the window direction is right-to-left.

## Try the example

The `docs/translations` crate in the repository is a complete English and French application. It contains the build script, the `KitTranslations` global with all kit text, the French catalogue and a window with a language switch. Run it with `cargo run -p kit-docs-translations`, or pass `fr` to start in French.

Headless tests verify the mechanism, the coverage of every `KitStrings` property and the placeholders of the French text. They do not qualify a screen reader or a right-to-left layout on a device.
