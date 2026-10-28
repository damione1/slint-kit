# Native documentation captures

The capture application compiles the exact files in `docs/examples` through the
public `@kit` imports. It uses Slint 1.18.1, **winit/Skia** and an active desktop.
Examples are presentation fixtures; the integration applications additionally
exercise real Rust controllers.

Build the embedded MCP server for development without changing the library’s
default features:

```sh
SLINT_EMIT_DEBUG_INFO=1 cargo build --locked -p kit-docs-capture --features slint/mcp
```

Open one example for interactive inspection:

```sh
SLINT_MCP_PORT=9315 SLINT_SCALE_FACTOR=1 target/debug/kit-docs-capture button --show
```

The server listens on `http://127.0.0.1:9315/mcp` (Streamable HTTP).
`list_windows` supplies window handles; `get_window_properties` supplies the root
element handle. Inspect with `get_element_tree`, interact with `click_element`
and `dispatch_key_event`, then call `take_screenshot`. Use a free loopback port
and close the example when inspection is finished. This endpoint is a tool
server, not a browser page.

The Python helper starts and closes a native example for each image. With no
slugs it captures the full catalogue in both languages. It allocates a separate
loopback port per process and writes the actual MCP screenshot bytes:

```sh
python3 docs/capture/mcp.py --binary target/debug/kit-docs-capture --existing-variants --exercise
# Render Retina pixels natively, using the existing logical 1x viewports:
python3 docs/capture/mcp.py --binary target/debug/kit-docs-capture --pixel-density 2
# Or capture selected components:
python3 docs/capture/mcp.py calendar date-picker --binary target/debug/kit-docs-capture
python3 docs/capture/provenance.py --method mcp
npm --prefix docs run build
```

`--pixel-density 2` sets the native scale factor to 2 and requests the same
logical window size as each 1x baseline. It saves `@2x.png` files for the
website’s `srcset`; width/height attributes continue using the 1x dimensions.
The docs build rejects a 2x image with a different logical viewport. This is a
fresh native render, with text rasterized at the target density.

`--existing-variants` renews existing dark, 200% text and dark/200% files as well
as the baseline. `--exercise` verifies Button pointer and Tab/Space activation
in separate fresh processes, in both languages. `--binary` also supports a
shared `CARGO_TARGET_DIR`. Inspect the resulting PNGs before recording a visual
review count with `provenance.py --reviewed-components N`; never infer that count
from successful compilation or capture.

For capture without MCP, the application also supports `--all`, `--dark`,
`--text-200`, `--check` and `--exercise`; see `--help`. Use `provenance.py --method snapshot`
for that direct capture workflow. A 200% capture uses a larger
viewport so typography can grow. IME, screen-reader, Android-device and other
backend qualification require their own actual target checks.
