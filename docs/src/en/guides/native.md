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
slint-kit = { version = "0.5", features = ["native"] }
```

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

Check `slint_kit::native::capabilities()` and project its typed results into [KitFilePicker](/components/file-picker/), [KitDirectoryPicker](/components/directory-picker/) and [KitFileDropZone](/components/file-drop-zone/). Picker support and external drop support are distinct capabilities. The provided native adapter reports drop support as unsupported; the window backend must supply that integration separately.

`HostRequestController<T>` issues tickets to reject obsolete asynchronous results. Keep pending, failure and cancellation presentation explicit. See the complete [showcase file controller](https://github.com/damione1/slint-kit/blob/main/showcase/src/form_files.rs) for native request handling and the [host adapter](https://github.com/damione1/slint-kit/blob/main/crates/kit-native/src/lib.rs) for platform behavior.
