---
layout: base.njk
locale: en
title: "Native integration"
translationKey: "guides/native/"
permalink: "{{ 'guides/native/' | localeUrl(locale) }}index.html"
---

# Native integration

Enable host services explicitly. The core domain and visual file components do not require a native picker dependency. Add the optional `native` feature only in an application that needs it:

```toml
slint-kit = { version = "={{ project.version }}", features = ["native"] }
```

## Typed picker bindings

Use `FilePickerBinding<W>` or `DirectoryPickerBinding<W>` for the ordinary component flow. Their explicit ports identify application properties without a root naming convention; the binding owns weak-window publication, asynchronous requests and generation checks. Receive accepted resources through `on_selection`.

```rust
let picker = slint_kit::FilePickerBinding::new(&app, slint_kit::FilePickerPorts {
    entries: App::set_entries, state: App::set_request_state, error: App::set_error,
});
picker.set_mode(slint_kit::FileSelectionMode::Single);
picker.set_access(slint_kit::FieldAccess::Editable);
picker.on_selection(|files| println!("{} resources selected", files.len()));
app.on_browse(picker.browse_handler());
app.on_remove(picker.remove_handler());
```

Initialize `slint_kit::install(&app)` before showing the window. See the complete [file picker](/components/file-picker/) and [directory picker](/components/directory-picker/) consumers. Cancellation preserves the accepted resources; destroying the window or superseding a request prevents obsolete results from publishing.

## Requests and outcomes

`slint_kit::native::pick_files(FileRequest)` and `pick_directory(DirectoryRequest)` are asynchronous. Construct and poll the picker on the host UI executor, for example `slint::spawn_local`. Return to a weak window handle when publishing the result. Do not block the UI event loop while awaiting a dialog.

```rust
use slint_kit::{FileRequest, FileSelectionMode, SelectionOutcome};

async fn choose_document() -> SelectionOutcome<Vec<slint_kit::FileHandle>> {
    slint_kit::native::pick_files(FileRequest {
        selection: FileSelectionMode::Single,
        filters: Vec::new(),
        suggested_name: None,
    }).await
}
```

Handle every `SelectionOutcome` variant: `Selected(value)`, `Cancelled`, or `Failed(HostError)`. Cancellation preserves the previously accepted selection. `HostError::Unsupported` is an explicit capability result, not an empty successful selection. Native APIs that cannot distinguish platform failure from cancellation do not fabricate permission errors.

`FileFilter::new(label, extensions)` validates the filter. `FileHandle` and `DirectoryHandle` expose a display name and `FileLocation`; a location may be `Path` or `Uri`. Choosing a resource neither reads its contents nor guarantees that all resources have a filesystem path. File sizes may be unavailable.

## Capabilities and pending requests

Check `slint_kit::native::capabilities()` and project its typed results into [KitFilePicker](/components/file-picker/), [KitDirectoryPicker](/components/directory-picker/) and [KitFileDropZone](/components/file-drop-zone/). Picker support and external drop support are distinct capabilities. The generic native adapter reports drop support as unsupported. For a winit window, opt into `native-winit` and call `FilePickerBinding::install_winit_drop` with the capability and hover ports. This bridge is separate from picker support; exposing a Slint DropArea does not establish OS drop support or device qualification.

`HostRequestController<T>` issues tickets to reject obsolete asynchronous results. Keep pending, failure and cancellation presentation explicit. See the complete [showcase file controller](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_files.rs) for native request handling and the [host adapter](https://github.com/damione1/slint-kit/blob/main/crates/kit-native/src/lib.rs) for platform behavior.

Keep `set_mode` and `set_access` synchronized with the picker when they change. Native drops use these same policies even through the window-wide winit bridge: single selection keeps the last dropped file, multiple selection appends unique locations, and read-only/disabled access ignores the drop.

## Localized built-in text

Set the window’s `KitStrings` once for file browse/remove/status text and step-state names. Every public message property remains an instance override; removal templates accept `{action}` and `{name}` in either order.

`FilePickerBinding::set_error_mapper` and `DirectoryPickerBinding::set_error_mapper` receive the typed `HostError` and return localized `SharedString` text. `HostError::Platform(detail)` retains the platform detail; the default mapper preserves it too. Unsupported capability is represented by the request state and never adds a field error, even with a custom mapper.

The facade bindings provide the Slint window as the native dialog parent. Raw host integrations can call pick_files_with_parent or pick_directory_with_parent with a raw-window-handle provider. Show the window first; parent support depends on the OS/backend (macOS, Windows and Linux XDG). Providers are read before returning the future, preserving weak-window lifetime; native modal behavior still needs target qualification.
