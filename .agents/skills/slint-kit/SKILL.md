---
name: slint-kit
description: Build native Rust and Slint applications with Slint Kit, using its public components, typed controllers, themes and build integration. Use when consuming Slint Kit or adapting an existing app to it.
---

# Consume Slint Kit

Read the target application’s dependency and build configuration before integrating it. Use the published `slint-kit` and `slint-kit-build` packages for application integration; a toolkit checkout is optional. This checkout targets `0.5.1`, including the component quality and modal interaction improvements in [CHANGELOG.md](../../../CHANGELOG.md). Verify that it is published before using its APIs from crates.io; use the matching documentation for an older resolved release. The compatible dependency range is `"0.5"` below. Keep the runtime and build packages on matching releases and Slint pinned to their supported version.

## Find documentation and examples

The public documentation is [English at the site root](https://slint-kit.dev/) and [French under `/fr/`](https://slint-kit.dev/fr/). Start with the relevant guide:

- [Getting started](https://slint-kit.dev/guides/getting-started/): a complete registry-based application and desktop prerequisites.
- [Editor setup](https://slint-kit.dev/guides/editor/): resolve `@kit` in VS Code and Zed through project-relative library paths.
- [Component catalogue](https://slint-kit.dev/components/): component screenshots and complete Slint examples.
- [Complete workspace application](https://slint-kit.dev/guides/workspace/): reusable account, project list and edit dialog, sharing Rust/Slint sources with the native showcase and browser demonstration.
- [Typed controller application](https://slint-kit.dev/guides/typed-example/): downloadable Cargo, build, Rust and Slint files, with callbacks connected.
- [Controllers](https://slint-kit.dev/guides/controllers/), [themes and accessibility](https://slint-kit.dev/guides/themes/) and [native integration](https://slint-kit.dev/guides/native/): domain ownership, presentation and host services.

For a registry consumer, inspect the source of its resolved package version when exact signatures matter; `cargo metadata --format-version 1` reports package manifest paths. The relative source links below are shortcuts when this skill lives in a toolkit checkout. If they are unavailable, use the installed package sources or the [published package source](https://docs.rs/crate/slint-kit-slint/0.5.1/source/). Keep runtime and build dependencies on the same `"0.5"` compatibility range.

## Wire the consumer

Merge these registry dependencies into the application’s `Cargo.toml`:

```toml
[dependencies]
slint-kit = "0.5"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.5"
```

Use edition 2024 and Rust 1.92 or later. Preserve the app's chosen backend and renderer when compatible; the manifest above is a desktop example. Add `features = ["native"]` to `slint-kit` only when host services are needed.

In `build.rs`, compile through `slint-kit-build` so the `@kit` library and contract verification remain configured:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

For custom Slint compiler configuration, start with `slint_kit_build::config()` and preserve its library paths. A minimal `ui/app.slint`:

```slint
import { KitButton, KitTextField } from "@kit/kit.slint";
export { KitFormValidation, HtmlInputKind, NumberInputFormat, KitInputConstraints, KitValidationResult, KitValidityState } from "@kit/kit.slint";
export component App inherits Window {
    preferred-width: 360px;
    preferred-height: 180px;
    in-out property <string> name;
    callback save(string);
    VerticalLayout {
        padding: 24px; spacing: 16px; alignment: start;
        KitTextField { label: "Name"; text <=> root.name; }
        KitButton { text: "Save"; activated => { root.save(root.name); } }
    }
}
```

In `src/main.rs`, include the generated module and connect the application callback:

```rust
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = App::new()?;
    slint_kit::init_form_validation!(self, app);
    app.on_save(|name| println!("Draft submitted: {name}"));
    app.run()
}
```

Run `cargo run`. Initialize form validation once per window before showing it. Set intrinsic options such as `required`, `pattern`, `min`, `max` and `step` on the control. Guard Rust submission with matching `FieldBinding::set_html_constraints(HtmlConstraints { ... })` rules before accepting a domain value; the example callback only logs the draft. See the [forms guide](https://slint-kit.dev/guides/forms/). Ordinary fields already include `label`, help and error presentation. `KitFieldLabel` is a composition primitive for custom skins; it is not needed to set a field label. Component catalogue examples export `Example` instead of `App`: use the `Example` entry point from the getting-started guide and remove callbacks from the previous example. Screenshots show the declared state; use the typed application guide for a complete interactive controller.

For local toolkit development, use `slint-kit = { path = "../slint-kit/crates/kit" }` and `slint-kit-build = { path = "../slint-kit/crates/kit-build" }` with paths adjusted to the checkout. The library files are bundled in the registry packages; consumers do not need a checkout.

The Slint language server does not read `build.rs`. For registry consumers, run `cargo vendor --locked target/slint-editor/vendor` from the workspace root, then map `kit` to `target/slint-editor/vendor/slint-kit-slint/ui` in VS Code's `slint.libraryPaths`, or pass `-L kit=target/slint-editor/vendor/slint-kit-slint/ui` through Zed's `lsp.slint.binary.arguments`. Vendoring copies all remote dependencies; refresh after lockfile changes or `cargo clean`. Check the vendor directory name when several kit versions are resolved. Local path dependencies are not vendored: point the editor at the checkout's `crates/kit-slint/ui` using a workspace-relative path. Preserve existing editor settings and library aliases. See the editor setup guide for complete JSON examples.

## Choose and compose

Choose components from the catalogue, then verify their public properties and typed callbacks against the resolved version. Read [the public exports](../../../crates/kit-slint/ui/kit.slint) to find current component names, then read the relevant source for properties and typed callback signatures. The public files are the maintained API reference; do not guess names from a web kit. [The showcase](../../../showcase/ui/forms-gallery.slint) and its [Rust field bindings](../../../showcase/src/form_fields.rs) demonstrate a complete consumer.

Keep accepted application values in Rust. `slint_kit` reexports the domain types and adapters: use enums for policies and states, `FieldBinding<T, E>` for typed draft/commit handling, and `SelectBinding` or `bind_select!` for stable keyed selection. Read [field.rs](../../../crates/kit-slint/src/field.rs) and [select.rs](../../../crates/kit-slint/src/select.rs) before wiring them; controlled ownership requires acknowledgment. Keep incomplete drafts editable and let the native editor own caret, selection, undo and IME.

Use `DecimalValue`, `Money` and explicit currency/locale/rounding rules for numeric domains. Use civil dates, local times and explicit time-zone resolution; never convert an ambiguous local time silently. Renderer positions and byte-channel projections do not replace validated domain values.

Pass `KitTheme` and `ControlSize` into composed controls, or assign `KitThemeContext.current` once per window (re-export the global from the main Slint file to set it from Rust). This is a default context, not a CSS ancestor cascade. A local component’s `theme` affects its known internal controls; application-supplied `@children` need explicit `theme: parent-component.theme` bindings. Keep neighboring regions independent. The built-in light and dark modes are `KitThemes.light` and `.dark`.

The kit ships one Tailwind-inspired presentation with `KitThemes.light` and `.dark`; no additional theme package is needed. It uses Lucide for built-in icon roles. `slint_kit_build::compile` registers `@kit`; `config_with_libraries(HashMap<String, PathBuf>)` preserves application aliases and reserves `kit`. Inter is optional: import `{ InterFont } from "@kit/fonts/inter.slint"` and set the Window's `default-font-family: InterFont.family`. Importing components alone does not embed Inter. The default themes do embed JetBrains Mono Regular (OFL 1.1) for `typography.family-monospace`; this named, registered font keeps code cells portable across native and WASM builds. A custom code font must be registered, monospaced and cover both the code alphabet and mask glyphs. The documentation examples explicitly choose Inter.

Use local presentation properties rather than partial recipe literals. `KitPressable` and buttons expose `corner-radius`, `padding-inline`, `padding-block`, `text-size`, and `text-weight`; text fields expose radius, paddings, text size and leading/trailing icons; `KitCard` and `KitSurface` expose radius and elevation. Preserve intrinsic layout and state resolution. Custom `text-size` values should include `theme.text-scale` when they must follow enlarged-text preferences. A custom theme must provide every field, including `shape`, `elevation`, `focus`, `typography`, `spacing`, `motion`, and `icons`; omitted Slint struct fields silently become zero or transparent. Prefer copying a shipped theme in Rust and changing individual fields.

When composing custom Pressable content, its padding is already applied to the insertion rectangle: center against the content dimensions rather than blindly using the lexical parent’s height or width. See the public KitButton implementation for leading/trailing placement.

Icons are `IconGlyph` values from named functions: `KitGlyphs.close()` for the kit's own glyphs, or `LucideIcons.trash()` after `import { LucideIcons } from "@kit/icons/lucide/all.slint";` (one file per Lucide category also exists; check names there, since Lucide renamed several Feather icons). Paint them with `KitIcon { glyph: ...; }` or `KitIconButton { glyph: ...; accessible-name: "..."; }`; there is no `IconKind` any more. Use `leading-icon` / `trailing-icon` directly on `KitButton` and text fields for ordinary icon-plus-text content, rather than rebuilding a Pressable. Built-in glyphs read `theme.icons` (`KitIconSet`): the built-in theme supplies Lucide. Distinguish these semantic roles from application-selected arbitrary Lucide icons. The [Lucide gallery](https://slint-kit.dev/components/icon/) generates import and integration snippets. Pass `ink` from the surrounding recipe. Icons are decorative by default; set `purpose: IconPurpose.informative` and a `label` only when no nearby text says what the icon means. Size follows `control-size` and `text-scale`; `stroke-mode: IconStroke.absolute` keeps a fixed stroke width. Never edit the generated Lucide files. `KitCard` and `KitHero` accept `LayoutSpace` inset and gap values. Choose `ControlWidth.fill` where the containing layout needs it. Build custom skins on the shared pressable, selection and dialog primitives so pointer, keyboard, focus and disabled behavior remain consistent. For larger screens, inspect the [layout wrappers](../../../crates/kit-slint/ui/layout.slint); for large collections, inspect [data components](../../../crates/kit-slint/ui/data.slint) and preserve model identity and virtualization.

The [display gallery](../../../showcase/ui/base-display.slint) and [its controller](../../../showcase/src/base_display.rs) show alerts, tooltips, toasts and progress. Mount one `KitTooltipLayer` as the final child of each window, provide its viewport size, and disable it while a modal or another screen hides its triggers. `KitTooltip` reports keyboard focus to that window-local layer. Bind `target-focused` to a KitButton/KitPressable’s `focus-visible`, so pointer activation does not open a keyboard hint. Keep toast lifetimes in Rust with `ToastQueue`; map `ToastInteraction.pointer` and `.focus` to separate `ToastPause` sources. Pause/resume must preserve the remaining lifetime.

`KitDrawer` fills its host viewport and anchors its bounded panel to the trailing edge. Mount it at the Window root after scrollable content; register each real field and preserve the exact opener in the focus registry. `KitAccordion` keeps the public single expansion default; the showcase demonstrates multiple expansion by default. Disclosure bodies remain mounted during their reduced-motion-aware height transition.

The [navigation gallery](../../../showcase/ui/base-navigation.slint) demonstrates keyed menus, tabs, notifications and a window-level modal drawer with a focus trap. Host controllers acknowledge activation with stable tokens and generation, own the selected content, skip disabled choices and restore focus when overlays close. Data views render finite viewports over `ModelRc` collections; do not materialize every UI delegate or use row position as identity. `KitVirtualTextView` virtualizes document lines and offers native editing of the selected line; use `KitTextArea` for a whole editable buffer.

`KitNotificationCenter` keeps the read action's slot stable using `mark-read-label` and `read-label`. A dismissed row disables its actions during a 150ms fade, then emits `dismiss-row` with the captured token and generation; reduced motion removes the delay. A changed identity cancels a pending dismissal. Keep popup placement and viewport bounds stable for the open session while the host acknowledges read, preview and dismissal actions.

`KitDateTimeField` and `KitZonedDateTimeField` accept host-localized labels/help and expose `editor-focused` plus `focus-observed(bool)`. Preserve ambiguity context on `ambiguity-started()` and accept only the completed `ambiguity-chosen` action. `KitDatePicker` and `KitCalendar` expose calendar action names; `KitDataTable` and `KitContentList` expose `empty-label`. Labels do not change Rust parsing.

For dates, use `DatePattern` to validate one explicit Gregorian date format and `DateNames` for localized names (`DateNames::french()` is included). Years are 1–9999; numeric input uses ASCII digits. HTML date inputs keep `YYYY-MM-DD`; custom localized entry combines a native `KitTextField` with the same Rust `DatePattern` and `DateConstraints` used by the calendar. Never try competing ambiguous orders or infer a century. `DateConstraints` validates bounds, unavailable weekdays and blackout intervals. `DateRangeController::with_rules` applies `DateRangeRules` to every day of a range and counts civil nights (same-day means zero); a rejected endpoint preserves the draft start. Bind calendar `previous-enabled`/`next-enabled` to `can_navigate`, localize weekday and action labels, and choose any of the seven `WeekStart` values. Read-only calendars browse without selecting. `DateSetController` stores multiple selected `CivilDate` keys; `grid_for_month` lets several calendars share one controller's constraints and selection.

`KitDialogSurface.tab-policy` defaults to `DialogTabPolicy.host`; `.native` lets Slint traverse the mounted controls. The host must disable every background focus target during the modal, including navigation composites/table `interactive: false` (`KitMenu`, `KitMenuBar`, `KitTabs`, `KitAccordion`, `KitCarousel`, `KitBreadcrumbs`, `KitPagination`), and ordinary buttons/fields through `disabled`/`FieldAccess.disabled`. A scrim does not block semantic accessibility actions. Disable custom children separately and restore the exact opener. Only topmost dialogs consume Escape/system Back; busy dialogs do not dismiss. Persistent menus and tabs let system Back reach the overlay. Preserve a menu row’s focus during pointer-down rather than stealing it before activation.

Native file requests are asynchronous and return explicit cancellation, failure or unsupported outcomes. Read [host integration](../../../crates/kit-native/src/lib.rs) and [the showcase integration](../../../showcase/src/form_files.rs) before enabling pickers or drag and drop. Capability depends on the actual backend and target; exposing a DropArea alone does not prove external file-drop support.

Verify the target app builds, exercise its real callbacks and inspect its native rendering in light/dark themes, at 200% text and with keyboard navigation. Native screenshots are visual examples, not screen-reader, IME or external-drop qualification. To build the showcase, follow [Build the showcase](https://slint-kit.dev/guides/getting-started/); cloning is needed for that source application, not for installing the library. Preserve the toolkit's [GPL-3.0-or-later license](../../../LICENSE) for its code and bundled assets, along with dependency notices. Choose and document application-specific assets separately.

`KitAvatar` is static by default. Use `AvatarInteraction.action` with `activated`, `disabled` and a meaningful `accessible-name` for an actionable profile; do not wrap it in another button. `KitOtpField` keeps one native editor in boxed and `FieldDecoration.underlined` presentations. Its `slots` property supplies the default minimum/maximum validation lengths, and `pattern` defaults to `[0-9]*`; use `[A-Za-z0-9]*` and matching Rust `OtpRules::new(OtpLength, HtmlPattern)` for letters and digits. Both use complete-draft HTML patterns and UTF-16 length units. The existing `OtpValue::parse`/`OtpAlphabet` API remains available. For a read-only masked OTP view, set `input-type: InputType.password` and `access: FieldAccess.read-only`. Editable password undo can panic in Slint 1.18.1 after a shorter selection replacement; do not claim that path is qualified. Slint 1.18.1 lacks a public before-insert constraint hook: these lengths validate rather than prevent overflow, and the kit keeps the real editor unchanged. Use `ValidationTiming.on-submit` when a partial code should remain visually neutral until acceptance. Optional formatting uses inherited `display-text` at rest and raw `text` while focused: `FG76657` is a seven-character value with `slots: 7` and pattern `[A-Z]{2}[0-9]{5}`, while `FG‑76657` is its host display projection. Do not count the separator as a slot or normalize pasted separators silently. Badges and stats accept optional decorative `leading-icon` glyphs.
