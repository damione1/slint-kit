# Documentation update for the integrated source

Historical verification for `78bbf70`, before the 0.2.0 publication. See
[release and Retina verification](RELEASE-0.2.0.md) for the subsequent release.

Published baseline: `bcbe17672fb40f6f8b85290416c1fcef2e3e1f05`.
Local source tip: `ca2a5e3fbe25e29f40a04a93cb622e4e7f7482a5`.
Integrated and tested Rust source: `ec301c9` (merge of those two histories).
Kit UI SHA-256: `55a8571405d3f3a186a6abb14d9c59d4be80e8245aeeba5a971f4afba17171b4`.

The update preserves the 12 original local commits and the two published SEO
commits. Work was performed in the worktree based on `origin/main`. At that revision, the website
explained source compatibility explicitly: the new APIs were on `main`,
while crates.io was at 0.1.0. That documentation update did not publish a registry release.

## Commit coverage

| Local commit | Documentation coverage |
| --- | --- |
| `534de80` | Homeward integration ownership, regression checks and the official gate; repository instructions retained. |
| `f60d916` | Intrinsic alert height with wrapped copy in conditional layouts. |
| `2f52130` | Filled button width, field padding focus and comfortable action targets. |
| `d94e4c2` | Topmost dialog/system Back handling, busy behavior and persistent composites. |
| `c6eb087` | Host/native modal traversal, background eligibility, menu focus and full focus outlines. |
| `7506444` | Stable reveal behavior for targets exceeding the viewport. |
| `a1ea581` | Wrapped label height propagation and layout-owned measurement. |
| `b5e7e3a` | Host-supplied date/time/zone and ambiguity labels. |
| `cc193c4` | Composite editor focus and ambiguity activation context. |
| `bc81173` | Calendar navigation labels, picker strings and temporal help. |
| `77c71a0` | Localized content-list empty message. |
| `ca2a5e3` | Localized table empty message. |

The bilingual source-updates guide, component prose, API extraction, French
fixtures, getting-started/controller/theme links, README and consumption skill
all reflect these changes. The dialog fixture demonstrates native traversal in
a window with no background controls. Typed-example image dimensions are now
read from the actual PNG headers, as with table captures, preserving reserved
preview geometry after recapture.

## Native capture method

Slint 1.18.1, macOS 27.0, Apple M4 Pro, winit/Skia, scale factor 1.
The capture app was built with `SLINT_EMIT_DEBUG_INFO=1` and `--features slint/mcp`;
the embedded server was reached through loopback Streamable HTTP.
[The capture workflow](capture/README.md) reproduces this setup without changing
production library defaults.

All 246 English/French component baselines and existing variants were genuinely
recaptured through `take_screenshot`. Additional temporal dark/200% variants
bring the catalogue manifest to **326 PNGs**. Image bytes, dimensions, fixture
sources, kit UI sources and capture-tool hashes are recorded in
`content/captures.json`. Thirty-two localized baselines were visually inspected,
alongside representative dark and 200% variants; this is not a claim to have
visually reviewed every capture. Button pointer and Tab/Space activation were
verified through MCP in separate fresh windows for each language.

The native table verification passed all eight EN/FR × light/dark × 100%/200%
combinations, exercising pointer/keyboard sorting, stable selection, Home/End,
Enter and stale/disabled rejection. Both typed native applications passed real
editor draft/commit, invalid-value preservation, pointer-open selection and
keyboard commit checks. Their public captures were refreshed too.

## Checks

- `cargo run -p xtask --locked -- check`: passed; formatting, generated contracts,
  142 tests and Clippy. This uses the unchanged official gate.
- `npm --prefix docs run build`: passed; 123 components, 246 localized references,
  264 canonical HTML pages, matching sitemap, source downloads, image dimensions,
  provenance, metadata, links and language alternates.

- `npm --prefix docs run check:browser`: 16 passing EN/FR mobile/desktop,
  light/dark and enlarged-text scenarios, delayed images/scripts, native menu
  keyboard/pointer interactions, no-JavaScript navigation and catalogue filtering.
  Initial-load CLS was 0 throughout.

Native captures and headless regressions do not qualify IME, a screen reader,
an Android device or other backends. Controller recipes retain their existing
registry-compatible source and dependencies. Deployment receipts and detailed
local/browser logs are stored under ignored `docs/.verification/`.
