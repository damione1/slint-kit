# Slint Kit Build

Build-time registration of the installed [Slint Kit](https://github.com/damione1/slint-kit) component library. The helper resolves `@kit` from Cargo's installed package and verifies shared Rust/Slint contracts before compilation. No source checkout is required.

```toml
[build-dependencies]
slint-kit-build = "0.7"
```

In `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Use [`slint-kit`](https://crates.io/crates/slint-kit) in application dependencies. Rust 1.92 or later and Slint 1.18.1 are required. For custom compiler configuration, start with `slint_kit_build::config()` and pass the result to `slint_kit_build::compile_with_config(path, config)`. Preserve its library paths; calling the low-level Slint compiler directly does not add the window service exports.

Windows requires 0.7.1 or later; 0.7.0 incorrectly escapes canonical source paths. The helpers accept relative, absolute and canonical paths, including spaces and Unicode, while preserving application include paths and library aliases. Paths containing quotes or line breaks cannot be represented in a Slint import and return a diagnostic.

## Editor imports

The Slint language server does not read `build.rs`. For a crates.io consumer, run `cargo vendor --locked target/slint-editor/vendor` from the application workspace root, then configure VS Code's `slint.libraryPaths` with `{"kit": "target/slint-editor/vendor/slint-kit-slint/ui"}`, or pass `-L kit=target/slint-editor/vendor/slint-kit-slint/ui` to `slint-lsp` in Zed. These paths assume one kit version and Cargo's default vendor directory naming. Vendoring copies all remote dependencies; rerun it after updating the lockfile or cleaning `target/`.

For complete editor settings, local path dependencies and troubleshooting, see [Editor setup](https://slint-kit.dev/guides/editor/) ([français](https://slint-kit.dev/fr/guides/editor/)).

## Shared modules and compatibility

Slint Kit pins Slint **1.18.1** and enables its experimental module-build feature inside the build package. Components, structs, enums and globals use the single `@kit` entry point; supplemental font/icon imports retain their paths. Keep the runtime dependency named `slint-kit` so generated references to `slint_kit::ui` resolve. `compile` supplies the window globals needed by `slint_kit::install`; no per-component export list is needed in application UI.

The experimental feature is isolated here rather than exposed as application configuration. If an upstream release removes it, the compatibility fallback is a build-generated Slint entry point that re-exports the application and kit contracts, with typed Rust glue included through `SLINT_INCLUDE_GENERATED`. That fallback preserves the public import/props/callback and per-window `install` contract; it is a migration strategy, not an alternative backend enabled in the current 1.18.1 package.

```rust
let config = slint_kit_build::config().with_style("fluent".into());
slint_kit_build::compile_with_config("ui/app.slint", config)
    .expect("compile application with shared kit services");
```

## Modules partagés et compatibilité

Slint Kit épingle Slint **1.18.1** et active sa compilation expérimentale de modules dans le paquet build. Composants, structs, enums et globals passent par l’unique point d’entrée `@kit` ; les imports supplémentaires de polices/icônes conservent leurs chemins. Gardez la dépendance runtime nommée `slint-kit` pour résoudre `slint_kit::ui`. `compile` fournit les globals nécessaires à `slint_kit::install` sans liste d’exports par composant dans l’application.

Pour personnaliser le compilateur, partez de `config()` puis appelez `compile_with_config(path, config)` : les chemins de bibliothèques et exports de services restent configurés. L’appel direct au compilateur Slint de bas niveau n’ajoute pas ces exports.

Sous Windows, utilisez la 0.7.1 ou une version ultérieure ; la 0.7.0 échappait incorrectement les chemins canonicalisés. Les helpers acceptent les chemins relatifs, absolus et canonicalisés, avec espaces et Unicode, en conservant les chemins d’inclusion et alias de bibliothèques. Les chemins contenant des guillemets ou sauts de ligne ne sont pas représentables dans un import Slint et renvoient un diagnostic.

La fonctionnalité expérimentale reste isolée dans ce paquet. Si une version amont la retire, le repli de compatibilité est un point d’entrée Slint généré qui réexporte l’application et les contrats du kit, avec une liaison Rust typée incluse par `SLINT_INCLUDE_GENERATED`. Ce repli préserve le contrat public import/propriétés/callback et `install` par fenêtre ; il s’agit d’une stratégie de migration, pas d’un autre backend activé dans le paquet 1.18.1 actuel.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
