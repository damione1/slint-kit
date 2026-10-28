# Slint Kit

Composable native Rust and Slint components with typed behavior, native text editing and customizable themes. Rust owns validated values and stable identity; Slint owns presentation and interaction.

```toml
[dependencies]
slint-kit = "0.2"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.2"
```

Compile your UI in `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Import public components in Slint:

```slint
import { KitButton, KitTextField } from "@kit/kit.slint";
```

Use `slint_kit` in Rust for strict domain values, typed field bindings and stable keyed selection. Enable the optional `native` feature for asynchronous file and directory pickers. Choose a compatible Slint backend/renderer for your target; the manifest above is a desktop example.

Rust 1.92 or later is required. [Source, showcase and bilingual documentation](https://github.com/damione1/slint-kit).

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
