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

`KitSlider` and `KitRangeSlider` expose read-only `spec-valid` and `value-valid`; both are false before `install(&app)` makes the numeric service available. A valid spec parses exact decimals with minimum < maximum, positive step and representable arithmetic. `value-valid` additionally requires a value on `minimum + n*step` within bounds; for a range it covers both ordered endpoints. Loading data or changing spec never rewrites it: the visual position is clamped to 0..1 without snapping an off-grid value. Next/Previous recover the adjacent step in their direction; Page Next uses floor(step position) + `page-steps` (ten when unset), Page Previous ceil(step position) - `page-steps`, then clamps to admissible bounds. Keyboard navigation from an out-of-range number recovers to an admissible bound. An admitted range edit repairs both endpoints before constraining the active thumb, so the inactive endpoint can change. Malformed data and reversed ranges remain rejected. Local ownership accepts the candidate; controlled ownership only proposes it. Pointer cancellation restores the pre-gesture snapshot. An invalid spec blocks edits, renders disabled and emits one debug diagnostic per distinct spec per window. Bind the flags to application-owned messages as needed; invalid accessible profiles do not offer an AT recovery action.

Accessible slider metadata uses the same domain: `spec.minimum`, `maximum` and `step`, the current decimal value, and the other endpoint as each range thumb’s inward bound. Accessible set-value follows pointer rounding and bounds through `DecimalSlider::at_value`; it emits `chosen` and `committed` once for a changed valid candidate, and preserves controlled ownership. Supply matching `spec`/`value` even when overriding normalized positions. Color channel sliders use 0..255 with step 1.

`value-label` and range `lower-label`/`upper-label` supply formatted accessible descriptions; the numeric accessible value remains the decimal text. Slint 1.18.1 has no independent value-text property and restricts accessible bounds/step to Float32, which can collapse large or closely spaced bounds. Do not use those projections to accept domain values. Metadata and synthetic action coverage is headless; qualify the named backend and screen reader separately.

`FieldBinding` and explicit `FieldController` remain available for custom integrations. The ordinary field path needs no component-specific macro. `init_form_validation!` is deprecated in favor of `install`.

The legacy `bind_select!` macro is retained for migration and deprecated in documentation. Replace its window callback convention with `Choices<K>`, the component’s `options`/`selected`/`chosen` contract and the shared installer. Explicit `SelectBinding` projections remain available for custom controlled integrations.

<h2 id="typed-temporal-host-values">Typed temporal host values</h2>

Temporal host inputs synchronize the accepted baseline and idle display after `slint_kit::install(&window)`, including controls mounted later. Supply typed values directly; no text-mirroring callback is needed. The complete installed starters load [dates](/components/date-field/), [times](/components/time-field/) and [date/time pairs](/components/date-time-field/) from typed properties without authored draft strings. Catalogue snippets keep explicitly labelled static text for pure-Slint previews; that preview text does not demonstrate service synchronization.

| Component | Host property | Absence |
| --- | --- | --- |
| `KitDateField` | in-out `accepted-date: CivilDateView` | `{}` |
| `KitMonthField` | in-out `accepted-year`, `accepted-month` | both `0` |
| `KitYearField` | in-out `accepted-year` | `0` |
| `KitIsoWeekField` | in-out `accepted-week-year`, `accepted-week` | both `0` |
| `KitTimeField` | in-out `accepted-time: LocalTimeValue` | `{present: false}` or `{}` |
| `KitTimePicker` | in-out `accepted-time: LocalTimeValue` | `{present: false}` or `{}` |
| `KitDateTimeField` | in-out `accepted-value: LocalDateTimeView` | `{}` |

`LocalTimeValue` contains `present: bool` and `time: LocalTimeView`. Midnight is `{present: true, time: {}}`; an empty wrapper means absence. **Migration:** replace every bare `accepted-time: {hour: 9, minute: 30}` with `accepted-time: {present: true, time: {hour: 9, minute: 30}}`. Wrap host assignments and two-way bindings too, and check `.present` before reading `.time`.

The accepted properties on DateField/MonthField/YearField/IsoWeekField/TimeField are in-out typed values, like DatePicker's. Under Local ownership an accepted commit parses the text in Rust and writes the typed value back (absence writes the absent value: `{}` or zeros), so a pure-Slint host can bind with `<=>` and read the value without parsing strings. A one-way binding is replaced by the first local write; use `<=>` to keep sharing it. Under Controlled ownership the properties are not written: accept the commit with the matching Rust `FieldSpec`, keep the returned typed value and assign the property to acknowledge. Invalid or incomplete text never reaches an accepted commit, so the typed value keeps its last accepted state. `acknowledge(FieldValue)` changes the native accepted baseline only; it does not set a typed host property or rewrite the active draft.

TimePicker additionally emits `chosen(LocalTimeView)` for an accepted present time and `cleared()` for valid optional absence, preserving `committed(FieldCommit)`. Local ownership updates its in-out model. Controlled ownership proposes without replacing it: acknowledge with the complete `accepted-time` wrapper. DateTime similarly publishes/proposes an entire `LocalDateTimeView` through `chosen`, and absence through `cleared()`; acknowledge by assigning the whole `accepted-value`. Both date and time must be valid together, or both empty for optional absence. Moving between its editors never accepts half a pair, and Enter followed by blur does not repeat an unchanged proposal.

A valid host assignment or clear updates idle text and the accepted baseline. An authored initial draft, including an explicitly empty one, is preserved. During native focus a host update acknowledges the new baseline while preserving draft, caret, selection and undo; blur/cancel reconciles through the existing ownership policy. Do not implement a host text rewrite to force synchronization. Invalid typed dates, times or year/month/week combinations are ignored, preserving the prior state; they are never converted to absence. Partial month/week pairs are invalid, while the historical all-zero pair means absence. These contracts do not qualify native IME, devices or screen readers.

