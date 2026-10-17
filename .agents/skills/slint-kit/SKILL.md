---
name: slint-kit
description: Build native Rust and Slint applications with this Slint Kit checkout, using its public components, typed controllers, themes and build integration. Use when consuming Slint Kit or adapting an existing app to it.
---

# Consume Slint Kit

Resolve the checkout from this skill's location. Read the target application's existing dependency and build configuration before integrating it. These crates are consumed from source; do not assume a crates.io release exists.

## Wire the consumer

For an application beside a `slint-kit` checkout, merge these dependencies into its `Cargo.toml`; adjust the paths to the actual checkout:

```toml
[dependencies]
kit = { path = "../slint-kit/crates/kit" }
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
kit-build = { path = "../slint-kit/crates/kit-build" }
```

Use edition 2024 and Rust 1.92 or later. Preserve the app's chosen backend and renderer when compatible; the manifest above is a desktop example. Add `features = ["native"]` to `kit` only when host services are needed.

In `build.rs`, compile through `kit-build` so the `@kit` library and contract verification remain configured:

```rust
fn main() {
    kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

For custom Slint compiler configuration, start with `kit_build::config()` and preserve its library paths. In Rust, use `slint::include_modules!()` as usual. A minimal `ui/app.slint`:

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

## Choose and compose

Read [the public exports](../../../crates/kit-slint/ui/kit.slint) to find current component names, then read the relevant source for properties and typed callback signatures. The public files are the maintained API reference; do not guess names from a web kit. [The showcase](../../../showcase/ui/forms-gallery.slint) and its [Rust field bindings](../../../showcase/src/form_fields.rs) demonstrate a complete consumer.

Keep accepted application values in Rust. `kit` reexports the domain types and adapters: use enums for policies and states, `FieldBinding<T, E>` for typed draft/commit handling, and `SelectBinding` or `bind_select!` for stable keyed selection. Read [field.rs](../../../crates/kit-slint/src/field.rs) and [select.rs](../../../crates/kit-slint/src/select.rs) before wiring them; controlled ownership requires acknowledgment. Keep incomplete drafts editable and let the native editor own caret, selection, undo and IME.

Use `DecimalValue`, `Money` and explicit currency/locale/rounding rules for numeric domains. Use civil dates, local times and explicit time-zone resolution; never convert an ambiguous local time silently. Renderer positions and byte-channel projections do not replace validated domain values.

Pass `KitTheme` and `ControlSize` into composed controls. Choose `ControlWidth.fill` where the containing layout needs it. Build custom skins on the shared pressable, selection and dialog primitives so pointer, keyboard, focus and disabled behavior remain consistent. For larger screens, inspect the [layout wrappers](../../../crates/kit-slint/ui/layout.slint); for large collections, inspect [data components](../../../crates/kit-slint/ui/data.slint) and preserve model identity and virtualization.

The [display gallery](../../../showcase/ui/base-display.slint) and [its controller](../../../showcase/src/base_display.rs) show alerts, tooltips, toasts and progress. Mount one `KitTooltipLayer` as the final child of each window, provide its viewport size, and disable it while a modal or another screen hides its triggers. `KitTooltip` reports keyboard focus to that window-local layer. Keep toast lifetimes in Rust with `ToastQueue`; map `ToastInteraction.pointer` and `.focus` to separate `ToastPause` sources. Pause/resume must preserve the remaining lifetime.

The [navigation gallery](../../../showcase/ui/base-navigation.slint) demonstrates keyed menus, tabs, notifications and an actual popup drawer with a focus trap. Host controllers acknowledge activation with stable tokens and generation, own the selected content, skip disabled choices and restore focus when overlays close. Data views render finite viewports over `ModelRc` collections; do not materialize every UI delegate or use row position as identity. `KitVirtualTextView` virtualizes document lines and offers native editing of the selected line; use `KitTextArea` for a whole editable buffer.

Native file requests are asynchronous and return explicit cancellation, failure or unsupported outcomes. Read [host integration](../../../crates/kit-native/src/lib.rs) and [the showcase integration](../../../showcase/src/form_files.rs) before enabling pickers or drag and drop. Capability depends on the actual backend and target; exposing a DropArea alone does not prove external file-drop support.

Verify the target app builds, exercise its real callbacks and inspect its native rendering. For toolkit changes, use the checks in [AGENTS.md](../../../AGENTS.md). Preserve the toolkit's [GPL-3.0-or-later license](../../../LICENSE) for its code and bundled assets, along with dependency notices. Choose and document application-specific assets separately.
