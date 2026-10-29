#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build on-demand browser previews from the public Rust starter sources."""
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import importlib.util

DOCS = Path(__file__).resolve().parents[1]
ROOT = DOCS.parent
BINDGEN_VERSION = '0.2.129'


def run(*args, **kwargs):
    return subprocess.run(args, check=True, cwd=ROOT, **kwargs)


def build():
    tools_spec = importlib.util.spec_from_file_location('web_tools', DOCS / 'scripts/web-tools.py')
    tools = importlib.util.module_from_spec(tools_spec)
    tools_spec.loader.exec_module(tools)
    tools.prepare()
    bindgen = os.environ.get('WASM_BINDGEN', 'wasm-bindgen')
    version = subprocess.check_output([bindgen, '--version'], text=True).strip()
    if version != f'wasm-bindgen {BINDGEN_VERSION}':
        raise SystemExit(f'Install wasm-bindgen-cli {BINDGEN_VERSION}; found {version}')
    target = Path(os.environ.get('SLINT_WEB_TARGET_DIR', ROOT / 'target/doc-previews')).resolve()
    environment = {**os.environ, 'CARGO_TARGET_DIR': str(target)}
    environment.pop('SLINT_PREVIEW_COMPONENTS', None)
    run('cargo', 'build', '--manifest-path', 'docs/preview/Cargo.toml', '--locked',
        '--release', '--target', 'wasm32-unknown-unknown', env=environment)
    output = DOCS / '.generated/web'
    shutil.rmtree(output, ignore_errors=True)
    temporary = output / 'build'
    temporary.mkdir(parents=True)
    run(bindgen, '--target', 'web', '--out-dir', str(temporary), '--out-name',
        'slint_kit_previews', str(target / 'wasm32-unknown-unknown/release/slint_kit_doc_previews.wasm'))
    # Archive the corresponding application, build tools and local platform fix.
    # Dependencies remain pinned by Cargo.lock and available from their source registries.
    files = [ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md']]
    files += [p for p in (ROOT / 'crates').rglob('*') if p.is_file() and 'target' not in p.parts]
    files += [p for p in (DOCS / 'preview').rglob('*') if p.is_file() and 'target' not in p.parts]
    files += [DOCS / 'scripts/build-web-previews.py', DOCS / 'scripts/web-tools.py', DOCS / 'content/starters.json']
    for slug in json.loads((DOCS / 'content/starters.json').read_text()):
        files += [DOCS / 'quickstart' / slug / name for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'src/main.rs', 'ui/app.slint']]
    with (temporary / 'source.tar.gz').open('wb') as compressed, gzip.GzipFile(fileobj=compressed, mode='wb', mtime=0, filename='') as gzip_stream, tarfile.open(fileobj=gzip_stream, mode='w') as archive:
        for file in sorted(set(files)):
            content = file.read_bytes()
            if file == ROOT / 'Cargo.toml':
                # The standalone archive contains only the library workspace.
                # Preview compilation uses precisely the same library sources.
                content = content.replace(b', \"showcase\", \"tests\", \"xtask\", \"docs/capture\", \"docs/integration\"', b'')
            info = tarfile.TarInfo('slint-kit/' + str(file.relative_to(ROOT)))
            info.mode = 0o644
            info.size = len(content)
            info.uid = info.gid = 0
            info.uname = info.gname = ''
            info.mtime = 0
            archive.addfile(info, io.BytesIO(content))
    shutil.copyfile(DOCS / 'preview/NOTICE.txt', temporary / 'NOTICE.txt')
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--manifest-path', str(DOCS / 'preview/Cargo.toml'), '--filter-platform', 'wasm32-unknown-unknown', '--locked', '--format-version', '1'], cwd=ROOT))
    notices = ['Third-party Rust dependency notices\nSlint is used under its GPL-3.0-only option for this distribution.\n']
    for package in sorted(metadata['packages'], key=lambda value: (value['name'], value['version'])):
        if not package.get('source'):
            continue
        folder = Path(package['manifest_path']).parent
        notices.append('\n' + '=' * 72 + '\n' + package['name'] + ' ' + package['version'] + '\n' + (package.get('license') or 'See original package license') + '\n' + (package.get('repository') or package.get('source')) + '\n')
        for file in sorted(folder.iterdir()):
            if file.is_file() and file.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE')):
                notices.append('\n--- ' + file.name + ' ---\n' + file.read_text(errors='replace') + '\n')
    (temporary / 'THIRD_PARTY_NOTICES.txt').write_text(''.join(notices))
    digest = hashlib.sha256()
    for file in sorted(temporary.iterdir()):
        digest.update(file.name.encode() + b'\0' + file.read_bytes())
        if file.stat().st_size > 25 * 1024 * 1024:
            raise SystemExit(f'Preview asset exceeds Cloudflare limit: {file.name}')
    fingerprint = digest.hexdigest()[:20]
    temporary.rename(output / fingerprint)
    manifest = {'base': '/assets/previews/' + fingerprint, 'components': sorted(json.loads((DOCS / 'content/starters.json').read_text())), 'bindgen': BINDGEN_VERSION, 'source_sha256': {str(file.relative_to(ROOT)): hashlib.sha256(file.read_bytes()).hexdigest() for file in sorted(set(files))}}
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Browser previews: {len(manifest["components"])} components, {manifest["base"]}')


if __name__ == '__main__':
    build()
