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

Number fields follow the spin button pattern of web number inputs: Up/Down move one `step`, PageUp/PageDown move `page-steps` steps (10 by default), and Home/End jump to `min`/`max` when that bound is set; otherwise they keep their native caret movement. Steps use exact decimal arithmetic on the `min`/`default-value` lattice and replace the draft like typed text, so `edited`, `value-edited`, dirty tracking, native undo and `commit-policy` apply unchanged, and a step clamped at a bound emits nothing. Fields with `step: "any"`, read-only or disabled fields do not step. Set `wheel-stepping: true` to also step the focused editor with the wheel; `step-by` and `step-to-bound` expose the same path to custom buttons. The `KitNumberStepper` buttons still commit each press at once.

## Drafts and acceptance

`text` remains the native editing draft, including incomplete values. `field-state` exposes accepted value, draft, dirty/touched flags and the last transition; `accepted-text` projects the last accepted text. `committed(FieldCommit)` contains the candidate and the constraints under which it was accepted. A named codec key is application data, independent of visual position.

`commit-policy` defaults to `CommitPolicy.on-blur`. Choose `on-submit` or `on-change-when-valid` when appropriate. `commit()` reports validity and requests acceptance, `cancel-edit()` restores the last accepted draft, and `reset-field()` restores `default-value` and validation state. `ownership: Ownership.controlled` emits proposals; acknowledge accepted host values explicitly with `acknowledge(FieldValue)`. The typed [controllers guide](/guides/controllers/) covers custom codecs and exact decimals.

`access: FieldAccess.read-only` retains native text selection and prevents acceptance. Disabled controls reject interaction. Native caret, selection, undo and IME stay in the underlying TextInput. Headless tests exercise public input paths; they do not qualify physical keyboards, composition engines or screen readers on every platform.

Multiline fields keep mouse drags for native text selection. Their vertical scrollbar stays in a theme-sized gutter; wheel scrolling remains available when the text overflows.

## Shared options and composition

Fields already include `label`, `description`, `error`, validation and accessible naming. `leading-icon` and `trailing-icon` are decorative `IconGlyph` values; they do not replace a textual accessible name. Presentation options such as `decoration`, `corner-radius`, padding and text size preserve the native editor and shared validation.

[KitFieldLabel](/components/field-label/), [KitFieldDescription](/components/field-description/) and [KitFieldError](/components/field-error/) are composition primitives for custom layouts. Their compatibility links come from actual inherited APIs. They do not supply an editor, domain parsing or an independent validation controller. `KitInputGroup` and its start/end/text parts arrange an existing field.

## Submit and reset

`KitSubmitButton` and `KitFormButton` express form actions. Before dispatching a submission, call each field's `report-validity()` and focus the first invalid field. The Rust command must also accept checked domain values or validate its incoming payload; a disabled button is not a submission guard. Form examples keep the command and domain validation in Rust. `KitResetButton` invokes the application's reset action, which calls field resets and clears business state deliberately.

Calendar/date-range composites validate availability as well as HTML bounds. The time-zone field refuses nonexistent times and asks for an explicit occurrence when clocks repeat. Its accepted `ZonedDateTimeCommit` converts to `ResolvedDateTime`; no time zone is inferred from a local date-time.

Typed dates and times follow the same `commit-policy` as text fields. With the default `CommitPolicy.on-blur`, `KitDatePicker` accepts a valid typed date when focus leaves its editor, `KitDateTimeField` accepts the pair when focus leaves both of its editors, and `KitZonedDateTimeField` resolves when focus leaves its date, time and zone editors. Moving between the editors of one group is not a blur. Invalid text keeps the accepted value and the draft, and an unchanged value does not repeat `chosen`.

`on-change-when-valid` emits the final value of each native edit. Arrow keys and modifier keys leave the acceptance state unchanged.

`report-validity()` never moves focus. Report every field first, then call `focus-if-invalid()` only for the first invalid field. This avoids blur side effects while collecting errors.

`KitSubmitButton` requires a connected `validation-requested` callback that returns the form’s validity, or explicit `form-no-validate`. Handle `form-action(FormAction)` for submit/reset; `activated` remains an observation after the gate and cannot replace it. `KitResetButton` skips validation.

Text editing exposes UTF-8 byte offsets through `cursor-position`, `selection-anchor` and `set-selection(anchor, cursor)`. `insert-at-cursor(text)` takes focus and follows native insertion/filtering; `undo()`, `redo()`, `cut-selection()`, `copy-selection()` and `paste()` keep the same editor. `blur()` releases it. Use `context-menu: TextContextMenu.disabled` to replace the standard localisable menu.

Hidden passwords suppress cut/copy in both the menu and public functions; other read-only fields retain copy and selection.

The themed popup opens with right-click, Menu or Shift+F10. Arrow keys and Home/End move between enabled actions; Enter activates, Escape returns to the editor, and Tab continues normal focus navigation. Opening the owned menu preserves the field’s logical focus and does not trigger a blur commit. The popup uses accessible list/list-item roles because Slint 1.18 lacks menu roles; localise its name with `KitStrings.text-menu-label`. Touch long-press is not provided. Slint 1.18 restores parent focus when closing the popup; its outside dismissal press can be consumed before the clicked control receives focus. Its composition uses public Slint APIs and needs no image decoder or renderer feature.

