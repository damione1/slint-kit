# Public documentation deployment

The canonical origin is `https://slint-kit.dev`. `docs/scripts/routes.mjs`
controls canonical URLs, language alternatives, structured data and the sitemap.
The `slint-kit` Cloudflare Worker and its custom domains are configured in the
repository-root `wrangler.toml`; deploy from that configuration so the migration
handler and asset binding are retained.

From the repository root:

```sh
npm ci --prefix docs
# Requires Rust's wasm32-unknown-unknown target and wasm-bindgen-cli 0.2.129.
npm --prefix docs run build
cargo run -p xtask --locked -- check
npx wrangler deploy --config wrangler.toml
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
only `slint-kit.damiengoehrig.ca`, never the parent `damiengoehrig.ca` property.
