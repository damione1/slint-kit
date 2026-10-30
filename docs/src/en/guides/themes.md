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

Each component defaults to `KitThemeContext.current`, initially `KitThemes.light`. Assign it once for the window's default presentation. An explicit `theme` overrides that component and its internal controls. Children supplied by the application through `@children` keep their own defaults: pass the parent theme explicitly when composing a themed region. To switch the window default from Rust, re-export `KitThemeContext` from the main Slint file, then call `KitThemeContext::get(&window).set_current(theme)`.

`ButtonIntent` has `primary`, `secondary`, `outline`, `quiet` and `destructive` variants. `secondary` is a filled neutral surface, `outline` a transparent bordered control and `quiet` a borderless one. Feedback uses semantic severity and tone enums. Choose a state by meaning, then let the theme supply its color. Avoid conveying an error solely through a red border; keep the message and field label available.

Each `AlertSeverity` takes its own subtle background, border and mark from the `information`, `success`, `warning` and `danger` tokens and their `*-subtle` backgrounds, so alerts stay distinguishable without reading their titles. Kit scroll containers, lists and tables draw their bars with [KitScrollBar](/components/scroll-bar/), which follows the theme. A std-widgets `ScrollView` or `ListView` in application code keeps the Slint style and `Palette.color-scheme`.

## Built-in Tailwind presentation

The kit ships one Tailwind-inspired visual system, with light and dark modes. `KitThemes.light` and `.dark` supply its slate surfaces, indigo actions, semantic feedback colors, typography, spacing, focus rings, elevation and Lucide icons. The standard `slint_kit_build::compile("ui/app.slint")` integration registers everything needed for the components.

```slint
import { KitThemes, KitThemeContext, KitButton } from "@kit/kit.slint";

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    in property <bool> dark;
    init => { KitThemeContext.current = root.dark ? KitThemes.dark : KitThemes.light; }
    changed dark => { KitThemeContext.current = root.dark ? KitThemes.dark : KitThemes.light; }
    background: KitThemeContext.current.canvas;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        KitButton { text: "Continue"; }
    }
}
```

**Inter is an optional font.** Import `{ InterFont } from "@kit/fonts/inter.slint"` and set `default-font-family: InterFont.family` on the window to use it, as the documentation examples do. Without this import, the theme uses the application's font. The four static weights total about 1.6 MB of source files; embedded applications can choose a font appropriate to the device.

For application library aliases, start with `slint_kit_build::config_with_libraries(...)`, then use `slint_build::compile_with_config`. The helper preserves application aliases and reserves `kit`. Component height stays intrinsic: content and padding can require more room than the nominal size.

The palette derives from Tailwind CSS under MIT, Inter is under the SIL Open Font License and Lucide carries ISC/MIT notices. Keep the [third-party notices](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md). Shadow support depends on the renderer; borders preserve surface boundaries when shadows are unavailable.

## Local changes and composed regions

Use the theme for shared identity, `control-size` for density, and local properties for an intentional exception. Their defaults remain bound to the active recipe; changing the corner radius or padding keeps hover, focus, disabled and validation behavior.

| Component | Local presentation |
| --- | --- |
| `KitButton`, `KitIconButton`, `KitPressable` | `corner-radius`, `padding-inline`, `padding-block`, `text-size`, `text-weight` |
| `KitTextField` and its derived field types | `corner-radius`, `padding-inline`, `padding-block`, `text-size`, `leading-icon`, `trailing-icon` |
| `KitCard`, `KitSurface` | `corner-radius`, `elevation`; existing inset and gap properties |
| `KitCheckbox`, `KitSwitch`, `KitOption`, `KitNavigationAction` | inherited Pressable settings on their label and surface; indicator geometry still follows the family recipe |
| `KitAlert` | `corner-radius`, `inset` (length); severity keeps resolving its colors |

Composite groups, selects, tabs and menus retain their theme, size and composition contracts; the table does not imply the same local properties exist on every container. Field labels and helper/error messages keep their typography roles rather than inheriting the editor’s `text-size`.

```slint
import { KitButton, ButtonIntent, KitThemeContext } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

component ExportAction inherits KitButton {
    text: "Export";
    intent: ButtonIntent.primary;
    leading-icon: LucideIcons.download();
    corner-radius: 10px;
    padding-inline: KitThemeContext.current.spacing.unit * 5;
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        ExportAction { }
    }
}
```

`text-size` is a rendered length. Its default includes the theme's text scale; bind a custom text size to `theme.text-scale` too when it must follow that preference. A full `recipe` replacement is an advanced override: preserve every field and the state logic it owns instead of constructing a partial struct literal.

A local region needs explicit bindings for application-supplied children:

```slint
import { KitCard, KitButton, KitThemes } from "@kit/kit.slint";

component DarkPanel inherits KitCard {
    theme: KitThemes.dark;
    KitButton { theme: root.theme; text: "Continue"; }
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        DarkPanel { }
    }
}
```

