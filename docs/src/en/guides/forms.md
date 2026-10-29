---
layout: base.njk
locale: en
title: "Forms, shared options and validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---
# Forms, shared options and validation

For a standard field, choose the control and set its options:

```slint
KitEmailField {
    label: "Email address";
    placeholder: "name@example.com";
    description: "Used for notifications.";
}
```

The label, help and error are already built into the control. [KitFieldLabel](/components/field-label/), [KitFieldDescription](/components/field-description/) and [KitFieldError](/components/field-error/) are primitives for custom composition. Their pages list controls with the corresponding built-in property; each reference also shows inherited properties.

## Validate the accepted value

The downloadable projects use Slint Kit 0.3.0 with a Rust `FieldBinding`. The project parser distinguishes the draft from the accepted value: an invalid draft remains editable and cannot replace the last valid value. Try the complete [email](/components/email-field/), [URL](/components/url-field/) and [masked field](/components/masked-field/) projects.

Email follows the HTML grammar: one `@`, a valid local part and a valid domain. A domain without a dot, such as `name@machine`, is allowed. A URL must be absolute; validation does not establish that the site exists. [HTML email and URL reference](https://html.spec.whatwg.org/multipage/input.html#e-mail-state-(type=email)).

An empty field is optional unless its controller uses `AbsencePolicy::Required`. Guard Rust submission too; showing an error does not replace refusing an invalid accepted value. [HTML constraints and submission](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constraints).

## Mask or regex

A mask checks a fixed sequence of slots. A regex expresses a more flexible rule. To add one to the [text field project](/components/text-field/), add `regress = "=0.12.0"` to its dependencies and replace its parser:

```rust
fn parse(text: &str) -> DraftValue<Value> {
    use std::sync::OnceLock;
    static PATTERN: OnceLock<regress::Regex> = OnceLock::new();
    let pattern = PATTERN.get_or_init(|| {
        regress::Regex::with_flags(r"^(?:[A-Z]{3}-\d{4})$", "v")
            .expect("valid application pattern")
    });
    if text.is_empty() {
        DraftValue::Empty
    } else if pattern.find(text).is_some_and(|m| m.range() == (0..text.len())) {
        DraftValue::Parsed(Value(text.into()))
    } else {
        DraftValue::Invalid(ParseIssue::InvalidSyntax)
    }
}
```

Also set the project’s initial value to `let initial = Value("ABC-1234".into());`.

`ABC-1234` is accepted; `abc-1234` and `XABC-1234` are rejected. Matching covers the entire value. The `v` mode uses ECMAScript syntax, including Unicode sets. [Regex engine API](https://docs.rs/regress/0.12.0/regress/struct.Regex.html).

HTML `pattern` applies to text, email, URL, telephone, search and password fields. It does not apply to `textarea`; a custom multiline rule remains explicit application validation. [The `pattern` attribute](https://html.spec.whatwg.org/multipage/input.html#the-pattern-attribute).

## Extend without replacing the editor

Keep the control’s native `TextInput` for selection, caret and editing history. Customize its theme, labels, prefixes and suffixes, and connect domain parsing and constraints through `FieldBinding`. For shared or asynchronous rules, see [typed controllers](/guides/controllers/).
