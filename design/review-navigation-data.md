# Navigation, selection, overlays and data

Reviewed against the public 0.4.0 API from `develop` (`c830f49c6d18f3db76fae7c9474ae04407367d4a`). The Mac design, design kit, design system, UX and layout grid skills inform hierarchy, semantic colors, focus, spacing and resizing. Existing Slint primitives and the cross-platform API remain the implementation reference; SwiftUI-only materials, symbols and window APIs are not introduced.

## Component coverage

Every public component exported by these five families was inspected. “Retained” means no confirmed defect justified a source change; it does not claim unperformed device qualification.

| Component | Review and result |
| --- | --- |
| `KitNavigationAction` | Retained intrinsic label measurement, ellipsis, option recipe and stable activation identity. |
| `KitMenu` | Active target and persistent selection remain independent; the focus outline now appears only while the menu owns focus. Pointer-down focus observation remains intact. |
| `KitMenuBar` | Active target outline now follows actual composite or row focus. Horizontal scrolling and arrow/typeahead translation retained. New `interactive` eligibility disables its real pointer, keyboard and semantic targets without replacing option identities. |
| `KitTabs` | Persistent selection underline retained; the active target gets an outline only while focused. `interactive` makes its real targets ineligible while preserving selection and model. |
| `KitCollapse` | Quiet disclosure treatment replaces the filled button treatment; disabled text and chevron now use disabled ink. Mounted disclosure content and reduced-motion transition retained. |
| `KitAccordion` | Stable keyed expansion, single/multiple policy, roving arrows, mounted descriptions and reduced-motion transition retained. `interactive` disables every header and clears pending focus requests. |
| `KitCarousel` | Intrinsic wrapped title/body, bounded manual navigation, explicit empty and current-position states retained. `interactive` disables both navigation actions. |
| `KitBreadcrumbs` | Separate parent actions and static current location retained; horizontal viewport preserves long paths. `interactive` disables every actionable ancestor. |
| `KitPagination` | Validated host-owned page projections, boundary-disabled actions and visible position retained. `interactive` also makes every page action ineligible during modality. |
| `KitNotificationButton` | Retained intrinsic icon/text/count geometry and a full accessible label. |
| `KitNotificationCenter` | Empty title and description wrap. Stable action slots, dismissal identity snapshots and bounded/virtual viewports retained. |
| `KitDrawer` | Header sizing is intrinsic; scrollable client content retains stretch allocation. Retained window-root modality, bounded trailing panel, explicit close name, busy/topmost guards and host focus restoration. |
| `KitDataTable` | Hover and press feedback restored for unselected rows; selection uses selection/on-selection tokens. Rows explicitly expose selectability, selection and collection position. Header alignment and virtualization retained. |
| `KitContentList` | Hover and press feedback restored; persistent selection and disabled appearance now use semantic tokens consistently. Empty thumbnail detection uses image dimensions, so the article fallback appears reliably. Rows expose selectability, selected state and collection position. |
| `KitVirtualTextView` | Editor footer measures its actual content, wraps errors and stacks controls in narrow/enlarged layouts. It preserves one mounted native editor. Document rows expose list-item role, selection and position; pointer feedback restored. |
| `KitSelectKeyboard` | Shared key translator, activation repeat suppression, skip-disabled navigation callbacks, typeahead and explicit dismissal reasons retained. Disabling its FocusScope now clears an armed Return/Space latch when FocusOut is suppressed by Slint. |
| `KitSelectTrigger` | Retained field recipe, editable/read-only/disabled distinctions and named combobox semantics. |
| `KitSelectSurface` | Search placeholder no longer overlaps active IME preedit text. Native editor, caret scrolling and active-row reveal retained. |
| `KitOption` | Retained stable token/generation snapshot, active/selected distinction and shared pressable behavior. |
| `KitSelect` | Retained native popup closure acknowledgement, exact Tab restoration, bounded virtual list and shared validation engine. |
| `KitDialogSurface` | Topmost/busy dismissal guards and explicit host/native Tab policy retained. Its backdrop now consumes wheel events, preventing the underlying page from scrolling while child panel viewports retain normal wheel handling. |
| `KitDialogPanel` | Retained named group, description and theme-driven two-layer elevation; content consumes backdrop hits. |
| `KitDialogHeader` | Retained intrinsic composition layout and theme spacing. |
| `KitDialogBody` | Retained intrinsic vertical composition and theme spacing. |
| `KitDialogFooter` | Retained trailing form actions and theme spacing; callers remain responsible for responsive composition. |
| `KitFocusTarget` | Retained eligible/token/generation/request guards and actual focus observation. |

