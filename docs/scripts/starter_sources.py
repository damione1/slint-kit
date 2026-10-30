# SPDX-License-Identifier: GPL-3.0-or-later
"""Prepare independent consumers and private local builds without patching downloads."""
from enum import Enum
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import tomllib

ROOT = Path(__file__).resolve().parents[2]
FILES = ('Cargo.toml', 'Cargo.lock', 'build.rs', 'src/main.rs', 'ui/app.slint')
KIT_PACKAGES = {'slint-kit-core': 'crates/kit-core', 'slint-kit-slint': 'crates/kit-slint',
                'slint-kit-native': 'crates/kit-native', 'slint-kit-build': 'crates/kit-build',
                'slint-kit': 'crates/kit', 'slint-kit-tailwind': 'crates/kit-tailwind'}
LOCK_PACKAGE = 'slint-kit-consumer-lock'
_prepared = {}


class BuildMode(Enum):
    LOCAL = 'local'
    RELEASE = 'release'


def build_mode(mode=None):
    return mode if isinstance(mode, BuildMode) else BuildMode(mode or os.environ.get('DOCS_BUILD_MODE', 'local'))


def write_changed(path, content):
    content = content.encode() if isinstance(content, str) else content
    if not path.is_file() or path.read_bytes() != content:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)


def digest(value):
    return hashlib.sha256(value).hexdigest()


def local_patches():
    return '\n[patch.crates-io]\n' + ''.join(
        f'{name} = {{ path = {json.dumps(str(ROOT / directory))} }}\n'
        for name, directory in KIT_PACKAGES.items())


def rename_package(source, name):
    result, count = re.subn(r'(?m)^(name\s*=\s*)"[^"]+"',
                            lambda match: match[1] + json.dumps(name), source, count=1)
    if count != 1:
        raise ValueError('A starter needs an explicit package name')
    return result


def rename_lock(source, previous, name):
    # The root package is never depended on by a dependency, so only its own
    # package table changes between otherwise identical consumer graphs.
    needle = 'name = ' + json.dumps(previous)
    if source.count(needle + '\n') != 1:
        raise ValueError('The prepared lock must contain exactly one root package')
    return source.replace(needle + '\n', 'name = ' + json.dumps(name) + '\n')


def validate_manifest(source, version):
    manifest = tomllib.loads(source)
    if 'patch' in manifest or 'replace' in manifest:
        raise ValueError('Public starter sources must not contain local dependency overrides')
    for section, package in [('dependencies', 'slint-kit'), ('build-dependencies', 'slint-kit-build')]:
        dependency = manifest[section][package]
        constraint = dependency if isinstance(dependency, str) else dependency.get('version')
        if constraint != '=' + version or (isinstance(dependency, dict) and any(key in dependency for key in ['path', 'git'])):
            raise ValueError(f'{package} must use the exact registry version ={version}')
    return manifest


def prepared_lock(mode, source, seed, cargo_version):
    manifest = rename_package(source, LOCK_PACKAGE)
    if mode is BuildMode.LOCAL:
        manifest += local_patches()
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']
    context = {'format': 1, 'mode': mode.value, 'cargo': cargo_version,
               'manifest': manifest, 'seed': digest(rename_lock(seed, tomllib.loads(source)['package']['name'], LOCK_PACKAGE).encode())}
    if mode is BuildMode.LOCAL:
        context['workspace'] = {key: workspace[key] for key in ['package', 'dependencies']}
        context['packages'] = {name: (ROOT / directory / 'Cargo.toml').read_text()
                               for name, directory in KIT_PACKAGES.items()}
    key = digest(json.dumps(context, sort_keys=True).encode())
    directory = ROOT / 'docs/.cache/consumer-locks' / mode.value / key
    record = directory / 'manifest.json'
    lock = directory / 'Cargo.lock'
    if record.is_file() and lock.is_file():
        try:
            stored = json.loads(record.read_text())
            if isinstance(stored, dict) and stored.get('key') == key and stored.get('sha256') == digest(lock.read_bytes()):
                return lock.read_text()
        except (OSError, ValueError):
            pass
    write_changed(directory / 'Cargo.toml', manifest)
    write_changed(directory / 'src/main.rs', 'fn main() {}\n')
    write_changed(lock, rename_lock(seed, tomllib.loads(source)['package']['name'], LOCK_PACKAGE))
    subprocess.run(['cargo', 'generate-lockfile', '--manifest-path', str(directory / 'Cargo.toml')],
                   cwd=ROOT, check=True)
    write_changed(record, json.dumps({'key': key, 'sha256': digest(lock.read_bytes())}, sort_keys=True) + '\n')
    return lock.read_text()


def prepare(mode=None):
    mode = build_mode(mode)
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    entries = json.loads((ROOT / 'docs/content/starters.json').read_text())
    cargo_version = subprocess.check_output(['cargo', '--version'], cwd=ROOT, text=True).strip()
    public = ROOT / 'docs/.generated/consumers'
    consumers = {}
    private = {}
    graphs = {}
    checksums = {}
    for slug in sorted(entries):
        if not re.fullmatch('[a-z]+(?:-[a-z]+)*', slug):
            raise ValueError(f'Invalid starter slug: {slug}')
        original = ROOT / 'docs/quickstart' / slug
        source = (original / 'Cargo.toml').read_text()
        manifest = validate_manifest(source, version)
        seed = (original / 'Cargo.lock').read_text()
        # Root package names differ; dependency graphs can share one resolution.
        normalized = rename_package(source, LOCK_PACKAGE)
        normalized_seed = rename_lock(seed, manifest['package']['name'], LOCK_PACKAGE)
        graph = digest((normalized + '\0' + normalized_seed).encode())
        if graph not in graphs:
            graphs[graph] = prepared_lock(mode, source, seed, cargo_version)
        lock = rename_lock(graphs[graph], LOCK_PACKAGE, manifest['package']['name'])
        destination = public / slug
        native = ROOT / 'docs/.cache/native-source' / slug if mode is BuildMode.LOCAL else destination
        for name in FILES:
            data = source.encode() if name == 'Cargo.toml' else lock.encode() if name == 'Cargo.lock' else (original / name).read_bytes()
            write_changed(destination / name, data)
            if native != destination:
                write_changed(native / name, data + local_patches().encode() if name == 'Cargo.toml' else data)
        consumers[slug], private[slug] = destination, native
        checksums[slug] = {name: digest((destination / name).read_bytes()) for name in FILES}
    write_changed(public / 'manifest.json', json.dumps({'schemaVersion': 1, 'mode': mode.value,
                  'version': version, 'slugs': sorted(consumers), 'source_sha256': checksums},
                  sort_keys=True, indent=2) + '\n')
    _prepared.clear()  # The public destination contains exactly one mode at a time.
    _prepared[mode] = consumers, private
    print(f'Prepared {len(consumers)} {mode.value} consumers with {len(graphs)} dependency graphs', flush=True)
    return consumers


def source_directory(slug, mode=None, native=False):
    mode = build_mode(mode)
    if mode not in _prepared:
        prepare(mode)
    consumers, private = _prepared[mode]
    if slug not in consumers:
        raise ValueError(f'Unknown starter: {slug}')
    return (private if native else consumers)[slug]
