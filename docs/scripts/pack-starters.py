#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Package the exact documented consumer sources as standalone projects."""
from pathlib import Path
import json
import shutil
import zipfile
import tomllib

root = Path(__file__).resolve().parents[2]
docs = root / 'docs'
output = docs / '.generated/starters'
shutil.rmtree(output, ignore_errors=True)
output.mkdir(parents=True)
for slug in json.loads((docs / 'content/starters.json').read_text()):
    folder = docs / 'quickstart' / slug
    files = {name: (folder / name).read_bytes() for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'src/main.rs', 'ui/app.slint']}
    dependency = tomllib.loads(files['Cargo.toml'].decode())['dependencies']['slint-kit']
    kit_version = (dependency if isinstance(dependency, str) else dependency['version']).removeprefix('=')
    files['LICENSE'] = (root / 'LICENSE').read_bytes()
    files['README.md'] = f'# Slint Kit: {slug}\n\nRequires Rust1.92+. Run `cargo run --locked` in this folder.\n\nThis native example consumes Slint Kit{kit_version} and Slint1.18.1 from crates.io.\n\nhttps://slint-kit.dev/components/{slug}/\n'.replace('Rust1.92', 'Rust 1.92').replace('Kit0.3', 'Kit 0.3').replace('Slint1.18', 'Slint 1.18').encode()
    with zipfile.ZipFile(output / f'{slug}.zip', 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(f'{slug}/{name}', date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
print(f'Packaged {len(list(output.glob("*.zip")))} runnable projects')
