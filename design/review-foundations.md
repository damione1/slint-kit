# Foundations and catalogue

Baseline: `origin/develop` at `c830f49c` (Slint Kit 0.4.0, Slint 1.18.1).
The five design/UX/grid skills are applied through the existing Slint primitives.
The catalogue has 124 documented public components, covered by this file and
the three family reviews. Six internal helpers are considered separately.

| Component | Review and result |
| --- | --- |
| `KitPressable` | Fixed semantic activation-started callbacks to reject disabled/busy input before notifying consumers. Retained pointer cancellation, Space-on-release, Return repeat guard, intrinsic/fill policy and keyboard focus outline. |
| `KitButton` | Retained native-sized single-line labels and intrinsic icon/label measurement. Busy marks now grow with text instead of remaining a tiny glyph at 200%. |
| `KitFormButton` | Retained explicit submit/reset intent and shared validation request before activation. |
| `KitSubmitButton` | Retained primary completing action and form validation. |
| `KitResetButton` | Retained secondary treatment and explicit reset action. |
| `KitIconButton` | Retained named accessible action, scaled square hit area and centered decorative icon. Busy glyph scales with text. |
| `KitButtonGroup` | Retained intrinsic horizontal grouping and token spacing; consumers choose wrapping when needed. |
| `KitIcon` | Retained decorative/informative distinction, scalable glyph/stroke geometry, theme colors and licensed generated source. |
| Internal `TurningIcon` | Retained theme motion and direct glyph switch for software-renderer fallback and reduced motion. |
| Internal `ElevatedSurface` | Retained concentric layer radii, theme-owned key/ambient shadows and flat fallback. |

## Theme and shell

Complete light/dark themes and all three densities retain their existing
geometry and palette. Busy marks use the same text scale as their labels.
The teal demonstration theme previously combined a bright selection fill and
white selected text in dark mode; it now supplies the appropriate on-accent ink.
A contrast regression covers both modes.

The wordmark keeps its natural size at 200% text, leaving space for the actual
controls and content. Every toolbar control follows the chosen density. The
sidebar uses its available width, balanced insets and a quiet selected tint
rather than a raised button. Catalogue typography reads shared roles and
weights. Supported banner, navigation and main landmarks name the regions.

The selected sidebar item now exposes its selection through the actual macOS
AX bridge. The gallery header, sidebar and examples become unavailable while a
modal owns input; semantic activation cannot navigate out through its backdrop.
Long French/RTL demonstration labels fill their bounded row and elide instead
of overflowing the window. Their complete accessible name is preserved and a
keyboard/hover tooltip reveals the full text; Escape dismisses it.

The pinned Slint accessibility roles do not include a heading role; a SwiftUI
heading trait cannot be copied here. Landmarks and visible section titles do
not constitute VoiceOver qualification.

## Checks

The existing button sizing, press cancellation, focus outline, keyboard and
theme completeness/contrast tests remain required. Expanded checks cover scaled
busy marks, equal toolbar control heights across densities, a stable wordmark,
and teal selection contrast. Native catalogue captures cover all nine pages at
top/middle/end in light/dark at 100% and 200% text.

Final tested revisions, image evidence and the official-gate outcome are
recorded in the campaign result. Native IME, VoiceOver, Increase Contrast,
external drops and other backends/devices are not inferred from these checks.

The final foundation interaction pass exercises four native flows in light and
dark/200%: button pointer/Space activation plus macOS AX auditing, search clear
and resumed editing in the same editor, visible toast dismissal, and keyboard
help for the long localized action. All **30 asserted outcomes** pass. The
actual AX trees contain 172 and 158 elements respectively and verify button
roles, disabled/busy availability, the selected page and the named navigation
region. These are AX bridge inspections, not screen-reader sessions.

Evidence: `docs/.verification/quality-0.4/interactions/foundations-qualified-*`.
The [campaign verification record](verification.json) pins source revisions,
binary/manifest hashes, counts and representative images.
