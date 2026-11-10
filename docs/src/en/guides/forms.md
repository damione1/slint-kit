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

`KitMaskedField` accepts a declarative `mask`: `#` is an ASCII digit, `A` an ASCII letter, `*` an ASCII alphanumeric slot, `\` escapes a literal, and other characters are literal separators (1–256 tokens, at least one slot). After `slint_kit::install(&app)`, the shared HTML engine derives `min-length`, `max-length`, a complete-value `pattern` and the `_`-slot placeholder. The default `TextLengthPolicy.validate` preserves host text and native deletion, cut, undo and redo: an invalid or overlong draft remains visible and cannot be accepted. Changing `mask` validates the existing text without cropping it. Ordinary ASCII key insertions and explicit native paste are checked before editing; an invalid draft can be repaired incrementally at its first mismatch. Non-ASCII input and editing paths without a public pre-insertion event remain native and are checked by complete-draft validation; this is not an IME qualification. Literals are typed or pasted explicitly; no automatic separator insertion rewrites the caret. Before install, a nonempty mask keeps keyboard and public paste/cut actions read-only. Invalid mask syntax is exposed through `format-error` and `KitStrings.mask-format-message`, also used as an intrinsic validation error. Pair with `FieldSpec::masked(MaskPattern::compile("AA-####").unwrap())` to guard Rust acceptance. The cache retains at most 64 sources bounded by `MaskPattern::MAX_SOURCE_BYTES`, including failed compilations; it never retains oversized source strings.

Number fields follow the spin button pattern of web number inputs: Up/Down move one `step`, PageUp/PageDown move `page-steps` steps (10 by default), and Home/End jump to `min`/`max` when that bound is set; otherwise they keep their native caret movement. Steps use exact decimal arithmetic on the `min`/`default-value` lattice and replace the draft like typed text, so `edited`, `value-edited`, dirty tracking, native undo and `commit-policy` apply unchanged, and a step clamped at a bound emits nothing. Fields with `step: "any"`, read-only or disabled fields do not step. Set `wheel-stepping: true` to also step the focused editor with the wheel; `step-by` and `step-to-bound` expose the same path to custom buttons. The `KitNumberStepper` buttons still commit each press at once.

### Numeric editor platform limits

With Slint 1.18.1, installing the shared validation engine makes the native editor of a field declared `input-type: InputType.decimal` use `InputType.text`. This keeps native caret, selection and undo while allowing drafts such as `1e` and a configured decimal comma. The shared HTML engine still rejects incomplete or invalid values at acceptance. Restoring native `Decimal` would filter those drafts before the kit can validate them.

The numeric field component exposes its spin button role, range and increment/decrement actions separately. Its inner native text editor remains a text input: the winit/AccessKit bridge selects `NumberInput` only for native `Number` or `Decimal`. Slint 1.18.1 [input-method hints](https://docs.slint.dev/docs/slint/reference/keyboard-input/textinput/#input-method-hints) cover capitalization, correction and completion; they provide no independent numeric purpose. The kit cannot promise both the complete draft grammar and that native numeric metadata with this SDK.

Backend behavior differs. The Android backend derives its text or numeric input class from that same native type; this is a source-level finding, without a device keyboard test. Desktop winit maps both text and decimal to its normal IME purpose, while the winit WebAssembly input helper does not project the numeric type into an HTML input mode. Numeric soft keyboards, IME composition and native screen-reader behavior require qualification on the target backend; no numeric keyboard restoration is claimed here.

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


Constraint-bearing text, choice, file and date/time controls share a non-focusing `report-validity()` and explicit `focus-if-invalid()`. Register reporting separately from focus in a mixed form. Zoned date-time reporting checks both local and zone parts before returning; its focus helper selects the first invalid part. **Behavior change:** choice, file and composed date/time reports no longer move focus automatically. `KitCheckbox`, `KitSwitch` and `KitRadioGroup` activate with `Space` and reject `Enter` (`Return`) so an enclosing form can handle submission; `KitRadioGroup` accepts both `UpArrow`/`DownArrow` and flow-aware `LeftArrow`/`RightArrow` navigation.

A read-only file or directory picker can remain a validation candidate while its browse action is inactive. Its focus helper returns false when browsing is disabled, unsupported or pending, so the form can try another invalid field or its fallback. An admitted request still requires platform qualification; keep an editable route available to correct the selection.


Compose interactive leading/trailing actions directly inside `KitTextField` with `KitInputGroupStart/End`. Bind each part’s `layout-order` to `field.leading-adornment-order` or `field.trailing-adornment-order`; the one native editor keeps filtering, selection, undo, constraints and commit policy. Give actions their own accessible names and access guards. Use `focus-on-pointer: false` for an auxiliary action that should not blur an edit. Slint’s visual `layout-order` leaves Tab in declaration order: editor, built-in actions, then declared children. `clearable` controls the optional single-line clear button, while `reveal-toggle: SecretToggleVisibility.hidden` hides the password toggle without overriding `secret-visibility`. Observe `cleared()` after a successful clear and `secret-visibility-changed(SecretVisibility)` after a state transition. `busy` and `busy-label` show activity independently of validation; they do not change `access` or hide validation checking. Configure the default with `KitStrings.busy-label`.


Keep accepted password values in typed application state. Show a nonsensitive acceptance status; never render accepted-text or committed value text beside a password field, and never print the password to logs.

Optional temporal values have an explicit absence event. On `KitDatePicker` and `KitDateTimeField`, handle `cleared()` as `None` in the Rust `Option<CivilDate>` / `Option<LocalDateTime>` model; `chosen` always carries a nonempty value. Enable the trailing action with `clearable` and localize `clear-label`. `clear()` edits native drafts, then acceptance follows `commit-policy`; `commit()` requests explicit validated acceptance. Both local date/time drafts must be empty to clear the pair; a partial pair, `required` or custom error preserves the accepted value. Controlled ownership waits for the host to acknowledge absence. Each native editor retains its own undo entry. The pair action stays within the focus group; `editor-focused` still reports the two native editors. The date/time clear action requests each editor’s public native clear separately. If one action is refused, the resulting incomplete draft does not clear the accepted pair. Slint 1.18.1 exposes no documented composition-state API or transaction across editors; do not infer composition from key events. Native IME behavior needs qualification on the target backend, which headless tests do not provide.

## Controlled color wheel

`KitColorWheel` proposes opaque `SrgbColorView` values at full HSV value. Install the kit once per window, bind `value` and `known`, apply `chosen` to a local draft and submit that draft through your typed Rust boundary. Increase `generation` on rebind to discard held input. Left/right adjust hue, up/down saturation, Enter requests commit and Escape requests cancellation. The owner restores the last accepted value on cancellation. Localize `label`, `unknown-label` and `value-format`, or their `KitStrings.color-wheel-*` defaults. The two-axis control exposes a named focus group and description; native screen-reader editing is not qualified.

## Editable combobox and remote search

`KitCombobox` places a native `KitTextField` inline with a trailing popup trigger and a non-focus-stealing suggestion popup. Typing filters local `options` immediately and emits `query-edited(string)` once `query` reaches `min-query-length` (or when cleared), after `query-debounce`. `retention` defaults to `SelectionRetention.retain`, so replacing `options` with a remote result page preserves the accepted `selected` token, its `available` state and its last known `selected-label`. Set `allow-custom-value: true` and handle `custom-chosen(string)` when free-form text outside `options` is valid; with `false` (the default), blur restores `selected-label`. `clearable` and `clear()` reset `text`, `query` and `selected` and emit `cleared()`. `search-state: SearchPresentation.loading` displays inline and popup busy feedback while keeping current rows visible, and `end-reached()` fires when the popup viewport enters the last `near-end-rows` rows.

## Text direction

Field labels, legends, help, errors, summaries, native editor, placeholder and formatted display default to logical start. The kit resolves Start/End to physical Left/Right using `KitTheme.direction`; it does not infer paragraph direction from text. Set `direction: ChoiceFlow.right-to-left` for RTL defaults. Slint 1.18.1 Software was observed rendering native Start at Left, so fields project these physical edges explicitly. Explicit `text-alignment: left/center/right` passes through unchanged. `flow` overrides only the field/summary chrome: built-in adornments and whole `KitInputGroupStart/End` parts mirror; bind their projected `layout-order`. Arrange content inside custom parts yourself. Tab follows declaration order. Slint’s ordinary focus loss can collapse a selected range; keyboard clear/reveal actions restore editor focus, while pointer actions retain focus and selection. OTP keeps explicit left alignment. Verify shaping, native editing, IME and accessibility on the target backend. On standalone Text-based helpers, an explicit inherited `horizontal-alignment` override goes directly to Slint; use a physical edge when needed.

Color fields and pickers treat `accepted-color` as the host value. Host changes and `alpha-mode` refresh idle hex/channel projections after earlier edits. A focused native editor keeps its draft, caret and undo history; after focus leaves, its text reflects the current accepted color. Controlled ownership waits for the host acknowledgement.

Valid native color hex/channel edits commit on blur as well as Enter; `chosen` emits once for that changed draft. Invalid or incomplete blurred drafts return to the accepted color text. `commit()` remains an explicit immediate request; `commit-requested` and channel request observations remain tied to Enter.

## Coordinated forms

`KitForm` supplies the named native form scope; `FormBinding<W, K>` reuses the shared HTML engine and `FormCoordinator`. Declare the application record once with `form_schema!(Record, Window, FieldKeys, { ... })`: `field: required(Type, FieldSpec, message)` for a required native draft, or `field: checked` for a checkbox boolean. The macro generates the typed record, Rust field ports and global form callbacks. The [registration starter](/components/form/) includes its four-field schema and business code in fifteen Rust lines total, with no helper module or per-field Rust wiring.

The Slint window exports `<field>-field` snapshots, `<field>-candidate` commits (or `<field>-state` for a checkbox), `report-<field>`, `focus-<field>`, `reset-<field>` and `<field>-error`. Export `valid`, `dirty`, `submitting`, `errors`, `focus-form` and the `validate-form`, `field-edited`, `reset-form`, `focus-error`, `send-form` callbacks. The macro connects these public conventions; missing ports, wrong codecs or an enum variant absent from the schema fail compilation. Field declaration order determines first-invalid focus. `Record::on_submit(&window, business)` admits the typed record and completes a synchronous success after business returns. For asynchronous submission and server errors, use `Record::bind(&window)` to obtain `TypedFormBinding`, then `submit()` and `finish(ticket, errors)` explicitly.

Declare each schema field once with `bind_required(key, FieldSpec, commit_getter, FormFieldPorts, error_text)` or `bind_checked` for checkbox state (`bind_value` accepts other already typed values). These methods register the same key and ports that supply the typed schema value. Combine values with `zip` and `map` to construct the Rust struct; no caller validation loop or separate parsing function is needed. `with_schema` returns `TypedFormBinding`, whose `submit()` reports all registered controls and runs each field codec once through the shared engine before constructing the record. The low-level `submit_typed(parser)` remains available for custom schemas. A parse failure returns nonempty `FormErrors<K>` and displays the keyed messages. Business code receives the typed struct and ticket, never an unchecked nonempty string. `submit(payload)` remains available for already typed payloads; `submit_without_validation` is an explicit validation opt-out that still enforces busy and weak-window guards.

`finish(ticket, Vec<FieldError<K, String>>)` maps server messages to the registered error setters and summary. Call `field_edited(&key)` from native edit callbacks to clear only that field’s server error, retaining other messages and native drafts, caret and undo. Reset clears errors, calls every field’s reset and invalidates prior tickets. Late or foreign responses cannot update the current form.

Reports collect every error before requesting focus. A focus callback returns whether it can request focus; an invalid read-only file stays invalid even if its browse action cannot take focus. The adapter tries subsequent invalid fields, then `set_focus_fallback` can call `KitForm.take-focus()` if none is focusable. Summary actions use `focus_target` with the supplied stable `FocusRequest`, not a visual row index.

Single-line implicit Enter waits for the native editor’s accepted callback. Multiline Enter, IME composition, consumed host shortcuts and auxiliary editors keep their behavior; set `implicit-form-submit: false` for an editor that owns Enter, or `implicit-submit: false` for the whole form. Checkbox/switch and shared radio/toggle Enter bubble to an enclosing form, while Space still changes choices. Selectors and tag entry keep their own Enter action. The form adds no blank Tab stop. Native IME, assistive technology and backend behavior still need target qualification.

Use `KitFormErrorSummary` for `FormProjection.errors` and each control’s error port for its field message. `valid` combines current HTML validity and server rejection; `dirty` aggregates field snapshots; `submitting` follows the active coordinator ticket. For custom `TextInput`, wire its accepted callback to `form.submit()` explicitly. Initialize `install(&window)` once before showing it.

Custom choice skins based on `KitPressable` can enable `form-submit-on-return`; its host key hook still runs first. Ordinary action buttons keep that option disabled and run their own action.

### Choice validation timing

`validation-timing` defaults to `ValidationTiming.on-blur`; choose `on-input` for admitted user edits or `on-submit` for explicit reporting. Render a host-owned `KitFieldError` only when `error-visible` is true; `validation-message` can already be nonempty while pristine. `check-validity()` leaves presentation unchanged, `report-validity()` reveals errors without moving focus, and `reset-validation()` clears interaction flags. Controlled proposals count as edits before acknowledgment; read-only/disabled actions and programmatic selection changes do not.

For Checkbox, Switch, RadioGroup, SegmentedControl, ToggleGroup, Select, Combobox, MultiSelect and TagInput, the form owns the message placement. MultiSelect and TagInput no longer add an internal error row. Keep existing domain errors such as TagInput.error in the same host message with explicit precedence. `has-focus` on MultiSelect/TagInput includes chips and the owned popup; moving within the group is not blur. Text-based fields expose the same `error-visible` constraint projection while retaining their built-in message and `validation-visible` presentation contract.

```slint
import { KitField, KitSelect, KitFieldError, ValidationTiming } from "@kit";
KitField {
    choice := KitSelect { required: true; validation-timing: ValidationTiming.on-blur; }
    if choice.error-visible: KitFieldError { text: choice.validation-message; }
}
```
