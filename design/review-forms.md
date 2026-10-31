# Form component quality review

Source baseline: `c830f49c` (`develop`, Slint Kit 0.4.0, Slint 1.18.1). Final form/library source: `5201a0aa`; final native component qualification: `97ccab8a` (same library and form gallery inputs).

The design review applies `mac-design`, `design-kit`, `design-system`, `ux-design` and `layout-grid` to the existing Slint public controls. Native text editing, the shared HTML constraint engine, stable identity and exact domain values remain the interaction foundation.

## Coverage

All 57 public controls/composition primitives in these families, plus the internal `FormConstraintState` and the `KitFormValidation` global, have been read against the following checks: semantic theme/recipe use; leading edges and inline gaps; intrinsic sizing; enlarged text; focus, disabled and read-only state; accessible label/value/actions; and host-owned validation/commit behavior. Static inspection alone is not native interaction or screen-reader qualification.

| Family | Components read | Result |
| --- | --- | --- |
| `field.slint` | `KitField`, `KitFieldLabel`, `KitFieldDescription`, `KitFieldError`, `KitFieldGroup`, `KitFieldSet`, `KitInputGroup`, `KitFieldSeparator`, `KitInputGroupStart`, `KitInputGroupEnd`, `KitInputGroupText`, `KitFormErrorSummary` | Token-driven composition, wrapping labels and error announcement reviewed. Complete field groups now cap their height intrinsically even inside a stretching drawer layout; dynamic multiline height still follows the native draft. Error-summary focus targets remain host-owned. |
| `input.slint` | `KitTextField`, `KitPasswordField`, `KitSearchField`, `KitEmailField`, `KitUrlField`, `KitTelephoneField`, `KitTextArea`, `KitOtpField`, `KitMaskedField` | One persistent native editor reviewed for all variants. Search clear now emits its empty semantic value. The placeholder stays hidden during native preedit; OTP placeholders reserve the final letter-spacing advance instead of eliding valid slots. |
| `number.slint` | `KitIntegerField`, `KitDecimalField`, `KitMoneyField`, `KitPercentField`, `KitQuantityField`, `KitDurationField`, `KitNumberStepper` | Exact domain payloads retained; stepper has named and disabled increment/decrement actions. Its actions now match the native editor height and remain square at all three densities and 200% text. No renderer-float arithmetic added. |
| `choice.slint` | `KitCheckbox`, `KitSwitch`, `KitRadioGroup`, `KitSegmentedControl`, `KitToggleGroup`, `KitChip`, `KitMultiSelect`, `KitTagInput`, `KitCombobox`, `KitSlider`, `KitRangeSlider`, `KitRating` | RTL segments now reverse their visual projection while retaining tokens and cancelling held activation when reordered. Radio marks and switch tracks centre beside long/enlarged labels; slider heights follow theme density at 200%. Selection identity and disabled/read-only guards reviewed. |
| `calendar.slint` | `KitCalendar`, `KitDateField`, `KitDatePicker`, `KitDateRangePicker`, `KitMonthField`, `KitYearField`, `KitIsoWeekField` | Roving keyboard, constrained date availability, provisional range confirmation and localizable labels reviewed. |
| `datetime.slint` | `KitTimeField`, `KitTimePicker`, `KitDateTimeField`, `KitZonedDateTimeField` | Civil time/date separation and explicit ambiguity choice reviewed. Ambiguity buttons now inherit the requested control density. |
| `color.slint` | `KitColorField`, `KitColorPicker` | Draft/commit separation reviewed. Channel inputs now filter numeric drafts and enforce exact integer constraints 0–255 before commit; channel sliders and popup actions inherit density. Swatch radii derive concentrically from their parent. |
| `files.slint` | `KitFileList`, `KitFilePicker`, `KitDirectoryPicker`, `KitFileDropZone` | Stable removal identity, host capability outcomes and browse alternatives reviewed. File-row padding now uses three spacing units, aligning it to the four-pixel grid. |
| `validation.slint` | `FormConstraintState` | Shared nonvisual HTML constraint state reviewed; validation initialization and Rust submission remain host responsibilities. |

## Showcase UX

- Flow: choose Profile, Billing, Choices, Schedule or Files in the showcase → edit or select → commit, apply, save or reset → inline result. Native editors retain selection and undo; provisional calendars and color drafts expose Cancel.
- Surfaces: fields and independent choices are inline; calendars, searchable choices and color drafts use anchored popovers; ambiguous local time choices remain adjacent to their context.
- States reviewed: pristine, invalid, checking, disabled/read-only, selected and pending/cancelled/failed/unsupported native requests. Long labels wrap; 200% text remains part of native visual verification.
- Words: visible labels identify values; action names say Save, Create, Reset, Apply, Cancel, Choose or Remove. Essential instructions stay visible.
- Access: Tab, arrows, Return, Escape and assistive actions follow the existing shared primitives. VoiceOver and IME require native qualification and are not inferred from compilation or headless tests.

## Regression checks

Added real public-consumer checks in `tests/src/part2.rs`:

- Search clear emits exactly one empty semantic value and keeps its native editor usable.
- RTL segments reverse visual order, keep stable tokens/generation, block disabled activation and cancel a pointer press when its represented identity changes.
- Standalone/range/channel sliders and ambiguity choices follow all three densities at 100% and 200%; ambiguity buttons are compared with a real peer button rather than an assumed line height.
- Color byte channels reject letter insertion, retain the persistent editor and block out-of-range and fractional commits.
- Switch tracks and radio marks remain geometrically centred beside both short and wrapping labels at every density and text scale.
- Numeric stepper actions are square, share their native editor height and align their upper edges at every density and text scale. The check failed before the correction with a real action size of 52×51.2 against a 52-high editor (`/tmp/slint-quality-stepper-red.log`).
- A real software-renderer snapshot of an empty OTP field paints placeholder ink in every one of its six slots in both themes at 100% and 200%, protecting against ellipsis hiding available slots.