A neighboring control without an explicit theme still uses the window context. This is an explicit composition contract, not a CSS ancestor cascade.

## Icons

`KitIcon` and `KitIconButton` paint an `IconGlyph`. The kit's own glyphs are `KitGlyphs` (`KitGlyphs.check()`, `KitGlyphs.chevron-down()`…). Every [Lucide](https://lucide.dev) 1.64.0 icon is also available: `@kit/icons/lucide/all.slint` exports `LucideIcons`, and each Lucide category has its own file with the same glyphs, for example `LucideArrows` in `@kit/icons/lucide/arrows.slint`. Each icon is a named function, so a misspelt name fails at compile time, and only the icons an application calls reach its binary.

[Browse all Lucide icons and copy their integration](/components/icon/).

```slint
import { KitIcon, KitIconButton, IconPurpose } from "@kit/kit.slint";
import { LucideIcons } from "@kit/icons/lucide/all.slint";

component Toolbar inherits HorizontalLayout {
    spacing: 8px;
    KitIconButton { glyph: LucideIcons.trash(); accessible-name: "Delete"; }
    KitIcon { glyph: LucideIcons.cloud-check(); purpose: IconPurpose.informative; label: "Saved"; }
}

export component Example inherits Window {
    preferred-width: 420px;
    preferred-height: 180px;
    VerticalLayout {
        padding: 24px;
        alignment: start;
        Toolbar { }
    }
}
```

Icon size follows `control-size` (14, 16 or 20 px) multiplied by the theme's `text-scale`. Icons inside controls use their host's size: `KitIconButton`, field actions, select and combobox chevrons and check marks, and number stepper buttons; a field's clear or reveal button grows with its icon at 200% text. The stroke width comes from the theme's `shape.icon-stroke` token, 2 in every shipped theme. With the default `stroke-mode: IconStroke.scaled` the token is in grid units and scales with the icon, as in SVG: Lucide's 2 units on 24 draw 1.33 px at 16 px and 2.67 px at 200% text. With `IconStroke.absolute` the stroke stays 2 px at every size. An `IconGlyph` has a `stroke` layer and a `fill` layer (empty for most icons): a few Lucide icons, such as `tag` or `palette`, have filled dots, painted with the same `ink`. Pass `ink` from the surrounding recipe. An icon is decorative by default and stays out of the accessibility tree; when it carries meaning that no nearby text gives, set `purpose: IconPurpose.informative` and a `label`, and it is announced as an image.

Built-in control icons read semantic roles from `theme.icons`, a `KitIconSet`. The built-in theme supplies Lucide glyphs for those roles. Override a role in a copied theme to change its consumers, or use a control’s explicit glyph property where available. Selects, comboboxes, disclosure rows and sorted table headers turn their chevron when they open, close or change direction, over the theme's `motion.fast` duration or at once with `reduced-motion`. At rest the chevron is drawn unrotated, so it points the right way under every renderer. Chips mark removal with the close glyph.

Moving from 0.3: `IconKind` no longer exists. Replace `KitIcon { kind: IconKind.close; }` with `KitIcon { glyph: KitGlyphs.close(); }`, and `KitIconButton { icon: IconKind.close; }` with `KitIconButton { glyph: KitGlyphs.close(); }`. The [changelog](https://github.com/damione1/slint-kit/blob/main/CHANGELOG.md) lists every 0.4.0 change.

The Lucide files are generated from a vendored release and never edited by hand. Lucide is under the ISC license, and the icons it derives from Feather are also under the MIT license; both notices are in [THIRD-PARTY-NOTICES.md](https://github.com/damione1/slint-kit/blob/main/THIRD-PARTY-NOTICES.md) and ship with the crate.

## Size and layout

`ControlSize` is `compact`, `medium` or `comfortable`. `ControlWidth.intrinsic` preserves the component's content width; `fill` asks it to use the space allocated by its layout. Most component heights are intrinsic. Set a finite viewport for [virtualized data components](/components/#data), and keep their model identity stable while scrolling.

`LayoutSpace` provides shared inset and gap values. `KitCard` and `KitHero` accept these values. Prefer [KitStack](/components/stack/), [KitInline](/components/inline/) and the other layout wrappers when their shared spacing contract suits the application.

## Text scaling and motion

For 200% text, copy a theme value and set its `text-scale` to `2`. For reduced motion set `reduced-motion` to `true`. These are application theme choices; a host must connect the actual operating-system preferences when needed. Let text and control heights grow instead of clipping them inside fixed-height content.

## Verify the native experience

Check light and dark themes, 200% text, keyboard navigation, focus visibility and disabled actions in the actual application. Preserve the native text editor for caret, selection, undo and IME. Provide meaningful accessible names, particularly for icon buttons, custom pressables and focus targets.

The site includes browser previews and native captures; their captions identify the rendering environment. A rendered example does not certify a screen reader, IME behavior, external file dropping or another backend. Native device and assistive-technology qualification belongs to the consuming application's target platforms. [Slint accessibility documentation](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) describes accessibility properties and native inspection practices.