Localise field defaults through the window-local `KitStrings` global, for example `KitStrings.required-message = "Ce champ est obligatoire."`. Format templates use `{count}`, `{value}`, `{min}` and `{max}`. Existing per-field message properties override those defaults. Set a `FieldBinding` message snapshot with `binding.set_messages(window.global::<slint_kit::ui::KitStrings>().get_fields())`; refresh that snapshot after changing locale. Typed parser failures are mapped to the supplied messages, and application/domain failures retain the text from their own error formatter.

The [text-field reference](/components/text-field/) explains callback payloads, native editing methods, focus, validation visibility and accessibility. Variants inherit those APIs. `focus-observed` tracks the editor and its owned editing menu, `secret-visibility` controls password presentation, and `input-method-hints` expresses the native capitalization/correction/completion preferences available in Slint. Localise action labels and count templates through `KitStrings`; per-field properties remain overrides.


Constraint-bearing text, choice, file and date/time controls share a non-focusing `report-validity()` and explicit `focus-if-invalid()`. Register reporting separately from focus in a mixed form. Zoned date-time reporting checks both local and zone parts before returning; its focus helper selects the first invalid part. **Behavior change:** choice, file and composed date/time reports no longer move focus automatically.

A read-only file or directory picker can remain a validation candidate while its browse action is inactive. Its focus helper records a request, not confirmation from the platform; keep an editable route available to correct the selection.


Compose interactive leading/trailing actions directly inside `KitTextField` with `KitInputGroupStart/End`. Bind each part’s `layout-order` to `field.leading-adornment-order` or `field.trailing-adornment-order`; the one native editor keeps filtering, selection, undo, constraints and commit policy. Give actions their own accessible names and access guards. Use `focus-on-pointer: false` for an auxiliary action that should not blur an edit. Slint’s visual `layout-order` leaves Tab in declaration order: editor, built-in actions, then declared children. `clearable` controls the optional single-line clear button, while `reveal-toggle: SecretToggleVisibility.hidden` hides the password toggle without overriding `secret-visibility`. Observe `cleared()` after a successful clear and `secret-visibility-changed(SecretVisibility)` after a state transition. `busy` and `busy-label` show activity independently of validation; they do not change `access` or hide validation checking. Configure the default with `KitStrings.busy-label`.


Keep accepted password values in typed application state. Show a nonsensitive acceptance status; never render accepted-text or committed value text beside a password field, and never print the password to logs.

Optional temporal values have an explicit absence event. On `KitDatePicker` and `KitDateTimeField`, handle `cleared()` as `None` in the Rust `Option<CivilDate>` / `Option<LocalDateTime>` model; `chosen` always carries a nonempty value. Enable the trailing action with `clearable` and localize `clear-label`. `clear()` edits native drafts, then acceptance follows `commit-policy`; `commit()` requests explicit validated acceptance. Both local date/time drafts must be empty to clear the pair; a partial pair, `required` or custom error preserves the accepted value. Controlled ownership waits for the host to acknowledge absence. Each native editor retains its own undo entry. The pair action stays within the focus group; `editor-focused` still reports the two native editors. The date/time clear action requests each editor’s public native clear separately. If one action is refused, the resulting incomplete draft does not clear the accepted pair. Slint 1.18.1 exposes no documented composition-state API or transaction across editors; do not infer composition from key events. Native IME behavior needs qualification on the target backend, which headless tests do not provide.

## Controlled color wheel

`KitColorWheel` proposes opaque `SrgbColorView` values at full HSV value. Install the kit once per window, bind `value` and `known`, apply `chosen` to a local draft and submit that draft through your typed Rust boundary. Increase `generation` on rebind to discard held input. Left/right adjust hue, up/down saturation, Enter requests commit and Escape requests cancellation. The owner restores the last accepted value on cancellation. Localize `label`, `unknown-label` and `value-format`, or their `KitStrings.color-wheel-*` defaults. The two-axis control exposes a named focus group and description; native screen-reader editing is not qualified.

## Text direction

Field labels, legends, help, errors, summaries, native editor, placeholder and formatted display default to logical start. The kit resolves Start/End to physical Left/Right using `KitTheme.direction`; it does not infer paragraph direction from text. Set `direction: ChoiceFlow.right-to-left` for RTL defaults. Slint 1.18.1 Software was observed rendering native Start at Left, so fields project these physical edges explicitly. Explicit `text-alignment: left/center/right` passes through unchanged. `flow` overrides only the field/summary chrome: built-in adornments and whole `KitInputGroupStart/End` parts mirror; bind their projected `layout-order`. Arrange content inside custom parts yourself. Tab follows declaration order. Slint’s ordinary focus loss can collapse a selected range; keyboard clear/reveal actions restore editor focus, while pointer actions retain focus and selection. OTP keeps explicit left alignment. Verify shaping, native editing, IME and accessibility on the target backend. On standalone Text-based helpers, an explicit inherited `horizontal-alignment` override goes directly to Slint; use a physical edge when needed.

Color fields and pickers treat `accepted-color` as the host value. Host changes and `alpha-mode` refresh idle hex/channel projections after earlier edits. A focused native editor keeps its draft, caret and undo history; after focus leaves, its text reflects the current accepted color. Controlled ownership waits for the host acknowledgement.
