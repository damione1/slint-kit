# Vendored Lucide icons

`<version>/` is an unmodified copy of `icons/`, `categories/` and `LICENSE`
from the Lucide release named in its `VERSION` file (tag, commit, source
archive and its SHA-256). Keep exactly one version here.

`cargo run -p xtask -- generate-icons` turns these SVG files into
`crates/kit-slint/ui/icons/lucide/*.slint`; `cargo run -p xtask -- contracts`
(and therefore `check`) fails when those files differ from what this snapshot
generates. Never edit the generated files.

To upgrade, replace the directory with the new release, update `VERSION`,
regenerate and review the diff: a removed or renamed icon breaks consumers and
needs a minor kit version with the names listed in the changelog; additions are
a patch. The notices in `LICENSE` must stay in `THIRD-PARTY-NOTICES.md`.
