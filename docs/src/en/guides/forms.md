---
layout: base.njk
locale: en
title: "Forms, shared options and validation"
translationKey: "guides/forms/"
permalink: "{{ 'guides/forms/' | localeUrl(locale) }}index.html"
---

# Forms, shared options and validation

Slint Kit forms share an HTML constraint engine and typed Rust codecs. They reject malformed email addresses and URLs, preserve incomplete drafts and return validated values to your business code. Native editing stays with Slint; your submission command also checks the Rust boundary.

Set up the application once using [Project setup](/guides/project-setup/). Import controls from `@kit` and call `slint_kit::install(&app)` before showing each window. This installs pure shared services; fields, calendars and selections keep their own instance state. Repeated installation is harmless. These APIs target Slint Kit **{{ project.version }}**; use matching runtime and build packages.

```slint
import { KitEmailField, FieldCommit } from "@kit";
export component App inherits Window {
    callback contact-changed(FieldCommit);
    KitEmailField {
        label: "Work email";
        required: true;
        spec-key: "contact";
        committed(value) => { root.contact-changed(value); }
    }
}
```

Register the domain codec once and guard the Rust boundary too:

```rust
let app = App::new()?;
slint_kit::install(&app);
let spec = slint_kit::FieldSpec::email();
slint_kit::install_field_specs(&app, [
    spec.named(slint_kit::FieldSpecKey::new("contact").unwrap())
]).unwrap();
app.on_contact_changed(move |commit| {
    if let Ok(Some(address)) = spec.accept(commit) {
        println!("Contact: {}", spec.format(&address));
    }
});
```

The HTML constraint engine enforces intrinsic email, URL, numeric, date and other input rules, including `required`, bounds, lengths, pattern and step. Nonempty text is insufficient for email or URL. A custom codec adds domain rules after those shared constraints. `FieldSpec::accept` returns the parsed value; pass that value to business code instead of parsing it again. Do not replace native editing with host-side string rewriting.

## Drafts and acceptance

`text` remains the native editing draft, including incomplete values. `field-state` exposes accepted value, draft, dirty/touched flags and the last transition; `accepted-text` projects the last accepted text. `committed(FieldCommit)` contains the candidate and the constraints under which it was accepted. A named codec key is application data, independent of visual position.

`commit-policy` defaults to `CommitPolicy.on-blur`. Choose `on-submit` or `on-change-when-valid` when appropriate. `commit()` reports validity and requests acceptance, `cancel-edit()` restores the last accepted draft, and `reset-field()` restores `default-value` and validation state. `ownership: Ownership.controlled` emits proposals; acknowledge accepted host values explicitly with `acknowledge(FieldValue)`. The typed [controllers guide](/guides/controllers/) covers custom codecs and exact decimals.

`access: FieldAccess.read-only` retains native text selection and prevents acceptance. Disabled controls reject interaction. Native caret, selection, undo and IME stay in the underlying TextInput. Headless tests exercise public input paths; they do not qualify physical keyboards, composition engines or screen readers on every platform.

## Shared options and composition

Fields already include `label`, `description`, `error`, validation and accessible naming. `leading-icon` and `trailing-icon` are decorative `IconGlyph` values; they do not replace a textual accessible name. Presentation options such as `decoration`, `corner-radius`, padding and text size preserve the native editor and shared validation.

[KitFieldLabel](/components/field-label/), [KitFieldDescription](/components/field-description/) and [KitFieldError](/components/field-error/) are composition primitives for custom layouts. Their compatibility links come from actual inherited APIs. They do not supply an editor, domain parsing or an independent validation controller. `KitInputGroup` and its start/end/text parts arrange an existing field.

## Submit and reset

`KitSubmitButton` and `KitFormButton` express form actions. Before dispatching a submission, call each field's `report-validity()` and focus the first invalid field. The Rust command must also accept checked domain values or validate its incoming payload; a disabled button is not a submission guard. Form examples keep the command and domain validation in Rust. `KitResetButton` invokes the application's reset action, which calls field resets and clears business state deliberately.

Calendar/date-range composites validate availability as well as HTML bounds. The time-zone field refuses nonexistent times and asks for an explicit occurrence when clocks repeat. Its accepted `ZonedDateTimeCommit` converts to `ResolvedDateTime`; no time zone is inferred from a local date-time.

`on-change-when-valid` emits the final value of each native edit. Arrow keys and modifier keys leave the acceptance state unchanged.

`report-validity()` never moves focus. Report every field first, then call `focus-if-invalid()` only for the first invalid field. This avoids blur side effects while collecting errors.

`KitSubmitButton` requires a connected `validation-requested` callback that returns the form’s validity, or explicit `form-no-validate`. Handle `form-action(FormAction)` for submit/reset; `activated` remains an observation after the gate and cannot replace it. `KitResetButton` skips validation.

Text editing exposes UTF-8 byte offsets through `cursor-position`, `selection-anchor` and `set-selection(anchor, cursor)`. `insert-at-cursor(text)` takes focus and follows native insertion/filtering; `undo()`, `redo()`, `cut-selection()`, `copy-selection()` and `paste()` keep the same editor. `blur()` releases it. Use `context-menu: TextContextMenu.disabled` to replace the standard localisable menu. Password menus suppress cut/copy; other read-only fields retain copy and selection.
