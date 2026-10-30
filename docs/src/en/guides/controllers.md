---
layout: base.njk
locale: en
title: "Typed controllers"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Typed controllers


Keep the accepted value in Rust and project its presentation into Slint. The `slint_kit` facade reexports core values and weak-window adapters. A view token is only valid within its model generation; it is never the application's stable identity.

[Run and read the complete typed application](/guides/typed-example/): native field, validation, stable selection and every signal connected.

## Field drafts and accepted values

`FieldBinding<T, E>` connects a `FieldController<T, E>` to a publisher. Choose a domain type for `T` and an error enum for `E`. Its constructor accepts the initial accepted value, ownership, commit policy, absence policy, parser, formatter, error formatter and projection publisher.

| Choice | Values and behavior |
| --- | --- |
| `Ownership` | `Local` accepts successful local commits. `Controlled` emits an intent and waits for the host to call `acknowledge`. |
| `CommitPolicy` | `OnBlur`, `OnSubmit`, `OnChangeWhenValid`. A host calls `commit` with the corresponding `CommitTrigger`. |
| `AbsencePolicy` | `Required` or `Optional`; an empty draft is not the same as an invalid value. |
| `DraftValue<T>` | `Empty`, `Incomplete`, `Parsed(T)` or `Invalid(ParseIssue)`. Keep incomplete input editable. |
| `FieldAccess` | `Editable`, `ReadOnly`, `Disabled`. Read-only text retains native selection. |

Feed editor changes into `edited(&str)`, wire blur and submit to `commit`, and use `cancel` or `reset` for explicit actions. Publish `FieldProjection.draft`, access, validation and error back into the view. The publisher should capture `window.as_weak()` and upgrade it only while updating the window. Keep the binding alive with the window's controller.

Do not replace the editor or rewrite its text on every keystroke. Native Slint text editing owns selection, caret, undo and IME. Formatting belongs at the acceptance boundary. See the complete [field bindings in the showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_fields.rs) and [editor signals](/components/text-field/).

## Validation and form submission

`begin_validation()` returns a `ValidationGeneration`; retain that ticket with the asynchronous request. `finish_validation(ticket, errors)` ignores a result superseded by a newer draft. Do not identify requests by a mutable row index. `FormCoordinator` similarly uses submission tickets and typed field keys. A submit button only emits the form action; the host owns validation and persistence.

## Stable keyed selection

`SelectBinding<K>` retains stable application keys and publishes `SelectProjection` rows, active and selected tokens, availability, open state and generation. Its `activate(token, generation)` rejects stale input. `reconcile` changes the source options; `query` filters the view without removing the accepted key. Duplicate keys are an error.

Use `on_intent` for `SelectionIntent::Commit(K)` and `SelectionIntent::Clear`. Under controlled ownership, acknowledge the accepted key with `set_selected`. The `bind_select!` macro wires the conventional window contract; its required `rows`, `select-state` and `select-*` callbacks are documented in the [public macro source](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/src/macros.rs).

`SelectionAvailability` distinguishes `Absent`, `Available` and `Unavailable`. A selected item hidden by filtering or removed from the current model is not automatically an instruction to clear the domain value. See [KitSelect](/components/select/) and [navigation examples](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs).

## Exact numbers and civil time

Use `DecimalValue`, `Money`, `CurrencyMetadata` and explicit `NumberLocale`, `DecimalScale` and `RoundingPolicy`. Slider positions and color byte projections are renderer values; they do not replace exact accepted amounts. Parse civil dates and local times with the core types, then resolve a zone explicitly. An ambiguous local time requires an explicit choice; never silently pick an instant.

## Overlays and feedback

Keep `FocusRegistry` and the exact opener in the host controller. Dialog and drawer primitives present focus requests; register real focus targets and restore the opener on close. A tooltip needs one window-level [KitTooltipLayer](/components/tooltip-layer/), mounted after the content. `ToastQueue` owns toast lifetimes; pointer and focus pauses use distinct `ToastPause` sources so overlapping interactions do not resume a toast prematurely.


## Tables and row identity

The [complete sortable table application](/guides/table-example/) demonstrates selection surviving sorting, keyboard navigation and rejection of events from an older generation.

