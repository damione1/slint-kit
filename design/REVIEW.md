# Component quality review

The review covers all **124 public components**, plus their internal composition
helpers, from `develop` at `c830f49c`. Three agents reviewed forms, navigation/data
and display/layout; the coordinating pass covered buttons, themes and the
catalogue shell. The five design skills are adapted to the existing portable
Slint controls in [KIT.md](KIT.md).

## Changes

- **Alignment and reading:** centered switch/radio indicators and labelled
  dividers; one timeline axis; intrinsic complete fields and document footers;
  square numeric step actions at 200%; shared gallery typography and spacing.
- **Input and identity:** complete OTP placeholders; no placeholder over native
  preedit; numeric byte constraints; semantic search clearing; RTL segmented
  order without changing option identity.
- **Modal interaction:** background controls become ineligible, semantic actions
  cannot escape to another page, the backdrop consumes wheel events, and the
  panel's own viewport remains scrollable. Disabling a keyboard translator
  clears a held activation, preserving the first Return after reopening.
- **States and feedback:** data hover/press/selection feedback, selected-row
  accessibility, a real thumbnail fallback, visible toast dismissal, solid
  stopped skeletons, enlarged busy marks and custom status badges.
- **Catalogue accessibility:** named regions, selected navigation items, native
  AX disabled/busy state, legible teal editor selection and a stable wordmark.
- **Long labels:** bounded French/RTL sample actions, complete accessible names
  and keyboard/hover help dismissed with Escape.

`KitMenuBar`, `KitTabs`, `KitAccordion`, `KitCarousel`, `KitBreadcrumbs` and
`KitPagination` now expose `interactive`, defaulting to `true`. Hosts set it to
`false` for background composites while a modal owns input. This preserves the
models and selection. The API, examples and integration guidance are updated in
English and French.

## Coverage

| Review | Public components |
| --- | ---: |
| [Forms](review-forms.md) | 57 |
| [Navigation and data](review-navigation-data.md) | 26 |
| [Display and layout](review-display-layout.md) | 33 |
| [Foundations](review-foundations.md) | 8 |
| Total | **124** |

`python3 tools/check-design-review.py` checks the inventory against the actual
public catalogue so a component cannot disappear from the coverage count.

## Verification

Final product source revision: `97ccab8a4dffe1ef6cc4937ad12286c941425807`.
The component library has been unchanged since `5201a0aa`; the later product
change bounds long localized labels in the showcase consumer.

- `cargo run -p xtask --locked -- check`: **312 tests passed**, generated contracts
  and icons match, token lint passes, Clippy passes with warnings denied.
- Native winit/Skia on macOS: nine navigation/data/modal paths in light/100%
  and dark/200%, **70 outcomes and 176 events**. Pointer, keyboard and semantic actions
  cover identity, sorting, editing, focus restoration and modal isolation.
- Foundation paths in both modes: **30 outcomes**, including actual macOS AX
  roles, selected page and unavailable controls. Search clears and resumes in
  the same native editor; the visible toast close action works. Keyboard focus
  reveals a complete long French label and Escape dismisses its tooltip.
- **496 final native component images** cover all 124 public examples in
  light/dark at 100%/200% text. Their manifest pins 197 library/example source
  hashes, independently checked against the library freeze.
- **108 baseline and 108 final native showcase images** cover all nine pages at
  top/middle/end in the same four appearance/text variants.
- Public previews: **450 verified scenes**, including all 124 starters.
  Documentation build/check pass for 248 localized references and 268 canonical
  pages. The French dialog example now uses French action labels.
- Browser checks pass at **DPR 1 and 2**: each covers 124 WASM startups,
  28 page captures and 23 Workspace renders plus editing/validation/focus flows.
  Search passes its fixed desktop-DPR-1/mobile-DPR-2 matrix without errors.
- The [verification record](verification.json) records revisions, commands,
  manifest hashes, browser checks and representative evidence.

The full local evidence lives under `docs/.verification/quality-0.4/`. The native
commands are reproducible with `tools/capture-component-quality.py`,
`tools/capture-quality.py`, `tools/exercise-navigation-data.py` and
`tools/exercise-foundations.py`.

Actual native event routes and an AX tree audit do **not** qualify a VoiceOver
flow, physical input devices, native IME, external file drops or other platforms.
Those limits are kept explicit in the evidence manifests.

## Representative native renders

These images are copied from the recorded captures without visual alteration.
The light comparison uses the same page, position and 100% text setting.

| View | Evidence |
| --- | --- |
| Baseline gallery | [Light, before](previews/gallery-before-light.png) |
| Final gallery | [Light, after](previews/gallery-after-light.png) · [Dark at 200%](previews/gallery-after-dark-200.png) |
| Complete long label at keyboard focus | [French at 200%](previews/long-label-keyboard.png) |
| Square numeric actions | [Dark at 200%](previews/number-stepper-dark-200.png) |
| Aligned timeline | [Dark at 200%](previews/timeline-dark-200.png) |
