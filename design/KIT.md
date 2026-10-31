# Slint Kit design

Feel: calm, precise, familiar.

## Applied skills

The installed instructions below guided the three family agents and the
coordinating review. Their SwiftUI examples were translated to existing Slint
APIs; they do not introduce a Swift runtime or replace native editing.

| Skill | Application to this kit |
| --- | --- |
| `mac-design` | Desktop hierarchy, restraint, keyboard conventions, light/dark appearance and native text behavior |
| `ux-design` | Clear actions and feedback, disabled/read-only behavior, modal ownership, focus restoration and accessible names |
| `layout-grid` | Shared leading edges, token spacing, intrinsic sizing, measured geometry and native image inspection at 200% text |
| `design-system` | Semantic color and typography roles, contrast, complete themes and three consistent densities |
| `design-kit` | Inventory of every public component, relevant states, composition boundaries and the parts gallery |

Installed sources for this campaign are the corresponding
`.agents/skills/<name>/SKILL.md` files in the original `slint-kit` workspace.
Slint syntax and runtime verification follow the installed `slint:slint` skill.

The catalogue is a desktop sidebar window: appearance controls above, component
families at the leading edge, examples in a single reading column. Product
examples demonstrate the same public components in realistic tasks. Keep native
window controls and native text editing. The kit remains portable across Slint
backends; SwiftUI APIs, SF Symbols and glass materials are not dependencies.

## Tokens and typography

`crates/kit-slint/ui/theme.slint` owns semantic colors, shape, elevation, focus,
typography, spacing, motion and family recipes. Components use those tokens;
consumers may supply a complete theme. Preserve the existing default design and
three densities. Text follows `KitMetrics.font-size` and `theme.text-scale`.
Body and control copy remain 14 logical pixels, captions 12, subtitles 18 and
page titles 30. These are the kit's portable roles, not a claim of AppKit styling.

Use the four-pixel spacing unit: 4 for help, 8 within related controls, 16 within
groups, 24 between sections. Each reading column shares a leading edge. Control
heights follow density and the actual content; a 200% text setting must not hide
labels, values, action glyphs or feedback. One-pixel strokes and optical icon
geometry are deliberate exceptions to the spacing grid.

## Icons and color

Use `KitIcon` and the generated, licensed Lucide set or original `KitGlyphs`.
Decorative icons remain silent; informative icons need meaningful labels.
Selection, focus, hover and press are distinct. Status always includes words or
a meaningful icon. Normal text needs 4.5:1 contrast, meaningful boundaries and
marks 3:1. Disabled controls retain a visible shape and label.

## Parts and states

The complete component inventory and per-family findings live in
`review-forms.md`, `review-navigation-data.md`, `review-display-layout.md` and
`review-foundations.md`. Composition primitives are reviewed separately from
complete controls; they do not imply independent application actions.

| Family | Required behavior and presentation |
| --- | --- |
| Buttons and pressables | Intrinsic width, stable busy geometry, pointer cancellation, Space on release, Return once, visible keyboard focus, disabled guard |
| Fields and field composition | Visible labels/help/errors, native selection/caret/undo/IME, read-only selection, matching intrinsic and host validation |
| Numeric and temporal fields | Exact domain values, editable partial drafts, explicit units/segments/ambiguity, bounded actions |
| Choices, color and files | Labels plus state, keyboard alternatives, stable identity, explicit native capability/cancellation outcomes |
| Selects and menus | Distinct active/selected/disabled rows, bounded scrolling, keyboard navigation and focus restoration |
| Dialogs and drawers | Labelled task, modal focus isolation, clear submit/cancel, exact opener restoration, busy dismissal guard |
| Navigation | Selected content is clear, disclosure is subordinate, no layout shift on hover, logical keyboard order |
| Data | Virtualized viewport, keyed rows, sort and empty feedback, hover/press/selected states, readable columns |
| Display and feedback | Intrinsic content, restrained emphasis, aligned timelines/steps, non-color status, reduced-motion fallback |
| Layout and scrolling | Shared edges, token insets, bounded content, no accidental stretch, scrolling only when needed |

Review rest, hover, press, keyboard focus, disabled, read-only, invalid, selected,
open, loading, empty and completed states where applicable. Inspect native
light/dark rendering and 200% text. Headless checks prove geometry and callbacks;
they do not qualify VoiceOver, IME, external file drops or other devices.

## Command map and feedback

Tab/Shift-Tab traverse controls; arrows navigate composites; Space activates a
button on release; Return confirms the current action (multiline editors retain
Return for a new line); Escape dismisses the
topmost dismissible overlay. Standard editor shortcuts retain native meanings.
The catalogue's appearance buttons change their visible state immediately.
Empty data explains the next action; invalid forms retain the draft and place
feedback by the affected field. A transient toast is never the only way to
recover from an error.
