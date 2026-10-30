---
layout: base.njk
locale: en
title: "Forms, shared options and validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---
# Forms, shared options and validation

Choose a field, then set its options:

```slint
KitEmailField {
    label: "Work email";
    required: true;
    pattern: ".+@example\\.com";
    pattern-message: "Use your example.com address.";
}
```

Fields already include `label`, `description` and `error`. [KitFieldLabel](/components/field-label/), [KitFieldDescription](/components/field-description/) and [KitFieldError](/components/field-error/) are primitives for custom composition. Their pages list compatible controls; each reference also shows inherited properties.

## Initialize validation

The complete projects install the engine once per window. Export these types from your root Slint file, then initialize the Rust engine:

```slint
export { KitFormValidation, HtmlInputKind, NumberInputFormat,
    KitInputConstraints, KitValidationResult, KitValidityState } from "@kit/kit.slint";
```

```rust
// After App::new(), before app.run().
slint_kit::init_form_validation!(self, app);
```

If generated code lives in `mod ui`, use `init_form_validation!(ui, app)`. Without initialization, `validation-ready` stays false and validity checks fail. The [email](/components/email-field/), [URL](/components/url-field/) and [text field](/components/text-field/) projects contain all files.

## Choose constraints

| Option | Purpose |
| --- | --- |
| `required` | Reject an empty value or a missing required selection. |
| `multiple` | Accept comma-separated email addresses. |
| `min-length`, `max-length` | Text length in UTF-16 code units; `-1` removes the limit. The native draft remains editable. |
| `pattern`, `pattern-message` | ECMAScript `v` regex, matching the whole value. |
| `min`, `max`, `step` | Canonical number/date/time bounds and steps; `step: "any"` removes the step constraint. |
| `default-value` | Reset value and step base when `min` is absent. |
| `validation-timing` | Defaults to `on-blur`; also `on-input` and `on-submit`. |
| `custom-validity` | Application error; an empty string clears it. |

An incomplete email shows an error on blur. A second `@` or an unsupported character reveals it while typing. HTML accepts `name@machine`: the domain does not require a dot. URLs must be absolute; `https://example.comddf.dofmo@` is invalid, while `mailto:a@example.com` is valid. Validation does not establish that a mailbox or website exists. [HTML email and URL rules](https://html.spec.whatwg.org/multipage/input.html).

`validity` exposes independent flags; `value` is canonical and `text` preserves the native draft. `check-validity()` checks; `report-validity()` also reveals the error and focuses the control. Disabled controls and read-only text fields are excluded. [HTML form validation](https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#constraints).

## Guard the accepted value in Rust

A visual error does not replace validation before accepting a value. The examples configure matching `FieldBinding` constraints:

```rust
field.set_html_constraints(slint_kit::HtmlConstraints {
    kind: slint_kit::HtmlInputKind::Email,
    required: true,
    pattern: Some(slint_kit::HtmlPattern::compile(r".+@example\.com").expect("valid application pattern")),
    ..Default::default()
});
```

The engine checks before the domain parser, which receives the normalized value. An invalid draft cannot replace the last accepted value. For several fields, use `HtmlFormValidation<FieldKey>` and `first_invalid()`, or connect `KitSubmitButton.validation-requested`. `form-no-validate` bypasses this guard; reset never submits.

## Mask or regex

A [mask](/components/masked-field/) guides a fixed structure. A regex validates without rewriting the draft. For example, `pattern: "[A-Z]{3}-[0-9]{4}"` accepts `ABC-1234`, rejects `abc-1234` and rejects `XABC-1234`. This is a shared option, not another component.

HTML does not apply `pattern` to `textarea`. For a multiline rule, use `HtmlPattern` in Rust and report the result through `custom-validity`. Invalid patterns impose no constraint, as in HTML; `pattern-error` exposes the diagnostic. `pattern-enabled: true` distinguishes an explicit empty pattern from no pattern. [HTML pattern attribute](https://html.spec.whatwg.org/multipage/input.html#the-pattern-attribute).

## Compose and extend

Keep the native editor for caret, selection and history. Themes, labels, messages, prefixes and suffixes remain customizable. For another skin, import the nonvisual `FormConstraintState` from `@kit/validation.slint`. Business rules, exact money, masks, OTP alphabets and zone resolution remain in typed controllers. [Controllers](/guides/controllers/).

The kit provides native validation based on HTML. HTTP form transport, browser autofill and platform-specific keyboards need host integration; color controls retain their typed sRGB model.
