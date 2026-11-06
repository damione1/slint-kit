#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build one shared WASM bundle for every interactive example and catalogue fixture."""
import argparse
import ast
import inspect
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib
import fcntl
from functools import wraps

from doc_inputs import ROOT, DOCS, build_inputs, digest, runtime_inputs, write_changed, write_json, consumer_identity, compiler_environment

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


def corresponding_sources(slug=None):
    from starter_sources import source_files
    files = [ROOT / name for name in ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'README.md']]
    files += [path for path in (ROOT / 'crates').rglob('*') if path.is_file() and 'target' not in path.parts]
    files += [path for path in (DOCS / 'preview').rglob('*') if path.is_file() and 'target' not in path.parts]
    files += [DOCS / 'scripts' / name for name in ['build-web-previews.py', 'web_cache.py', 'presentation.py', 'doc_inputs.py']]
    starters = [DOCS / 'quickstart' / slug] if slug else sorted((DOCS / 'quickstart').iterdir())
    for starter in starters:
        if (starter / 'src/main.rs').is_file():
            files += [starter / name for name in source_files(starter)]
    files += [path for path in (DOCS / 'examples').rglob('*') if path.is_file()]
    return sorted(set(files))


def archive_sources(directory, slug=None):
    files = corresponding_sources(slug)
    with (directory / 'source.tar.gz').open('wb') as compressed, gzip.GzipFile(fileobj=compressed, mode='wb', mtime=0, filename='') as gzip_stream, tarfile.open(fileobj=gzip_stream, mode='w') as archive:
        for path in files:
            content = path.read_bytes()
            if path == ROOT / 'Cargo.toml':
                # Keep the exact library workspace while removing absent apps.
                content = content.replace(b', "showcase", "tests", "xtask", "docs/capture", "docs/integration"', b'')
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
    # Keep the generated bindings byte exact. The authored loader decompresses
    # one transport asset into the original single WebAssembly module.
    assets = {name: (temporary / name).read_bytes() for name in ASSETS}
    assets['slint_kit_previews_bindings.js'] = assets.pop('slint_kit_previews.js')
    assets['slint_kit_previews.js'] = (DOCS / 'preview/loader.js').read_bytes()
    assets['slint_kit_previews_bg.wasm.gz'] = gzip.compress(assets.pop('slint_kit_previews_bg.wasm'), compresslevel=6, mtime=0)
    value = hashlib.sha256()
    for name, data in sorted(assets.items()):
        value.update(name.encode() + b'\0' + data + b'\0')
    code_base = f'/assets/previews/{slug}/{family}/{value.hexdigest()[:20]}'
    for name, data in assets.items():
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
            'assets': {name: digest(data) for name, data in assets.items()},
            'transport': {'encoding': 'gzip', 'moduleSha256': digest((temporary / 'slint_kit_previews_bg.wasm').read_bytes())},
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


def fixture_code_key(task, rustc, bindgen, environment):
    files = ('build.rs', 'src/lib.rs', 'Cargo.toml', 'Cargo.lock')
    host = {name: digest((DOCS / 'preview' / name).read_bytes()) for name in files}
    flags = compiler_flags(environment)
    return digest({'schema': 3, 'scene': task['sceneKey'], 'rustc': rustc,
                   'bindgen': bindgen, 'host': host, 'flags': flags})


def compiler_flags(environment):
    return {name: value for name, value in environment.items()
            if name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')
            or name.startswith(('CARGO_PROFILE_RELEASE_', 'CARGO_TARGET_WASM32_'))}


def bundle_key(inputs, rustc, bindgen, environment, packaging_hash=None):
    # A binary contains every route, so any reachable fixture or host change
    # invalidates the bundle. Each screenshot still has its own scene identity.
    hosts = {str(path.relative_to(ROOT)): digest(path.read_bytes())
             for path in corresponding_sources()
             if path.suffix in ('.rs', '.slint', '.toml', '.lock')
             or path.name == 'build-web-previews.py'}
    if packaging_hash is not None:
        hosts['docs/scripts/build-web-previews.py'] = packaging_hash
    return digest({'schema': 1, 'inputs': {name: entry['identity'] for name, entry in inputs['entries'].items()},
                   'hosts': hosts, 'runtime': runtime_inputs(), 'rustc': rustc,
                   'bindgen': bindgen, 'flags': compiler_flags(environment)})


def compile_bundle(target, environment, bindgen, destination):
    log_path = DOCS / '.verification/wasm/documentation-bundle.log'
    log_path.parent.mkdir(parents=True, exist_ok=True)
    with log_path.open('w') as log:
        result = subprocess.run(['cargo', 'build', '--manifest-path', 'docs/preview/Cargo.toml', '--locked', '--release',
                                 '--target', 'wasm32-unknown-unknown'], cwd=ROOT, env=environment,
                                stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f'Documentation bundle compilation failed; see {log_path}')
    subprocess.run([bindgen, '--target', 'web', '--out-dir', str(destination), '--out-name', 'slint_kit_previews',
                    str(target / 'wasm32-unknown-unknown/release/slint_kit_doc_previews.wasm')], cwd=ROOT, check=True)


