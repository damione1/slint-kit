# Slint Kit Build

Build-time registration of the installed [Slint Kit](https://github.com/damione1/slint-kit) component library. The helper resolves `@kit` from Cargo's installed package and verifies shared Rust/Slint contracts before compilation. No source checkout is required.

```toml
[build-dependencies]
slint-kit-build = "0.3"
```

In `build.rs`:

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile application UI");
}
```

Use [`slint-kit`](https://crates.io/crates/slint-kit) in application dependencies. Rust 1.92 or later and Slint 1.18.1 are required. For custom compiler configuration, start with `slint_kit_build::config()` and preserve its library paths.

Original implementation under [GNU GPL v3.0 or later](LICENSE). Dependencies retain their own licenses.
