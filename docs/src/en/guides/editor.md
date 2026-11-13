---
layout: base.njk
locale: en
title: "Editor setup"
translationKey: "guides/editor/"
permalink: "{{ 'guides/editor/' | localeUrl(locale) }}index.html"
---

# Resolve `@kit` imports in your editor

`slint-kit-build` registers `@kit` for Cargo compilation. The Slint language server does not read your `build.rs`, so configure its library paths separately to enable diagnostics, completion and navigation for imports such as `@kit`.

Use the Slint extension for your editor and a language server matching the application's Slint version (currently **{{ project.slint }}**).

## Applications using crates.io

Open the application's Cargo workspace root in your editor. After creating `Cargo.lock` with `cargo check`, run this command from that root:

```sh
cargo vendor --locked target/slint-editor/vendor
```

Cargo copies the remote dependencies selected by your lockfile into this project-local directory, including `slint-kit-slint/ui` and its fonts, icons and license notices. No toolkit checkout or machine-specific Cargo cache path is needed. Vendoring copies **all remote dependencies**, so allow extra disk space and time for the first run.

For editor-only use, you can leave the Cargo source-replacement configuration printed by this command unapplied: `build.rs` continues to resolve the same locked kit through Cargo. If you also want builds to use the vendored sources, merge the printed configuration into `.cargo/config.toml` as described in the [Cargo vendoring guide](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html).

The settings below assume one resolved `slint-kit-slint` version and Cargo's default directory naming. If your dependency graph contains several versions, inspect `target/slint-editor/vendor/` and select the directory matching the kit used by your application; Cargo adds version suffixes when needed. Do not use `--versioned-dirs` with the paths below unchanged.

## VS Code

Merge this setting into `.vscode/settings.json` at the workspace root:

```json
{
  "slint.libraryPaths": {
    "kit": "target/slint-editor/vendor/slint-kit-slint/ui/kit.slint"
  }
}
```

