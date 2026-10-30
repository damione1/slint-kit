#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build only invalidated component/theme previews, with independent public URLs."""
import argparse
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import fcntl
from functools import wraps

from doc_inputs import ROOT, DOCS, build_inputs, digest, runtime_inputs, write_changed, write_json, consumer_identity, compiler_environment
from presentation import ThemeFamily, themed_source
from web_cache import ASSETS, restore, save

BINDGEN_VERSION = '0.2.129'
EXTRA_ASSETS = ('source.tar.gz', 'NOTICE.txt', 'THIRD_PARTY_NOTICES.txt', 'component.json')
COMPILED_EXTRAS = ('source.tar.gz', 'component.json')


def code_key(slug, family, inputs, rustc, bindgen, environment):
    # Changes in another Slint declaration never enter this key. Rust adapters
    # and toolchains are conservative shared inputs because they can affect
    # callback behavior even when a UI declaration remains unchanged.
    source = DOCS / 'quickstart' / slug
    files = [source / 'src/main.rs', source / 'build.rs', DOCS / 'preview/build.rs',
             DOCS / 'preview/src/lib.rs', DOCS / 'preview/Cargo.toml', DOCS / 'preview/Cargo.lock']
    flags = {name: value for name, value in environment.items() if name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')
             or name.startswith(('CARGO_PROFILE_RELEASE_', 'CARGO_TARGET_WASM32_'))}
    host = {str(path.relative_to(ROOT)): digest(path.read_bytes()) for path in files}
    manifest = tomllib.loads((DOCS / 'preview/Cargo.toml').read_text())
    for section in ['dependencies', 'build-dependencies']:
        for name, value in manifest.get(section, {}).items():
            if name in ('slint-kit', 'slint-kit-build'):
                if isinstance(value, dict):
                    value['version'] = '<resolved-kit-sources>'
                else:
                    manifest[section][name] = '<resolved-kit-sources>'
    packages = tomllib.loads((DOCS / 'preview/Cargo.lock').read_text())['package']
    external = [package for package in packages if package.get('source') and not package['name'].startswith('slint-kit')]
    host['docs/preview/Cargo.toml'] = digest(manifest)
    host['docs/preview/Cargo.lock'] = digest(external)
    host['workspace-configuration'] = digest(tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['dependencies'].get('slint'))
    host['compiler-configuration'] = digest({'style': 'fluent', 'resources': 'embedded', 'debug': False})
    return digest({'schema': 2, 'slug': slug, 'family': family, 'ui': inputs['entries'][f'starter:{slug}:{family}']['identity'],
                   'host': host,
                   'consumer': consumer_identity(slug), 'runtime': digest(runtime_inputs()), 'rustc': rustc, 'bindgen': bindgen, 'flags': flags})


def content_hash(directory, names):
    value = hashlib.sha256()
    for name in sorted(names):
        value.update(name.encode() + b'\0' + (directory / name).read_bytes() + b'\0')
    return value.hexdigest()[:20]