TimePicker adjustment follows the shared time constraints and the intersection of their exact lattice with the selected minute/second precision. An empty first click seeds the first legal representable point (valid min, otherwise a supplied valid default, otherwise midnight/the first permitted point), without another increment; both directions use the same seed. With step 900 seconds, minute/second adjustment moves 15 minutes and hour adjustment one hour. Whole-time carry is retained; no-bound movement wraps on the grid, ordinary bounds clamp, and min > max admits a periodic interval across midnight. Malformed bounds are ignored; invalid/nonpositive step uses the HTML 60-second default and any removes the grid restriction. No representable candidate means no edit/acceptance. Nonempty invalid drafts remain editable; adjust observes intent only. Native replace-draft preserves undo/edit notifications; the existing Local/Controlled typed value contract remains in force. See [time adjustments](/guides/forms/#time-adjustments).

The pure core `HtmlConstraints::step_time(current: Option<LocalTime>, direction: StepDirection, segment_seconds: NonZeroU32, precision: TimePrecision) -> Option<LocalTime>` accepts an already parsed time; `current: None` requests the empty seed and a returned `None` means no admissible adjustment. Segment duration is a nonzero number of seconds. Keep a nonempty invalid draft distinct from `None`.

<h2 id="duration-units-and-money-rules">Duration units and money rules</h2>

`KitDurationField` edits a decimal quantity. Its suffix is visual: pair `text: "5"; suffix: "s";` with `FieldSpec::duration(DurationUnit::Seconds, rules)`. The required unit is one of `Milliseconds`, `Seconds`, `Minutes` or `Hours`. `DecimalRules` checks the input quantity's locale, scale, sign and bounds. Widget `min`, `max` and `step` use that same visible quantity and canonical attribute syntax; they are not milliseconds unless the selected unit is milliseconds. Clock text such as `01:30` is not a duration codec format.

The factory returns `DurationFieldSpec`, which reuses the shared field engine and keeps the parsed quantity for formatting accepted drafts. `accept(commit)` returns `Result<Option<DurationValue>, FieldSpecError<ParseIssue>>`; pass the returned value on. Register `named(key)` normally; `with_display` and `with_grouping` remain available. `DurationValue` stores nonnegative whole `u64` milliseconds: `5` seconds → `5000` ms, `5` minutes → `300000` ms, and `1.5` seconds → `1500` ms. With scale 3, `0.001` seconds → `1` ms. Sub-millisecond results and excessive input scale return `ScaleExceeded`; conversion overflow or negative converted values return `Overflow`, while `SignPolicy::NonNegative` rejects negatives before conversion.

For loading and controlled acknowledgement, first handle `spec.format(&value) -> Result<String, ParseIssue>`. It uses `DurationValue::quantity(unit)` for exact inverse conversion and rejects quantities outside the configured scale or bounds. A recurring decimal such as `1` ms in minutes/hours returns `ScaleExceeded`; no rounding or floating-point conversion hides the loss. Handle the error before changing the UI value. Other `FieldSpec` formatters still return `String`.

`KitMoneyField` is a decimal presentation alias. `FieldSpec::money(currency, rules)` attaches the supplied `CurrencyCode` and applies the supplied `DecimalRules`; it does not infer currency precision from `CurrencyMetadata`. To take the scale from the currency, use `FieldSpec::money_in(metadata, syntax, sign, bounds)`: input finer than the currency's minor units is rejected with `ScaleExceeded`, the at-rest display shows the symbol and grouping, and the editable draft stays a plain decimal. `CurrencyMetadata::iso(&code)` returns the ISO 4217 scale and a customary symbol for common currencies (`None` when the code is not listed); `CurrencyMetadata::new(code, symbol, scale)` builds any other. The application chooses supported currencies, sign, bounds and business rounding. For explicit formatting, call `Money::format(&metadata, syntax, rounding)` and handle its `Result`: it checks matching currency and applies metadata scale with the explicit rounding policy. Prefix/suffix text enforces none of those rules.

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

`KitSelect` and `KitMultiSelect` share one two-way `query` with the popup search editor. A single Select choice or actual popup session closure clears it; MultiSelect keeps it after a toggle while `close-on-select:false` leaves the popup open. With `close-on-select:true`, an admitted popup toggle closes and resets it. Escape, Tab, outside and programmatic popup closure clear the old query; persistent lists retain their independent filter. A nonempty reset emits `query-edited("")`, so remote providers can cancel the previous search and restore an unfiltered page. The callback for a closing choice sees the cleared filter; an open MultiSelect toggle sees the retained filter. Reopened sessions/new callback queries survive cleanup, and cleanup never acknowledges controlled `selected`. Combobox still cancels pending debounce on suggestion closure without rewriting native text or accepted selection. Custom surfaces bind the same query that filters their displayed rows.

`KitMultiSelect.close-on-select` now defaults to false: add/remove several choices in one open session with its active focus and filter preserved. Local ownership updates `selected`; controlled ownership emits `selection-changed(row, added)` and keeps both old membership and checkmarks until the host acknowledges. Limit/stale/disabled rejections do not toggle or close. Select and MultiSelect accept `[ChoiceRow] choice-rows` for explicit markers, matched by token; MultiSelect derives them from the full selected set, so preselected, reordered and visible filtered choices remain marked with a check glyph and `accessible-item-selected`. Use `close-on-select:true` to restore the former popup behavior. Advanced composition defaults are closing `KitSelection` and keeping-open `KitMultipleSelection`. Core `SelectController::activate_keeping_open` / `SelectEvent::ActivateKeepingOpen(K)` retain the session; ordinary `activate` still closes. Manual `KitSelectionService.activate` callbacks now take a sixth `close_on_select: bool` argument after ownership; ordinary `install` wiring handles it. Update exhaustive `SelectEvent` matches for the new variant.

`KitSelectKeyboard`, `KitSelectTrigger` and `KitSelectSurface` are composition primitives. Their public props allow you to share one selection instance between parts; they are not separate application controllers. The complete Select/Combobox controls provide this composition already. Calendar/date/time components likewise keep local state; receive `CivilDate`, `DateRange`, `LocalDateTime` or `ResolvedDateTime` by checked conversion of their `chosen` payload.

`KitSelect` and `KitMultiSelect` now default to `searchable:false`: no search row or reserved editor height, and printable keys keep prefix typeahead. Set `searchable:true` to opt into native filtering; `focus-search-on-open:true` (the new Surface default too) focuses its real TextInput. On an inline/embedded popup path, opening by typing sends the first character to the native editor. With focus-search-on-open:false, the list receives focus first; forward Tab or supported printable redirection enters search. Tab/Backtab from the editor still closes and traverses normally; arrows/Page keys, Return and Escape retain navigation/choice/close behavior. Forward `search-label` and `empty-label` for localized search/zero-result text. `KitSelectSurface` remains searchable by default and exposes `type-ahead(string)` for custom input: native editing when searchable, prefix navigation otherwise. Call `slint_kit::install(&window)` once; do not build query by concatenating keystrokes. Selection ownership, MultiSelect close-on-select, real-close query reset and persistent filters are unchanged. These native input paths do not qualify a physical IME or screen reader.

Slint 1.18.1 limitation: with a separate native popup Window (the Qt backend path), the opening printable character and printable redirection from a list-first popup cannot reach the popup editor through host-Window replay. Open with an arrow and focus-search-on-open:true, or use forward Tab from the list, then type normally in the focused editor. Inline Surface is unaffected by this Window boundary; a Surface placed in a separate native PopupWindow is affected. The SDK exposes no public popup Window/input target for retargeting; query is not manually rewritten. This is not a complete Qt search fix or physical IME/AT qualification.

`KitSelectKeyboard.move-active(NavigationStep)` and `type-ahead(string)` route custom navigation surfaces through the same session as native keyboard events. Use `choose-option(token, generation)` for activation; keep host input gates on the calling surface.

Choice and navigation composites expose configurable keyboard policies while keeping existing defaults: `wrap` defaults to `true` on `KitRadioGroup`, `KitSegmentedControl`, `KitToggleGroup`, `KitMenu`, `KitMenuBar`, `KitTabs` and `KitCarousel`, and `false` on `KitSelect`, `KitSelectTrigger`, `KitSelectSurface`, `KitSelectKeyboard`, `KitMultiSelect` and `KitCombobox`. Closed `KitSelect` triggers open and move to the first, last or matched option on `Home`, `End` or printable typeahead. `typeahead-timeout-ms` defaults to `700`, accepts spaces inside an active query, and resets on arrow/Home/End/PageUp/PageDown navigation. `PageUp`/`PageDown` step by visible `page-rows` (`viewport-height / row-height`, `0` until measured; unmeasured PageUp/PageDown are rejected). `KitTabs.activation` selects `TabActivation.manual` (default) or `.automatic`; `KitToggleGroup` adds `direction` and `flow`; `KitMenuBar`, `KitTabs` and `KitCarousel` inherit `flow: theme.direction`; and `KitCarousel` handles `LeftArrow`/`RightArrow` (mirrored in RTL) plus `Home`/`End`.

Nonvirtualized MenuBar/Tabs and choice/rating controls keep native focus on persistent scopes and delegate once to their materialized semantic option. Physical RTL/grid ordinals differ from logical item index/count. Radio/Toggle retain their single native keyboard scope across orientation/model replacement; delegation accounts for both materialized layouts, including the hidden set. One Tab stop and host editor redirection are preserved. `KitSelectSurface.accessible-name` names its list; custom `KitOption` loops must bind inherited `accessible-item-index` and `accessible-item-count` themselves. Virtualized Menu/Select and editable Combo/search retain their native scope/editor: Slint 1.18.1 has no safe public virtualized-child ordinal or editor-to-option identifier relation. An ephemeral row can be destroyed without FocusOut, so cached focus must never trigger background refocus. Tabs-to-panel identifier relations and VoiceOver/NVDA/physical IME qualification remain unsupported or unqualified; model-position getters do not establish an AT result.

## KeyedModel for host-owned data

`KeyedModel<K, R>` associates a unique typed key with each public row. `R: KeyedRow` defines the kit projection; built-in rows cover table, content, text, chip, file, notification and toast views. `model()` provides a virtualizable model. `replace` rejects duplicate keys before changing anything and retains tokens for surviving keys. Removed tokens are never recycled.

Each `KeyedModel` owns a wide generation sequence; its positive Slint tag cycles after `i32::MAX`, cancelling live component gestures on every change. Do not persist a bare generation as a durable version: an arbitrarily old tag can repeat after a complete cycle. Row tokens remain process-unique and never recycled. If the finite token space is exhausted, new/replace/insert return `ModelError::IdentityExhausted` transactionally instead of panicking.

For collection views with explicit ports, `bind` takes the window and setters for rows, generation and active index. The binding retains a weak window. `navigation()` implements shared roving navigation; `activation(handler)` updates `SelectableRow` state and calls the business handler with `K`. Validate other intents through `accept(token, generation)` or `remove_fenced`. Token-only chip callbacks use `remove(token)` with a non-reused token. Changing source content invalidates generation-fenced gestures; do not substitute an index for a key.


`KitDataTable` and `KitContentList` report a typed `gesture(token, generation, RowGesture)` next to `activate`: a click is `select`, `toggle` with Control or Meta held and `extend` with Shift held; Space is `toggle`; Enter, or a double-click on a row without an editable cell, is `open`. `activate` keeps firing for every activation, so connect one of the two. For several selected rows, bind `KeyedModel::gestures(SelectionMode::Multiple, handler)`: it applies `select`, `toggle` and `extend` (a range from the last plain or toggled row, disabled rows skipped) and calls `handler(RowGesture, K)` so the application acts on `open`; `SelectionMode::Single` reduces every selecting gesture to `select`. `selected_keys()`, `select_all()`, `clear_selection()` and the fenced `toggle` and `select_range` are available for bulk actions. `SelectableRow` now also requires `selected()`. The kit paints no selection checkbox column and reports no context-menu gesture yet.

`KitDataTable` and `KitContentList` take `state: DataState` (`ready`, `loading`, `failed`). While the model is empty, `loading` shows a spinner (`loading-label`) and `failed` shows `error-label` with a retry action (`retry-label`) that emits `retry-requested()`; restart the load there and set `state` back to `loading`. Rows already shown stay visible. `empty-content: DataEmptyContent.custom` shows the component's children in place of `empty-label`. `KitDataTable` also takes `surface-style: DataSurface.flush` (no fill, border or shadow, for use inside a card) and `striped: false`; set `min-height` on an instance for a table shorter than 160px.

`KitContentList` takes `control-size: ControlSize` (`compact`, `medium`, `comfortable`) for the row height and thumbnail size, `thumbnail: ContentThumbnail` (`image`, `icon`, `none`) for the leading cell, `excerpt-lines` (default `2`) for the excerpt height, and `vertical-scrollbar-policy` (default `always-on`; `ScrollBarPolicy.as-needed` hides the bar until rows overflow). `interactive: false` makes rows, links and keys inert and takes the list out of keyboard focus, which blocks it behind a dialog without covering it. A row with a `link-label` draws it as its own button: `link-activated(token, generation)` fires and the row is not activated. A row is named from its non-empty title and metadata and described by its excerpt, so an item without metadata reads `Title`, not `Title, `.

The application still chooses data, sorting, filtering, loading and commands. A large model belongs in a virtualized collection control, not repeated visual children. `action`, `event`, `event2` and `event3` connect business callbacks with a weak window and pass a temporary live handle to the handler.

For `KitVirtualTextView`, `active-index` is a visual coordinate only. Native line editing, Save and Copy require `TextViewAccess.edit-line` and an index within the current `lines` model. Shrinking or clearing a model must not create a token from an invalid index. Save/Copy reject a captured token/generation that becomes stale, including around Copy’s focus/clipboard effects. For admitted lines Copy uses the current native draft and restores editor focus; read-only mode keeps the footer hidden. `font: TextViewFont.monospace` and `show-line-numbers` set the typeface and the gutter; Ctrl/Cmd+C and `copy-active-line()` copy the stored line text in any access mode without moving focus and emit `copy-line-text(token, generation, text)`, which hosts validate like `copy-requested`. Validate `commit-line`/`copy-requested` with `KeyedModel::accept`; use `update_active` and `bind_active` to keep edits attached to the stable key rather than a displayed row or line number.

For numeric codecs, pair the control’s number-format with DecimalRules.syntax: canonical with NumberSyntax::en_ca()/en_us(), decimal-comma with fr_ca()/fr_fr(). The shared grammar admits strict grouping, signs and outer whitespace before exact scale/sign/bounds checks; min/max/step/default-value remain canonical attributes. FieldSpec::accept returns the parsed T: pass it to business code instead of parsing its text again. Plain draft formatting is separate from format_decimal grouped display; with_display can configure at-rest presentation. FieldSpec::integer(syntax, bounds) takes the syntax and the bounds; NumberSyntax is an open type (de_de, de_ch, pt_br, en_in or NumberSyntax::new). The native number-format control knows only canonical and decimal-comma: for another syntax, use a text control with the typed codec.

## Global state with Store

Rust reaches a Slint window only through its root component and its `global`s. Use that: one Slint `global` is one store. The global declares the properties every component reads; a `Store<T>` in Rust owns the typed value and projects it into the global.

```slint
export global Session {
    in-out property <string> user;
    in-out property <int> unread;
}
```

```rust
#[derive(Clone)]
struct SessionState { user: String, unread: i32 }

let store = Store::new(SessionState { user: "Ada".into(), unread: 0 });
let weak = app.as_weak();
let _projection = store.observe(move |state| {
    if let Some(app) = weak.upgrade() {
        let session = app.global::<Session>();
        session.set_user(state.user.as_str().into());
        session.set_unread(state.unread);
    }
});
store.update(|state| state.unread += 1);
```

Any component that reads `Session.unread` shows the new value, whichever screen it is on. In React terms, the `global` is the context consumed by components and the `Store<T>` is the reducer state that feeds it. Keep the `_projection` alive as long as the window; dropping the `Subscription` stops the projection.

- `get`, `with`, `set` and `update` read and write on the UI thread. Observers run after each change with a snapshot. An observer that changes the store sees the change applied at once, and observers run again with the latest value, so none stays stale.
- `subscribe` reports changes only; `observe` also calls the observer with the current value.
- `store.remote()` returns a `StoreRemote<T>` that is `Send`. Call `update` or `set` from a thread or an asynchronous task: the change is queued with `slint::invoke_from_event_loop` and applied on the UI thread. It returns `StoreError` when no event loop runs, and a change for a dropped store is discarded.
- Business state belongs to your application. The store holds the projection the views need, not the source of truth.
- Use one store per domain (session, settings, cart) rather than one store for the whole application: a component then depends only on the global it reads.
- Globals exist once per window. To keep several windows in step, observe the same store once per window; each projection writes into that window's global.

## Screen navigation with Router

Declare the screens as one route type. A plain enum names them; an enum whose variants carry data carries the route parameters with the screen. Rust owns the stack. Slint only shows the screen that is current.

```rust
#[derive(Clone)]
enum AppRoute { List, Detail { id: u32 }, Edit { id: u32 } }

let router = Router::new(AppRoute::List);
let weak = app.as_weak();
let _projection = router.observe(move |navigation| {
    if let Some(app) = weak.upgrade() {
        app.set_change(navigation.change().into());
        app.set_route(match navigation.route() {
            AppRoute::List => Screen::List,
            AppRoute::Detail { .. } => Screen::Detail,
            AppRoute::Edit { .. } => Screen::Edit,
        });
        app.set_can_go_back(navigation.can_go_back());
    }
});
router.push(AppRoute::Detail { id: 7 });
router.pop();
```

```slint
KitRouter {
    can-go-back: root.can-go-back;
    back => { root.back(); }
    KitScreen { active: root.route == Screen.list; change: root.change; /* list */ }
    KitScreen { active: root.route == Screen.detail; change: root.change; /* detail */ }
}
```

- `push`, `pop` and `replace` change the stack. `pop` at the root does nothing and returns `false`. `replace` swaps the current route and keeps the history.
- The `match` that converts the route to the Slint enum has no wildcard arm. A new route does not compile until that match handles it.
- `on_navigated(|route, change| ...)` reports each navigation. `observe` also reports the current route, which is how the window is projected.
- `KitRouter` turns Escape and the system back key into `back()` while a screen is behind the current one. Call `router.pop()` from that callback. At the root, and while a modal layer is open, the key is left alone.
- Each `KitScreen` moves with the theme motion tokens: a push slides in from the trailing edge, a pop from the leading edge, a replace fades. Reduced motion removes the movement. A hidden screen takes no pointer, focus or accessibility.
- Deep links and a browser history are not part of the router.

## Async calls

Slint changes a window only from its event loop, so a result that arrives later needs the window again. `spawn` and `Latest` hide that step: the closure receives the window, already upgraded, and application code holds no `as_weak` and no `upgrade`.

```rust
#[derive(Clone)]
struct Users { rows: Vec<String> }

let latest = Latest::new();
let store = Store::new(Loading::<Users, String>::Loading);

app.on_reload({
    let app = app.as_weak();
    let latest = latest.clone();
    let store = store.clone();
    move || {
        let app = app.upgrade().unwrap();
        latest
            .load(&app, fetch_users(), move |_, state| store.set(state))
            .unwrap();
    }
});
```

- `spawn(&app, future, |app, result| ...)` runs the future on the event loop and applies the result with the window. If the window was destroyed, the result is dropped. Every call delivers its own result.
- `Latest` guards against stale responses. Each `spawn` or `load` makes the earlier requests stale: the older future is dropped at its next wake-up and its result is never applied, even when it arrives after the newer one. Create one `Latest` per source of requests (a search box, a list that reloads). Clones share the guard; `cancel()` discards the request in flight.
- `Latest::load(&app, future, |app, state| ...)` takes a future that returns a `Result` and reports a `Loading<T, E>`: `Loading::Loading` at once, then `Loading::Ready(value)` or `Loading::Failed(error)`. `Loading` offers `is_loading`, `ready` and `failed`. Project it into component properties, or into a `Store` as above, to show a spinner, the list or an error message.
- `Fence` and `FenceTicket` (in `kit-core`) are the generation guard behind `Latest`, for code that runs its own tasks: `begin()` returns a ticket and makes older ones stale, and `is_current(ticket)` tells whether a response may still be applied.
- The kit imposes no asynchronous runtime. The future is polled by Slint's event loop through `slint::spawn_local`, so it need not be `Send`. A future that needs a runtime reactor (tokio sockets or timers, for example) must bring it: wrap it with `async_compat::Compat`, or await the `JoinHandle` of a task that your own runtime runs. A future that uses only plain wakers, such as channels, needs nothing more.
- Call `spawn` and `load` on the UI thread, for example from a callback. They return `SpawnError` when no event loop runs. To report a result from another thread instead, use `Store::remote()`.

## Host effects and notifications

Localize built-in action and region names once per window through `KitStrings.dismiss-label-template` (`Dismiss {title}`), `breadcrumbs-label`, `pagination-label` and `notifications-label`, or override each component’s `dismiss-label` / `accessible-name`. `ToastEntry.dismiss-label` carries a complete per-entry name through `KitToastHost`; empty uses the global template, and `KitToast.dismiss-label` can override it. Complete Rust `ToastEntry` literals now need `dismiss_label: Default::default()` (or a localized name); literals ending with `..Default::default()` remain compatible.

`KitStrings.carousel-position-template` and `pagination-position-template` default to `{current} / {total}`. For example, use `Page {current} of {total}` for pagination. The kit substitutes the bounded one-based current position and total, or zero/zero for an empty collection. `KitCarousel.position-label` and `KitPagination.position-label` accept a complete host-formatted override; bind it to current state so it follows navigation. There is no locale inference or automatic translation. Existing previous/next/empty/action label properties remain independently overridable.

For numbered pagination, set `KitPagination.display: PaginationDisplay.numbered` and translate `first-label`, `last-label`, `previous-label`, `next-label` and `page-label-template` (`Page {page}`) per instance; they are not part of `KitStrings`.

Choice controls use the same window defaults: `search-options-label`, `no-matching-options-label`, `chip-remove-template` (`Remove {label}`), `tag-input-placeholder`, `checkbox-mixed-label`, `rating-clear-label`, `range-lower-thumb-template` and `range-upper-thumb-template` (`{label}` stands for the field label). `KitSelect`, `KitMultiSelect`, `KitCombobox`, `KitChip`, `KitTagInput`, `KitCheckbox`, `KitRating` and `KitRangeSlider` also expose a property for each string. Read-only controls report `accessible-read-only` instead of a spoken "Read only" description.

`FilePickerBinding` and `DirectoryPickerBinding` accept explicit projection ports, install weak-window handlers and deliver typed `FileHandle` or `DirectoryHandle` values. Request generations discard stale asynchronous results. The optional `native-winit` feature adds a window-level drop bridge with an explicit capability. See [Native integration](/guides/native/) for platform limits.

`ToastBinding<Action>` stores typed actions in `ToastQueue`, projects `ToastEntry` rows and manages a weak timer. Capacity and `ToastLifetime` are explicit. Its three handlers route action, dismissal and independent pointer/focus pauses; the application only performs the resulting business action. Surviving toast delegates retain keyboard focus and independent hover/focus pauses when another entry is appended, dismissed, expires or is evicted. Keep the `model()` handle installed; the binding applies incremental keyed insertions/removals. `KeyedModel::insert` and `remove_by_key` preserve surviving delegates while advancing the generation; `replace` and sorting still reset their projection.

Each `KitToast` draws a severity mark from the alert recipe. Escape on its focused action or dismiss button closes a dismissible toast. For a hotkey the application defines (web toasts use F8), call `KitToastHost.take-focus()` to focus the first toast's action, or its dismiss button when it has no action; the kit binds no key itself.

Use the stable `ToastId` from `ToastBinding::push` with `dismiss_id(id) -> Option<T>`, rather than treating a visual token as a queue ID. `clear()`, `len()` and `is_empty()` operate on the binding. `update(id, view, lifetime) -> bool` changes only the view and duration: it preserves the ID/token, typed action and separate hover/focus pauses, restarts lifetime at now and emits no close notification for the update itself. A missing or expired ID returns false and is not resurrected.

Register `on_closed(&self, Fn(ToastId, T, ToastCloseReason) + 'static)` once; registering another observer replaces it. The reason is `Expired`, `Dismissed`, `Action`, `Evicted` or `Cleared`. `T: Clone + 'static` remains required: dismissal returns the action while the observer gets its clone. Notifications follow model/timer publication with no binding borrow held; callbacks may reenter without losing their mutations. `action_handler` delivers the Action close notification before the business action, exactly once. Persistent lifetime does not prevent capacity eviction or explicit removal. Capture `app.as_weak()` and upgrade it inside a callback that touches UI; do not capture a strong window.

After creating `toasts`, this observer retains the complete typed records without logging user payloads:

```rust
use std::{cell::RefCell, rc::Rc};

let closed = Rc::new(RefCell::new(Vec::new()));
toasts.on_closed({
    let closed = closed.clone();
    move |id, action, reason| closed.borrow_mut().push((id, action, reason))
});
```

For a custom runtime, consume every core closure record. `ToastQueue::push` now returns `Result<ToastChange<T, ToastId>, ToastQueueError>`: use `.result` for the new ID and handle `.closed`, including expirations before any capacity eviction. `advance(now)` and `clear(now)` return `Vec<ToastClosed<T>>`, each carrying `id`, full `value` and `reason`. `set_paused` and `update(id, lifetime, now, FnOnce(&mut T))` return `ToastChange<T, bool>`; process `.closed` even if `.result` is false because an entry may have expired. Core update explicitly mutates its payload; binding update preserves its typed action. `ToastBinding::push` keeps its existing `Result<ToastId, ToastQueueError>` return type.

`ToastBinding` schedules only the next unpaused timed expiry with a single-shot timer. Empty, persistent-only and fully paused queues do not wake the event loop. Push, dismissal, pause changes and expiry recompute the next delay; releasing one pause source does not override the other. Custom runtime adapters can call `ToastQueue::next_timeout(now)` using the same injected monotonic clock as `advance(now)`: it returns the remaining delay, `None` when nothing is running, or zero for an overdue record. Reading it does not mutate the queue; call `advance` to remove expired records. The timer closure retains only weak binding state, and dropping the last binding stops it even if its projected model survives.

`KeyedModel::sorting_by` turns a domain-key comparator into an alternating sortable-header handler. Sorting preserves selection and active identity and advances the generation, so a pointer gesture started before the reorder cannot target the new projection. The [sortable table application](/guides/table-example/) wires `bind`, `navigation()`, `activation` and `sorting_by` to a `KitDataTable` in one short file.

`KeyedModel::cell_commits(handler)` is the `KitDataTable.cell-committed` handler for in-place cell edits: it admits a commit only under the current generation and for an enabled row, then passes the row key, the column token and the `FieldCommit` (it receives the `CellCommit` payload of `cell-committed`, whose fields are named `row`, `column`, `generation` and `commit`), which the application runs through that column's `FieldSpec::accept` before republishing the row.

Use `Choices::selections(&keys)` for initial multiple selections; missing or repeated keys reject the complete projection. `KeyedModel::bind_active` publishes the active row through a weak window to an editor. Use `update_active` to reject commits for a line that is no longer active.

Match a decimal codec’s `DecimalRules.syntax` to the component’s `number-format`, and a time codec’s `ClockFormat` to the component’s `clock-format`. The HTML engine validates the native grammar; the codec then receives that grammar, including decimal commas and 12-hour clocks. Accepted formatting round-trips through the same parser, so cancelling restores an editable value in the declared format.

`bind_active` publishes only when the active key or its record revision changes. Updating another row and reordering unchanged rows preserve an editor draft. Explicit `replace` supplies fresh records and republishes the active record. Use `get`, `len`, `is_empty`, `update_by_key` and `remove_by_key` for domain access, including disabled records. In-place updates retain the interaction generation and use row notifications; membership changes and replacement invalidate old gestures.

Column resize ignores a click without movement. Escape or a secondary press cancels a drag preview. Double-clicking a divider emits `autosize-requested(column-token)`; calculate the domain-appropriate width and update the column policy.

Drag a column header onto another header to request moving it to that column’s current visible position. `reorder-requested(source-token, target-token)` uses stable tokens and leaves the model unchanged until the host accepts; remove the source and insert it at the target’s previous position, preserving each column’s `source`, width and token. Keep canonical row data unchanged and republish `active-column` for the accepted column key. Drops from another model, stale column revisions, hidden/duplicate tokens and external payloads are rejected. Releasing outside a valid header cancels the move. The divider remains a separate resize target (24 logical pixels at 100%, scaled with text). Ctrl+Shift+Left/Right requests an adjacent move on a focused sortable header or in the roving header region. `reorder-active-column(ColumnStep.previous/next)` exposes the same bounded request to host actions, including for static headers. A disabled table emits no reorder or autosize request. Headless contracts do not qualify OS-native drag-and-drop or screen readers.

`ColumnStep` is the shared presentation enum (`previous`, `next`). `TableColumnDrop` (`valid`, `token`) is the Slint projection of the internal drag-service result, not a component or a host reorder command; ordinary consumers handle `reorder-requested`.

A table cell uses the native field editor. With a cell cursor, Home and End jump to the first and last cell of the row and Control or Meta with Home or End moves to the first or last row; Control or Meta with C (or `copy-active-cell()`) copies the displayed text of the active cell and emits `cell-copied(row-token, column-token, generation, text)`. Click chooses the row and cell cursor, double-click or F2 opens an editable field, Enter commits, Escape cancels and Tab validates before moving. `edit-active()`, `commit-edit()` and `cancel-edit()` expose those operations to host actions. Install the kit once per window, including for plain text cells. `active-index` and `active-column` are view cursors; commit identity is always `(row-token, column-token, generation, FieldCommit)`. Read `edit-error` for a rejected codec or HTML constraint. The accessible active cell exposes its value and the localized `editable-label` / `read-only-label` hint; headless evidence does not qualify a screen reader.

Localize document editing with `editor-label`, `editor-placeholder`, `save-label` and `copy-label`; their defaults remain English.


### Keyboard hooks from Rust

Public kit keyboard hooks return `KitKeyResult`, a shared presentation enum exported by `@kit` and as `slint_kit::ui::KitKeyResult`. Return `KitKeyResult::Accept` from Rust to consume a shortcut, or `Reject` to preserve the control’s built-in behavior. Slint bindings use `KitKeyResult.accept` and `.reject`; the kit converts them at the native `FocusScope` or `TextInput` boundary. Do not use the unrelated winit backend `EventResult` or Slint private APIs.

This result applies to `KitTextField.key-pressed`, `KitDataTable.key-event`, inherited `KitPressable.key-event`, selector `tab-navigation`, and public selector/navigation `handle-key` functions. A text-field hook runs only without active IME preedit. Preserve rejection for normal text and navigation so native editing keeps selection, undo and IME.

`CellContent::tone_text(text, tone)` paints a text cell in the ink of a `Tone` (a negative amount in `Tone::Danger`); a selected or disabled row keeps its own ink. `CellContent::icon(glyph, text, tone)` paints a generated icon: a non-empty `text` makes it an informative image with that accessible label, an empty one hides it from assistive technology. `CellContent::progress(fraction, text, tone)` paints a read-only bar named by `text`; `ProgressFraction::try_from(f32)` accepts only a finite value from 0 to 1, so an invalid fraction fails where the cell is built. A Slint `TableCell` literal sets `kind: TableCellKind.icon` or `.progress` with `glyph` or `fraction`. These kinds are display only: a toggle, a button or link and select, number or date editors are not available yet.

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

A second click on `KitDateRangePicker` before the start restarts the range from that date. Set `earlier-end: EarlierEndPolicy.swap` to turn the two dates into the range, or `.reject` to refuse. A refused choice keeps the draft and sets `rejection` (`DateRangeRejection`: `unavailable-date`, `end-before-start`, `blocked-interior`, `too-short`, `too-long`, or `invalid-configuration`) with a matching message property (`too-short-message` and so on). Read the pending selection with `draft` and `draft-changed(DateRangeDraft)`. In Rust, `DateRangeRules::with_earlier_end` sets the policy and `DateRangeController::choose` returns `RangeRejection`.

Calendar date configuration exposes `CalendarConfigurationState.unchecked` before `install`, then `.valid` or `.invalid`. Malformed/reversed min/max, invalid civil dates or malformed unavailable ranges produce a disabled reference grid plus a visible `configuration-message`; navigation/acceptance cannot bypass those rules. Translate the message once with `KitStrings.calendar-configuration-error`, or override an instance. Repairing the configuration restores the existing component. Range length rules are validated separately by the range reducer and Apply. Complete Rust `CalendarProjection` literals must now include `configuration_state`; `calendar_projection()` supplies Valid for a validated controller.


Calendar `weekdays` models always contain seven names in Monday-to-Sunday order. Set `week-start: WeekStart.sunday` (or any other weekday) to rotate both headers and grid columns. `KitDatePicker` and `KitDateRangePicker` forward the same convention to their calendar. Do not rotate localized names yourself when switching the first day.

All three calendar/date-picker controls expose `outside-days: CalendarOutsideDays` and `week-rows: CalendarWeekRows`; the pickers forward both to their calendar. The corrected default `dim` uses secondary text for unselected adjacent-month days. `show` opts into their previous ordinary text treatment; `hide` leaves blank, nonactivating, accessibility-hidden cells while preserving seven-column positions. `fixed` keeps six rows; `variable` removes whole weeks after the month’s final day, producing four to six rows. Month-crossing keyboard navigation and the existing date constraints, ownership, RTL and localized labels keep their meaning. The public examples explicitly use `dim`/`fixed` and therefore adopt the corrected adjacent-day appearance. `KitCalendar` also takes `day-marks: [CalendarDayMark]` (`date`, `marker: CalendarMarker` = none/dot/bar, `caption`, `description`): the marker and caption render in the cell and the description is appended to the accessible label. Children of `KitCalendar` render as a footer row (Today, Clear, presets). The pickers do not forward these yet.

For custom projections, populate `CalendarDay.outside-month` accurately relative to the displayed month and year; do not mark adjacent dates as inside merely to fill the grid. `CalendarView.week-rows` carries the requested week policy. Complete generated Rust `ui::CalendarDay` and `ui::CalendarView` literals now require `outside_month` and `week_rows` respectively. Both presentation enums are new; match all their variants when using them in Rust. No existing Rust enum gains a variant in this change.

Calendar localization uses explicit text properties on all three controls. Keep `weekdays` as the short visual headers (repeated initials are allowed); provide full `weekday-names` for header names and `%A`, and separate `weekday-short-names` for `%a`, both Monday-first. `month-names`/`month-short-names` supply `%B`/`%b` in January-to-December order. Each formatting array requires twelve or seven nonempty, distinct names; no locale is inferred. `accessible-date-pattern` defaults to `%A %-d %B %Y`, and `title-pattern` to `%B %Y`. The installed service formats each actual date and appends `today-label`, range-boundary labels or `in-range-label`, separated by commas. A single-day range gets both boundary suffixes; a provisional start gets `range-start-label`. Interior suffixes exclude endpoints, and empty suffixes are omitted cleanly. The pickers forward these properties. Bad names/patterns disable the calendar projection, not the DatePicker native editor; accepted dates and drafts are preserved. While the end is pending, `KitDateRangePicker` previews the days from the chosen start to the hovered or keyboard-focused day with the in-range fill (a day before the start only with `earlier-end: swap`). The preview is visual and does not change labels or the draft.

`DateDisplayPattern` is a date-only formatter for these display patterns: it accepts incomplete fragments such as `%B %Y`, exposes `new`, `as_str`, `format` and `format_with_names(CivilDate, &DateNames)`, and has no parser. It shares `%Y %m %d %e %b %B %a %A %j %G %V %u %F %%` and numeric `-`, `_`, `0` padding modifiers with `DatePattern`; time fields, two-digit years and malformed syntax are rejected. `DatePattern` still requires a complete date for input parsing. Native ISO entry and date/range constraints are unchanged. Manual service callbacks now receive `CalendarText`: `project(CalendarView, CalendarText)` and `range-project(CalendarView, DateRangeDraft, CalendarText)` replace the old month-name array. Ordinary `install` wiring and the Rust `calendar_projection` signature are preserved.

`KitDatePicker.display-pattern` applies a `DateDisplayPattern` to the resting text only (for example `%d/%m/%Y` or `%-d %b %Y`), using the picker's `month-names`, `month-short-names`, `weekday-names` and `weekday-short-names`; empty keeps ISO and an invalid pattern falls back to ISO. While the field has focus, editing, native date constraints and `accepted-date` remain `YYYY-MM-DD`; typing a non-ISO date is not supported. A standalone `KitDateField` takes the same data through `display-format: DateDisplay`, and `spec-key` selects a registered `FieldSpec` display formatter when no pattern is set.

`KitCalendar`, `KitDatePicker` and `KitDateRangePicker` expose `flow: ChoiceFlow`, defaulting to `theme.direction`. Set `flow: ChoiceFlow.right-to-left` on one calendar or picker to override its theme; each picker forwards this value to its calendar. RTL mirrors weekday/date columns and the month-navigation controls: previous month is on the right, next month on the left, with matching chevrons and action names. Left moves to the next day and Right to the previous day. Up/Down, Home/End and Page Up/Down (including Shift for years) keep their existing meaning. Keep the weekday names model unchanged: `week-start`, civil-date values, bounds, availability and range validation are independent of reading direction.

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

`Instant::from_unix_seconds` validates stored Unix seconds. `TimeZoneId::local_at` converts that typed instant to a civil date and time in an explicit zone and rejects civil years outside 1–9999; neither helper reads a clock or infers a system zone. Preserve the instant when an overlap projects two occurrences to the same local value. `TimeZoneId::offset_at(instant)` returns the `UtcOffset` (seconds east of UTC, `seconds()`) in force at that instant, `TimeZoneId::all()` iterates every IANA zone the kit resolves (for building a zone list), and `CivilDate::weekday()` returns the day of the week as a `WeekStart`. `ResolvedDateTime::utc_offset()` gives the offset of the accepted occurrence; `ZonedDateTimeCommit.utc-offset-seconds` and `ZonedDateTimeResult.utc-offset-seconds` / `later-utc-offset-seconds` expose it to pure-Slint code.

## Tab strips and stable panels

KitTabs uses horizontal Left/Right navigation (mirrored by flow), Home/End and Enter/Space. Up/Down, page keys, Escape and Control/Alt/Meta chords remain available to the enclosing scope. Printable keys also reach the application unless typeahead-enabled is explicitly true. Space always accepts the active tab. tab-width: ControlWidth.fill gives every tab an equal share of the strip (labels elide instead of scrolling); the default intrinsic sizes tabs to their labels and scrolls. Manual activation changes focus first; automatic activation proposes a selection on arrows or optional prefix navigation. Controlled ownership still waits for the owner to acknowledge the chosen domain key.

Bind each KitTabPanel.state to the strip state and its tab to a SelectionRef from the same Choices source. Resolve references with Choices::selection(&DomainKey), not a visible row index. Accepted selection chooses the panel; a merely focused tab does not. Reordering keeps the association. An unavailable/absent selection hides and collapses the panel according to the strip's existing selection-retention policy. Local bound-to-options selection may clear on removal; controlled values stay owned by the host.

Group panels with KitStack { gap: LayoutSpace.none; ... } so collapsed siblings add no inter-panel gaps. The panel composes KitStack and forwards theme/gap/inset; pass the theme explicitly to supplied children. Children stay mounted with native drafts, caret and undo history. The Slint 1.18.1 native replacement path used in tests records deletion and insertion as separate undo entries; the kit preserves that history. Physical IME is qualified separately.

Leave tab-stop false for interactive content so Tab enters its native controls directly; set it true for static content to add a named panel focus stop. take-focus() explicitly focuses the active panel scope and renders the theme ring. Selection changes never request background focus. The tab-panel role and accessible-name are exposed; Slint 1.18.1 has no public tab-to-panel identifier relation, and headless metadata does not qualify a screen reader.

The new keyboard properties and KitTabPanel require the matching source kit until published in a later release. The low-level KitSelectKeyboard keeps vertical list/typeahead/Escape defaults; navigation-direction, flow, typeahead-enabled and dismiss-on-escape let composed headers select their keyboard profile without reimplementing the state machine.

`KitMenu.items` takes one `MenuItem` per entry of `options`, in the same order, to draw a `kind` (`action`, `checkbox`, `radio`, `separator`, `group`), a `leading-icon`, a `shortcut`, a `destructive` ink and a `checked` mark. The `OptionRow` keeps identity and activation, so the `SelectController` identity and typeahead still apply. Mark `separator` and `group` options `disabled` so navigation skips them, and update `checked` in `chosen`.

`KitSelect.items` and `KitCombobox.items` take one `OptionItem` per decorated option, matched by `token`, to draw a `kind` (`option`, `group`, `separator`), a `leading-icon` and a `trailing-text`. The `OptionRow` keeps identity and activation, so the `SelectController` identity and typeahead still apply. Mark `group` and `separator` options `disabled` so navigation skips them; the kit also makes them ignore the pointer. `KitSelect.clearable` shows a button that empties the selection and emits `cleared()`; with `Ownership.controlled` the host clears `selected` itself. Accent-insensitive or host-defined filtering is not provided: filter in the host and pass the result as `options`.
