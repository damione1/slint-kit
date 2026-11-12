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
        pattern: "[^@]+@[^@]+[.][^@]+";
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

Numeric drafts have two explicit formats: `NumberInputFormat.canonical` accepts dot decimals and correctly grouped comma thousands (`1,234.50`, EnCa/EnUs); `.decimal-comma` accepts comma decimals and space, NBSP or narrow NBSP thousands (`1 234,50`, FrCa/FrFr). Every group after the first has exactly three digits. Leading `+`/`-`, surrounding whitespace and valid exponents follow the shared numeric grammar. Malformed grouping stays editable but fails complete-value validation; no ambiguous separator is guessed, and normalization never rewrites the native draft/caret/undo.

Match the widget and codec: for a French decimal draft, use `number-format: NumberInputFormat.decimal-comma` and `FieldSpec::decimal(DecimalRules { locale: NumberLocale::FrCa, scale: DecimalScale::new(2).unwrap(), sign: SignPolicy::Signed, bounds: None })`. Register the named spec as usual. Keep `min`, `max`, `step` and numeric `default-value` in canonical ungrouped syntax, for example `min: "0"; max: "2000"; step: "0.01";`; these attributes do not use the editing locale. `FieldSpec::integer()` still uses EnCa with no locale argument. Only the four listed locales are supported; this does not add Indian grouping or other locale families.