def corresponding_sources(slug):
    files = [ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md']]
    files += [path for path in (ROOT / 'crates').rglob('*') if path.is_file() and 'target' not in path.parts]
    files += [path for path in (DOCS / 'preview').rglob('*') if path.is_file() and 'target' not in path.parts]
    files += [DOCS / 'scripts' / name for name in ['build-web-previews.py', 'web_cache.py', 'presentation.py', 'doc_inputs.py']]
    files += [DOCS / 'quickstart' / slug / name for name in ['Cargo.toml', 'Cargo.lock', 'build.rs', 'src/main.rs', 'ui/app.slint']]
    return sorted(set(files))


def archive_sources(directory, slug):
    files = corresponding_sources(slug)
    with (directory / 'source.tar.gz').open('wb') as compressed, gzip.GzipFile(fileobj=compressed, mode='wb', mtime=0, filename='') as gzip_stream, tarfile.open(fileobj=gzip_stream, mode='w') as archive:
        for path in files:
            content = path.read_bytes()
            if path == ROOT / 'Cargo.toml':
                # Keep the exact library workspace while removing absent apps.
                content = content.replace(b', "showcase", "tests", "xtask", "docs/capture", "docs/integration"', b'')
            if path == DOCS / 'quickstart' / slug / 'ui/app.slint':
                content = themed_source(content.decode(), ThemeFamily.TAILWIND.value).encode()
            info = tarfile.TarInfo('slint-kit/' + str(path.relative_to(ROOT)))
            info.mode, info.size, info.mtime = 0o644, len(content), 0
            archive.addfile(info, io.BytesIO(content))
    return {str(path.relative_to(ROOT)): digest(path.read_bytes()) for path in files}


def third_party_notices():
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--manifest-path', str(DOCS / 'preview/Cargo.toml'),
            '--filter-platform', 'wasm32-unknown-unknown', '--locked', '--format-version', '1'], cwd=ROOT))
    result = ['Third-party Rust dependency notices\nSlint is used under its GPL-3.0-only option for this distribution.\n']
    for package in sorted(metadata['packages'], key=lambda value: (value['name'], value['version'])):
        if not package.get('source'):
            continue
        result.append('\n' + '=' * 72 + '\n' + package['name'] + ' ' + package['version'] + '\n' + (package.get('license') or 'See original package license') + '\n' + (package.get('repository') or package.get('source')) + '\n')
        for path in sorted(Path(package['manifest_path']).parent.iterdir()):
            if path.is_file() and path.name.upper().startswith(('LICENSE', 'LICENCE', 'COPYING', 'COPYRIGHT', 'NOTICE')):
                result.append('\n--- ' + path.name + ' ---\n' + path.read_text(errors='replace') + '\n')
    return ''.join(result)


def publish_assets(temporary, output, slug, family):
    code_hash = content_hash(temporary, ASSETS)
    code_base = f'/assets/previews/{slug}/{family}/{code_hash}'
    for name in ASSETS:
        data = (temporary / name).read_bytes()
        if len(data) > 25 * 1024 * 1024:
            raise ValueError(f'Preview asset exceeds hosting limit: {slug}/{name}')
        write_changed(output / code_base.lstrip('/') / name, data)
    sources = {}
    for name in EXTRA_ASSETS:
        data = (temporary / name).read_bytes()
        if len(data) > 25 * 1024 * 1024:
            raise ValueError(f'Preview source asset exceeds hosting limit: {slug}/{name}')
        url = f'/assets/preview-sources/{digest(data)[:20]}/{name}'
        write_changed(output / url.lstrip('/'), data)
        sources[name] = url
    record = json.loads((temporary / 'component.json').read_text())
    return {'key': record['key'], 'base': code_base, 'sources': sources,
            'assets': {name: digest((temporary / name).read_bytes()) for name in ASSETS},
            'sourceAssets': {name: digest((temporary / name).read_bytes()) for name in EXTRA_ASSETS}}


def prune_generated(output, previews):
    referenced = set()
    for variants in previews.values():
        for entry in variants.values():
            if not isinstance(entry.get('sources'), dict):
                return
            referenced.update((entry['base'] + '/' + name).lstrip('/') for name in entry['assets'])
            referenced.update(url.lstrip('/') for url in entry['sources'].values())
    assets = output / 'assets'
    if not assets.exists():
        return
    for path in assets.rglob('*'):
        if path.is_file() and str(path.relative_to(output)) not in referenced:
            path.unlink()
    for path in sorted(assets.rglob('*'), reverse=True):
        if path.is_dir() and not any(path.iterdir()):
            path.rmdir()


def exclusive_build(function):
    @wraps(function)
    def locked(*arguments, **keywords):
        folder = DOCS / '.cache'
        folder.mkdir(parents=True, exist_ok=True)
        with (folder / 'preview-build.lock').open('a') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            return function(*arguments, **keywords)
    return locked


