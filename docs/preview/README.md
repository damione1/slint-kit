# Browser previews

The documentation runs its Rust starter applications in a canvas using Slint’s
WebAssembly backend. Previews load only after the reader chooses **Run preview**.
Native file dialogs and OS drag-and-drop require the downloaded native project.

Install the matching build tools, then build the site:

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
npm ci --prefix docs
npm --prefix docs run build:web
```

`WASM_BINDGEN` can select an existing wasm-bindgen executable.
`SLINT_WEB_TARGET_DIR` can select a shared Cargo build directory.
`npm --prefix docs run build` also builds previews; `build:pages` rebuilds the
HTML using the existing generated previews.

The preview package compiles each exact `docs/quickstart/<component>/ui/app.slint`
and its Rust `main` as an isolated module. The browser package uses the local
kit adapter for its browser clock fix; downloadable native starters consume
crates.io. `NOTICE.txt` and the corresponding source archive accompany each
content-addressed preview build. See Slint’s [web backend documentation](https://docs.slint.dev/latest/docs/slint/guide/platforms/web/).
