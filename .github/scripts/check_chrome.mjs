// SPDX-License-Identifier: GPL-3.0-or-later
// Exercise the actual browser and renderer before a long documentation build.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

const chrome = process.env.CHROME_PATH;
assert(chrome, 'CHROME_PATH must identify the pinned capture browser');
const profile = fs.mkdtempSync(path.join(os.tmpdir(), 'slint-docs-chrome-check-'));
try {
  const document = '<canvas></canvas><script>document.body.dataset.webgl=Boolean(document.querySelector("canvas").getContext("webgl2"))</script>';
  const result = spawnSync(chrome, [
    '--headless=new', '--no-first-run', '--no-default-browser-check',
    '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
    `--user-data-dir=${profile}`, '--dump-dom',
    `data:text/html,${encodeURIComponent(document)}`,
  ], {encoding: 'utf8', timeout: 30000, maxBuffer: 1024 * 1024});
  assert.ifError(result.error);
  assert.equal(result.status, 0, `Chrome startup failed: ${result.stderr}`);
  assert.match(result.stdout, /data-webgl="true"/, `Chrome cannot create WebGL2: ${result.stdout}\n${result.stderr}`);
  console.log('Pinned Chrome startup and WebGL2 passed with the sandbox enabled.');
} finally {
  fs.rmSync(profile, {recursive: true, force: true, maxRetries: 5, retryDelay: 100});
}