The internal `FocusOutline` helper was also inspected: it remains the single theme-driven outside/inset ring renderer.

## Gallery composition

Navigation and data galleries use the shared typography roles, weights, gaps and motion tokens. Nested navigation layout primitives receive the gallery theme explicitly. Content-list/preview columns derive their widths from their real content container and stack when the available width or text size requires it, instead of assuming a fixed card inset. Existing previews, search, sort, notifications and editable document callbacks remain connected. The gallery disables every background composite, action and field while the drawer or another host modal is open; foreground drawer editing remains active.

UX flows remain direct: choose a destination or tab to reveal its content; search/sort records while identity stays stable; select an article to reveal its preview and return focus on close; open collection settings, edit, save or cancel, and restore the exact opener. Errors remain next to editable content. Notifications retain separate read, preview and dismiss actions with visible outcomes.

## Verification

Regression checks cover disabling all six navigation families across armed pointer, keyboard, semantic action and focus-request routes; re-enabling after a held Return; backdrop-wheel isolation with an independently scrolling panel; collection selection metadata and model updates, absent/present thumbnail transitions, the editor/footer/error bounds at 100% and 200% in a narrow window, intrinsic drawer composition, and the menu focus outline entering/leaving the composite without changing the active target. Existing checks cover stable generations, skip-disabled activation, large-model virtualization, native editor retention, clipboard, table alignment, dialogs, drawer focus traps and carousel geometry.

The 26 public examples were inspected in light/dark at 100%/200% using native winit/Skia captures under `docs/.verification/quality-0.4/components/verified/`; its manifest records the example and library sources used for those images. Baseline showcase navigation/data pages were also inspected at all four appearance/text variants. Those images revealed the simultaneous unfocused rings, empty thumbnail fallback and inflated drawer content that prompted corrections. Drawer was re-inspected in all four modes after its intrinsic field/caption fixes. The final showcase manifest in `docs/.verification/quality-0.4/after/manifest.json` records source `5201a0aa`; all 24 navigation/data frames (top, middle and end in the four variants) were inspected after the final product freeze.

The final product source is `5201a0aab4961c645644db05f730aa4811aa46da`. The official gate passed 311 checks plus clippy with warnings denied. `tools/exercise-navigation-data.py` exercised these actual native flows through embedded Slint MCP on macOS 27.0, arm64, Mac16,7 with winit/Skia:

| Flow | Evidence checked in light 100% and dark 200% |
| --- | --- |
| Menu and tabs | Pointer selection, arrow/Return navigation, disabled Archive semantic action rejected. |
| Notifications | Stable Mark read slot, preview/back, dismissal identity, Escape and restored-trigger Return. |
| Drawer | Empty-name error, native text editing, save, restored opener and dismissal. |
| Data table | Pointer selection, End to product #10000 and stable selected record after sorting; accessible delegate projections rather than offscreen status text. |
| Article list | Closing preview restores the list target; Down opens the next keyed article. |
| Virtual text | Bounded delegates for 50,000 lines, native editing and save, Copy callback. Shared clipboard preserved in memory and restored in `finally`. |
| Modal background | Every mounted background pressable ineligible; semantic actions cannot change the opener page; Escape restores navigation eligibility and the same semantic action works again. Scrim wheel leaves page offset unchanged. |
| Custom select | Expressive option rows retain avatar bounds on both axes. Long labels elide rather than overflow at 200%. |
| Modal wheel fixture | Real native overflowing child viewport scrolls; outside wheel and inner wheel both preserve the background offset. Fixture derives from the tested public-component composition. |

Both modes passed all nine flows: **70 asserted outcomes and 176 recorded events, with no dropped or unknown events**. Manifests, individual results, element properties and screenshots are under `docs/.verification/quality-0.4/native-navigation-data/verified-light/` and `verified-dark-200/`. They record exact source and binary hashes. Notification screenshots were replaced after waiting for the theme's fade to settle; their manifest repair records verify identical binaries and sources. Final light/dark200 native frames were inspected, including modal disabled states, focus restoration, table selection, responsive document footer and expressive select geometry.

Kit defects and host composition defects were treated separately. Modal wheel leakage and the disabled keyboard activation latch are fixed in shared primitives. The public navigation composites now expose `interactive` for their own targets. The showcase host disables its background targets while modal, and consumer-provided Drawer captions explicitly retain intrinsic height. No new Drawer content policy was necessary.

This document does not claim VoiceOver, physical keyboard, IME, Increase Contrast, Reduce Transparency or non-Mac backend qualification from headless checks or screenshots. Native MCP dispatch exercises the real native event route; it does not replace those device and assistive-technology qualifications.
