---
name: slint-kit
description: Build native Rust and Slint applications with Slint Kit, using its public components, typed controllers, themes and build integration. Use when consuming Slint Kit or adapting an existing app to it.
---

# Consume Slint Kit

Read the target application’s dependency and build configuration before integrating it. Use the published `slint-kit` and `slint-kit-build` packages for application integration; a toolkit checkout is optional. The current release is `0.2.0`, with the compatible dependency range `"0.2"` used below. Keep the runtime and build packages on matching releases and Slint pinned to their supported version.

## Find documentation and examples

The public documentation is [English at the site root](https://slint-kit.damiengoehrig.ca/) and [French under `/fr/`](https://slint-kit.damiengoehrig.ca/fr/). Start with the relevant guide:

- [Getting started](https://slint-kit.damiengoehrig.ca/guides/getting-started/): a complete registry-based application and desktop prerequisites.
- [Component catalogue](https://slint-kit.damiengoehrig.ca/components/): component screenshots and complete Slint examples.
- [Typed controller application](https://slint-kit.damiengoehrig.ca/guides/typed-example/): downloadable Cargo, build, Rust and Slint files, with callbacks connected.
- [Controllers](https://slint-kit.damiengoehrig.ca/guides/controllers/), [themes and accessibility](https://slint-kit.damiengoehrig.ca/guides/themes/) and [native integration](https://slint-kit.damiengoehrig.ca/guides/native/): domain ownership, presentation and host services.

For a registry consumer, inspect the source of its resolved package version when exact signatures matter; `cargo metadata --format-version 1` reports package manifest paths. The relative source links below are shortcuts when this skill lives in a toolkit checkout. If they are unavailable, use the installed package sources or the [versioned repository](https://github.com/damione1/slint-kit/tree/v0.2.0) for release 0.2.0. The component reference and examples document 0.2.0, including temporal/empty-state labels, composite focus signals, `DialogTabPolicy`, and menu/table `interactive`. Keep both kit dependencies on the same `"0.2"` compatibility range.

## Wire the consumer

Merge these registry dependencies into the application’s `Cargo.toml`:

```toml
[dependencies]
slint-kit = "0.2"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.2"
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
    app.on_save(|name| println!("Draft submitted: {name}"));
    app.run()
}
```

Run `cargo run`. The example logs the draft; validate it before accepting it into domain state. Component catalogue examples export `Example` instead of `App`: use the `Example` entry point from the getting-started guide and remove callbacks from the previous example. Screenshots show the declared state; use the typed application guide for a complete interactive controller.

For local toolkit development, use `slint-kit = { path = "../slint-kit/crates/kit" }` and `slint-kit-build = { path = "../slint-kit/crates/kit-build" }` with paths adjusted to the checkout. The library files are bundled in the registry packages; consumers do not need a checkout.

## Choose and compose

Choose components from the catalogue, then verify their public properties and typed callbacks against the resolved version. Read [the public exports](../../../crates/kit-slint/ui/kit.slint) to find current component names, then read the relevant source for properties and typed callback signatures. The public files are the maintained API reference; do not guess names from a web kit. [The showcase](../../../showcase/ui/forms-gallery.slint) and its [Rust field bindings](../../../showcase/src/form_fields.rs) demonstrate a complete consumer.

Keep accepted application values in Rust. `slint_kit` reexports the domain types and adapters: use enums for policies and states, `FieldBinding<T, E>` for typed draft/commit handling, and `SelectBinding` or `bind_select!` for stable keyed selection. Read [field.rs](../../../crates/kit-slint/src/field.rs) and [select.rs](../../../crates/kit-slint/src/select.rs) before wiring them; controlled ownership requires acknowledgment. Keep incomplete drafts editable and let the native editor own caret, selection, undo and IME.

Use `DecimalValue`, `Money` and explicit currency/locale/rounding rules for numeric domains. Use civil dates, local times and explicit time-zone resolution; never convert an ambiguous local time silently. Renderer positions and byte-channel projections do not replace validated domain values.

Pass `KitTheme` and `ControlSize` into composed controls. Supply semantic `success` and `warning` colors when composing a custom theme, together with accent, hover, pressed and focus colors. `KitCard` and `KitHero` accept `LayoutSpace` inset and gap values. Choose `ControlWidth.fill` where the containing layout needs it. Build custom skins on the shared pressable, selection and dialog primitives so pointer, keyboard, focus and disabled behavior remain consistent. For larger screens, inspect the [layout wrappers](../../../crates/kit-slint/ui/layout.slint); for large collections, inspect [data components](../../../crates/kit-slint/ui/data.slint) and preserve model identity and virtualization.

The [display gallery](../../../showcase/ui/base-display.slint) and [its controller](../../../showcase/src/base_display.rs) show alerts, tooltips, toasts and progress. Mount one `KitTooltipLayer` as the final child of each window, provide its viewport size, and disable it while a modal or another screen hides its triggers. `KitTooltip` reports keyboard focus to that window-local layer. Keep toast lifetimes in Rust with `ToastQueue`; map `ToastInteraction.pointer` and `.focus` to separate `ToastPause` sources. Pause/resume must preserve the remaining lifetime.

`KitDrawer` fills its host viewport and anchors its bounded panel to the trailing edge. Mount it at the Window root after scrollable content; register each real field and preserve the exact opener in the focus registry. `KitAccordion` keeps the public single expansion default; the showcase demonstrates multiple expansion by default. Disclosure bodies remain mounted during their reduced-motion-aware height transition.

The [navigation gallery](../../../showcase/ui/base-navigation.slint) demonstrates keyed menus, tabs, notifications and a window-level modal drawer with a focus trap. Host controllers acknowledge activation with stable tokens and generation, own the selected content, skip disabled choices and restore focus when overlays close. Data views render finite viewports over `ModelRc` collections; do not materialize every UI delegate or use row position as identity. `KitVirtualTextView` virtualizes document lines and offers native editing of the selected line; use `KitTextArea` for a whole editable buffer.

`KitNotificationCenter` keeps the read action's slot stable using `mark-read-label` and `read-label`. A dismissed row disables its actions during a 150ms fade, then emits `dismiss-row` with the captured token and generation; reduced motion removes the delay. A changed identity cancels a pending dismissal. Keep popup placement and viewport bounds stable for the open session while the host acknowledges read, preview and dismissal actions.

`KitDateTimeField` and `KitZonedDateTimeField` accept host-localized labels/help and expose `editor-focused` plus `focus-observed(bool)`. Preserve ambiguity context on `ambiguity-started()` and accept only the completed `ambiguity-chosen` action. `KitDatePicker` and `KitCalendar` expose calendar action names; `KitDataTable` and `KitContentList` expose `empty-label`. Labels do not change Rust parsing.

`KitDialogSurface.tab-policy` defaults to `DialogTabPolicy.host`; `.native` lets Slint traverse the mounted controls. The host must disable every background focus target during the modal, including menu/table `interactive: false`, and restore the exact opener. Only topmost dialogs consume Escape/system Back; busy dialogs do not dismiss. Persistent menus and tabs let system Back reach the overlay. Preserve a menu row’s focus during pointer-down rather than stealing it before activation.

Native file requests are asynchronous and return explicit cancellation, failure or unsupported outcomes. Read [host integration](../../../crates/kit-native/src/lib.rs) and [the showcase integration](../../../showcase/src/form_files.rs) before enabling pickers or drag and drop. Capability depends on the actual backend and target; exposing a DropArea alone does not prove external file-drop support.

Verify the target app builds, exercise its real callbacks and inspect its native rendering in light/dark themes, at 200% text and with keyboard navigation. Native screenshots are visual examples, not screen-reader, IME or external-drop qualification. To build the showcase, follow [Build the showcase](https://slint-kit.damiengoehrig.ca/guides/getting-started/); cloning is needed for that source application, not for installing the library. For toolkit changes in a checkout, use the checks in [AGENTS.md](../../../AGENTS.md). Preserve the toolkit's [GPL-3.0-or-later license](../../../LICENSE) for its code and bundled assets, along with dependency notices. Choose and document application-specific assets separately.