def restore_packaging_only(cache, key, destination, inputs, rustc, bindgen, environment):
    """Reuse a proven binary when only this Python packager changed.

    Verify its original key against the current compiler inputs/toolchain/flags,
    then compare the Cargo+bindgen recipe with the archived source. Rust/Slint
    edits or different compiler commands cannot enter this path.
    """
    source_name = 'docs/scripts/build-web-previews.py'
    recipe = ast.dump(ast.parse(inspect.getsource(compile_bundle)), include_attributes=False)
    for record_path in sorted(cache.glob('*/component.json')):
        try:
            record = json.loads(record_path.read_text())
            archived_hash = record['source_sha256'][source_name]
            original = record['key']
            if original != bundle_key(inputs, rustc, bindgen, environment, archived_hash):
                continue
            if not restore(cache, original, destination, COMPILED_EXTRAS):
                continue
            with tarfile.open(destination / 'source.tar.gz', 'r:gz') as archive:
                archived_source = archive.extractfile('slint-kit/' + source_name).read()
            if digest(archived_source) != archived_hash:
                continue
            tree = ast.parse(archived_source)
            function = next(node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == 'compile_bundle')
            old_recipe = ast.dump(ast.Module(body=[function], type_ignores=[]), include_attributes=False)
            if old_recipe != recipe:
                continue
        except (OSError, ValueError, KeyError, TypeError, StopIteration, tarfile.TarError):
            continue
        record.update({'key': key, 'reusedCompilation': original, 'source_sha256': archive_sources(destination)})
        write_json(destination / 'component.json', record)
        save(cache, key, destination, COMPILED_EXTRAS)
        print(f'WASM compilation reused after verified packaging-only change: {original}', flush=True)
        return True
    return False


@exclusive_build
def build():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('slugs', nargs='*', help='Validate requested routes; the shared bundle always contains every component')
    parser.add_argument('--check', action='store_true')
    parser.add_argument('--package-only', action='store_true', help='Refuse any compiler invocation; require a verified compiled cache')
    args = parser.parse_args()
    from starter_sources import prepare
    prepare()
    inputs = build_inputs()
    entries = json.loads((DOCS / 'content/starters.json').read_text())
    if not set(args.slugs) <= set(entries):
        parser.error('Unknown preview component')
    slugs = sorted(entries)
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
    key = bundle_key(inputs, rustc, version, environment)
    old = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    if args.check:
        if old.get('bundle', {}).get('key') != key or old.get('components') != slugs:
            raise SystemExit('Stale browser bundle; rebuild it')
        print(f'Browser bundle verified: {len(slugs)} interactive examples + {2 * len(slugs)} localized fixtures')
        return
    cache = DOCS / '.cache/preview-bundles'
    with tempfile.TemporaryDirectory(prefix='bundle-', dir=output) as folder:
        temporary = Path(folder)
        hit = restore(cache, key, temporary, COMPILED_EXTRAS)
        if not hit:
            hit = restore_packaging_only(cache, key, temporary, inputs, rustc, version, environment)
        if not hit:
            if args.package_only:
                raise RuntimeError('No compiled bundle matches the current sources; package-only never builds')
            print(f'WASM build once: {len(slugs)} interactive examples + {2 * len(slugs)} localized fixtures', flush=True)
            compile_bundle(target, environment, bindgen, temporary)
            # Reject a concurrently edited source tree rather than attesting a
            # binary under a hash computed before its sources changed.
            if bundle_key(build_inputs(), rustc, version, environment) != key:
                raise RuntimeError('Documentation sources changed during compilation; rebuild the bundle')
            hashes = archive_sources(temporary)
            write_json(temporary / 'component.json', {'key': key, 'components': slugs,
                       'source_sha256': hashes, 'revision': subprocess.check_output(
                           ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()})
            save(cache, key, temporary, COMPILED_EXTRAS)
        write_changed(temporary / 'NOTICE.txt', (DOCS / 'preview/NOTICE.txt').read_bytes())
        write_changed(temporary / 'THIRD_PARTY_NOTICES.txt', third_party_notices())
        bundle = publish_assets(temporary, output, 'bundle', 'shared')
        previews = {slug: {'tailwind': {**bundle, 'key': code_key(slug, 'tailwind', inputs, rustc, version, environment)}}
                    for slug in slugs}
        write_json(manifest_path, {'schema': 2, 'defaultTheme': 'tailwind', 'bindgen': BINDGEN_VERSION,
                                  'components': slugs, 'entries': previews, 'bundle': bundle})
    prune_generated(output, previews)
    print(f'Browser bundle: {"reused" if hit else "built once"}; {3 * len(slugs)} runtime routes', flush=True)


if __name__ == '__main__':
    build()
