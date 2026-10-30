# Changelog

## 0.4.0 — unreleased

0.4.0 changes the theme and icon contracts. The Slint compiler reports every site that must change, so upgrade by following its errors. This list names each break.

### Theme

- **`KitTheme`** grows typed token groups: `shape` (`KitShape`: radii by role, border width, icon stroke), `elevation` (`KitElevation` of `KitShadow { key, ambient }`), `focus` (`KitFocusRing`), `typography` (`KitTypography`), `spacing` (`KitSpacing` with per-`ControlSize` `KitControlDensity`) and `motion` (`KitMotion`). It also gains these colors: `placeholder`, `accent-hover`, `accent-pressed`, `accent-subtle`, `selection`, `on-selection`, `danger-hover`, `danger-pressed`, `danger-subtle`, `success-subtle`, `warning-subtle`, `information`, `information-subtle` and `border-strong`. Write a custom theme as a complete literal: Slint sets an omitted field to zero or transparent, without a warning.
- The flat **`focus-ring`** color is removed; use `focus.color`.
- **`KitThemeContext`** is a new global. Every component's `theme` now defaults to `KitThemeContext.current` instead of `KitThemes.light`. Assign it once, or from Rust after re-exporting `KitThemeContext`, so a component you did not pass a theme no longer renders light in a dark application.
- **`KitMetrics`**: `label-gap`, `help-gap`, `field-gap`, `section-gap` and `footer-gap` become functions of the theme. `font-size(role, scale)` becomes `font-size(theme, role)`, and `control-height(size, scale)` becomes `control-height(theme, size)`. `KitLayoutMetrics.space` now takes the theme. New: `density`, `inline-gap`, `row-height` and `icon-size`.
- **`ButtonRecipe`**: `focus-color`, `focus-width` and `focus-offset` are replaced by `focus: KitFocusRing`. It also gains `busy-mark-size`, `busy-mark-stroke`, `font-weight` (button labels use `typography.weight-medium`) and `elevation` with its `shadow` layer: primary, secondary and destructive buttons lift by `elevation.control` (`xs` in the Tailwind themes, none in the defaults), painted by a child so the shadow never changes a button's measured height.
- **`OptionRecipe`** drops `active-border` and `border-width` and gains `pressed-fill`, `selected-fill` and `selected-foreground`. Active and selected rows of lists, menus and tabs take the `accent-subtle` tint, and the active row paints the theme's inset `FocusOutline` instead of a 1 px border. Menu and navigation rows read the recipe's label color and size; `KitOption` now uses the quiet button geometry.
- **`KitThemes.choice-recipe(theme)`** becomes **`choice-recipe(theme, size)`**: checkbox and radio boxes are 16, 20 or 24 px by `ControlSize`, and switch, slider and marks scale with them; the box corner is at most a quarter of the box, and choice groups nest `radius-control` inside their padding (11 px in the default themes, was 10). `KitSlider` and `KitRangeSlider` gain `control-size`.
- Field labels, file and folder picker labels and data-table headers use `typography.weight-medium` instead of semibold. `KitIconButton` uses `radius-control` (was 6 px); the data table reads its surface recipe (container radius and shadow); badges, skeletons and tooltips use `radius-control`, the hero the container radius, and tooltips the popover shadow.
- **`TailwindThemes.dark`**: `surface-raised` is slate-900 (was slate-800), so hover, pressed and `border-subtle` stay distinct on popovers and dialogs. `on-accent` is slate-950 (was indigo-950), raising labels on the indigo-400 accent from 5.4:1 to 6.8:1.
- **`ButtonIntent.outline`** is added between `secondary` and `quiet`: a transparent fill with the strong border. Rust code matching `ButtonIntent` exhaustively must handle it.
- `SurfaceLevel` (now with `overlay` and `popover`) and `AlertSeverity` are declared in `theme.slint`. They are still re-exported from their former files.
- Every component family reads recipes from `KitThemes` (button, field, trigger, choice, option, surface, alert, table row, scroll bar, icon) instead of literals. `KitDialogPanel` and `KitSelectSurface` paint a stacked shadow under a transparent host, so a `background` set on the instance is now hidden; `border-radius` still applies.
- New opt-in **`TailwindThemes`** (`light`, `dark`) in `@kit/themes/tailwind.slint`, with Inter 4.1 embedded in four static weights (about 1.6 MB). Nothing is embedded unless the file is imported.

### Icons

