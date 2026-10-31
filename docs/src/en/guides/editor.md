---
layout: base.njk
locale: en
title: "Editor setup"
translationKey: "guides/editor/"
permalink: "{{ 'guides/editor/' | localeUrl(locale) }}index.html"
---

# Resolve `@kit` imports in your editor

`slint-kit-build` registers `@kit` for Cargo compilation. The Slint language server does not read your `build.rs`, so configure its library paths separately to enable diagnostics, completion and navigation for imports such as `@kit/kit.slint`.

Use the Slint extension for your editor and a language server matching the application's Slint version (currently **1.18.1**).

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
    "kit": "target/slint-editor/vendor/slint-kit-slint/ui"
  }
}
```

The key is `kit`, without `@`; the value points to the directory containing `kit.slint`. The Slint extension resolves relative library paths against the workspace root. Preserve any other library aliases already in this map. See the [Slint extension's library path setting](https://github.com/slint-ui/slint/blob/v1.18.1/editors/vscode/package.json).

## Zed

Merge these language-server arguments into `.zed/settings.json` at the workspace root:

```json
{
  "lsp": {
    "slint": {
      "binary": {
        "arguments": [
          "-L",
          "kit=target/slint-editor/vendor/slint-kit-slint/ui"
        ]
      }
    }
  }
}
```

Keep existing server arguments and add this `-L` pair. Zed starts the server in the project worktree, so this path is relative to that root. The [Slint Zed extension](https://github.com/slint-ui/slint/blob/v1.18.1/editors/zed/src/slint.rs) forwards `binary.arguments` to `slint-lsp`. Other editors can pass the same arguments with the workspace root as the server's working directory.

## Check and refresh

Restart the Slint language server (or reload the editor), then open a `.slint` file that imports `KitButton` from `@kit/kit.slint`. The import should resolve, and Go to Definition should open the vendored component. Keep compiling with `slint_kit_build::compile` in `build.rs`; editor settings do not configure Cargo compilation.

Rerun the vendoring command after updating `Cargo.lock` or running `cargo clean`, which removes `target/`. Do not edit the vendored files: refresh them from the resolved dependency instead. You can commit the relative editor settings; `target/` remains generated and ignored by Git.

## Applications using a local checkout

Cargo does not vendor local path dependencies. If your application uses `slint-kit` and `slint-kit-build` from a sibling `../slint-kit` checkout, use `../slint-kit/crates/kit-slint/ui` as the VS Code library path, or `kit=../slint-kit/crates/kit-slint/ui` in Zed's `-L` argument. Adjust that relative path to your workspace layout. When opening the Slint Kit repository itself, use `crates/kit-slint/ui`.

If the editor still reports “Cannot find requested import”, check that the configured directory contains `kit.slint`, the opened workspace matches the path's root, and the selected library version matches your Cargo dependency. An include path alone does not register the `@kit` library alias. See [Slint component libraries](https://docs.slint.dev/latest/docs/slint/guide/language/coding/file/#component-libraries).
