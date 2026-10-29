# Public documentation deployment

The canonical origin is `https://slint-kit.dev`. `docs/scripts/routes.mjs`
controls canonical URLs, language alternatives, structured data and the sitemap.
The `slint-kit` Cloudflare Worker and its custom domains are configured in the
`docs/wrangler.toml`; deploy from that configuration so the migration
handler and asset binding are retained.

From the repository root:

```sh
npm ci --prefix docs
# Requires Rust's wasm32-unknown-unknown target and wasm-bindgen-cli 0.2.129.
npm --prefix docs run build
cargo run -p xtask --locked -- check
npx wrangler deploy --config docs/wrangler.toml
```

`docs/worker.mjs` permanently redirects `slint-kit.damiengoehrig.ca` and
`www.slint-kit.dev` to the canonical HTTPS origin, preserving paths and query
parameters. Legacy `/en` prefixes and retired source-update guides resolve in
the same redirect. Other requests are served by the ASSETS binding, including
normal 404 responses. `run_worker_first` is required for host redirects to run
before matching static files. `docs/src/_redirects` handles legacy paths on the
canonical host. `npm run build:pages` includes migration regression checks.

The domain was migrated on 2026-10-29 from the committed 0.3 documentation
revision `5ca2e34fb17fb8f483b7f4691dfe9c18c946696c`. Keep the old hostname, its
certificate and redirects for at least a year (preferably indefinitely). Keep
Google's DNS verification TXT record and both Search Console properties.
Monitor the new sitemap and indexing reports as Google recrawls the site.
The new domain is verified separately; the Change of Address action must target
only `slint-kit.damiengoehrig.ca` (or its HTTPS URL-prefix property), never the
parent `damiengoehrig.ca` property.

Workers Builds runs `npm run build` in `docs`. In that environment only,
`WORKERS_CI=1` provisions Rust 1.98.1, the wasm32 target and wasm-bindgen
0.2.129 under ignored `.cache/tools`; release downloads are checksum-checked.
Local builds continue to use the developer's installed tools.
Workers Builds uses Thin LTO and 16 codegen units for the generated preview
bundle, instead of the monolithic full-LTO profile used by local release builds.
This limits the resources required to optimize all 123 demos together.

## Persistent build cache

Cloudflare caches Eleventy's `docs/.cache` directory when build caching is
enabled (npm's cache alone does not retain Rust compilation). In Workers Builds,
Rust tools, the Cargo registry and compilation target live under this directory.
GitHub Actions separately restores `docs/.cache/previews`, in addition to its
existing Rust dependency cache.

The final JavaScript/WASM artifacts are cached by source contents, actual Rust
compiler identity, wasm-bindgen version and compilation flags. A fresh checkout's
timestamps do not force a recompile. Modifying a starter, kit source, lockfile,
toolchain or build configuration invalidates the cache. Ordinary website text
and screenshot changes reuse the compiled previews. Asset checksums reject a
corrupted cache entry. Source archives and dependency notices are regenerated
on every build so downloads still match the current source.

Logs explicitly report `WASM cache hit` or `WASM cache miss`. An expired or evicted
provider cache causes a normal cold build. Run the cache regressions with:

```sh
python3 -m unittest discover -s docs/scripts -p 'test_web_cache.py'
```
