---
layout: base.njk
locale: en
title: "Themes and accessibility"
translationKey: "guides/themes/"
permalink: "{{ 'guides/themes/' | localeUrl(locale) }}index.html"
---

# Themes and accessibility

This reference documents Slint Kit 0.2.0. For localized labels, composite focus and native modal traversal, see [updates and compatibility](/guides/source-updates/). Upgrade both kit dependencies to `"0.2"` when moving from 0.1.

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

Custom themes provide canvas, surface and raised surface; primary and secondary text; accent and on-accent; danger, success and warning; borders, focus, hover, pressed and disabled colors; scrim; `reduced-motion` and `text-scale`. Supply the full structure rather than leaving semantic colors unset. The [public theme declarations](https://github.com/damione1/slint-kit/blob/main/crates/kit-slint/ui/theme.slint) define the complete contract.

`ButtonIntent` has `primary`, `secondary`, `quiet` and `destructive` variants. Feedback uses semantic severity and tone enums. Choose a state by meaning, then let the theme supply its color. Avoid conveying an error solely through a red border; keep the message and field label available.

## Size and layout

`ControlSize` is `compact`, `medium` or `comfortable`. `ControlWidth.intrinsic` preserves the component's content width; `fill` asks it to use the space allocated by its layout. Most component heights are intrinsic. Set a finite viewport for [virtualized data components](/components/#data), and keep their model identity stable while scrolling.

`LayoutSpace` provides shared inset and gap values. `KitCard` and `KitHero` accept these values. Prefer [KitStack](/components/stack/), [KitInline](/components/inline/) and the other layout wrappers when their shared spacing contract suits the application.

## Text scaling and motion

For 200% text, copy a theme value and set its `text-scale` to `2`. For reduced motion set `reduced-motion` to `true`. These are application theme choices; a host must connect the actual operating-system preferences when needed. Let text and control heights grow instead of clipping them inside fixed-height content.

## Verify the native experience

Check light and dark themes, 200% text, keyboard navigation, focus visibility and disabled actions in the actual application. Preserve the native text editor for caret, selection, undo and IME. Provide meaningful accessible names, particularly for icon buttons, custom pressables and focus targets.

The component captures in this site are native Slint renders using **winit and Skia on macOS**. They show the declared example state with labels adapted to the page’s language. They do not certify a screen reader, IME behavior, external file dropping or another backend. Native device and assistive-technology qualification belongs to the consuming application's target platforms. [Slint accessibility documentation](https://docs.slint.dev/latest/docs/slint/guide/development/best-practices/#accessibility) describes accessibility properties and native inspection practices.
