# Slint Kit Components

Public Slint components, model projections and weak-window Rust adapters for [Slint Kit](https://github.com/damione1/slint-kit). The package includes the complete `ui/` library and its generated shared contracts.

Applications normally use the [`slint-kit` facade](https://crates.io/crates/slint-kit) and [`slint-kit-build`](https://crates.io/crates/slint-kit-build). The build helper registers the installed library as `@kit`:

```slint
import { KitButton, KitTextField } from "@kit/kit.slint";
```

Rust 1.92 or later and Slint 1.18.1 are required. The default `adapter` feature exposes Rust bindings; disabling it leaves the installed Slint library available to build tools without a UI runtime.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
