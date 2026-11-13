---
layout: base.njk
locale: en
title: "Development loop"
translationKey: "guides/development-loop/"
permalink: "{{ 'guides/development-loop/' | localeUrl(locale) }}index.html"
---

# Development loop: preview, rebuild and build times

A Slint Kit application is a normal Cargo project: `build.rs` compiles your `.slint` files, then `rustc` compiles the generated Rust. A running application does not reload a changed `.slint` file. Choose the loop that matches what you are changing.

## Which loop to use

| You are changing | Use | Needs a Rust rebuild | Shows |
| --- | --- | --- | --- |
| Layout, spacing, theme tokens, declared content | Editor preview (VS Code Live Preview) or `slint-viewer --auto-reload` | No | Static declarations; no Rust services |
| A single component in isolation | `slint-viewer` with `--component` | No | The same static rendering |
| Behavior backed by `slint_kit::install`: filtering, validation, typed values, accepted selections | `cargo run` | Yes | The real application |
| Behavior in a browser | WASM build from `slint-kit new --web` | Yes | Browser rendering; see [Project setup](../project-setup/) |
| Rust logic only | `cargo check`, then `cargo run` | Yes | Compiler diagnostics first |

Keep the editor configured for `@kit` first: see [Editor setup](../editor/).

## Editor preview and `slint-viewer`

The editor preview and `slint-viewer` compile your `.slint` file directly and reload it when it changes. They never run your Rust code or `slint_kit::install`, so they show layout and declared state only. The [editor guide](../editor/#static-previews-and-rust-services) lists what each component family renders without services.

Use a viewer and language server whose version matches the application's Slint version (currently **{{ project.slint }}**). A viewer built for an older Slint release cannot compile the kit's library files.

For a standalone preview of one file, point the viewer at the kit's `ui` directory. The preview file must import `@kit/kit.slint`, because a directory alias resolves suffixed imports:

```sh
slint-viewer --auto-reload -L kit=target/slint-editor/vendor/slint-kit-slint/ui --component App ui/preview.slint
```

Keep `@kit` as the import in your application and use a separate preview entry file, as described in the editor guide. Add `--load-data` to start a preview from saved property values.

## `SLINT_LIVE_PREVIEW` is not supported

Slint can generate Rust that reloads the `.slint` file at run time when the `SLINT_LIVE_PREVIEW` environment variable is set. In Slint {{ project.slint }} that generator omits the library globals that `slint_kit::install` needs ([upstream issue #14812](https://github.com/slint-ui/slint/issues/14812)), so Slint Kit stops the build with an explanation. Do not set the variable, not even to `0` or an empty value. Re-check this page when you upgrade Slint.

## Build times

These numbers come from one machine, so use them to compare steps, not to predict your own times. Each value is a single run, in seconds, of the project created by `slint-kit new` (Skia renderer, the desktop default).

- Machine: Apple silicon, 12 cores, 24 GB of memory, macOS.
- Toolchain: Rust 1.99.1, Slint 1.18.1, Slint Kit 0.7.2 plus the fix below, `cargo build` with the default `dev` profile and `CARGO_BUILD_JOBS=6`.
- Cargo dependencies were already downloaded. The Skia binaries were part of the cold build.

| Step | Time |
| --- | --- |
| Cold build (empty `target/`) | 48 to 54 s |
| Build with no change | 1.6 to 1.8 s |
| Build after editing a `.slint` file | 5.0 to 5.6 s |
| Build after editing a Rust file | 3.1 to 3.5 s |
| `cargo check` after a `.slint` or Rust edit | 1.2 to 3.4 s |
| First `cargo check` after a clean `target/` | 12 to 31 s |

A `.slint` edit costs more than a Rust edit because the build script runs again: it verifies the kit's contracts, compiles your `.slint` files and generates the Rust that `rustc` then compiles. A first build after the build script has run still recompiles the application once, so a second build is the representative "no change" time.

Earlier versions rewrote an internal wrapper file on every build, which made Cargo recompile the application even when nothing had changed. A build with no change took 5.0 to 6.5 s and a Rust edit 5.5 to 8.0 s before the fix. Upgrade to a release that includes it.

Two settings made no measurable difference for this small project, so there is no reason to change them for speed alone:

- `CARGO_INCREMENTAL=1` or `0`: rebuilds stayed within 1 s of each other.
- `CARGO_PROFILE_DEV_DEBUG=0` instead of the default debug information: cold and rebuild times were within the same range.

Times were not compared with a build before library mode, so this page makes no before/after claim for it.

## Settings for your project

- **Iterate with `cargo check`.** It type-checks Rust and runs `build.rs` without producing a binary.
- **Limit parallel jobs when memory is tight.** Large UIs can use a lot of memory when several `rustc` processes run at once. Lower the count with `CARGO_BUILD_JOBS` or `build.jobs` in `.cargo/config.toml`, and avoid running several separate Cargo builds at the same time.
- **Stay on the `dev` profile for everyday UI work.** The measurements above use it. Switch to `--release` only to judge animation and scrolling performance.
- **Keep your `target/` directory.** A cold build is the slowest step by a wide margin, so run `cargo clean` only when you must.
- **Linker.** Linker choice was not measured here. If link time dominates your own rebuilds, measure a faster linker for your platform before adopting it.