- **`IconKind` is removed**, along with its single ternary chain. Glyphs are `IconGlyph { stroke, fill, viewbox }` values returned by named functions: the kit's 28 original glyphs are in **`KitGlyphs`** (`IconKind.chevron-down` becomes `KitGlyphs.chevron-down()`), and all 1,999 Lucide 1.64.0 icons are in `@kit/icons/lucide/all.slint` (`LucideIcons`) and in one file per category. An icon you do not call adds nothing to the binary.
- **`KitIcon`**: `kind` becomes `glyph`, which has no default. `KitIcon` now inherits `Rectangle` instead of `Path`, so the `Path` properties are no longer public. New properties: `control-size` (14, 16 or 20 px, multiplied by `text-scale`), `purpose: IconPurpose` with `label` (an informative icon is announced as an image), `stroke-mode: IconStroke` (`scaled` or `absolute`), and the read-only `stroke-width`.
- **`KitIconButton.icon`** becomes **`glyph: IconGlyph`**. Icons inside controls follow their host's `control-size`: icon buttons, field actions, select and combobox chevrons and checks, stepper buttons, disclosure, breadcrumb and notification icons. A field's clear or reveal button grows with its icon, so it still fits at 200% text.
- An information `KitAlert` draws `KitGlyphs.information()` instead of the plus glyph.
- `IconRecipe` becomes `{ size, color, disabled-color, stroke }`. `KitThemes.icon-recipe` takes a `ControlSize`, and the new `KitThemes.icon-stroke-width(recipe, glyph, mode, extent)` computes the stroke.
- The kit glyphs move to Lucide's 24-unit grid, with round caps and joins and the theme's `shape.icon-stroke` token (2). At 16 px they now draw a 1.33 px stroke, where they drew 1.7 px before.
- Selects, comboboxes, disclosure rows and sorted table headers turn their chevron over `motion.fast`, or instantly with `reduced-motion`. At rest each draws the glyph of its state unrotated, so renderers that do not rotate paths still point the right way.
- **`KitChip`** inherits `KitPressable` instead of `KitButton`: it no longer has a `text` property, and it marks removal with the close glyph instead of a `" ×"` suffix.

### Controls

- **`KitSelect.disabled: bool`** becomes **`access: FieldAccess`**, and `KitSelect` gains `validation`. A read-only select keeps focus but never opens. `KitSelectTrigger` gains `access` and `validation` and no longer reads `intent`.
- **`KitScrollBar`** is new: a themed scroll bar that replaces the std-widgets bars inside the kit. **`KitScrollArea`** now inherits `Rectangle`. It keeps `theme`, `gap`, `inset`, `enabled`, `vertical-scrollbar-policy`, `mouse-drag-pan-enabled`, the `visible-*` and content sizes, `content-y`, `scrolled`, `reveal()` and `scroll-to-start()`. The rest of the std `ScrollView` API is gone (`content-x`, the horizontal policy and the deprecated `viewport-*`).
- `KitPressable` gains `focus-indicated` and `focus-ring`, and `take-focus()` does nothing while it is disabled. `KitInputGroup` gains `out hovered`.
- New: `theme` on `KitField`, `KitFieldGroup`, `KitInputGroupStart`/`End`, `KitContainer`, `KitStack`, `KitInline`, `KitWrap`, `KitGrid`, `KitScrollArea`, `KitButtonGroup`, `KitCardBody` and `KitCardFooter`; `palette` on `KitColorField`; `failed-label` on `KitFilePicker` and `KitDirectoryPicker`.
- Segmented-control and toggle-group labels use the button recipe's `font-weight` (medium). Disabled checkboxes, radios and switches draw their mark's edge in `disabled-text`, so the mark stays visible on the disabled row fill; it used to vanish wherever `border-subtle` equalled `disabled-fill`, as in `TailwindThemes.dark`.
- States that used to render nothing now have a look: read-only, hover, checking (with a spinner) and invalid fields, distinct alert tones, the selected tab's underline, the selected palette swatch and a disabled drop zone.

### Build and notices

- `rust-toolchain.toml` pins Rust **1.99.1** for development and CI. The supported minimum (`rust-version`) stays 1.92, and Slint stays at `=1.18.1`.
- `THIRD-PARTY-NOTICES.md` (Tailwind CSS MIT, Inter OFL 1.1, Lucide ISC and Feather MIT) ships with `slint-kit` and `slint-kit-slint`.
- `cargo run -p xtask -- generate-icons` regenerates the Lucide files from `vendor/lucide/<version>/`. `xtask check` rejects stale icon files and, through `lint-tokens`, any literal in the kit sources that is not a theme token or listed in `xtask/token-allowlist.txt`.
