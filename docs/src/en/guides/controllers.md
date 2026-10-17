---
layout: base.njk
locale: en
title: "Typed controllers"
translationKey: "guides/controllers/"
permalink: /en/guides/controllers/index.html
---

# Typed controllers

Keep the accepted value in Rust and project its presentation into Slint. The `kit` facade reexports core values and weak-window adapters. A view token is only valid within its model generation; it is never the application's stable identity.

[Run and read the complete typed application](/en/guides/typed-example/): native field, validation, stable selection and every signal connected.

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

Do not replace the editor or rewrite its text on every keystroke. Native Slint text editing owns selection, caret, undo and IME. Formatting belongs at the acceptance boundary. See the complete [field bindings in the showcase](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_fields.rs) and [editor signals](/en/components/text-field/).

## Validation and form submission

`begin_validation()` returns a `ValidationGeneration`; retain that ticket with the asynchronous request. `finish_validation(ticket, errors)` ignores a result superseded by a newer draft. Do not identify requests by a mutable row index. `FormCoordinator` similarly uses submission tickets and typed field keys. A submit button only emits the form action; the host owns validation and persistence.

## Stable keyed selection

`SelectBinding<K>` retains stable application keys and publishes `SelectProjection` rows, active and selected tokens, availability, open state and generation. Its `activate(token, generation)` rejects stale input. `reconcile` changes the source options; `query` filters the view without removing the accepted key. Duplicate keys are an error.

Use `on_intent` for `SelectionIntent::Commit(K)` and `SelectionIntent::Clear`. Under controlled ownership, acknowledge the accepted key with `set_selected`. The `bind_select!` macro wires the conventional window contract; its required `rows`, `select-state` and `select-*` callbacks are documented in the [public macro source](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/src/macros.rs).

`SelectionAvailability` distinguishes `Absent`, `Available` and `Unavailable`. A selected item hidden by filtering or removed from the current model is not automatically an instruction to clear the domain value. See [KitSelect](/en/components/select/) and [navigation examples](https://github.com/damione1/slint-kit/blob/main/showcase/src/base_navigation.rs).

## Exact numbers and civil time

Use `DecimalValue`, `Money`, `CurrencyMetadata` and explicit `NumberLocale`, `DecimalScale` and `RoundingPolicy`. Slider positions and color byte projections are renderer values; they do not replace exact accepted amounts. Parse civil dates and local times with the core types, then resolve a zone explicitly. An ambiguous local time requires an explicit choice; never silently pick an instant.

## Overlays and feedback

Keep `FocusRegistry` and the exact opener in the host controller. Dialog and drawer primitives present focus requests; register real focus targets and restore the opener on close. A tooltip needs one window-level [KitTooltipLayer](/en/components/tooltip-layer/), mounted after the content. `ToastQueue` owns toast lifetimes; pointer and focus pauses use distinct `ToastPause` sources so overlapping interactions do not resume a toast prematurely.
