# Slint Kit logo assets

Vector artwork licensed GPL-3.0-or-later. The repository README displays the
stacked logo and selects its light or dark variant to match the reader's GitHub theme.

| Asset | Intended use |
| --- | --- |
| [slint-kit-symbol.svg](slint-kit-symbol.svg) | Standalone transparent symbol, 512 × 512 viewBox |
| [slint-kit-horizontal.svg](slint-kit-horizontal.svg) | Horizontal logo with outlined wordmark |
| [slint-kit-stacked.svg](slint-kit-stacked.svg) | Stacked logo with outlined wordmark |
| [slint-kit-monochrome.svg](slint-kit-monochrome.svg) | Solid navy symbol |
| [slint-kit-white.svg](slint-kit-white.svg) | Solid white symbol for dark backgrounds |
| [slint-kit-icon.svg](slint-kit-icon.svg) | Color symbol inside a rounded navy app tile |

Horizontal and stacked logos have `-dark.svg` variants with white `Slint` and
light blue `Kit`. Production files contain paths and require no fonts, images,
stylesheets, scripts or network access. `slint-kit.svg` is a compatibility copy
of the standalone symbol.

The `-editable.svg` and `-dark-editable.svg` lockups retain live text using
Inter SemiBold (600) with −0.015 em tracking. Install Inter to edit them; fallback
fonts may change their appearance. Production outlines use Inter 4.001 at optical
size 14 and render independently of installed fonts. The font is not bundled;
its copyright notice and license are preserved in [Inter-OFL.txt](Inter-OFL.txt).

The symbol preserves the reference's tall proportions and approximately 29°
diagonals. Its visible bounds are about 324 × 400 units, centered in a 512 × 512
viewBox, with 11% minimum clear space. The connector sits behind both opaque
ribbons at 55% opacity. Single-color variants use one opaque path.

The SVG family was parsed as XML and rendered with Rust `resvg` at widths
16, 24, 32, 64, 128 and 512 pixels. Production assets were also visually inspected
in Chrome on light and dark backgrounds.
