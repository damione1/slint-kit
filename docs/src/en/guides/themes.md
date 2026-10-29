---
layout: base.njk
locale: en
title: "Themes and accessibility"
translationKey: "guides/themes/"
permalink: "{{ 'guides/themes/' | localeUrl(locale) }}index.html"
---

# Themes and accessibility


A theme is a `KitTheme` value. Pass it to each composed component; the public defaults are `KitThemes.light` and `KitThemes.dark`. Use the same value for the window background and text colors.

```slint
import { KitTheme, KitThemes, KitButton, ControlSize, ControlWidth } from "@kit/kit.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in-out property <KitTheme> theme: KitThemes.dark;
    background: root.theme.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton {
            theme: root.theme;
            text: "Continue";
            control-size: ControlSize.comfortable;
            control-width: ControlWidth.fill;
        }
    }
}
```

## Semantic colors

Custom themes provide canvas, surface and raised surface; primary, secondary and placeholder text; accent, on-accent, accent hover and pressed, subtle accent and text selection; danger with its hover and pressed tones, success, warning and information with their subtle backgrounds; subtle and strong borders, hover, pressed and disabled colors; scrim; `reduced-motion` and `text-scale`. Beyond colors, a theme carries typed token groups: `shape` (radii by role for controls, containers, overlays and pills, plus border widths), `elevation` (a level per role, and a key and an ambient shadow layer per level), `focus` (ring style, width, offset and colors), `typography` (family, weights, sizes), `spacing` (a base unit and the height and paddings of each `ControlSize`) and `motion` (fast, base and slow durations). Slint has no spread syntax and an omitted struct field silently becomes zero or transparent, so write a custom theme as a complete literal, or copy a shipped theme and change fields from Rust. The [public theme declarations](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/ui/theme.slint) define the complete contract.

Every component's `theme` defaults to `KitThemeContext.current`, which starts as `KitThemes.light`. Assign it once to theme a whole window, including components that were not passed a theme; a component's own `theme` property still overrides it for that subtree. To switch it from Rust, re-export the global from the application's main file with `export { KitThemeContext } from "@kit/kit.slint";`, then call `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` has `primary`, `secondary`, `outline`, `quiet` and `destructive` variants. `secondary` is a filled neutral surface, `outline` a transparent bordered control and `quiet` a borderless one. Feedback uses semantic severity and tone enums. Choose a state by meaning, then let the theme supply its color. Avoid conveying an error solely through a red border; keep the message and field label available.

Each `AlertSeverity` takes its own subtle background, border and mark from the `information`, `success`, `warning` and `danger` tokens and their `*-subtle` backgrounds, so alerts stay distinguishable without reading their titles. Kit scroll containers, lists and tables draw their bars with [KitScrollBar](/components/scroll-bar/), which follows the theme. A std-widgets `ScrollView` or `ListView` in application code keeps the Slint style and `Palette.color-scheme`.

## Tailwind-inspired theme