## Calendar dates, formats and availability

`CivilDate` uses the proleptic Gregorian calendar, years 1–9999. `KitDateField` and `KitDatePicker` keep their HTML date contract: drafts, `min`, `max` and accepted strings use `YYYY-MM-DD`. A display label and a date input format are separate choices. There is no automatic inference between day-first and month-first input, and no implicit time zone.

For application-owned text fields or date labels, validate a `DatePattern` once and keep it with the controller:

```rust
use slint_kit::{CivilDate, DateNames, DatePattern, DraftValue};
let pattern = DatePattern::new("%d/%m/%Y").unwrap();
let DraftValue::Parsed(date) = pattern.parse("29/02/2024") else { return };
assert_eq!(date, CivilDate::new(2024, 2, 29).unwrap());
assert_eq!(date.iso_string(), "2024-02-29");
let spoken = DatePattern::new("%A %-d %B %Y").unwrap();
assert_eq!(spoken.format_with_names(date, &DateNames::french()), "jeudi 29 février 2024");
```

Supported directives are `%Y`, `%m`, `%d`, `%e`, `%b`, `%B`, `%a`, `%A`, `%j`, `%G`, `%V`, `%u`, `%F` and `%%`; numeric directives accept `-`, `_` or `0` padding. Unpadded or space-padded numeric fields require nonnumeric separators: `%Y%m%d` is valid, while `%Y%-m%-d` is rejected as ambiguous. This covers numeric orders and separators, compact dates, English month names, ordinal dates and ISO week dates. `%y` is rejected because a two-digit year requires a century policy. Plain `parse`/`format` use English names. `parse_with_names`/`format_with_names` use validated host-owned `DateNames`; a French preset is included. Names must be nonempty and have no surrounding whitespace. Matching uses Unicode lowercase, retains accents and requires configured abbreviation punctuation. Other calendar systems and locale-specific digit shapes require an explicit host adapter.

Keep the native `KitTextField` editor for custom-format text. Parse its draft with the chosen pattern, then validate the resulting `CivilDate` against the same rules as calendar clicks. Do not attach `HtmlInputKind.date` to localized text; the shared HTML date engine expects ISO. If the field participates in a form, apply its required/custom validity rules and guard the Rust submit path too.

`DateConstraints` combines optional inclusive `NumericBounds<CivilDate>`, unavailable `DateRange` values and unavailable weekdays. Pass it to `CalendarController::with_constraints`. `choose` rejects blocked dates while keyboard focus may inspect them. All seven `WeekStart` values are available; changing `set_week_start` also requires rotating the host's weekday labels. `can_navigate` drives `previous-enabled` and `next-enabled`; month/year paging must change the visible month, while day/week actions must move the focused date. The bounds-only `CalendarController::new` rejects an out-of-bounds initial selection to `None` and clamps focus; use `with_constraints` when construction must return an explicit error. `show_month` and `focus_date` move the visible keyboard target without committing a value. `grid_for_month` lets an application compose several `KitCalendar` views over the same selection and rules.

If the host supplies a business error through `error`, clear that stale error on `edited` before the next submission. Keep the accepted date unchanged until the newly edited draft passes the same constraints.

`DateRangeRules` adds optional minimum/maximum nights. A same-day range is zero nights; the inclusive range may not cross an unavailable day. `DateRangeController::with_rules` preserves the previous accepted range until Apply and restores it on Cancel. Failed end-date selection preserves the start. `DateSetController` supports multiple dates keyed by `CivilDate`: project `selected().contains(&date)` into `CalendarDay.selected`, then call `toggle` from the calendar action. There is no visual-index identity.

Calendar keyboard actions use arrows, Home/End and Page Up/Down; Shift changes the paging unit to years. Read-only calendars remain browsable and cannot accept a date. Supply localized month titles, weekday labels, accessible date labels and range action labels. The [calendar](/components/calendar/), [date picker](/components/date-picker/) and [range picker](/components/date-range-picker/) executable examples combine the kit's card, badge and helper primitives with real availability rules. Their reference date is fixed for reproducible examples. These APIs do not by themselves qualify a screen reader or IME on a particular backend.
