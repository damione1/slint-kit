# Slint Kit

Composable native Rust and Slint components with typed behavior, native text editing and customizable themes. Rust owns validated values and stable identity; Slint owns presentation and interaction.

```toml
[dependencies]
slint-kit = "0.7"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.7"
```

Compile your UI in `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Import public components in Slint:

```slint
import { KitButton, KitTextField } from "@kit";
```

Call `slint_kit::install(&window)` once per window before showing it. It installs validation and the pure calculation services; component state stays local to each instance. See the [forms guide](https://slint-kit.dev/guides/forms/). Phone fields separate a typed canonical `PhoneNumber` string from their configurable display mask and digit count.

Use `slint_kit` in Rust for strict domain values, `Choices<K>`, `FieldSpec<T,E>` and `KeyedModel<K,Row>`. Enable `native` for asynchronous file/directory bindings, or `native-winit` for the explicit winit file-drop bridge. Choose a compatible Slint backend/renderer for your target; the manifest above is a desktop example. Use matching runtime and build releases as described in [Project setup](https://slint-kit.dev/guides/project-setup/).

Appelez `slint_kit::install(&window)` une fois par fenêtre avant son affichage. Les composants possèdent leur état local et les services calculent les projections typées. `Choices<K>`, `FieldSpec<T,E>` et `KeyedModel<K,Row>` gardent valeurs métier et clés stables en Rust. Utilisez des versions correspondantes des paquets runtime et build, comme décrit dans [Configurer le projet](https://slint-kit.dev/fr/guides/project-setup/).

Rust 1.92 or later is required. [Source, showcase and bilingual documentation](https://github.com/damione1/slint-kit).

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
