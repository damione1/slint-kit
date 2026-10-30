---
layout: base.njk
locale: en
title: "Compose a workspace application"
translationKey: "guides/workspace/"
permalink: "{{ 'guides/workspace/' | localeUrl(locale) }}index.html"
---

# Compose a workspace application

The workspace example assembles account settings, a project list and an edit dialog using public Slint Kit components. A single composition works with the minimal theme or the optional Tailwind presentation. The Rust host owns the project data, filtering and in-memory updates.

## Try the complete composition

- [Account settings](/examples/workspace/?theme=tailwind&mode=light&scale=1&scene=account&lang=en): edit the profile, choose a language and toggle notifications.
- [Projects](/examples/workspace/?theme=tailwind&mode=light&scale=1&scene=projects&lang=en): search, filter, select a project and create or edit one.
- [Edit dialog](/examples/workspace/?theme=tailwind&mode=light&scale=1&scene=dialog&lang=en): inspect the fields, validation, save state and focus return.

The demo controls switch theme, light/dark presentation and text scale. Compare the same data and layout, including a local customization, rather than maintaining a second set of components. Changes in the demo are held in memory.

## Run it in a native application

Download [workspace.slint](/assets/examples/workspace.slint) as `ui/app.slint` and [workspace.rs](/assets/examples/workspace.rs) as `src/main.rs`. Add these two files to the same project:

### Cargo.toml

```toml
[package]
name = "workspace-example"
version = "0.1.0"
edition = "2024"

[dependencies]
slint-kit = "=0.4.0"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "=0.4.0"
```

### build.rs

```rust
fn main() {
    slint_kit_build::compile("ui/app.slint").expect("compile workspace");
}
```

Run `cargo run` from the project directory. The source includes the main entry point, form-validation initialization, a model keyed by project identity, filtering and save callbacks. The initial application uses the base theme.

For Tailwind, add `slint-kit-tailwind = "=0.4.0"` to build-dependencies and call `slint_kit_tailwind::compile` in `build.rs`. Import `TailwindThemes` from `@kit-tailwind/theme.slint` in `ui/app.slint`, then set `App.theme` to `TailwindThemes.light` or `.dark`. To choose Inter, also import `{ InterFont }` from `@kit-tailwind/fonts/inter.slint` and set `default-font-family: InterFont.family` on `App`. [Theme setup and local styling](/guides/themes/).

## Keep application ownership clear

`ProductWorkspace.projects` is a controlled projection supplied by the host. Search and filter requests return to Rust; project identity stays independent of row order. Save callbacks validate before applying changes and acknowledge completion. Replace the in-memory store with your own application services while preserving those ownership boundaries.

The composite passes its theme to its children and dialog explicitly. A component’s local radius, padding or icon does not change neighboring regions. The dialog disables background controls while open, preserves the opener and returns focus after dismissal or completion.

When adapting the example, keep the native editors, keyboard handling, field constraints and named actions. Test the real target backend with long labels and enlarged text; the browser demo does not qualify platform IME or screen readers.
