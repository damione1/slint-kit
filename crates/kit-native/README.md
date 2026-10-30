# Slint Kit Native

Optional native host services for [Slint Kit](https://github.com/damione1/slint-kit): asynchronous file and directory pickers with explicit cancellation, failure and unsupported outcomes. External file dropping requires separate window-backend integration.

Applications normally enable these services through the facade:

```toml
[dependencies]
slint-kit = { version = "0.4", features = ["native"] }
```

Access the adapter through `slint_kit::native`. Preserve the UI event loop while awaiting a dialog and reject stale results with typed host-request tickets. Native capabilities depend on the actual platform and backend. Rust 1.92 or later is required.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