The key is `kit`, without `@`; the value points to the `kit.slint` file itself, because the import is `@kit` without a filename. The Slint extension resolves relative library paths against the workspace root. Preserve any other library aliases already in this map. See the [Slint extension's library path setting](https://github.com/slint-ui/slint/blob/v1.18.1/editors/vscode/package.json).

## Zed

Merge these language-server arguments into `.zed/settings.json` at the workspace root:

```json
{
  "lsp": {
    "slint": {
      "binary": {
        "arguments": [
          "-L",
          "kit=target/slint-editor/vendor/slint-kit-slint/ui/kit.slint"
        ]
      }
    }
  }
}
```

Keep existing server arguments and add this `-L` pair. Zed starts the server in the project worktree, so this path is relative to that root. The [Slint Zed extension](https://github.com/slint-ui/slint/blob/v1.18.1/editors/zed/src/slint.rs) forwards `binary.arguments` to `slint-lsp`. Other editors can pass the same arguments with the workspace root as the server's working directory.

## Check and refresh

Restart the Slint language server (or reload the editor), then open a `.slint` file that imports `KitButton` from `@kit`. The import should resolve, and Go to Definition should open the vendored component. Keep compiling with `slint_kit_build::compile` in `build.rs`; editor settings do not configure Cargo compilation.

Rerun the vendoring command after updating `Cargo.lock` or running `cargo clean`, which removes `target/`. Do not edit the vendored files: refresh them from the resolved dependency instead. You can commit the relative editor settings; `target/` remains generated and ignored by Git.

## Applications using a local checkout

Cargo does not vendor local path dependencies. If your application uses `slint-kit` and `slint-kit-build` from a sibling `../slint-kit` checkout, use `../slint-kit/crates/kit-slint/ui/kit.slint` as the VS Code library path, or `kit=../slint-kit/crates/kit-slint/ui/kit.slint` in Zed's `-L` argument. Adjust that relative path to your workspace layout. When opening the Slint Kit repository itself, use `crates/kit-slint/ui/kit.slint`.

If the editor still reports “Cannot find requested import”, check that the configured file is `kit.slint` and exists, the opened workspace matches the path's root, and the selected library version matches your Cargo dependency. An include path alone does not register the `@kit` library alias. See [Slint component libraries](https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/#component-libraries).

## Supplemental imports in Slint 1.18.1

The language server resolves a bare alias as a file and joins a suffixed import onto the configured path. A single alias cannot therefore resolve both `@kit` and `@kit/fonts/inter.slint` or `@kit/icons/lucide/all.slint`. Mapping `kit` to the `ui` directory supports those supplemental imports, but makes bare `@kit` unresolved. The file mapping above prioritizes the component API.

Keep the documented supplemental imports in your application: Cargo compilation through `slint-kit-build` resolves them correctly using its module metadata and directory alias. With the editor file mapping, Slint 1.18.1 can still report unresolved supplemental imports. This is a language-server limitation, not a failure of the Cargo configuration; do not replace the application's root imports with the old `@kit/kit.slint` path.

<h2 id="static-previews-and-rust-services">Static previews and Rust services</h2>

Starting with Slint Kit 0.7.2, VS Code Live Preview and `slint-viewer` display declarative selection options even though no Rust code runs. For a local checkout, the viewer can use a directory alias if the preview file imports `@kit/kit.slint`; use a separate preview entry file and keep the application's public imports as `@kit`. A file alias for `@kit` has the supplemental-import limitation described above.

These previews show layout and declared state; they do not install Rust services. Use `cargo run` with `slint_kit::install(&app)` once per window, or the component page's browser demo, for service-backed behavior.

| Component family | Pure-Slint preview | Runtime service needed for |
| --- | --- | --- |
| Tabs, menus/menu bars, breadcrumbs, carousels | Declared `options`, labels and disabled state; declared single selection | Filtering, reconciliation, navigation and accepted `chosen` values; carousels preview their first source slide |
| Radio and segmented groups | Declared options and single selection | Keyboard selection, validation and generation checks |
| Toggle groups | Declared option labels; explicit `rows` for exact selected marks | Projecting the selected set and accepting toggles |
| Accordion | Declared option headers, initially collapsed; use explicit `rows: [DisclosureRow]` instead of `options` to preview expanded bodies | Expansion-set projection and keyboard navigation |
| Rating | Configured score scale and value; explicit `scores` also work | Score acceptance and navigation |
| Select, combobox, multi-select and tag input | Authored placeholders/text and explicit presentation properties; static option rows where exposed | Selected-label lookup, chips, filtering and popup navigation |
| Calendar, date picker and date range picker | Explicit `days`, `title` and navigation properties | Computing month grids, bounds, ranges and accepted dates |
| Text/numeric, color and time/date-time fields | Authored native text; use `text` or `hex-text` where exposed | Typed formatting/parsing, commit rules, HTML validation and time-zone resolution |
| Buttons, layout, feedback and data display | Declared content and presentation | Only the application actions or models your example uses |

Do not set a service's `ready` property manually: it means its Rust callbacks are installed. Local ownership does not replace installation. Static preview does not qualify keyboard/IME or assistive technology behavior on a target device.

## Rust live-preview limitation in Slint 1.18.1

The `SLINT_LIVE_PREVIEW` environment variable selects a different Rust code generator. That generator omits the shared library global exports and `Global` implementations required by Slint Kit ([upstream issue #14812](https://github.com/slint-ui/slint/issues/14812)). This affects every window using `slint_kit::install`, including windows whose visible controls are unrelated to selection.

Slint Kit 0.7.2 reports this limitation during the build instead of emitting Rust that fails with unresolved globals. Remove `SLINT_LIVE_PREVIEW` from the shell, `build.rs` and development scripts, then rebuild. Setting it to `0` or an empty string still selects Slint's live-preview generator; remove the variable entirely. Enabling the `slint/live-preview` Cargo feature alone does not select it.

For runtime inspection, keep `slint/mcp` and `SLINT_EMIT_DEBUG_INFO=1` with the normal generator. The editor's pure-Slint Live Preview and `slint-viewer` do not need `SLINT_LIVE_PREVIEW`; their static rendering follows the table above.

## Runtime inspection metadata

The build helper follows Slint’s default debug metadata policy. For runtime element inspection with the testing backend or MCP, set `SLINT_EMIT_DEBUG_INFO=1` before building, or pass `slint_kit_build::config().with_debug_info(true)` to `compile_with_config`. Ordinary release builds do not force element names and source locations into the generated application.

For the editing loop, preview options and measured build times, see the [development loop guide](../development-loop/).
