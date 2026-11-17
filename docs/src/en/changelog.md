---
layout: base.njk
locale: en
title: "Changelog"
translationKey: "changelog/"
permalink: "{{ 'changelog/' | localeUrl(locale) }}index.html"
---

# Changelog

## 0.9.0 — Slint 1.19

Slint Kit 0.9.0 upgrades to Slint 1.19. Use matching 0.9.0 runtime and build packages; [Project setup](/guides/project-setup/) provides the manifest and source-checkout alternatives. Applications staying on an earlier release should keep its documentation and dependencies.

### Slint 1.19 integration

- **Toolchain:** Rust 1.95+ and Slint 1.19 are required. The kit accepts Slint `~1.19.0`; examples pin 1.19.0. Keep the compiler and runtime aligned.
- **Language edition:** the kit uses its own `slint-project.json` with edition 1.19. Applications keep edition 1.18 by default; opt in explicitly with `slint_kit_build::config().with_edition((1, 19))`. Conditional and repeated children now contribute to intrinsic sizing inside the kit, replacing duplicate caption and chip measurements.
- **Accessibility:** text and numeric fields expose one accessible owner and delegate text and selection to the native editor. This removes duplicate accessible nodes; it does not qualify a physical screen reader or IME.
- **Native reveals:** list, tab, menu and field reveals use Slint's scrolling APIs. Collection restoration remains immediate, including before first layout; rapid tab/menu reversals keep the latest item visible. Scroll-area keyboard steps are immediate so repeated input retains its full distance.
- **Reduced motion:** animation blocks use `animate.enabled`; the application `theme.reduced-motion` override remains available. Platform support and timer/tick responsibilities are detailed in [themes and accessibility](/guides/themes/#text-scaling-and-motion).

See the [official Slint 1.19 announcement](https://slint.dev/blog/slint-1.19-released) and [upstream release notes](https://github.com/slint-ui/slint/releases/tag/v1.19.0) for all Slint changes. Slint's additional capabilities are distinct from new Slint Kit components.

### Migration changes

| Previous integration | 0.9.0 integration |
| --- | --- |
| Component sub-imports such as `@kit/button.slint` | Import the public component from `@kit`. |
| `@kit/icons/lucide/all.slint` or category files | Use `@kit-icons/all.slint` or the matching category file. |
| `@kit/fonts/inter.slint` | Use `@kit-fonts/inter.slint`. |
| Editor `kit` alias pointing to `kit.slint` | Point `kit` to `ui/slint-project.json`; configure the separate icon/font aliases. See [editor setup](/guides/editor/). |
| `scroll-to-index` always aligns a row at the top | It reveals the nearest edge and leaves a visible row in place. Wait for animated reveals before reading final offsets; `set-scroll` restores signed offsets immediately. See [controllers](/guides/controllers/). |

Custom theme literals must include the added control/choice colors, inset surface and display typography tokens. See [themes and accessibility](/guides/themes/) and the complete change list below when upgrading a custom theme or an exhaustive enum match.

### Building an application

The [getting-started guide](/guides/getting-started/) explains the Slint UI/Rust application split. [Project setup](/guides/project-setup/#desktop-and-mobile-targets) now links to official desktop packaging, Android, iOS, WebAssembly and embedded deployment guides. Slint's microcontroller support does not imply a `no_std` port of Slint Kit.

All 139 public components now have a complete runnable starter and browser preview, including screen navigation, the sidebar, popovers, menus and dialogs. Dialogs show one-line action results at their full height. Longer content scrolls vertically while the footer remains visible.

For all kit API and behavior changes, including changes beyond the Slint upgrade and earlier versions, read the [complete changelog](https://github.com/damione1/slint-kit/blob/v0.9.0/CHANGELOG.md).