Duration fields use decimal quantities and a required Rust codec unit: pair `text: "5"; suffix: "s";` with `FieldSpec::duration(DurationUnit::Seconds, rules)` (5000 ms stored). A minutes codec maps `5` to 300000 ms. Keep suffix, constraints and codec in the same unit; a suffix performs no conversion. Handle the fallible `DurationFieldSpec::format` when loading values: recurring or overly precise quantities are rejected without rounding. Money fields apply the currency and DecimalRules supplied to `FieldSpec::money`; visual symbols do not impose currency precision or business rounding. See [duration units and money rules](/guides/controllers/#duration-units-and-money-rules).

### Numeric editor platform limits

With Slint 1.18.1, installing the shared validation engine makes the native editor of a field declared `input-type: InputType.decimal` use `InputType.text`. This keeps native caret, selection and undo while allowing drafts such as `1e` and a configured decimal comma. The shared HTML engine still rejects incomplete or invalid values at acceptance. Restoring native `Decimal` would filter those drafts before the kit can validate them.

The numeric field component exposes its spin button role, range and increment/decrement actions separately. Its inner native text editor remains a text input: the winit/AccessKit bridge selects `NumberInput` only for native `Number` or `Decimal`. Slint 1.18.1 [input-method hints](https://docs.slint.dev/docs/slint/reference/keyboard-input/textinput/#input-method-hints) cover capitalization, correction and completion; they provide no independent numeric purpose. The kit cannot promise both the complete draft grammar and that native numeric metadata with this SDK.

Backend behavior differs. The Android backend derives its text or numeric input class from that same native type; this is a source-level finding, without a device keyboard test. Desktop winit maps both text and decimal to its normal IME purpose, while the winit WebAssembly input helper does not project the numeric type into an HTML input mode. Numeric soft keyboards, IME composition and native screen-reader behavior require qualification on the target backend; no numeric keyboard restoration is claimed here.

## Drafts and acceptance

`text` remains the native editing draft, including incomplete values. `field-state` exposes accepted value, draft, dirty/touched flags and the last transition; `accepted-text` projects the last accepted text. `committed(FieldCommit)` contains the candidate and the constraints under which it was accepted. A named codec key is application data, independent of visual position.

`commit-policy` defaults to `CommitPolicy.on-blur`. Choose `on-submit` or ### Loading typed dates and times

After the per-window `install`, `accepted-date`, year/month/week inputs, `accepted-time` and the atomic date/time `accepted-value` initialize accepted baselines and idle displays. Their installed starters supply only typed values; the catalogue’s matching draft strings are explicitly static preview content. An authored initial draft, even empty, remains untouched. Host updates or clears preserve focused native drafts, caret, selection and undo, then reconcile at blur/cancel. Invalid typed models are ignored rather than cleared.

`accepted-time` now uses `LocalTimeValue {present, time}`: `{}` is absence, while `{present: true, time: {}}` is midnight. TimeField keeps this property as host input; TimePicker makes it in-out and emits typed `chosen(LocalTimeView)` / `cleared()` as well as its existing `committed(FieldCommit)`. Local ownership updates the picker model; Controlled proposes and waits for a complete wrapper assignment. DateTime acknowledges a whole `LocalDateTimeView`, never half a pair. See [typed temporal host values](/guides/controllers/#typed-temporal-host-values) for the property table, Rust commit boundary and migration.

<h3 id="time-adjustments">Time adjustments</h3>

`KitTimePicker` applies time constraints before editing or proposing an adjusted value. With `min: "09:00"; max: "17:00"; step: "900";`, a minute adjustment moves 09:30 to 09:45 and an hour adjustment moves it to 10:30. `step` is in seconds. A button advances `ceil(segment duration / representable step)` points of the exact admissible grid; off-grid drafts recover in the requested direction.

An empty draft selects a seed on the first click, with no extra increment: use the first legal, exactly representable point from a valid minimum, otherwise the shared constraints’ valid default value when supplied, otherwise midnight/the first permitted point. Both directions use that same seed. TimePicker exposes no new `default-value` property. A nonempty incomplete or invalid draft stays unchanged.

Whole-time arithmetic carries between segments: 10:59 → 11:00. Without valid bounds, movement wraps across midnight on the grid without changing any date. Ordinary bounds clamp to legal endpoints. `min: "23:00"; max: "02:00";` is a periodic range: movement crosses midnight within it, then clamps at its endpoints rather than entering the excluded daytime interval. `step: "any"` removes the step-grid restriction; malformed/nonpositive step falls back to HTML time’s 60 seconds and malformed bounds are ignored.

Precision intersects the exact grid: only whole-minute or whole-second candidates representable by `LocalTime` are admitted. Fractional steps do not silently round values; if no representable legal candidate exists, adjustment does nothing. Disabled/read-only, unavailable services and active native preedit block replacement. The shared `KitTextField.replace-draft(string) -> bool` performs the admitted edit through the mounted native editor, retaining undo and normal edit notifications before ordinary commit validation. `adjust` is an observation, not an accepted value; use the existing typed commit/proposal/acknowledgement contract. Press-and-hold repetition is not provided by this change. Native IME and screen readers remain separately qualified.

`on-change-when-valid` when appropriate. `commit()` reports validity and requests acceptance, `cancel-edit()` restores the last accepted draft, and `reset-field()` restores `default-value` and validation state. `ownership: Ownership.controlled` emits proposals; acknowledge accepted host values explicitly with `acknowledge(FieldValue)`. The typed [controllers guide](/guides/controllers/) covers custom codecs and exact decimals.

`access: FieldAccess.read-only` retains native text selection and prevents acceptance. Disabled controls reject interaction. Native caret, selection, undo and IME stay in the underlying TextInput. Headless tests exercise public input paths; they do not qualify physical keyboards, composition engines or screen readers on every platform.

Multiline fields keep mouse drags for native text selection. Their vertical scrollbar stays in a theme-sized gutter; wheel scrolling remains available when the text overflows.

## Shared options and composition

Fields already include `label`, `description`, `error`, validation and accessible naming. `leading-icon` and `trailing-icon` are decorative `IconGlyph` values; they do not replace a textual accessible name. Presentation options such as `decoration`, `corner-radius`, padding and text size preserve the native editor and shared validation.

[KitFieldLabel](/components/field-label/), [KitFieldDescription](/components/field-description/) and [KitFieldError](/components/field-error/) are composition primitives for custom layouts. Their compatibility links come from actual inherited APIs. They do not supply an editor, domain parsing or an independent validation controller. `KitInputGroup` and its start/end/text parts arrange an existing field.

## Submit and reset

`KitSubmitButton` and `KitFormButton` express form actions. Before dispatching a submission, call each field's `report-validity()` and focus the first invalid field. The Rust command must also accept checked domain values or validate its incoming payload; a disabled button is not a submission guard. Form examples keep the command and domain validation in Rust. `KitResetButton` invokes the application's reset action, which calls field resets and clears business state deliberately.

Calendar/date-range composites validate availability as well as HTML bounds. The time-zone field refuses nonexistent times and asks for an explicit occurrence when clocks repeat. Its accepted `ZonedDateTimeCommit` converts to `ResolvedDateTime`; no time zone is inferred from a local date-time.

`KitZonedDateTimeField.occurrence-selection` exposes `ZonedOccurrenceSelection.unselected`, `.earlier` or `.later` independently of `resolution`. A repeated local time remains `ambiguous` after its occurrence is accepted: the time has two possible instants, but the user has answered. Acceptance removes the internal ambiguity blocker, preserving host `custom-validity` and all other constraints; normal validity/reporting can then pass when those constraints permit it. Both occurrence buttons reject further choices for that resolution and expose the accepted checked state. Localize `earlier-accepted-message` and `later-accepted-message` for the existing polite confirmation. Editing any date/time/zone draft or calling `resolve()` again clears the selection; access/stale guards and reentrant host updates remain effective. Keep consuming the typed `chosen` commit for business logic; the presentation enum alone is not a checked instant. Polite live-region metadata is not native screen-reader qualification.

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

KitSelect, KitCombobox and KitMultiSelect: With services installed for the window, the popup chooses space below or above its trigger, shifts horizontally when needed and bounds its width and height to that window. A tall option list scrolls within the available space and remains virtualized. Popup placement does not change controlled selection, native editor focus, or query reset behavior. For a custom KitSelectSurface composition, use its read-only chrome-height instead of estimating search height: requested option height + chrome-height, plus any separate status content, gives the desired popup content height before window bounds. This does not add a placement mode or width policy, and the separate native-popup Qt typing limitation above remains.

`KitCombobox` places a native `KitTextField` inline with a trailing popup trigger and a non-focus-stealing suggestion popup. Typing filters local `options` immediately and emits `query-edited(string)` once `query` reaches `min-query-length` (or when cleared), after `query-debounce`. `retention` defaults to `SelectionRetention.retain`, so replacing `options` with a remote result page preserves the accepted `selected` token, its `available` state and its last known `selected-label`. Set `allow-custom-value: true` and handle `custom-chosen(string)` when free-form text outside `options` is valid; with `false` (the default), blur restores `selected-label`. `clearable` and `clear()` reset `text`, `query` and `selected` and emit `cleared()`. `search-state: SearchPresentation.loading` displays inline and popup busy feedback while keeping current rows visible, and `end-reached()` fires when the popup viewport enters the last `near-end-rows` rows.

## Text direction

Field labels, legends, help, errors, summaries, native editor, placeholder and formatted display default to logical start. The kit resolves Start/End to physical Left/Right using `KitTheme.direction`; it does not infer paragraph direction from text. Set `direction: ChoiceFlow.right-to-left` for RTL defaults. Slint 1.18.1 Software was observed rendering native Start at Left, so fields project these physical edges explicitly. Explicit `text-alignment: left/center/right` passes through unchanged. `flow` overrides only the field/summary chrome: built-in adornments and whole `KitInputGroupStart/End` parts mirror; bind their projected `layout-order`. Arrange content inside custom parts yourself. Tab follows declaration order. Slint’s ordinary focus loss can collapse a selected range; keyboard clear/reveal actions restore editor focus, while pointer actions retain focus and selection. OTP keeps explicit left alignment. Verify shaping, native editing, IME and accessibility on the target backend. On standalone Text-based helpers, an explicit inherited `horizontal-alignment` override goes directly to Slint; use a physical edge when needed.

Color fields and pickers use atomic `accepted-value: ColorValue { present, color }` as the host value. The default is present opaque black; `{present:false}` (or `{}`) is absence, never an alpha-zero sentinel. `accepted-color` is removed. Replace every assignment or binding with a complete `accepted-value: {present:true,color:...}`; read `accepted-value.color` only when `accepted-value.present` is true. Publish the complete value to reintroduce a color after absence, including the same RGB. Host updates and alpha policy refresh idle projections; a focused native editor retains its draft, caret and undo until the existing synchronization boundary.

For optional colors set `required:false`. A valid empty commit emits `cleared()` and maps naturally to Rust `None`; `chosen(SrgbColorView)` remains present-only. Required/custom-invalid empties cannot clear the accepted value. Controlled ownership leaves accepted-value unchanged until the host acknowledges either outcome. `clear()` uses native editing, then ordinary blur or `commit()` decides acceptance; `clearable:true` opts into its shared clear action, false remains the default. Localize `clear-label` and `empty-label` (`KitStrings.clear-label` / `color-empty-label`, default “No color”). The empty preview is distinct from a transparent color. Absence initializes hex/channel editors empty; an admitted channel edit starts from RGB0/alpha255, a popup color choice establishes presence, and Cancel preserves prior absence. The public examples keep the clear button hidden and expose `clear-accent()` as a native-draft helper. Manual service bindings must adapt to `ColorResult.value: ColorValue`: after checking result.valid, read `.value.present` then `.value.color`. Parsing an empty string is valid/absent; wheel/hue/validate/channel results remain present and the other color grammar is unchanged.

The color palette popup is bounded in width and height by the window passed to install(&window). It opens above the field when space below is insufficient and shifts horizontally to remain within that window. Native scrolling with themed scrollbars keeps the palette and Cancel/Apply actions reachable when the content exceeds the available area, including at 200% text. This does not change color acceptance, controlled acknowledgement or cancellation.

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

Select and MultiSelect distinguish `label` (stable accessible name), `placeholder` (empty hint) and `value-label` (accessible current value). No extra visible label row is inserted; compose `KitFieldLabel` for a caption and connect its focus request to `take-focus()`. **Migration:** stop putting the selected display value or field name in `placeholder`. Set the name with `label`; for explicit `rows`/`state` projections, supply `selected-label` instead. An available empty option label stays empty. An unavailable controlled choice displays `unavailable-label` and `theme.icons.warning` without changing the host selection.

MultiSelect defaults to a count-and-label summary of its chips. Localize `KitStrings.multi-selection-count-format` (`{count}`), `multi-selection-value-format` (`{count}`/`{labels}`), and `selection-label-separator`; `select-label`, `multi-select-label`, the Select/MultiSelect placeholders, `combobox-placeholder` and `selection-unavailable-label` are window defaults. Per-component `value-label` can override the summary without changing membership. A controlled removal updates it after host acknowledgment. Combobox publishes `placeholder` separately from its stable `label`, defaulting the hint to `KitStrings.combobox-placeholder`. Its accessible value is nonempty native `text`, otherwise the accepted `selected-label`; preserve the native editor’s draft value while editing and its accepted selected-label projection independently of search results.

## One-time codes

Compose `KitKeypad` with `KitCodeIndicator` explicitly. The keypad only emits digit/delete/confirm callbacks; the host owns the secret, `filled` count, localized progress `label` and `validation: ValidationPresentation`. `invalid` and `valid` paint danger/success marks, while `pristine` and `checking` stay neutral. Include localized error/success feedback in the label when it should be announced: the indicator declares the standard Slint `accessible-live-region: polite`, but adds no internal message or automatic keypad association. `activation-started` may precede a cancelled press, so update accepted progress from digit/delete callbacks. `control-size: ControlSize.medium` preserves the default 12px mark with shipped spacing; compact/comfortable use two/four spacing units, all multiplied by text-scale. Presentation is not business validation: keep real acceptance in the host and expose counts/status, never the code. Qualify actual announcements on the target backend/screen reader separately.

`alphabet` defaults to `OtpAlphabet.ascii-digits`; `ascii-alphanumeric` also accepts ASCII letters. It supplies the default HTML `pattern`. For the default alphabet pattern, a compiled `edit-filter` checks ordinary insertions and explicit raw paste before native editing, including the resulting UTF-16 length after selection replacement. Rejected input preserves selection and existing undo history. A custom whole-value pattern keeps filtering off by default, so a valid prefix is not rejected; explicitly supply a prefix-accepting edit-filter when needed. Custom Unicode patterns remain supported. The ASCII digit pattern requests native `InputType.number`; other patterns use text input. This is a platform hint, without qualification of a device keyboard or OTP autofill. `completed(string)` reports a complete valid native edit once per distinct complete draft, after edited/value-edited callbacks; clearing or editing to an incomplete draft rearms it. Host text assignments do not simulate completion. Completion is not acceptance: retain FieldSpec::accept and submission guards. `FieldDecoration.cells` is the OTP default; `group-size` adds decorative boundaries without adding characters or editors. The inherited boxed/underlined styles remain available. Native formatted paste remains unsupported in Slint 1.18.1: copy the raw code without spaces, hyphens or trailing newline. Ctrl/Cmd+V, explicit paste and the kit menu retain the native paste path and never rewrite the clipboard. Core OtpRules::normalize_paste and OtpDraft::normalize_paste provide typed normalization for an explicit data boundary; they are not a native editor paste hook. Paths that bypass ordinary key/paste interception, custom unfiltered edits and host text/limit changes can still invoke strict overflow correction and discard native history. Composition and screen readers require target-backend qualification.

### Email domain policy and composite error ownership

HTML email syntax intentionally permits a single-label domain such as `john@acme`. The public single-address examples choose an additional product policy with `pattern: "[^@]+@[^@]+[.][^@]+"`. It accepts dotted domains/subdomains and rejects a missing suffix without checking DNS or mailbox delivery. The generic component keeps HTML-compatible defaults; omit that pattern when a single-label domain is allowed. This policy reports `patternMismatch`, independently of the intrinsic `typeMismatch` rule. `FieldSpec::email().accept(commit)` and `form_schema!` submission read the same configured constraints, so Rust cannot accept a draft rejected by the example's pattern. Initialize services once with `install` as above.

DateTimeField checks its native date/time parts as well as whole-pair constraints. A required pair requires both parts; an optional pair may be empty only when both are empty. Error presentation and `focus-if-invalid()` target the invalid part. Complete-pair bounds/custom validity remain at the pair level, and ZonedDateTimeField keeps zone errors with the zone editor. Resetting pair validation clears child error presentation too; native drafts, caret and undo stay in their existing editors.
