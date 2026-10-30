# Slint Kit Components

Public Slint components, model projections and weak-window Rust adapters for [Slint Kit](https://github.com/damione1/slint-kit). The package includes the complete `ui/` library and its generated shared contracts.

Applications normally use the [`slint-kit` facade](https://crates.io/crates/slint-kit) and [`slint-kit-build`](https://crates.io/crates/slint-kit-build). The build helper registers the installed library as `@kit`:

```slint
import { KitButton, KitTextField } from "@kit/kit.slint";
```

Rust 1.92 or later and Slint 1.18.1 are required. The default `adapter` feature exposes Rust bindings; disabling it leaves the installed Slint library available to build tools without a UI runtime.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

## Form constraints and custom fields

The native editor owns text selection, caret, composition and undo. HTML constraint
validation is shared by the field variants rather than being reimplemented in each
application parser. The reference is the [HTML Living Standard](https://html.spec.whatwg.org/multipage/input.html),
which is also the target of [W3C's HTML reference](https://www.w3.org/TR/html/).

Export the validation global and its boundary types from the application UI:

```slint
export { KitFormValidation, HtmlInputKind, NumberInputFormat,
    KitInputConstraints, KitValidationResult, KitValidityState } from "@kit/kit.slint";
```

Install the Rust engine once per window, before showing it:

```rust
mod ui { slint::include_modules!(); }
// In main, after constructing the window:
slint_kit::init_form_validation!(ui, window);
```

Use the facade's actual dependency name (`slint_kit` for a registry application;
`kit` is this workspace's alias). Installation is mandatory for validating controls: `validation-ready` is false and checks fail closed until the engine is installed. Draft editing and static presentation fixtures remain available. Do not treat an uninitialized control as a validated input.

```slint
KitEmailField {
    label: "Work email";
    required: true;
    pattern: ".+@example\\.com";
    pattern-message: "Use your example.com address.";
}
```

| Option | Applies to | Behavior |
| --- | --- | --- |
| `label`, `description`, `placeholder` | Text field family | Built-in presentation; no separate label component is needed. |
| `required` | Fields, checkboxes, radio/selection groups, file pickers | Empty optional values are allowed; an unchecked required checkbox or an empty required selection is invalid. |
| `multiple` | Email | A comma-separated list; validate each address and each pattern match. |
| `min-length`, `max-length` | Text, email, URL, password, search, telephone, textarea | UTF-16 code units, after user editing; `-1` means absent. The grapheme character counter remains a separate feature. Long drafts are reported invalid, without truncating native composition. |
| `pattern`, `pattern-enabled`, `pattern-message` | Text, email, URL, password, search, telephone | ECMAScript `v` syntax, full-value match. `pattern-enabled: true` distinguishes an explicit empty pattern from no pattern. Invalid patterns impose no constraint, as in HTML, and expose `pattern-error` to the author. |
| `min`, `max`, `step`, `default-value` | Number and ISO temporal fields | Canonical HTML strings. Exact step arithmetic; `step: "any"` removes the step constraint. Minimum, then initial value, then the type default establishes the step base. |
| `number-format` | Numeric fields | `canonical` uses a dot; `decimal-comma` accepts localized comma decimals and grouped digits, exposing a canonical `value` without rewriting the editor. |
| `validation-timing` | Text field family | Default `on-blur`, also `on-input` or `on-submit`. Invalid email characters and a second `@` are shown while typing. |
| `custom-validity` / `set-custom-validity` | Validating controls | An application error; clear with an empty string. This complements intrinsic validation. |
| `access` | Controls exposing `FieldAccess` | Disabled controls are excluded from validation. HTML-compatible read-only text controls are excluded; read-only selection controls are a native extension and remain subject to selection constraints. |

`KitIntegerField` defaults to `step: "1"`; decimal variants correspond to an HTML
number input configured with `step="any"`. Range and color controls keep their existing typed controller projections; the low-level validator does not replace their value sanitizers. Currency precision, business bounds,
OTP alphabets, masks, and accepted domain values still belong to the typed Rust
controller. Twelve-hour time editing projects onto the same canonical time constraints. Zoned date/time fields additionally check IANA identifiers; their typed codecs retain responsibility for zone resolution and ambiguity.

`pattern` does not apply to a textarea in HTML. A custom textarea rule can use
`HtmlPattern` in Rust and report its outcome with custom validity. A mask guides a
specific structure; a regex validates a complete value without rewriting it.

`validity` exposes the independent HTML flags. `value` is the sanitized semantic
value; `text` and `edited` remain native drafts. `check-validity()` returns status
and emits `invalid`; `report-validity()` also exposes the error and focuses the
control. Enter never emits a field commit for an intrinsically invalid value.
The host must guard its submit path too: use `HtmlFormValidation<K>` with stable
field keys and `first_invalid()`, or wire `KitSubmitButton.validation-requested`.
`form-no-validate` bypasses the submit guard, and reset buttons never validate.
`reset-field()` restores `default-value` and editing presentation; custom validity
is retained until explicitly cleared, as in HTML. Restore controlled accepted
values through the host's reset handler as well.

For Rust submissions that bypass UI callbacks, call
`FieldBinding::set_html_constraints(HtmlConstraints { ... })` before accepting
values. These rules guard blur and submit before the domain parser runs; its input
is the sanitized HTML value. Use a canonical parser and a localized formatter when
necessary. `HtmlPattern::compile` reports authoring errors directly.

To build another skin, import `FormConstraintState` from `@kit/validation.slint`
and compose it with the existing editor or selection primitive. It provides the
same flags, timing and messages without drawing another input. Custom messages,
controllers, themes, prefix/suffix slots and callbacks remain independently
replaceable. `KitFieldLabel` is a composition primitive: ordinary fields already
provide their own `label` property and focus wiring.

This is native constraint validation, not a browser DOM. HTTP form actions,
credential/password-manager autofill, browser datalists, browser-specific soft
keyboards and Display-P3 CSS color parsing require separate host/backend support.
The platform may ignore input-method hints. No placeholder attributes are exposed
as promises of unsupported browser services.


## Phone numbers

`KitTelephoneField` adds strict phone rules to the native editor. It blocks typed
letters and separators; formatted paste is accepted without silently dropping
letters. The canonical `value` and phone commit event contain digits and an
optional leading `+`. `edited` retains the native draft; `value-edited` supplies
the canonical value separately. Use the typed `PhoneNumber` string in Rust so a
national leading zero is preserved.

Choose `number-mode: PhoneNumberMode.national`, `.international` or `.either`.
Set `expected-digits` for an exact count, or `min-digits` and `max-digits` for a
range within 1–15 digits. The country calling code counts; `+` does not. Match
these settings with `PhoneRules::exact` or `PhoneRules::new` in the host parser.
`pattern` remains a regex on the canonical value. Use
`FieldBinding::set_draft_normalizer` with `normalize_phone` before HTML rules so
formatted paste receives the same validation in Rust.

`display-mask` uses `#` digit slots: `###-###-####`, `## ## ## ## ##`, or
`+# (###) ###-####`. It formats the field at rest; focus preserves the native
draft, cursor, selection and undo. It never changes the canonical value or hides
overflowing digits. Invalid masks expose `format-error` and display the canonical
value. Masks do not verify country allocation or whether a line exists.

HTML's unopinionated telephone state is still available with
`KitTextField { input-kind: HtmlInputKind.telephone; }`. Its HTML validation rules
have not been narrowed globally.

## Tailwind presentation

Every component uses the shipped Tailwind presentation by default. Compile
through `slint_kit_build::compile("ui/app.slint")` and import the public
`@kit/kit.slint` library. `KitThemes.light` and `.dark` provide complete themes,
including semantic colors, spacing, elevation and Lucide icon roles.

The theme embeds no font. Import `{ InterFont }` from
`@kit/fonts/inter.slint` separately and set the Window's
`default-font-family: InterFont.family` to use Inter. Its four static font files
are about 1.6 MB. Keep the third-party notices supplied with the kit.

Assign a theme to `KitThemeContext.current` for the window default, or pass it
explicitly to a component. Application-supplied children do not inherit a
container's theme automatically: bind their `theme` to the container's theme.
See the themes guide for local radius, padding, typography and elevation.

## Icons

`KitIcon` and `KitIconButton` paint an `IconGlyph`: one of the kit's own
`KitGlyphs`, or a Lucide icon. Buttons and text fields accept `leading-icon` and `trailing-icon`; built-in action glyphs read semantic roles from `theme.icons`. `@kit/icons/lucide/all.slint` exports
`LucideIcons` with every icon of the vendored Lucide release as a named pure
function (`LucideIcons.arrow-up()`), and each Lucide category has a file holding
the same glyphs (`LucideArrows` in `@kit/icons/lucide/arrows.slint`). The files
are generated by `cargo run -p xtask -- generate-icons` and never edited by
hand. An icon that is not called adds nothing to a binary. Icon size follows
`control-size` and the theme's `text-scale`; the stroke width is the theme's
`shape.icon-stroke` token, scaled with the icon or, with
`stroke-mode: IconStroke.absolute`, fixed in logical pixels. A glyph's `fill`
layer paints its filled shapes, if any, with the same ink. An informative icon
(`purpose: IconPurpose.informative`) is announced as an image named by `label`.
Lucide (ISC) and the icons it derives from Feather (MIT) keep their licenses;
see `THIRD-PARTY-NOTICES.md`.
