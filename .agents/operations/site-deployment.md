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

## Automatic deployment

GitHub Actions is the build and deployment owner. The Check workflow runs the
workspace gate and builds the documentation, records the source SHA in
`deployment-revision.txt`, and uploads the validated `_site` directory as an
artifact named for that SHA. The deployment job waits for both checks, downloads
that exact artifact, verifies its revision, and runs Wrangler with
`docs/wrangler.toml`. It performs no Rust compilation. Deployments are serialized;
obsolete main revisions are skipped before deployment. Pull requests and tags
do not deploy. `workflow_dispatch` can retry main after configuring credentials.

One-time account setup:

1. Create a scoped Cloudflare API token for the existing account, with Workers
   Scripts Edit, Workers Routes Edit and Zone Read for the two managed zones
   (`slint-kit.dev` and `damiengoehrig.ca`). The Cloudflare “Edit Cloudflare
   Workers” token template is the starting point.
2. Add it as the GitHub repository Actions secret `CLOUDFLARE_API_TOKEN`. Set the
   Actions variable `CLOUDFLARE_ACCOUNT_ID` to `38a68fae0765d8ec046acaaed0cbf111`.
   The account ID is public; the token belongs in the secret, not source code.
3. In the `slint-kit` Worker’s Settings > Builds, disconnect the Git repository
   so Workers Builds stops launching a second Rust compilation. Existing Worker
   deployments and custom domains remain in place. GitHub Actions then owns
   future deployments through Wrangler.

Workers Builds timed out on a cold compile of the generated preview bundle.
Caching alone cannot fix a cold build that never finishes and therefore never
populates its cache. The GitHub documentation build completes successfully and
provides the deployable artifact. No Cloudflare-side Rust bootstrap is required.

## Persistent build cache

GitHub Actions restores `docs/.cache/previews`, in addition to its Rust dependency
cache. Local builds use the same preview cache directory. The preview crate uses
Thin LTO and 16 codegen units on both local and GitHub builds.

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
