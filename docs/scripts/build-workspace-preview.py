#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build the exact workspace consumer, its GPL source bundle and integrity record."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import tempfile
from doc_inputs import ROOT, DOCS, compiler_environment, source_hashes, write_changed, write_json

OUTPUT = DOCS / '.generated/workspace-preview'
MANIFEST = DOCS / 'workspace-preview/Cargo.toml'
TARGET = DOCS / 'workspace-preview/target'
ASSETS = ('workspace.js', 'workspace_bg.wasm')
DOWNLOADS = ('source.tar.gz', 'LICENSE', 'NOTICE.txt', 'THIRD_PARTY_NOTICES.txt')


def checksum(data):
    return hashlib.sha256(data).hexdigest()


def source_files():
    files = [ROOT / name for name in ('Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md', 'THIRD-PARTY-NOTICES.md', 'rust-toolchain.toml')]
    files += [path for directory in ('crates', 'vendor') for path in (ROOT / directory).rglob('*')
              if path.is_file() and 'target' not in path.parts and path.name != '.DS_Store']
    files += [DOCS / name for name in ('src/assets/examples/workspace.slint', 'src/assets/examples/workspace.rs',
              'src/examples/workspace.njk', 'workspace-preview/Cargo.toml', 'workspace-preview/Cargo.lock',
              'workspace-preview/build.rs', 'workspace-preview/src/lib.rs', 'scripts/build-workspace-preview.py',
              'scripts/doc_inputs.py', 'scripts/presentation.py')]
    return sorted(set(files))


def source_archive(destination, files):
    def append(archive, name, content):
        info = tarfile.TarInfo('slint-kit/' + name)
        info.mode, info.size, info.mtime = 0o644, len(content), 0
        archive.addfile(info, io.BytesIO(content))
    with destination.open('wb') as stream, gzip.GzipFile(fileobj=stream, mode='wb', mtime=0, filename='') as compressed, tarfile.open(fileobj=compressed, mode='w') as archive:
        for path in files:
            content = path.read_bytes()
            if path == ROOT / 'Cargo.toml':
                members = [str(path.parent.relative_to(ROOT)) for path in sorted((ROOT / 'crates').glob('*/Cargo.toml'))]
                content = re.sub(r'(?m)^members = \[.*\]$', 'members = ' + json.dumps(members), content.decode()).encode()
            append(archive, str(path.relative_to(ROOT)), content)
        append(archive, 'REBUILD-WORKSPACE.txt', b'''Corresponding source for the Slint Kit workspace WASM demonstration.
License: GPL-3.0-or-later; dependency notices are shipped beside the module.
Install the pinned Rust toolchain, wasm32-unknown-unknown target and wasm-bindgen-cli 0.2.129.
From slint-kit/ run:
  cargo build --manifest-path docs/workspace-preview/Cargo.toml --locked --release --target wasm32-unknown-unknown
  wasm-bindgen --target web --out-dir web --out-name workspace docs/workspace-preview/target/wasm32-unknown-unknown/release/slint_kit_workspace_preview.wasm
The exact Slint/Rust consumer and browser configuration are under docs/src/assets/examples/ and docs/workspace-preview/.
Workspace membership in the root manifest is narrowed to the bundled libraries; package source and compilation settings are unchanged.
''')


def dependency_notices():
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--manifest-path', str(MANIFEST), '--filter-platform', 'wasm32-unknown-unknown', '--locked', '--format-version', '1'], cwd=ROOT))
    notices = ['Third-party Rust dependency notices\nSlint is used under its GPL-3.0-only option for this distribution.\n']
    notices.append((ROOT / 'THIRD-PARTY-NOTICES.md').read_text())
    for package in sorted(metadata['packages'], key=lambda package: (package['name'], package['version'])):
        if not package.get('source'):
            continue
        notices.append('\n' + '=' * 72 + '\n' + package['name'] + ' ' + package['version'] + '\n' + (package.get('license') or 'See package license') + '\n' + (package.get('repository') or package['source']) + '\n')
        for path in sorted(Path(package['manifest_path']).parent.iterdir()):
            if path.is_file() and path.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE')):
                notices.append('\n--- ' + path.name + ' ---\n' + path.read_text(errors='replace'))
    return ''.join(notices)


