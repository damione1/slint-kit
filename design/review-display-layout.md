# Display, feedback and layout review

Baseline: `origin/develop` at `c830f49c6d18f3db76fae7c9474ae04407367d4a` (0.4.0). Display fixes: `6665fa4b`; gallery and example polish: `fd4763b1`.
Review lens: `mac-design`, `design-kit`, `design-system`, `ux-design`, `layout-grid`, and the Slint skill. The transferable rules are restrained hierarchy, semantic colors, theme-owned geometry, readable scaled text, named controls, keyboard operation and reduced motion. SwiftUI types, Liquid Glass and SF Symbols do not replace the existing cross-platform Slint primitives.

## Coverage

All 33 public components in these families were inspected at source level. `ScrollFrame` and `ListView` are internal composition details and were inspected separately. “Retained” means the source review found no correction to make; it does not claim every platform or assistive technology has been qualified.

| Component | Review and result |
| --- | --- |
| `KitBadge` | Intrinsic text/icon sizing, semantic tone, label and scaled typography retained. |
| `KitAvatar` | Static image and action modes, image fallback, clipping, disabled state and shared pressable behavior retained. Existing pointer/keyboard/disabled regression covers the action mode. |
| `KitCard` | One padding owner, natural content height and repeated shadow geometry retained; existing exact-height regression remains required. |
| `KitCardHeader` | Wrapped title/description, family/size/weight tokens retained. |
| `KitCardBody` | Intrinsic vertical composition retained. The display demo now passes its theme explicitly. |
| `KitCardFooter` | Reflowing actions retained. Demo actions now distinguish one primary action from neutral alternatives. |
| `KitHero` | Introductory surface, wrapping and intrinsic layout retained. Demo uses a content title instead of the generic “Hero”. |
| `KitFooter` | Content-info semantics and intrinsic trailing section retained. |
| `KitDivider` | Fixed: both rules now share the scaled label’s vertical center instead of aligning to its top edge. Theme-owned separator thickness retained. |
| `KitStat` | Wrapped labels, value and descriptions retained; display demo preserves column/stack reflow at enlarged text. |
| `KitChatBubble` | Author/message/metadata grouping, asymmetric alignment, neutral received surface and on-accent sent text retained. |
| `KitSkeleton` | Fixed: stopping in the dim pulse phase now restores a solid placeholder. Reduced motion remains static. |
| `KitEmptyState` | Width-bound copy, intrinsic height and caller-supplied first action retained. |
| `KitTimeline` | Fixed: marker and connector use the same horizontal axis, marker aligns to the rendered first text line at either scale, geometry derives from tokens, and no connector trails the final entry. |
| `KitSteps` | Current/complete/error/pending visual distinctions and accessible state words retained. |
| `KitAlert` | Width-bound copy, severity icon, error live region and accessible dismiss button retained; existing conditional geometry regression remains required. |
| `KitSpinner` | Named activity, reduced motion and stopped presentation retained. |
| `KitProgress` | Bounded value semantics, anchored fill and reduced-motion transition retained. Demo names the operation being performed. |
| `KitRadialProgress` | Bounded value semantics, visible percentage and scaled intrinsic size retained. |
| `KitTooltip` | Input-transparent native hover help, keyboard trigger and Escape suppression retained. |
| `KitTooltipLayer` | Window-local ownership, bounded placement and modal invalidation retained. |
| `KitToast` | Fixed: the dismiss button now draws `theme.icons.close`; its former empty glyph left an invisible clickable target. Stable identity, live region and independent pointer/focus pause sources retained. |
| `KitToastHost` | Natural repeated height and model-owned lifetimes retained; existing exact-height regression remains required. |
| `KitContainer` | Finite width, reading-width cap and centered gutters retained. |
| `KitStack` | Theme-owned vertical spacing and intrinsic sizing retained. |
| `KitInline` | Explicit single-row composition retained; callers choose wrapping through `KitWrap`. |
| `KitWrap` | Instance-preserving reflow retained; existing native editor/focus preservation regression remains required. |
| `KitGrid` | Native Slint grid semantics and theme-owned spacing retained. |
| `KitSection` | Wrapped title/description and one leading edge retained. EN/FR examples and the quickstart remove their redundant repeated title. |
| `KitSurface` | One surface/padding owner, token recipe and caller-controlled elevation retained. |
| `KitScrollArea` | Finite viewport, vertical reflow, reveal behavior and themed scrollbar retained. Existing oversized-focus regression remains required. |
| `KitAppShell` | Header/content/footer composition with caller-controlled stretch retained. EN/FR examples and the quickstart now explicitly choose `LayoutSpace.regular` to separate their children. |
| `KitScrollBar` | Pointer drag, track paging, wheel, disabled state, bound projection and reduced motion retained. Internal bars intentionally do not add focus stops or accessibility nodes. |
| Internal `ScrollFrame`, `ListView` | Overlay bars and virtualized model/view identity retained. |

## Changes and checks

- `labelled_divider_rules_share_the_labels_vertical_center`: checks both separator rules against the label center in both themes at 100% and 200% text.

- `timeline_markers_share_their_connectors_axis_at_each_text_scale`: checks a shared marker/rule axis, first-line vertical centering, scaled marker size and absence of a dangling final rule in both shipped themes at 100% and 200% text.
- `stopping_a_skeleton_restores_a_solid_placeholder_after_either_pulse_phase`: checks that stopped animation remains at full opacity regardless of its previous phase.
- `toast_dismissal_draws_the_themes_close_glyph_inside_its_named_button`: checks that the dismiss control renders a fitted stroked glyph and keeps it centered at both text scales.
- The display demo now uses typography, spacing, radius and motion tokens; explicit themes reach composition parts. A single primary action is distinguished from neutral alternatives in each applicable example. The tooltip target is now a working “Publish Update” action with an Undo notification, rather than an inert button. Copy names the import and its completion/cancellation states.
- Existing feedback, scrollbar, avatar, focus, exact-height and editor-preserving reflow tests remain part of the official gate.

## Qualification

`cargo run -p xtask --locked -- check` passed formatting, generated contracts/icons, token lint, 305 workspace tests and Clippy with warnings denied. The 160 public kit tests include the four added here and the existing unlabelled-divider pixel regression.

All 32 dedicated example images in these families were visually inspected in light and dark at 100% and 200% text, for 128 views. `KitScrollBar` was inspected within `scroll-area`, covering the 33rd public component. The changed timeline, toast dismiss glyph and labelled divider were also reviewed at full image size. No remaining clipping or alignment defect was observed in these fixtures. Static captures show their declared state; tooltip dismissal, toast interaction sources, reduced motion and native editor preservation are checked separately by the testing backend.

Native review used winit/Skia on macOS 27.0 arm64, `Mac16,7`. Snapshots and consolidated source provenance are under `docs/.verification/quality-0.4/`. Final proof is regenerated after the full source tree is frozen; the discovery capture pass correctly refuses to write a manifest if another family's source changes during capture.

Slint 1.18.1 supplies landmark roles but no heading role, so SwiftUI’s heading trait is not claimed here. These checks do not qualify VoiceOver, native IME, Increase Contrast, external drag and drop, touch hardware or another desktop backend.
