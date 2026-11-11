---
layout: base.njk
locale: en
title: "Typed controllers"
translationKey: "guides/controllers/"
permalink: "{{ 'guides/controllers/' | localeUrl(locale) }}index.html"
---

# Typed values and owned collections

`FieldSpec<T, E>` returns validated domain values; `Choices<K>` retains choice keys; `KeyedModel<K, Row>` connects rows to stable identities. These APIs share the validation and selection behavior of Slint Kit components. Your application supplies its types, rules and data.

Start with [Project setup](/guides/project-setup/) and one `slint_kit::install(&app)` per window. Interactive components own their local view state and call pure shared reducers. Business callbacks receive a checked projection that converts to a Rust domain type. Applications retain ownership of their documents, lists, tables, files and asynchronous work.

Controls mounted later by `if`, model delegates or a popup initialize their own selection fence from the already installed window services. Keep one installation per window; no second install or artificial options replacement is needed.

## FieldSpec and native drafts

`FieldSpec<T, E>` contains a parser, accepted-value formatter, display formatter, domain validator and error formatter. `FieldSpec::codec` is suitable when parsing expresses the full rule; `FieldSpec::new` adds a domain error type and validator. Built-ins cover text, email, URL, integers, decimal, money, percent, quantity, temporal values, telephone, OTP and masks (`FieldSpec::masked(MaskPattern::compile("AA-####")?)`, sharing the same `#`/`A`/`*`/`\` grammar as `KitMaskedField.mask`).

Register named codecs through `install_field_specs(&app, specs)` before showing the window; duplicate `FieldSpecKey` values are rejected atomically. A codec is an immutable shared definition. It never stores the draft or accepted value of a component. `KitFieldService.reduce` receives explicit `FieldState`, action, rules and codec key; fields keep the returned state themselves.

Slint stores native drafts and formatted accepted-value projections. The pure service uses the codec to validate and format a candidate. At the Rust business boundary `spec.accept(FieldCommit)` decodes that projection and returns `T`, applying the same intrinsic constraints and domain rule. Keep that returned value. `with_display` and `with_grouping` affect the resting presentation without changing the accepted value or rewriting native edits. An error or incomplete draft leaves the last accepted value intact.

Money and decimals stay exact: use `Money`, `DecimalValue` and their explicit rules. Slider specs use decimal strings and a shared exact lattice; normalized floats are only pointer/display projections. `SliderSpec` defines minimum, maximum and step. `DecimalRangeView` converts to `OrderedRange<DecimalValue>`, whose endpoints cannot cross.

Accessible slider metadata uses the same domain: `spec.minimum`, `maximum` and `step`, the current decimal value, and the other endpoint as each range thumb’s inward bound. Accessible set-value follows pointer rounding and bounds through `DecimalSlider::at_value`; it emits `chosen` and `committed` once for a changed valid candidate, and preserves controlled ownership. Supply matching `spec`/`value` even when overriding normalized positions. Color channel sliders use 0..255 with step 1.

`value-label` and range `lower-label`/`upper-label` supply formatted accessible descriptions; the numeric accessible value remains the decimal text. Slint 1.18.1 has no independent value-text property and restricts accessible bounds/step to Float32, which can collapse large or closely spaced bounds. Do not use those projections to accept domain values. Metadata and synthetic action coverage is headless; qualify the named backend and screen reader separately.

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

Tokens are projections of stable keys, never array positions. `Choices::replace` preserves identity across reordering and invalidates stale source generations. Disabled choices cannot be accepted. Components keep independent selected, active, open and search state. `ownership: Ownership.controlled` emits an intent and waits for host acknowledgement through the selected property. MultiSelect and ToggleGroup own sets and emit `selection-changed(row, added)`; `Choices::accept` still returns the domain key. For remote combobox search, `KitCombobox` defaults to `retention: SelectionRetention.retain`, preserving the accepted token, availability and last known label across page replacements; `Choices::retain` and `Choices::label(&key)` expose the same retention and cached label lookup in Rust, and `SelectBinding::set_removal_policy(RemovalPolicy::Retain)` configures explicit controlled bindings.

`KitSelectKeyboard`, `KitSelectTrigger` and `KitSelectSurface` are composition primitives. Their public props allow you to share one selection instance between parts; they are not separate application controllers. The complete Select/Combobox controls provide this composition already. Calendar/date/time components likewise keep local state; receive `CivilDate`, `DateRange`, `LocalDateTime` or `ResolvedDateTime` by checked conversion of their `chosen` payload.

`KitSelectKeyboard.move-active(NavigationStep)` and `type-ahead(string)` route custom navigation surfaces through the same session as native keyboard events. Use `choose-option(token, generation)` for activation; keep host input gates on the calling surface.

Choice and navigation composites expose configurable keyboard policies while keeping existing defaults: `wrap` defaults to `true` on `KitRadioGroup`, `KitSegmentedControl`, `KitToggleGroup`, `KitMenu`, `KitMenuBar`, `KitTabs` and `KitCarousel`, and `false` on `KitSelect`, `KitSelectTrigger`, `KitSelectSurface`, `KitSelectKeyboard`, `KitMultiSelect` and `KitCombobox`. Closed `KitSelect` triggers open and move to the first, last or matched option on `Home`, `End` or printable typeahead. `typeahead-timeout-ms` defaults to `700`, accepts spaces inside an active query, and resets on arrow/Home/End/PageUp/PageDown navigation. `PageUp`/`PageDown` step by visible `page-rows` (`viewport-height / row-height`, `0` until measured; unmeasured PageUp/PageDown are rejected). `KitTabs.activation` selects `TabActivation.manual` (default) or `.automatic`; `KitToggleGroup` adds `direction` and `flow`; `KitMenuBar`, `KitTabs` and `KitCarousel` inherit `flow: theme.direction`; and `KitCarousel` handles `LeftArrow`/`RightArrow` (mirrored in RTL) plus `Home`/`End`.

Nonvirtualized MenuBar/Tabs and choice/rating controls keep native focus on persistent scopes and delegate once to their materialized semantic option. Physical RTL/grid ordinals differ from logical item index/count. Radio/Toggle layout scopes survive model replacement, preserve one Tab stop and allow a host editor redirect to win. `KitSelectSurface.accessible-name` names its list; custom `KitOption` loops must bind inherited `accessible-item-index` and `accessible-item-count` themselves. Virtualized Menu/Select and editable Combo/search retain their native scope/editor: Slint 1.18.1 has no safe public virtualized-child ordinal or editor-to-option identifier relation. An ephemeral row can be destroyed without FocusOut, so cached focus must never trigger background refocus. Tabs-to-panel identifier relations and VoiceOver/NVDA/physical IME qualification remain unsupported or unqualified; model-position getters do not establish an AT result.

## KeyedModel for host-owned data

`KeyedModel<K, R>` associates a unique typed key with each public row. `R: KeyedRow` defines the kit projection; built-in rows cover table, content, text, chip, file, notification and toast views. `model()` provides a virtualizable model. `replace` rejects duplicate keys before changing anything and retains tokens for surviving keys. Removed tokens are never recycled.

Each `KeyedModel` owns a wide generation sequence; its positive Slint tag cycles after `i32::MAX`, cancelling live component gestures on every change. Do not persist a bare generation as a durable version: an arbitrarily old tag can repeat after a complete cycle. Row tokens remain process-unique and never recycled. If the finite token space is exhausted, new/replace/insert return `ModelError::IdentityExhausted` transactionally instead of panicking.

For collection views with explicit ports, `bind` takes the window and setters for rows, generation and active index. The binding retains a weak window. `navigation()` implements shared roving navigation; `activation(handler)` updates `SelectableRow` state and calls the business handler with `K`. Validate other intents through `accept(token, generation)` or `remove_fenced`. Token-only chip callbacks use `remove(token)` with a non-reused token. Changing source content invalidates generation-fenced gestures; do not substitute an index for a key.

The application still chooses data, sorting, filtering, loading and commands. A large model belongs in a virtualized collection control, not repeated visual children. `action`, `event`, `event2` and `event3` connect business callbacks with a weak window and pass a temporary live handle to the handler.

## Host effects and notifications

`FilePickerBinding` and `DirectoryPickerBinding` accept explicit projection ports, install weak-window handlers and deliver typed `FileHandle` or `DirectoryHandle` values. Request generations discard stale asynchronous results. The optional `native-winit` feature adds a window-level drop bridge with an explicit capability. See [Native integration](/guides/native/) for platform limits.

`ToastBinding<Action>` stores typed actions in `ToastQueue`, projects `ToastEntry` rows and manages a weak timer. Capacity and `ToastLifetime` are explicit. Its three handlers route action, dismissal and independent pointer/focus pauses; the application only performs the resulting business action. Surviving toast delegates retain keyboard focus and independent hover/focus pauses when another entry is appended, dismissed, expires or is evicted. Keep the `model()` handle installed; the binding applies incremental keyed insertions/removals. `KeyedModel::insert` and `remove_by_key` preserve surviving delegates while advancing the generation; `replace` and sorting still reset their projection.

`ToastBinding` schedules only the next unpaused timed expiry with a single-shot timer. Empty, persistent-only and fully paused queues do not wake the event loop. Push, dismissal, pause changes and expiry recompute the next delay; releasing one pause source does not override the other. Custom runtime adapters can call `ToastQueue::next_timeout(now)` using the same injected monotonic clock as `advance(now)`: it returns the remaining delay, `None` when nothing is running, or zero for an overdue record. Reading it does not mutate the queue; call `advance` to remove expired records. The timer closure retains only weak binding state, and dropping the last binding stops it even if its projected model survives.

`KeyedModel::sorting_by` turns a domain-key comparator into an alternating sortable-header handler. Sorting preserves selection and active identity and advances the generation, so a pointer gesture started before the reorder cannot target the new projection. The [sortable table application](/guides/table-example/) wires `bind`, `navigation()`, `activation` and `sorting_by` to a `KitDataTable` in one short file.

`KeyedModel::cell_commits(handler)` is the `KitDataTable.cell-committed` handler for in-place cell edits: it admits a commit only under the current generation and for an enabled row, then passes the row key, the column token and the `FieldCommit`, which the application runs through that column's `FieldSpec::accept` before republishing the row.

Use `Choices::selections(&keys)` for initial multiple selections; missing or repeated keys reject the complete projection. `KeyedModel::bind_active` publishes the active row through a weak window to an editor. Use `update_active` to reject commits for a line that is no longer active.

Match a decimal codec’s `DecimalRules.locale` to the component’s `number-format`, and a time codec’s `ClockFormat` to the component’s `clock-format`. The HTML engine validates the native grammar; the codec then receives that grammar, including decimal commas and 12-hour clocks. Accepted formatting round-trips through the same parser, so cancelling restores an editable value in the declared format.

`bind_active` publishes only when the active key or its record revision changes. Updating another row and reordering unchanged rows preserve an editor draft. Explicit `replace` supplies fresh records and republishes the active record. Use `get`, `len`, `is_empty`, `update_by_key` and `remove_by_key` for domain access, including disabled records. In-place updates retain the interaction generation and use row notifications; membership changes and replacement invalidate old gestures.

Column resize ignores a click without movement. Escape or a secondary press cancels a drag preview. Double-clicking a divider emits `autosize-requested(column-token)`; calculate the domain-appropriate width and update the column policy.

Drag a column header onto another header to request moving it to that column’s current visible position. `reorder-requested(source-token, target-token)` uses stable tokens and leaves the model unchanged until the host accepts; remove the source and insert it at the target’s previous position, preserving each column’s `source`, width and token. Keep canonical row data unchanged and republish `active-column` for the accepted column key. Drops from another model, stale column revisions, hidden/duplicate tokens and external payloads are rejected. Releasing outside a valid header cancels the move. The divider remains a separate resize target (24 logical pixels at 100%, scaled with text). Ctrl+Shift+Left/Right requests an adjacent move on a focused sortable header or in the roving header region. `reorder-active-column(ColumnStep.previous/next)` exposes the same bounded request to host actions, including for static headers. A disabled table emits no reorder or autosize request. Headless contracts do not qualify OS-native drag-and-drop or screen readers.

`ColumnStep` is the shared presentation enum (`previous`, `next`). `TableColumnDrop` (`valid`, `token`) is the Slint projection of the internal drag-service result, not a component or a host reorder command; ordinary consumers handle `reorder-requested`.

A table cell uses the native field editor. Click chooses the row and cell cursor, double-click or F2 opens an editable field, Enter commits, Escape cancels and Tab validates before moving. `edit-active()`, `commit-edit()` and `cancel-edit()` expose those operations to host actions. Install the kit once per window, including for plain text cells. `active-index` and `active-column` are view cursors; commit identity is always `(row-token, column-token, generation, FieldCommit)`. Read `edit-error` for a rejected codec or HTML constraint. The accessible active cell exposes its value and the localized `editable-label` / `read-only-label` hint; headless evidence does not qualify a screen reader.

Localize document editing with `editor-label`, `editor-placeholder`, `save-label` and `copy-label`; their defaults remain English.


### Keyboard hooks from Rust

Public kit keyboard hooks return `KitKeyResult`, a shared presentation enum exported by `@kit` and as `slint_kit::ui::KitKeyResult`. Return `KitKeyResult::Accept` from Rust to consume a shortcut, or `Reject` to preserve the control’s built-in behavior. Slint bindings use `KitKeyResult.accept` and `.reject`; the kit converts them at the native `FocusScope` or `TextInput` boundary. Do not use the unrelated winit backend `EventResult` or Slint private APIs.

This result applies to `KitTextField.key-pressed`, `KitDataTable.key-event`, inherited `KitPressable.key-event`, selector `tab-navigation`, and public selector/navigation `handle-key` functions. A text-field hook runs only without active IME preedit. Preserve rejection for normal text and navigation so native editing keeps selection, undo and IME.

Use `CellContent::constrained_field(text, codec, access, constraints)` when a cell has intrinsic form rules. `KitInputConstraints` is the same projection used by the shared HTML engine; use `kind: HtmlInputKind::Number` for native numeric filtering and set absent `min_length` / `max_length` to -1. The default value supplies the step base and `custom_error` remains a persistent constraint; cell `FieldAccess` controls access. Required, pattern, min/max and step remain validated again through `FieldSpec::accept(commit)` in Rust. For a domain lock or another synchronous business rule, implement `validate-edit(row_token, column_token, generation, commit) -> string`: an empty string accepts, a localized message keeps the draft open. Enter, Tab and `commit-edit()` recheck the same draft after a temporary rejection; the canonical row remains unchanged until acceptance. Avoid silently reverting a rejected committed value.

Use `header-focus: TableHeaderFocus.roving` for one table Tab stop: Up from the first row (or the empty body) enters the header cursor, Left/Right changes columns, Down returns to the body, Enter/Space sorts, and Alt+Left/Right requests bounded resizing. The default `individual` policy keeps each sortable header and resize handle tabbable before the body. Localize `header-label`; `sort-active-column()` and `resize-active-column(delta)` share the guarded token callbacks. Resize uses the currently measured header width: show/measure the view first and let the next Slint update turn deliver its callback.

Moving an edited row entirely outside the table viewport cancels its draft without committing. This includes wheel scrolling and shrinking the view. Returning the row keeps its accepted value and does not restore an editor; start a new session explicitly.

Observe `edit-started(row, column, generation)` and `edit-ended(row, column, generation, outcome, message)` to track a cell session by its armed identity. `TableEditOutcome` distinguishes committed, unchanged, cancelled, rejected-on-blur, superseded and viewport-exit. Read `edit-draft` for recovery and `edit-bounds` (`valid`, window-logical x/y/width/height) after layout. `edit-blur-policy: TableEditBlurPolicy.keep-open` keeps an invalid native editor focused; its explicit cancel remains available. The default commit-or-drop preserves native blur behavior and reports rejection. `unchanged-edit-policy: TableUnchangedEditPolicy.skip` suppresses an unchanged cell proposal while still ending the session. End events snapshot their identity, so a handler can open another cell; canonical updates still use the fenced, typed cell-committed path.

Read each collection’s `page-rows` when handling PageUp/PageDown: it reflects the body viewport after the table header or document editor footer, density and text scaling. Expose it on your window and connect `model.navigation(&window, Window::get_page_rows)`; this weak handler reads the getter again for every key. A direct controller passes that value to `navigate_page`. `navigate(step, page_rows)` also requires an explicit capacity; `navigate_page` remains an explicit compatibility alias.

Collection viewports expose `first-visible-index`, `visible-row-count` and signed `scroll-x`/`scroll-y` offsets independently of selection. Use `scroll-to-index` for a visual position, `reveal-active` for the current active identity, `scroll-to-top` for both axes, and `set-scroll(x,y)` to restore saved offsets within current bounds. A saved position belongs to the model identity/order it was captured with. `near-end-rows`, `end-near` and the edge-triggered `near-end()` support incremental loading; if an appended page still leaves `end-near` true, continue filling explicitly instead of waiting for another edge. Programmatic scrolling also ends a table editing session if its row leaves the viewport.

When a table’s column count changes, `active-column` clamps to the remaining visible positions in both body and header navigation. Removing every column returns the body to the whole-row cursor (-1); later columns do not revive an obsolete cell. A removed edited cell still ends as `superseded`, without committing the draft.
## Interactive steps and event timelines

Use `KeyedModel<K, StepEntry>` and resolve `step-activated(token, generation)` with `accept`. Set `enabled` explicitly for reachable steps; the kit emits requests and your workflow decides whether and how to change states. The generation fence rejects held gestures after replacement. `interactive` defaults to false; orientation and descriptions work in either mode. Timeline rows are ordered display projections: supply a semantic `tone` and an optional decorative `glyph`, while retaining the event meaning in title and description.

For attached command strips, supply `KitButtonGroup.actions` from `KeyedModel<K, ButtonGroupEntry>` and resolve `activated(token, generation)` with `accept`. Each button keeps an independent Tab stop and action; this is not a selection group. The host owns busy/disabled state and commands. Model replacement cancels held gestures. Use @children when you need custom composition, passing each child its own theme and disabled state.

Calendar constraints may change while an existing record is open. A now-unavailable selected date is retained in your accepted value until a new date is chosen, while the grid projects without an allowed selection and still offers valid replacements. `DateRangeController::reduce` recovers an old invalid completed range or start draft by treating the next valid date as a fresh start. Invalid new choices remain rejected, and Apply validates the whole new range. Constructors that explicitly validate an accepted range remain strict. Keep min/max and availability configuration valid; malformed configuration is a distinct error.

Calendar date configuration exposes `CalendarConfigurationState.unchecked` before `install`, then `.valid` or `.invalid`. Malformed/reversed min/max, invalid civil dates or malformed unavailable ranges produce a disabled reference grid plus a visible `configuration-message`; navigation/acceptance cannot bypass those rules. Translate the message once with `KitStrings.calendar-configuration-error`, or override an instance. Repairing the configuration restores the existing component. Range length rules are validated separately by the range reducer and Apply. Complete Rust `CalendarProjection` literals must now include `configuration_state`; `calendar_projection()` supplies Valid for a validated controller.


Calendar `weekdays` models always contain seven names in Monday-to-Sunday order. Set `week-start: WeekStart.sunday` (or any other weekday) to rotate both headers and grid columns. `KitDatePicker` and `KitDateRangePicker` forward the same convention to their calendar. Do not rotate localized names yourself when switching the first day.

After `slint_kit::install(&window)`, table columns may be projected independently of their row data. `TableColumn.source` defaults to `ColumnSourceMode.position`: its index in the **input** columns model selects the cell, before hidden columns are filtered. Use `ColumnDataIndex::try_from(canonical_index)?.into()` to keep the source fixed when reordering or omitting input columns. `visibility: ColumnVisibility.hidden` hides a column without rewriting `TableRow.cells` or `.content`; `visible` is the default. `active-column` is a visible position, while sort, resize and edit callbacks retain the stable column token. Missing or negative source indices paint empty cells and cannot open an editor; extra row values are ignored. A removed/hidden edited column or changed source supersedes its session without committing the draft. Keep column tokens unique and preserve their source when reordering.

Synchronous `validate-edit` callbacks and `edit-ended` observers cannot redirect an older proposal into a reopened editor or a different canonical source. Each private session owns a fresh native editor. The public end event still precedes `cell-committed`; if its observer hides, removes or retargets the original row/column/source, that proposal is suppressed. Opening another editor, reordering an explicitly sourced column, or hiding a preceding column leaves the original token/source eligible. The lookup uses the stable token in the input column model, with positional sources resolved before visibility filtering. Install all shared services with `slint_kit::install`; installing codecs alone does not install table projection/session guards.

Select/Combobox/MultiSelect share an expandable trigger. The accessible Expand action opens a closed popup; it never toggles an open one. Collapse and Accordion use the same distinction between Expand and ordinary toggle activation. Disabled/read-only guards and local/controlled ownership remain in force: publish the proposed state when controlling a component. Slint1.18.1 maps the properties and action into AccessKit, but accesskit_macos0.26.3 does not expose expanded state or Expand to macOS accessibility; qualify your target platform separately.

`SelectBinding` with `RemovalPolicy::Retain` preserves the accepted off-page identity without aliasing a current row token. `Choices::retain` can register a key/label absent from the current page; for an accepted key never displayed in the component, supply its label through `selected-label` using `Choices::label`. The component automatically retains the last label it actually displayed. The `Choices` key/label registry lasts for the lifetime of that source handle.

Wrapped segmented grids preserve source column positions, including disabled cells and a partial last row. Vertical arrows skip disabled cells within that column; `wrap` wraps within the same column. `SelectController::{previous_row,next_row}(columns)` and `KitSelection.{previous-row,next-row}(columns)` share this behavior; `KitSelectionService` exposes the corresponding pure transitions. Page navigation in a list continues to count eligible rows.

Carousel arrow/Home/End navigation belongs to its own focus target and built-in navigation actions. A native editor or checkbox placed in `@children` keeps its editing and form keys; rejected child keys bubble to the enclosing application. Activate slide actions through their dedicated button.

`SelectBinding::navigate(step, page_rows)` takes the current `Option<NonZeroUsize>` viewport capacity on every call; `None` means unmeasured and rejects page navigation without changing or committing selection. Ordinary arrows/Home/End remain available. `navigate_page` takes an explicit `NonZeroUsize`. The deprecated `bind_select!` window convention now requires `select-page-rows`, projected from the active surface; it reads the weak window getter again for every key. In Slint, `page-rows: 0` means unmeasured and PageUp/PageDown are rejected until layout supplies a positive capacity.

A declarative mask uses the same compiled `MaskPattern` and HTML constraint engine as Rust acceptance. Keep native drafts intact after deletion or host updates; submit only after the shared constraints and `FieldSpec::masked` both accept the full value.

Horizontal wheel scrolling changes a table viewport without changing the active cell or submitting a value. The next Left/Right cell-navigation action reveals its new active cell. Cell focus frames follow the cell bounds independently of the native editor used during an edit.

Select and MultiSelect distinguish `label` (stable accessible name), `placeholder` (empty hint) and `value-label` (accessible current value). No extra visible label row is inserted; compose `KitFieldLabel` for a caption and connect its focus request to `take-focus()`. **Migration:** stop putting the selected display value or field name in `placeholder`. Set the name with `label`; for explicit `rows`/`state` projections, supply `selected-label` instead. An available empty option label stays empty. An unavailable controlled choice displays `unavailable-label` and `theme.icons.warning` without changing the host selection.

MultiSelect defaults to a count-and-label summary of its chips. Localize `KitStrings.multi-selection-count-format` (`{count}`), `multi-selection-value-format` (`{count}`/`{labels}`), and `selection-label-separator`; `select-label`, `multi-select-label`, the Select/MultiSelect placeholders, `combobox-placeholder` and `selection-unavailable-label` are window defaults. Per-component `value-label` can override the summary without changing membership. A controlled removal updates it after host acknowledgment. Combobox publishes `placeholder` separately from its stable `label`, defaulting the hint to `KitStrings.combobox-placeholder`. Its accessible value is nonempty native `text`, otherwise the accepted `selected-label`; preserve the native editor’s draft value while editing and its accepted selected-label projection independently of search results.

Set a window-local calendar clock explicitly after `install` and before `show`:

```rust
let zone = slint_kit::TimeZoneId::new("America/Toronto")?;
let today = slint_kit::refresh_calendar_today(&app, zone)?;
```

For deterministic/replayed time use `set_calendar_today_at(&app, zone, instant)` instead. Both helpers validate the projected civil date before changing `KitCalendarService.today`; they add no timer and infer no system zone. Refresh at your date rollover. Each calendar/picker may override `today`. Empty initial focus chooses today (or a stored value), then `DateConstraints::nearest_allowed` inside the bounds; ties prefer the future. Blocked intervals are skipped and wholly blocked weekdays return no eligible date, so long unavailable spans are not scanned day by day. A calendar with no allowed date remains browsable but cannot accept a date. Without a supplied today or opt-in clock, the deterministic fallback remains January2000. Updating today marks that date without replacing a user-navigated focused value.

`Instant::from_unix_seconds` validates stored Unix seconds. `TimeZoneId::local_at` converts that typed instant to a civil date and time in an explicit zone and rejects civil years outside 1–9999; neither helper reads a clock or infers a system zone. Preserve the instant when an overlap projects two occurrences to the same local value.