Implementation and initial regressions are committed in `80b61361`; documentation example consistency in `a3a69ada`; intrinsic complete-field geometry in `5702e038`; exact stepper action alignment in `21d9ddc5`. `CARGO_TARGET_DIR=/Users/damien/Projects/slint-kit-incremental-docs/target CARGO_INCREMENTAL=0 cargo test -p kit-tests --locked` passed all 159 tests on the first combined review tree (`/tmp/slint-quality-form-tests.log`), then all 160 after the drawer/field regression (`/tmp/slint-quality-field-intrinsic-green.log`), and all 161 after the stepper geometry regression (`/tmp/slint-quality-stepper-green.log`).

## Native visual inspection

The `before` captures under `docs/.verification/quality-0.4/` use the actual winit/Skia showcase. Inspected `choices-light-2x-top`, `choices-dark-2x-middle`, `profile-light-2x-middle`, `profile-dark-2x-end`, `billing-light-1x-end`, `schedule-light-2x-middle`, `schedule-dark-2x-top` and `files-dark-2x-end`. They confirm the original switch/radio centring defect and the elided OTP placeholder at enlarged text. General field, suffix, numeric, calendar and channel layouts have a shared leading edge and readable unscaled/scaled typography.

Inspected all 57 component examples in each of light/dark and 100%/200% text: 228 native discovery renders under `components/final/{light,dark}-{1,2}x`. The `form-sheets` contact sheets aid inspection; original captures remain intact. OTP and radio were also inspected at full resolution. The discovery pass intentionally changed sources and did not receive a provenance manifest; its filenames do not imply final qualification.

This inspection exposed seven documentation-example issues, fixed in `a3a69ada`: `combobox` omitted its component theme and did not scale its heading/body; `chip` and `multi-select` did not scale helper/result text; `field`, `field-group`, `input-group-start` and `input-group-end` omitted the composition primitive's parent theme. English/French examples were corrected; the affected quickstart heading/gap was kept consistent. The existing quickstarts already forwarded the theme and default body size. All seven exact examples compile with the capture interpreter in both languages (`/tmp/slint-quality-form-examples-en.log` and `-fr.log`).

The drawer discovery also exposed a complete field wrapper absorbing spare height, despite zero stretch. Its regression checks the native editor/label bounds in a real public drawer, and failed before the `KitField` intrinsic-height correction (`/tmp/slint-quality-field-intrinsic-red.log`). All 160 component tests then passed, including wrapped-field and native textarea growth/history checks; the correction and regression are committed together in `5702e038`.

All seven corrected documentation slugs have subsequently been reread in all four `components/verified` variants; the combobox's unintended light surface in dark mode is gone and helper/result text scales with the controls. The four `form-sheets/verified-examples-*.jpg` contact sheets record that review. The corrected drawer composition was also inspected in `verified/dark-2x/drawer-dark-text-200.png`: header, caption, field and action stay grouped at the top.

## Final source capture record

The showcase after bundle is recorded in `docs/.verification/quality-0.4/after/manifest.json` at form/library source `5201a0aab4961c645644db05f730aa4811aa46da`, binary SHA-256 `ca0f40db0b1b248ca892bf4d108bcdc1e10c68260b8b3ad31088067b42c81e8f`. It uses native winit/Skia on Mac16,7, macOS 27 arm64. The manifest records all 108 showcase frames.

All 60 frames belonging to Profile, Billing, Choices, Schedule and Files were inspected: each page in light/dark, 100%/200% text, at top/middle/end scroll positions. The five `form-sheets/after-*.jpg` contact sheets preserve the inspection index. Full-resolution checks include `profile-dark-2x-end.png` (all six OTP placeholder slots), `choices-light-2x-top.png` (centred switch/radio visuals and field grouping) and `billing-light-2x-end.png` (native numeric editor and square stepper actions). The final gallery review found no new regression introduced by the corrections.

The final complete component matrix is `docs/.verification/quality-0.4/components/qualified/manifest.json`: 496 fresh native frames for all 124 components, at revision `97ccab8a4dffe1ef6cc4937ad12286c941425807`, binary SHA-256 `1a5d66ed5036f31a4634410d88711e4abb6905db60f77870be193f7421280237`. Its recorded backend/device are winit/Skia, Mac16,7, macOS 27 arm64. There is no diff in `crates/kit-slint/ui` or `showcase/ui/forms-gallery.slint` between this revision and `5201a0aa`; the later RootShowcase correction concerns the French/RTL example button.

The final 200% light and dark views were reread for `number-stepper`, `otp-field`, `color-picker`, `color-field`, `radio-group`, `switch` and `date-range-picker` (14 frames). The four `form-sheets/qualified-forms-*-2x-*.jpg` contact sheets index those views. Stepper actions remain square and aligned with their editor; OTP shows all six available slots and six entered/masked characters; channel labels, native values and slider tracks remain aligned; radio/switch indicators centre beside their labels; range-calendar rows, headings and actions remain legible. No new regression was observed in these final views.

The earlier component discovery and `components/verified` runs remain separate evidence and are not substituted for the `qualified` source record. The official gate is recorded in the shared campaign review. Screenshot inspection and headless tests do not qualify VoiceOver, native IME, external file drops, other backends/devices or interactive states absent from the fixtures.
