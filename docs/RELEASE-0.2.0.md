# Slint Kit 0.2.0 and Retina documentation images

Release tag: `v0.2.0` at `3c49410b48edd2fdb67b541e336aca8a54b8970b`.
[Publication workflow](https://github.com/damione1/slint-kit/actions/runs/40618554847).

The five libraries (`slint-kit-core`, `slint-kit-slint`, `slint-kit-native`,
`slint-kit-build`, `slint-kit`) were published in dependency order. Each registry
archive was downloaded and checked against the locally verified release archive:
checksum, clean VCS revision, normalized manifest, license and source bytes.
The workflow ran release safeguards, the official workspace gate, bilingual docs
checks and verification of all five package archives before publication.

Internal workspace requirements are exactly `=0.2.0`. Public application examples,
crate READMEs, the consumption skill and bilingual guides now use the matching
`"0.2"` runtime/build packages. Example applications keep their own package
version. The migration guide explains that `"0.1"` does not automatically select
0.2. The former source-only warnings have been replaced with release guidance.
The site deployment follows successful registry publication.

## Screenshot sharpness

The previous images had one pixel per logical display point. Documentation now
keeps those 1x baselines and supplies a separate native 2x source through `srcset`.
The browser chooses the denser image on Retina displays while the HTML width and
height continue to reserve the same logical size. Preview CSS preserves those
attributes. The 2x images were freshly rendered by Slint, rather than produced
by resizing the 1x pixels.

All 246 localized component previews have native `@2x.png` counterparts captured
through the embedded Slint MCP server, with winit/Skia, scale factor 2 and explicit
logical viewports matching the 1x baseline. Their source, pixel density, PNG
geometry and hashes are recorded in `content/captures.json`. The catalogue now
records 572 PNGs, including the existing light/dark and enlarged-text variants.
Representative native 2x renders and actual browser previews were inspected;
this is not a claim that every Retina image was individually reviewed.

The typed and table applications also have 1x/2x native captures in both
languages. The typed application now permits its declared 560x480 window size,
which previously contracted to its inferred content maximum; its 2x capture is
1120x960. Both the downloadable and verification UI sources match. Their actual
field, selection, sorting and keyboard interactions were exercised after capture.

## Validation

- Release safeguards: seven passing checks; tagged release identity validated.
- `cargo run -p xtask --locked -- check`: passed, including 142 tests,
  formatting, generated contracts and Clippy, after the capture/viewport changes.
- All five `cargo package --locked` archives built and verified locally and in
  the publication workflow; registry archives match the tagged source.
- Both standalone downloadable applications compile and test against crates.io
  0.2.0 without workspace paths. The table identity regression passes.
- Native table: eight language/theme/text scenarios at 1x plus both localized
  baseline scenarios at 2x; all interactions pass. Native typed application:
  both languages at 1x and 2x; all interactions pass.
- Documentation build: 123 components, 246 localized references, 264 canonical
  pages; native 2x image dimensions, provenance, `srcset`, downloads, metadata,
  internal links and language alternates verified.
- Browser: 22 passing scenarios, including six Retina cases checking the actual
  `currentSrc` for Button, table and typed previews in both languages. Initial
  CLS is 0, including delayed 1x/2x preview images and script loading. No-JavaScript
  navigation, native disclosure keyboard/pointer input, catalogue filtering,
  dark mode and enlarged text also pass.

Environment: macOS 27.0, Apple M4 Pro, Slint 1.18.1, native winit/Skia at pixel
densities 1 and 2. This does not qualify IME, a screen reader or other platforms.
Detailed registry verification and deployment receipts are retained under ignored
`docs/.verification/`. See [capture reproduction](capture/README.md).