@exclusive_build
def build_fixture(task, inputs):
    """An exact catalogue fixture, compiled only when its old render is stale."""
    bindgen = os.environ.get('WASM_BINDGEN', 'wasm-bindgen')
    version = subprocess.check_output([bindgen, '--version'], text=True).strip()
    if version != f'wasm-bindgen {BINDGEN_VERSION}':
        raise ValueError('Unexpected wasm-bindgen version')
    rustc = subprocess.check_output(['rustc', '-vV'], cwd=ROOT, text=True).strip()
    target = Path(os.environ.get('SLINT_WEB_TARGET_DIR', ROOT / 'target/doc-previews')).resolve()
    environment = {**compiler_environment(), 'CARGO_TARGET_DIR': str(target)}
    key = digest({'schema': 1, 'scene': task['sceneKey'], 'rustc': rustc, 'bindgen': version,
                  'driver': digest((DOCS / 'preview/build.rs').read_bytes()),
                  'manifest': digest((DOCS / 'preview/Cargo.toml').read_bytes())})
    output = DOCS / '.generated/web'
    output.mkdir(parents=True, exist_ok=True)
    slug = task['slug']
    family = 'fixture-' + task['locale'] + '-' + task['recipe']['theme'] + '-' + str(task['recipe']['textScale'])
    with tempfile.TemporaryDirectory(prefix='fixture-', dir=output) as folder:
        temporary = Path(folder)
        if not restore(DOCS / '.cache/previews', key, temporary, EXTRA_ASSETS):
            source = DOCS / 'examples' / ('fr' if task['locale'] == 'fr' else '') / (slug + '.slint')
            text = source.read_text()
            if task['recipe']['theme'] == 'dark':
                text = text.replace('KitThemes.light', 'KitThemes.dark')
            ui_dir = DOCS / '.generated/preview-ui' / family
            write_changed(ui_dir / (slug + '.slint'), text)
            viewport = task['recipe']['viewport']
            env = {**environment, 'SLINT_PREVIEW_COMPONENTS': slug, 'SLINT_PREVIEW_SOURCE_DIR': str(ui_dir),
                   'SLINT_PREVIEW_FIXTURE': '1', 'SLINT_PREVIEW_TEXT_SCALE': str(task['recipe']['textScale']),
                   'SLINT_PREVIEW_VIEWPORT': f'{viewport[0]}x{viewport[1]}'}
            subprocess.run(['cargo', 'build', '--manifest-path', 'docs/preview/Cargo.toml', '--locked', '--release',
                            '--target', 'wasm32-unknown-unknown'], cwd=ROOT, env=env, check=True)
            subprocess.run([bindgen, '--target', 'web', '--out-dir', str(temporary), '--out-name', 'slint_kit_previews',
                            str(target / 'wasm32-unknown-unknown/release/slint_kit_doc_previews.wasm')], cwd=ROOT, check=True)
            hashes = archive_sources(temporary, slug)
            # Fixtures are an internal screenshot tool; record their exact UI too.
            write_json(temporary / 'component.json', {'key': key, 'slug': slug, 'scene': task['sceneKey'],
                       'source_sha256': hashes, 'fixtureSource': text,
                       'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()})
            write_changed(temporary / 'NOTICE.txt', (DOCS / 'preview/NOTICE.txt').read_bytes())
            write_changed(temporary / 'THIRD_PARTY_NOTICES.txt', third_party_notices())
            save(DOCS / '.cache/previews', key, temporary, EXTRA_ASSETS)
        return publish_assets(temporary, output, slug, family)


@exclusive_build

def build():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('slugs', nargs='*', help='Build a subset for development; production validation requires every component')
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    from starter_sources import prepare
    prepare()
    inputs = build_inputs()
    entries = json.loads((DOCS / 'content/starters.json').read_text())
    if not set(args.slugs) <= set(entries):
        parser.error('Unknown preview component')
    slugs = sorted(args.slugs or entries)
    bindgen = os.environ.get('WASM_BINDGEN', 'wasm-bindgen')
    version = subprocess.check_output([bindgen, '--version'], text=True).strip()
    if version != f'wasm-bindgen {BINDGEN_VERSION}':
        raise SystemExit(f'Install wasm-bindgen-cli {BINDGEN_VERSION}; found {version}')
    target = Path(os.environ.get('SLINT_WEB_TARGET_DIR', ROOT / 'target/doc-previews')).resolve()
    environment = {**compiler_environment(), 'CARGO_TARGET_DIR': str(target)}
    rustc = subprocess.check_output(['rustc', '-vV'], cwd=ROOT, text=True).strip()
    output = DOCS / '.generated/web'
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = output / 'manifest.json'
    old = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    previews = {slug: {'tailwind': entry['tailwind']} for slug, entry in old.get('entries', {}).items() if 'tailwind' in entry} if old.get('schema') == 2 else {}
    if args.check:
        for slug in slugs:
            for family in ThemeFamily:
                expected = code_key(slug, family.value, inputs, rustc, version, environment)
                if previews.get(slug, {}).get(family.value, {}).get('key') != expected:
                    raise SystemExit(f'Stale browser preview: {slug}/{family.value}; rebuild it')
        print(f'Browser preview keys verified: {len(slugs) * len(ThemeFamily)}')
        return
    cache = DOCS / '.cache/previews'
    notices = None
    hits = 0
    for slug in slugs:
        previews.setdefault(slug, {})
        for family in ThemeFamily:
            key = code_key(slug, family.value, inputs, rustc, version, environment)
            with tempfile.TemporaryDirectory(prefix='preview-', dir=output) as folder:
                temporary = Path(folder)
                restored = restore(cache, key, temporary, COMPILED_EXTRAS)
                if restored:
                    hits += 1
                    print(f'WASM reused {slug}/{family.value}', flush=True)
                else:
                    print(f'WASM build {slug}/{family.value}', flush=True)
                    ui_dir = DOCS / '.generated/preview-ui' / family.value
                    write_changed(ui_dir / f'{slug}.slint', themed_source((DOCS / 'quickstart' / slug / 'ui/app.slint').read_text(), family.value))
                    env = {**environment, 'SLINT_PREVIEW_COMPONENTS': slug, 'SLINT_PREVIEW_SOURCE_DIR': str(ui_dir)}
                    log_path = DOCS / '.verification/wasm' / f'{slug}-{family.value}.log'
                    log_path.parent.mkdir(parents=True, exist_ok=True)
                    with log_path.open('w') as log:
                        result = subprocess.run(['cargo', 'build', '--manifest-path', 'docs/preview/Cargo.toml', '--locked', '--release',
                            '--target', 'wasm32-unknown-unknown'], cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
                    if result.returncode:
                        raise RuntimeError(f'{slug}/{family.value} compilation failed; see {log_path}')
                    subprocess.run([bindgen, '--target', 'web', '--out-dir', str(temporary), '--out-name', 'slint_kit_previews',
                                    str(target / 'wasm32-unknown-unknown/release/slint_kit_doc_previews.wasm')], cwd=ROOT, check=True)
                    hashes = archive_sources(temporary, slug)
                    if notices is None:
                        notices = third_party_notices()
                    write_changed(temporary / 'NOTICE.txt', (DOCS / 'preview/NOTICE.txt').read_bytes())
                    write_changed(temporary / 'THIRD_PARTY_NOTICES.txt', notices)
                    write_json(temporary / 'component.json', {'key': key, 'slug': slug, 'theme': family.value,
                            'uiIdentity': inputs['entries'][f'starter:{slug}:{family.value}']['identity'], 'source_sha256': hashes,
                            'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()})
                    save(cache, key, temporary, EXTRA_ASSETS)
                # Packaging is independent of compilation: corrected notices
                # are published even when the binary and its original sources
                # are reused from an earlier source revision.
                if notices is None:
                    notices = third_party_notices()
                write_changed(temporary / 'NOTICE.txt', (DOCS / 'preview/NOTICE.txt').read_bytes())
                write_changed(temporary / 'THIRD_PARTY_NOTICES.txt', notices)
                save(cache, key, temporary, COMPILED_EXTRAS)
                previews[slug][family.value] = publish_assets(temporary, output, slug, family.value)
                # Checkpoint after each expensive link; interrupted builds resume.
                write_json(manifest_path, {'schema': 2, 'defaultTheme': 'tailwind', 'bindgen': BINDGEN_VERSION,
                                          'components': sorted(previews), 'entries': previews})
    prune_generated(output, previews)
    print(f'Browser previews: {hits} reused, {len(slugs) * len(ThemeFamily) - hits} built', flush=True)


if __name__ == '__main__':
    build()