def verify():
    record = json.loads((OUTPUT / 'manifest.json').read_text())
    assert record['schema'] == 1
    assert set(record['assets']) == set(ASSETS) and set(record['downloads']) == set(DOWNLOADS)
    folder = OUTPUT / record['base'].rsplit('/', 1)[1]
    for group in ('assets', 'downloads'):
        for name, expected in record[group].items():
            data = (folder / name).read_bytes()
            assert len(data) == expected['bytes'] and checksum(data) == expected['sha256'], name
    assert record['sources'] == source_hashes(source_files()), 'Workspace source bundle is stale'
    assert record['module'] == record['base'] + '/workspace.js'
    identity = checksum(json.dumps({**record['assets'], **record['downloads']}, sort_keys=True).encode())[:20]
    assert folder.name == identity, 'Workspace content-address does not match its assets'
    print('Workspace module, sources, licenses and integrity match.')


def build():
    bindgen = os.environ.get('WASM_BINDGEN', 'wasm-bindgen')
    environment = {**compiler_environment(), 'CARGO_TARGET_DIR': str(TARGET)}
    files = source_files()
    producer = {name: subprocess.check_output(command, text=True).strip() for name, command in {
        'rustc': ['rustc', '--version', '--verbose'], 'cargo': ['cargo', '--version'],
        'bindgen': [bindgen, '--version'],
    }.items()}
    flags = {name: value for name, value in environment.items() if name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')
             or name.startswith(('CARGO_PROFILE_RELEASE_', 'CARGO_TARGET_WASM32_'))}
    context = {'sources': source_hashes(files), 'producer': producer, 'flags': flags,
               'target': 'wasm32-unknown-unknown', 'profile': 'release', 'style': 'fluent',
               'resources': 'embedded', 'debug': True, 'sourceRoot': str(ROOT)}
    input_identity = checksum(json.dumps(context, sort_keys=True).encode())
    try:
        previous = json.loads((OUTPUT / 'manifest.json').read_text())
        if previous.get('inputIdentity') == input_identity:
            verify()
            print(f'Workspace preview reused from {previous["revision"]}: {previous["base"]}')
            return
    except (OSError, ValueError, KeyError, AssertionError):
        pass
    subprocess.run(['cargo', 'build', '--manifest-path', str(MANIFEST), '--locked', '--release', '--target', 'wasm32-unknown-unknown'], cwd=ROOT, env=environment, check=True)
    revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    with tempfile.TemporaryDirectory(prefix='workspace-preview-') as directory:
        temporary = Path(directory)
        subprocess.run([bindgen, '--target', 'web', '--out-dir', str(temporary), '--out-name', 'workspace', str(TARGET / 'wasm32-unknown-unknown/release/slint_kit_workspace_preview.wasm')], check=True)
        assert context['sources'] == source_hashes(files), 'Workspace sources changed during compilation'
        source_archive(temporary / 'source.tar.gz', files)
        (temporary / 'LICENSE').write_bytes((ROOT / 'LICENSE').read_bytes())
        (temporary / 'NOTICE.txt').write_text(f'Slint Kit workspace demonstration\nGPL-3.0-or-later\nSource revision: {revision}\nCorresponding source: source.tar.gz\nSlint dependency uses GPL-3.0-only. See THIRD_PARTY_NOTICES.txt.\n')
        (temporary / 'THIRD_PARTY_NOTICES.txt').write_text(dependency_notices())
        entries = {name: {'sha256': checksum((temporary / name).read_bytes()), 'bytes': (temporary / name).stat().st_size} for name in ASSETS + DOWNLOADS}
        identity = checksum(json.dumps(entries, sort_keys=True).encode())[:20]
        for name in entries:
            assert entries[name]['bytes'] <= 25 * 1024 * 1024, f'{name}: hosting file limit'
            write_changed(OUTPUT / identity / name, (temporary / name).read_bytes())
        base = '/assets/workspace-preview/' + identity
        record = {'schema': 1, 'inputIdentity': input_identity, 'producer': producer, 'flags': flags, 'base': base, 'module': base + '/workspace.js', 'revision': revision,
                  'sources': source_hashes(files), 'kitSourceHashes': source_hashes([path for path in files if 'crates' in path.parts]),
                  'assets': {name: entries[name] for name in ASSETS}, 'downloads': {name: entries[name] for name in DOWNLOADS}}
        write_json(OUTPUT / 'manifest.json', record)
        verify()
        print(f'Workspace preview {identity}: {entries["workspace_bg.wasm"]["bytes"]} WASM bytes; sources/licenses bundled.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    build() if not parser.parse_args().check else verify()
