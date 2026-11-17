# Slint Kit Core

The domain and interaction engine behind [Slint Kit](https://slint-kit.dev/guides/why-slint-kit/), a Rust and Slint UI component library. It supplies exact decimal and money arithmetic, HTML-style form constraints, typed field state, civil dates, stable-key selection, focus, overlays and generation guards. This crate has no UI backend.

These shared rules let the UI and Rust business boundary validate the same candidate without replacing native editing. Applications normally use the `slint-kit` facade, which re-exports the domain types. See [typed form validation](https://slint-kit.dev/guides/forms/) and [stable collection identity](https://slint-kit.dev/guides/controllers/) ([français](https://slint-kit.dev/fr/guides/why-slint-kit/)).

These sources prepare the unpublished 0.9.0 release. Use matching `develop` packages until publication; use the corresponding release documentation for published packages.

```toml
[dependencies]
slint-kit-core = { git = "https://github.com/damione1/slint-kit", branch = "develop" }
```

For applications, start with the [`slint-kit` facade](https://crates.io/crates/slint-kit) and [`slint-kit-build`](https://crates.io/crates/slint-kit-build). Rust 1.95 or later is required.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.

## Slint Kit licensing

The base kit components are **forever free for open-source use**, under GPL-3.0-or-later. A commercial license for proprietary applications is coming, on request; **Slint Kit Pro** advanced components are also coming. [Contact the author](https://github.com/damione1/slint-kit/discussions) for availability.
