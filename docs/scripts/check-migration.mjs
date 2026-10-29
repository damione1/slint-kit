// SPDX-License-Identifier: GPL-3.0-or-later
import assert from 'node:assert/strict';
import worker from '../worker.mjs';

const assets = { fetch: async () => new Response('asset', { status: 200 }) };
for (const host of ['slint-kit.damiengoehrig.ca', 'www.slint-kit.dev']) {
  for (const [from, to] of [
    ['/', '/'],
    ['/fr/components/button/?ref=old&lang=fr', '/fr/components/button/?ref=old&lang=fr'],
    ['/assets/components/button%402x.png', '/assets/components/button%402x.png'],
    ['/en', '/'], ['/en/components/button/', '/components/button/'],
    ['/fr/guides/source-updates/', '/fr/guides/getting-started/'],
    ['/en/guides/source-updates/', '/guides/getting-started/'],
    ['/quickstart/button/project.tar.gz', '/quickstart/button/project.tar.gz'],
    ['/does-not-exist', '/does-not-exist'],
  ]) {
    const response = await worker.fetch(new Request(`https://${host}${from}`), { ASSETS: assets });
    assert.equal(response.status, 301);
    assert.equal(response.headers.get('location'), `https://slint-kit.dev${to}`);
  }
}
const response = await worker.fetch(new Request('https://slint-kit.dev/fr/'), { ASSETS: assets });
assert.equal(response.status, 200);
assert.equal(await response.text(), 'asset');
const insecure = await worker.fetch(new Request('http://slint-kit.dev/fr/?ref=http'), { ASSETS: assets });
assert.equal(insecure.status, 301);
assert.equal(insecure.headers.get('location'), 'https://slint-kit.dev/fr/?ref=http');
console.log('Domain migration: 19 redirects preserve paths/query strings and enforce HTTPS; canonical host serves assets.');
