# SEO implementation and verification

Baseline: `origin/main` at `135e9dd189c824943230a7bfc4ad35c15c07fba4`.
Implementation is isolated on `codex/seo-improvements`. Unpushed changes from the
main checkout are not part of this branch. No production deployment, registry
publication, Search Console submission or Cloudflare policy change is included.

## Changes

- The documentation menu starts closed on mobile and uses a native `details`
  disclosure. A separate desktop container renders the same shared navigation
  include; CSS exposes one navigation at a time. JavaScript no longer changes
  the menu state at startup or on resize. Both layouts work without JavaScript.
- `content/pages.json` supplies explicit English/French titles and descriptions
  for all general pages. Component descriptions still use their own summaries.
  The two new table guides bring the site to 262 canonical HTML pages.
- The homepage exposes the version, Rust/Slint compatibility, pre-1.0 status,
  package source, repository and license. Version facts come from `Cargo.toml`.
  The Cargo homepage now points to the documentation site for the next release.
- Canonical metadata feeds Open Graph, large sharing cards, and a safely escaped
  JSON-LD graph (`WebSite`, `SoftwareSourceCode`, the actual page, and its visible
  breadcrumb). Two original 1200×630 share cards reuse the licensed project mark.
- All HTML images reserve their actual dimensions, including the existing typed
  example. General metadata, structured facts, breadcrumb agreement, image
  dimensions and download/source agreement are checked during the docs build.
- A complete bilingual sortable table application is available under
  `/guides/table-example/` and `/fr/guides/table-example/`. It uses registry
  dependencies, public `@kit` imports, stable domain keys, generation guards,
  weak window references and a retained `VecModel`. The workspace verifier
  compiles the exact same public Rust and Slint sources.
- Hero, SegmentedControl, UrlField and ZonedDateTimeField explanations now cover
  concrete use cases and integration boundaries in both languages. Their source
  components are unchanged. The table guide is linked from the sidebar, controller
  guides and DataTable reference.
- A documentation CI job runs the build and its checks. The existing official
  Rust gate remains unchanged.

## Verification

Environment: macOS 27.0, Apple M4 Pro, Rust 1.98.1, Slint 1.18.1. Native table
captures use **winit/Skia**, scale factor 1. These observations do not qualify
Windows/Linux, a physical mobile device, IME or a screen reader.

| Check | Result |
|---|---|
| `cargo run -p xtask --locked -- check` | Pass: workspace formatting, contracts, tests and Clippy |
| `npm --prefix docs run build` | Pass: 123 components, 246 component references, 262 canonical pages, EN/FR links, sitemap, image provenance and SEO contracts |
| Standalone copy of `table-consumer`, `cargo test` against crates.io | Pass: builds independently and checks selection, generation rejection, disabled rows and generation exhaustion |
| Native table, EN/FR × light/dark × 100%/200% text | Eight passing scenarios; actual pointer header sorting at 100%, keyboard header sorting at 200%, Home/End/Enter navigation and selection checked |
| `npm --prefix docs run check:browser` | Pass: 12 inspected captures, delayed script, keyboard/pointer, no-JS menu, responsive navigation and catalogue filter |
| Browser initial-load CLS | 0 in all 12 scenarios, including a 1.2-second script delay; intentional text enlargement is a separate visual check |
| Lighthouse 13.5.0, local Button page, simulated mobile | Performance 100, accessibility 100, SEO 100; lab LCP 1.05 s, CLS 0, TBT 0 ms |

Screenshots and machine-readable results are saved under `docs/.verification/`
(ignored by Git). Native variants and selected states were inspected, together
with browser mobile/desktop, French/English, dark and enlarged-text captures.
The two public table screenshots are committed with source/image hashes in
`content/table-captures.json`. JSON-LD syntax and agreement are checked locally;
the graph has not been submitted to a remote rich-result validator.

The Lighthouse result is a local lab measurement. Hosting, Cloudflare and real
visitor conditions differ from production; it is not a promise of a production
score, a field CWV result or a ranking gain. The original production Button CLS
was 0.527/0.440. The controlled delayed-script check verifies the menu cause has
been removed locally.

## Reproduce the native recipe verification

```sh
cargo build -p kit-docs-integration --bin kit-docs-table --locked
SLINT_BACKEND=winit SLINT_RENDERER=skia SLINT_SCALE_FACTOR=1 \
  target/debug/kit-docs-table --verify
# Repeat with --fr, --dark and --text-200, including their combinations.
node docs/scripts/table-provenance.mjs
npm --prefix docs run build
npm --prefix docs run check:browser
```

`check:browser` uses an installed Chrome in a temporary profile and a loopback
server. Set `CHROME_PATH` if Chrome is installed elsewhere. It does not use the
user's signed-in Chrome profile. The original typed-example command remains the
default binary for `cargo run -p kit-docs-integration`.

## Follow-up after publication

Search Console already reported the sitemap as successful with 260 discovered
pages. Its index report dated October 22 showed 187 indexed URLs, 69 HTML pages
awaiting exploration, four crawled HTML pages not indexed, and 17 non-indexed
downloads. The four HTML pages were individually inspected: successful fetch,
indexing allowed and matching canonical. This does not establish a specific
content-quality cause.

After deployment, confirm the two new guide URLs appear in the sitemap and the
mobile menu remains stable on production. Then compare the 69 waiting pages and
four crawled pages after one to two weeks. Use the initial 177 impressions and
one click as a small baseline, with comparable reporting periods; no automatic
resubmission or scheduled account action is required by this change.

Cloudflare warnings were conditional observations, with no demonstrated block
of Googlebot. They do not justify weakening the site's protections. `llms.txt`
and `lastmod` were optional audit notes, not blockers; no speculative policy or
fabricated modification dates have been added.
