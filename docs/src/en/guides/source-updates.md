---
layout: base.njk
locale: en
title: "Updates and compatibility"
translationKey: "guides/source-updates/"
permalink: "{{ 'guides/source-updates/' | localeUrl(locale) }}index.html"
---

# Source updates

Slint Kit **0.2.0** is available on crates.io. The component reference, downloadable Slint fixtures and native screenshots document this release. Localized labels, composite editor focus and native modal traversal are included in the published packages; a source checkout is optional.

## Upgrade from 0.1

Update both kit dependencies together. Cargo’s `"0.1"` range does not select 0.2, because minor releases before 1.0 have separate compatibility ranges. Keep Slint pinned to 1.18.1 and use Rust 1.92 or later.

```toml
[dependencies]
slint-kit = "0.2"
slint = { version = "=1.18.1", default-features = false, features = ["std", "compat-1-18", "backend-winit", "renderer-skia", "accessibility"] }

[build-dependencies]
slint-kit-build = "0.2"
```

Keep the [same build.rs and entry point](/guides/getting-started/), then run `cargo check` to resolve and verify the new dependencies. The [typed application](/guides/typed-example/) and [sortable table application](/guides/table-example/) use the 0.2 registry packages. For a source integration, select [tag v0.2.0](https://github.com/damione1/slint-kit/tree/v0.2.0) and use matching local paths for both packages.

## Localize dates and empty states

| Component | Host-supplied strings |
| --- | --- |
| [KitCalendar](/components/calendar/) | `previous-month-label`, `next-month-label`; continue supplying the title, weekdays and day names in the model. |
| [KitDatePicker](/components/date-picker/) | `placeholder`, `description`, `calendar-button-label`, `calendar-accessible-name`, and both month-navigation labels. An empty description hides the help. |
| [KitDateTimeField](/components/date-time-field/) | `date-label`, `time-label`, `date-placeholder`, `date-description`, `time-description`. |
| [KitZonedDateTimeField](/components/zoned-date-time-field/) | The date/time strings above plus `zone-label`, `zone-placeholder`, `zone-description`, `earlier-label`, `later-label`. |
| [KitDataTable](/components/data-table/), [KitContentList](/components/content-list/) | `empty-label`, shown when the model has no rows/items. |

These strings change presentation and accessible names. They do not change date parsing, IANA resolution or the host’s accepted values. For example, a French `AAAA-MM-JJ` placeholder still describes the existing ISO year-month-day order. Translate [KitDateField](/components/date-field/) and [KitTimeField](/components/time-field/) help through their existing inherited `description` property.

## Observe composite editor focus

`KitDateTimeField` and `KitZonedDateTimeField` expose `editor-focused` and `focus-observed(bool)` across their native text editors. The host can use these observations in its focus registry. They are not commit signals, and focus changes between individual editors may produce observations.

`KitZonedDateTimeField.ambiguity-started()` reports the beginning of an earlier/later action before its completed `ambiguity-chosen(AmbiguousTimeChoice)` callback. Preserve the pending resolution context during focus changes; accept an occurrence only after the completed choice. Keep all three draft strings and the resolution generation together in the host.

## Choose modal traversal explicitly

[KitDialogSurface](/components/dialog-surface/) now exports `DialogTabPolicy`. Its default `.host` policy captures Tab/Shift+Tab and calls `focus-next(bool)` for the host registry. The `.native` policy leaves traversal to Slint’s mounted controls.

With native traversal, the window host must make **every background focus target ineligible** while the modal is open. Set fields to `FieldAccess.disabled`, pressables to `disabled: true`, and [KitMenu](/components/menu/) and `KitDataTable` to `interactive: false` as appropriate. Restore access and the exact opener when the modal closes. Native policy alone does not establish this boundary.

`KitMenu.has-focus` and `focus-observed(bool)` include a row receiving focus on pointer-down, before activation. Preserve that focus during the press so the click can complete.

Only the topmost dialog handles Escape or system Back (`Key.Back`); a busy dialog consumes these keys without dismissing. An open selector reports `CloseReason.system-back` through [KitSelectKeyboard](/components/select-keyboard/). Its `dismiss-on-system-back` defaults to `true`; persistent menus, menu bars and tabs disable it so the containing overlay can receive Back. Handling the key is implemented; Android device qualification remains a separate check.

## Let layouts measure content

Filled buttons accept their allocated width even with long labels. Comfortable text-field clear/reveal actions include the surrounding padding in their hit target; pressing field padding focuses the existing editor. Keyboard traversal reveals the button’s full focus outline in ancestor scroll areas.

[KitFieldLabel](/components/field-label/) propagates wrapped height through layouts, including at 200% text. [KitAlert](/components/alert/) measures wrapped copy at its allocated width inside conditional layouts. Avoid fixed heights around these components.

For a target taller than the viewport, [KitScrollArea.reveal(top, bottom)](/components/scroll-area/) keeps a saved position when the leading edge is already visible; otherwise it reveals that edge. Repeated focus restoration remains stable.

Homeward is the first integration proof of concept. Confirmed reusable kit defects belong in this repository with regression coverage; application transport, installation identity and server authorization stay in the host. Keep the official `cargo run -p xtask --locked -- check` gate, and inspect native light/dark rendering, enlarged text and actual interactions. Screenshots and headless tests do not qualify IME, screen readers or every target platform.
