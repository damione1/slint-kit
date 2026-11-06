// SPDX-License-Identifier: GPL-3.0-or-later
// Transport only: bindings and the single compiled module remain unmodified.
import initialize from './slint_kit_previews_bindings.js';
export * from './slint_kit_previews_bindings.js';

export default async function init(options) {
  if (options !== undefined) return initialize(options);
  const response = await fetch(new URL('./slint_kit_previews_bg.wasm.gz', import.meta.url));
  if (!response.ok || !response.body) throw new Error(`Preview download failed: ${response.status}`);
  const decoded = response.body.pipeThrough(new DecompressionStream('gzip'));
  return initialize({module_or_path: new Response(decoded, {headers: {'Content-Type': 'application/wasm'}})});
}
