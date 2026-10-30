---
layout: base.njk
locale: en
title: "Migrating to 0.4"
translationKey: "guides/migration-0-4/"
permalink: "{{ 'guides/migration-0-4/' | localeUrl(locale) }}index.html"
---

# Migrating to Slint Kit 0.4

Keep `slint-kit`, its build integration and any presentation pack on matching versions. The component behavior stays in the base kit; the optional Tailwind package supplies presentation and font resources.

## Minimal application

An application using the built-in light or dark theme keeps `slint-kit-build` and `slint_kit_build::compile("ui/app.slint")`. It does not need a Tailwind package or font import.

Custom `KitTheme` literals now include typed token groups and `icons: KitIconSet`. Copy a shipped theme and modify its fields instead of maintaining a partial literal: Slint fills omitted fields with zero or transparent values. Use a component's local presentation properties for a single exception.

## Move the optional Tailwind import

The former `@kit/themes/tailwind.slint` file has moved to `slint-kit-tailwind`. Add the pack to build-dependencies:

```toml
[build-dependencies]
slint-kit-tailwind = "0.4"
```

```rust
fn main() {
    slint_kit_tailwind::compile("ui/app.slint").expect("compile UI");
}
```

Replace the old import with:

```slint
import { TailwindThemes } from "@kit-tailwind/theme.slint";
```

The theme no longer registers Inter implicitly. To keep Inter, import `{ InterFont } from "@kit-tailwind/fonts/inter.slint"` and set the Window's `default-font-family: InterFont.family`. Omit both to use the same theme with the application's font.

For custom library aliases, pass their map to `slint_kit_tailwind::config_with_libraries(...)` and compile through `slint_build::compile_with_config`. The pack checks its version against kit-build and rejects collisions with `kit` or `kit-tailwind` aliases.

## Icons and buttons

`IconKind` is replaced by `IconGlyph` values. Replace `kind: IconKind.close` on `KitIcon` and `icon: IconKind.close` on `KitIconButton` with `glyph: KitGlyphs.close()`. Lucide glyph functions use the same type.

Buttons accept `leading-icon` and `trailing-icon`; ordinary icon-plus-label actions no longer require a custom Pressable. `KitIconButton.intent` now selects its visual intent, consistently with text buttons. Verify the appearance of any existing icon button whose inherited `intent` was set but previously ignored.

Internal icon roles follow `theme.icons`, so a theme change updates built-in chevrons and actions as well as application icons. Custom roles must be valid `IconGlyph` values, not string names. [Browse the icon gallery](/components/icon/).

## Theme scope and local styling

`KitThemeContext.current` supplies the window default. An explicit `theme` overrides a component and its internal controls; it does not inject a theme into arbitrary `@children`. Pass the region's theme explicitly to children composed by your application. Avoid setting the global in a panel's initializer to emulate nested scope.

For local changes, use `corner-radius`, padding and typography properties where exposed, or a surface's `elevation`. Their defaults follow the active recipe. Keep complete recipes for advanced overrides, and preserve their state handling. [Theme and composition examples](/guides/themes/).

Run the application in light and dark themes, with enlarged text, keyboard navigation and disabled/read-only/invalid controls. In particular, check custom icon sizes, text-field affixes and icon-only action labels.
