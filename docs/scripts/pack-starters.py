#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Package deterministic standalone consumers, retaining unchanged archives."""
import io
import json
import zipfile
import tomllib
from doc_inputs import ROOT, DOCS, write_changed
from starter_sources import FILES, prepare

output = DOCS / '.generated/starters'
output.mkdir(parents=True, exist_ok=True)
consumers = prepare()
changed = 0
for slug, folder in consumers.items():
    files = {name: (folder / name).read_bytes() for name in FILES}
    dependency = tomllib.loads(files['Cargo.toml'].decode())['dependencies']['slint-kit']
    kit_version = (dependency if isinstance(dependency, str) else dependency['version']).removeprefix('=')
    files['LICENSE'] = (ROOT / 'LICENSE').read_bytes()
    files['README.md'] = f'# Slint Kit: {slug}\n\nRequires Rust 1.92+. Run `cargo run --locked` in this folder.\n\nThis native example consumes Slint Kit {kit_version} and Slint 1.18.1 from crates.io.\n\nhttps://slint-kit.dev/components/{slug}/\n'.encode()
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(f'{slug}/{name}', date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    changed += write_changed(output / f'{slug}.zip', stream.getvalue())
for path in output.glob('*.zip'):
    if path.stem not in consumers:
        path.unlink()
print(f'Starter archives: {changed} changed, {len(consumers) - changed} reused')
