---
layout: base.njk
locale: en
title: "Typed controllers"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Typed values and owned collections

Start with [Project setup](/guides/project-setup/) and one `slint_kit::install(&app)` per window. Interactive components own their local view state and call pure shared reducers. Business callbacks receive a checked projection that converts to a Rust domain type. Applications retain ownership of their documents, lists, tables, files and asynchronous work.

## FieldSpec and native drafts

`FieldSpec<T, E>` contains a parser, accepted-value formatter, display formatter, domain validator and error formatter. `FieldSpec::codec` is suitable when parsing expresses the full rule; `FieldSpec::new` adds a domain error type and validator. Built-ins cover text, email, URL, integers, decimal, money, percent, quantity, temporal values, telephone, OTP and masks.

Register named codecs through `install_field_specs(&app, specs)` before showing the window; duplicate `FieldSpecKey` values are rejected atomically. A codec is an immutable shared definition. It never stores the draft or accepted value of a component. `KitFieldService.reduce` receives explicit `FieldState`, action, rules and codec key; fields keep the returned state themselves.

Slint stores native drafts and formatted accepted-value projections. The pure service uses the codec to validate and format a candidate. At the Rust business boundary `spec.accept(FieldCommit)` decodes that projection and returns `T`, applying the same intrinsic constraints and domain rule. Keep that returned value. `with_display` and `with_grouping` affect the resting presentation without changing the accepted value or rewriting native edits. An error or incomplete draft leaves the last accepted value intact.

Money and decimals stay exact: use `Money`, `DecimalValue` and their explicit rules. Slider specs use decimal strings and a shared exact lattice; normalized floats are only pointer/display projections. `SliderSpec` defines minimum, maximum and step. `DecimalRangeView` converts to `OrderedRange<DecimalValue>`, whose endpoints cannot cross.

`FieldBinding` and explicit `FieldController` remain available for custom integrations. The ordinary field path needs no component-specific macro. `init_form_validation!` is deprecated in favor of `install`.

The legacy `bind_select!` macro is retained for migration and deprecated in documentation. Replace its window callback convention with `Choices<K>`, the component’s `options`/`selected`/`chosen` contract and the shared installer. Explicit `SelectBinding` projections remain available for custom controlled integrations.

## Stable choices

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
enum Destination { Overview, Activity }
let choices = slint_kit::Choices::new([
    (Destination::Overview, "Overview"),
    (Destination::Activity, "Activity"),
]).unwrap();
app.set_options(choices.model());
app.on_chosen(move |row| {
    if let Some(destination) = choices.accept(row) {
        println!("Open {destination:?}");
    }
});
```

Tokens are projections of stable keys, never array positions. `Choices::replace` preserves identity across reordering and invalidates stale source generations. Disabled choices cannot be accepted. Components keep independent selected, active, open and search state. `ownership: Ownership.controlled` emits an intent and waits for host acknowledgement through the selected property. MultiSelect and ToggleGroup own sets and emit `selection-changed(row, added)`; `Choices::accept` still returns the domain key.

`KitSelectKeyboard`, `KitSelectTrigger` and `KitSelectSurface` are composition primitives. Their public props allow you to share one selection instance between parts; they are not separate application controllers. The complete Select/Combobox controls provide this composition already. Calendar/date/time components likewise keep local state; receive `CivilDate`, `DateRange`, `LocalDateTime` or `ResolvedDateTime` by checked conversion of their `chosen` payload.

`KitSelectKeyboard.move-active(NavigationStep)` and `type-ahead(string)` route custom navigation surfaces through the same session as native keyboard events. Use `choose-option(token, generation)` for activation; keep host input gates on the calling surface.

## KeyedModel for host-owned data

`KeyedModel<K, R>` associates a unique typed key with each public row. `R: KeyedRow` defines the kit projection; built-in rows cover table, content, text, chip, file, notification and toast views. `model()` provides a virtualizable model. `replace` rejects duplicate keys before changing anything and retains tokens for surviving keys. Removed tokens are never recycled.

For collection views with explicit ports, `bind` takes the window and setters for rows, generation and active index. The binding retains a weak window. `navigation()` implements shared roving navigation; `activation(handler)` updates `SelectableRow` state and calls the business handler with `K`. Validate other intents through `accept(token, generation)` or `remove_fenced`. Token-only chip callbacks use `remove(token)` with a non-reused token. Changing source content invalidates generation-fenced gestures; do not substitute an index for a key.

The application still chooses data, sorting, filtering, loading and commands. A large model belongs in a virtualized collection control, not repeated visual children. `action`, `event`, `event2` and `event3` connect business callbacks with a weak window and pass a temporary live handle to the handler.

## Host effects and notifications

`FilePickerBinding` and `DirectoryPickerBinding` accept explicit projection ports, install weak-window handlers and deliver typed `FileHandle` or `DirectoryHandle` values. Request generations discard stale asynchronous results. The optional `native-winit` feature adds a window-level drop bridge with an explicit capability. See [Native integration](/guides/native/) for platform limits.

`ToastBinding<Action>` stores typed actions in `ToastQueue`, projects `ToastEntry` rows and manages a weak timer. Capacity and `ToastLifetime` are explicit. Its three handlers route action, dismissal and independent pointer/focus pauses; the application only performs the resulting business action.

`KeyedModel::sorting_by` turns a domain-key comparator into an alternating sortable-header handler. Sorting preserves selection and active identity and advances the generation, so a pointer gesture started before the reorder cannot target the new projection.

Use `Choices::selections(&keys)` for initial multiple selections; missing or repeated keys reject the complete projection. `KeyedModel::bind_active` publishes the active row through a weak window to an editor. Use `update_active` to reject commits for a line that is no longer active.

Match a decimal codec’s `DecimalRules.locale` to the component’s `number-format`, and a time codec’s `ClockFormat` to the component’s `clock-format`. The HTML engine validates the native grammar; the codec then receives that grammar, including decimal commas and 12-hour clocks. Accepted formatting round-trips through the same parser, so cancelling restores an editable value in the declared format.
