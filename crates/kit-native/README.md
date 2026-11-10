# Slint Kit Native

The optional native host layer for [Slint Kit](https://slint-kit.dev/guides/why-slint-kit/), the Rust and Slint UI component library. It provides asynchronous file and directory pickers with typed selected, cancelled and failed outcomes. Unsupported capabilities are explicit rather than presented as successful empty results.

The facade's weak-window bindings reject stale request results. File dialogs are desktop services; Android/iOS return `SelectionOutcome::Failed(HostError::Unsupported)`. External file dropping requires the separate window-backend bridge. See [native integration](https://slint-kit.dev/guides/native/) ([français](https://slint-kit.dev/fr/guides/native/)).

Applications normally enable these services through the facade:

```toml
[dependencies]
slint-kit = { version = "0.7", features = ["native"] }
```

Access the adapter through `slint_kit::native`. Preserve the UI event loop while awaiting a dialog and reject stale results with typed host-request tickets. Native capabilities depend on the actual platform and backend. Rust 1.92 or later is required.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

## Slint Kit licensing

The base kit components are **forever free for open-source use**, under GPL-3.0-or-later. A commercial license for proprietary applications is coming, on request; **Slint Kit Pro** advanced components are also coming. [Contact the author](https://github.com/damione1/slint-kit/discussions) for availability.

Use `pick_files_with_parent(request, &provider)` and `pick_directory_with_parent(request, &provider)` when the picker belongs to a window. Providers implement `DialogParent` through the raw-window-handle traits. Their handles are copied before returning the future, which retains no provider reference. Facade picker bindings supply the visible Slint window automatically. Parent support depends on the platform/backend; qualify native modal behavior on the target.