`@kit/themes/tailwind.slint` is an opt-in theme with the look of Tailwind CSS: a reduced slate, indigo, red, green, amber and sky palette, `rounded-md` controls, `rounded-lg` containers and `rounded-xl` overlays, Tailwind shadows, a two-pixel ring with a two-pixel offset, the 36 px default control height and the [Inter](https://rsms.me/inter/) typeface. Import it and assign one of its values; components never import it, so an application that does not import it compiles and embeds nothing from it. There is no Cargo feature to enable.

```slint
import { KitThemeContext, KitButton } from "@kit/kit.slint";
import { TailwindThemes } from "@kit/themes/tailwind.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in property <bool> dark;
    init => { KitThemeContext.current = root.dark ? TailwindThemes.dark : TailwindThemes.light; }
    changed dark => { KitThemeContext.current = root.dark ? TailwindThemes.dark : TailwindThemes.light; }
    background: KitThemeContext.current.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton { text: "Continue"; }
    }
}
```

`TailwindThemes.light` and `TailwindThemes.dark` are complete `KitTheme` values, so the usual rules apply: change fields from Rust, set `text-scale` for 200% text, or pass a value to one subtree's `theme`. Importing the file embeds Inter 4.1 in four static weights (400, 500, 600 and 700), about 1.6 MB of font data in the application binary; its `typography.family` is `"Inter"`. Kit text reads `typography.family`, while the application's own text keeps the window's `default-font-family`.

The palette values come from the Tailwind CSS palette under the MIT license, and Inter is under the SIL Open Font License 1.1; both notices are in [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) and ship with the crate. Slint draws one shadow per element and has no shadow spread: cards and toasts paint the dominant layer of their Tailwind shadow, while dialogs and popovers stack both layers on rectangles inset by each layer's spread. Controls stay flat. Under the software renderer, which draws no shadows, surfaces keep their border.

## Icons

`KitIcon` and `KitIconButton` paint an `IconGlyph`. The kit's own glyphs are `KitGlyphs` (`KitGlyphs.check()`, `KitGlyphs.chevron-down()`…). Every [Lucide](https://lucide.dev) 1.64.0 icon is also available: `@kit/icons/lucide/all.slint` exports `LucideIcons`, and each Lucide category has its own file with the same glyphs, for example `LucideArrows` in `@kit/icons/lucide/arrows.slint`. Each icon is a named function, so a misspelt name fails at compile time, and only the icons an application calls reach its binary.

```slint
import { KitIcon, KitIconButton, IconPurpose } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

export component Toolbar inherits HorizontalLayout {
    spacing: 8px;
    KitIconButton { glyph: LucideIcons.trash(); accessible-name: "Delete"; }
    KitIcon { glyph: LucideIcons.cloud-check(); purpose: IconPurpose.informative; label: "Saved"; }
}
```

Icon size follows `control-size` (14, 16 or 20 px) multiplied by the theme's `text-scale`. The stroke width comes from the theme's `shape.icon-stroke` token, 2 in every shipped theme. With the default `stroke-mode: IconStroke.scaled` the token is in grid units and scales with the icon, as in SVG: Lucide's 2 units on 24 draw 1.33 px at 16 px and 2.67 px at 200% text. With `IconStroke.absolute` the stroke stays 2 px at every size. An `IconGlyph` has a `stroke` layer and a `fill` layer (empty for most icons): a few Lucide icons, such as `tag` or `palette`, have filled dots, painted with the same `ink`. Pass `ink` from the surrounding recipe. An icon is decorative by default and stays out of the accessibility tree; when it carries meaning that no nearby text gives, set `purpose: IconPurpose.informative` and a `label`, and it is announced as an image.

The kit's components use the same glyphs. Selects, comboboxes, disclosure rows and sorted table headers turn their chevron when they open, close or change direction, over the theme's `motion.fast` duration or at once with `reduced-motion`. At rest the chevron is drawn unrotated, so it points the right way under every renderer. Chips mark removal with the close glyph.

Moving from 0.3: `IconKind` no longer exists. Replace `KitIcon { kind: IconKind.close; }` with `KitIcon { glyph: KitGlyphs.close(); }`, and `KitIconButton { icon: IconKind.close; }` with `KitIconButton { glyph: KitGlyphs.close(); }`. The [changelog](https://github.com/damione1/slint-kit/blob/main/CHANGELOG.md) lists every 0.4.0 change.

The Lucide files are generated from a vendored release and never edited by hand. Lucide is under the ISC license, and the icons it derives from Feather are also under the MIT license; both notices are in [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) and ship with the crate.

## Size and layout

`ControlSize` is `compact`, `medium` or `comfortable`. `ControlWidth.intrinsic` preserves the component's content width; `fill` asks it to use the space allocated by its layout. Most component heights are intrinsic. Set a finite viewport for [virtualized data components](/components/#data), and keep their model identity stable while scrolling.

`LayoutSpace` provides shared inset and gap values. `KitCard` and `KitHero` accept these values. Prefer [KitStack](/components/stack/), [KitInline](/components/inline/) and the other layout wrappers when their shared spacing contract suits the application.

## Text scaling and motion

For 200% text, copy a theme value and set its `text-scale` to `2`. For reduced motion set `reduced-motion` to `true`. These are application theme choices; a host must connect the actual operating-system preferences when needed. Let text and control heights grow instead of clipping them inside fixed-height content.

## Verify the native experience

Check light and dark themes, 200% text, keyboard navigation, focus visibility and disabled actions in the actual application. Preserve the native text editor for caret, selection, undo and IME. Provide meaningful accessible names, particularly for icon buttons, custom pressables and focus targets.

The component captures in this site are native Slint renders using **winit and Skia on macOS**. They show the declared example state with labels adapted to the page’s language. They do not certify a screen reader, IME behavior, external file dropping or another backend. Native device and assistive-technology qualification belongs to the consuming application's target platforms. [Slint accessibility documentation](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) describes accessibility properties and native inspection practices.
