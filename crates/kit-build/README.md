# Slint Kit Build

Build-time registration of the installed [Slint Kit](https://github.com/damione1/slint-kit) component library. The helper resolves `@kit` from Cargo's installed package and verifies shared Rust/Slint contracts before compilation. No source checkout is required.

```toml
[build-dependencies]
slint-kit-build = "0.5"
```

In `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Use [`slint-kit`](https://crates.io/crates/slint-kit) in application dependencies. Rust 1.92 or later and Slint 1.18.1 are required. For custom compiler configuration, start with `slint_kit_build::config()` and preserve its library paths.

## Editor imports

The Slint language server does not read `build.rs`. For a crates.io consumer, run `cargo vendor --locked target/slint-editor/vendor` from the application workspace root, then configure VS Code's `slint.libraryPaths` with `{"kit": "target/slint-editor/vendor/slint-kit-slint/ui"}`, or pass `-L kit=target/slint-editor/vendor/slint-kit-slint/ui` to `slint-lsp` in Zed. These paths assume one kit version and Cargo's default vendor directory naming. Vendoring copies all remote dependencies; rerun it after updating the lockfile or cleaning `target/`.

For complete editor settings, local path dependencies and troubleshooting, see [Editor setup](https://slint-kit.dev/guides/editor/) ([français](https://slint-kit.dev/fr/guides/editor/)).

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
