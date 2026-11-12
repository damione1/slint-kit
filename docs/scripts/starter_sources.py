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
                'slint-kit': 'crates/kit'}
LOCK_PACKAGE = 'slint-kit-consumer-lock'
_prepared = {}
PROJECT_TEMPLATES = ROOT / 'crates/kit-cli/templates/project'


def source_files(folder):
    assets = folder / 'ui/assets'
    return FILES + tuple(str(path.relative_to(folder)) for path in sorted(assets.rglob('*')) if path.is_file())


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


def with_kit_version(source, version):
    """Project registry dependency versions while retaining every other option."""
    if not re.fullmatch(r'\d+\.\d+\.\d+', version):
        raise ValueError('Expected a stable workspace version')
    parsed = tomllib.loads(source)
    for section, name in [('dependencies', 'slint-kit'), ('build-dependencies', 'slint-kit-build')]:
        dependency = parsed[section][name]
        if isinstance(dependency, dict) and any(key in dependency for key in ('path', 'git')):
            raise ValueError(f'{name}: a downloadable consumer requires a registry dependency')
        pattern = rf'(?m)^({re.escape(name)}\s*=\s*)("[^"]+"|\{{[^\n]*\}})(\s*(?:#.*)?)$'
        def replace(match):
            value = match[2]
            if value.startswith('"'):
                value = json.dumps('=' + version)
            else:
                value, count = re.subn(r'(\bversion\s*=\s*)"[^"]+"',
                                      lambda part: part[1] + json.dumps('=' + version), value)
                if count != 1:
                    raise ValueError(f'{name}: expected one explicit registry version')
            return match[1] + value + match[3]
        lines = []
        active_section = None
        count = 0
        for line in source.splitlines(keepends=True):
            heading = re.fullmatch(r'\s*\[([^\]]+)\]\s*(?:#.*)?', line.strip())
            if heading:
                active_section = heading[1]
            if active_section == section:
                line, matches = re.subn(pattern, replace, line)
                count += matches
            lines.append(line)
        source = ''.join(lines)
        if count != 1:
            raise ValueError(f'{name}: expected one dependency declaration')
    return source


def prepared_lock(mode, source, seed, cargo_version):
    manifest = rename_package(source, LOCK_PACKAGE)
    if mode is BuildMode.LOCAL:
        manifest += local_patches()
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']
    context = {'format': 2, 'mode': mode.value, 'cargo': cargo_version,
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
    # Keep the seed's external graph; resolve only changes required by the
    # consumer manifest instead of upgrading unrelated transitive packages.
    subprocess.run(['cargo', 'update', '--workspace', '--manifest-path', str(directory / 'Cargo.toml')],
                   cwd=ROOT, check=True)
    write_changed(record, json.dumps({'key': key, 'sha256': digest(lock.read_bytes())}, sort_keys=True) + '\n')
    return lock.read_text()


def template_literal(value):
    """Render only the value types admitted in standalone Cargo dependencies."""
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, bool):
        return 'true' if value else 'false'
    if isinstance(value, list):
        return '[' + ', '.join(template_literal(item) for item in value) + ']'
    if isinstance(value, dict):
        if any(not re.fullmatch(r'[A-Za-z0-9_-]+', key) for key in value):
            raise ValueError('Unsupported inline Cargo key')
        return '{ ' + ', '.join(key + ' = ' + template_literal(item) for key, item in value.items()) + ' }'
    raise ValueError(f'Unsupported Cargo dependency value: {type(value).__name__}')


def project_manifest(manifest, version):
    """Use the CLI project shell; keep each component's explicit dependencies."""
    template = (PROJECT_TEMPLATES / 'Cargo.toml.in').read_text()
    empty = template
    for marker in ('__RUNTIME_DEPENDENCY__', '__SLINT_DEPENDENCY__', '__EXTRA_DEPENDENCIES__', '__LIBRARY_CONFIGURATION__'):
        empty = empty.replace(marker, '')
    defaults = tomllib.loads(empty)['package']
    package = manifest['package']
    if set(manifest) - {'package', 'dependencies', 'build-dependencies', 'workspace'}:
        raise ValueError('Standalone starter has unsupported manifest tables')
    if set(package) - set(defaults) or any(package.get(key, value) != value for key, value in defaults.items() if key != 'name'):
        raise ValueError('Standalone starter package metadata differs from the shared project template')
    if set(manifest['build-dependencies']) != {'slint-kit-build'}:
        raise ValueError('Extend the shared template for additional build dependencies')
    if manifest.get('workspace', {}):
        raise ValueError('Standalone starter workspace must be empty')
    dependencies = manifest['dependencies']
    def dependency(name):
        return name + ' = ' + template_literal(dependencies[name]) if name in dependencies else ''
    return (template.replace('"__PACKAGE_NAME__"', template_literal(package['name']))
            .replace('__RUNTIME_DEPENDENCY__', dependency('slint-kit'))
            .replace('__SLINT_DEPENDENCY__', dependency('slint'))
            .replace('__EXTRA_DEPENDENCIES__', '\n'.join(dependency(name) for name in dependencies if name not in {'slint-kit', 'slint'}))
            .replace('__KIT_VERSION__', version)
            .replace('__LIBRARY_CONFIGURATION__', ''))


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
        source = with_kit_version((original / 'Cargo.toml').read_text(), version)
        manifest = validate_manifest(source, version)
        source = project_manifest(manifest, version)
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
        names = source_files(original)
        for target in {destination, native}:
            for asset in (target / 'ui/assets').rglob('*'):
                if asset.is_file() and str(asset.relative_to(target)) not in names:
                    asset.unlink()
        for name in names:
            data = source.encode() if name == 'Cargo.toml' else lock.encode() if name == 'Cargo.lock' else (PROJECT_TEMPLATES / 'build.rs').read_bytes() if name == 'build.rs' else (original / name).read_bytes()
            write_changed(destination / name, data)
            if native != destination:
                write_changed(native / name, data + local_patches().encode() if name == 'Cargo.toml' else data)
        consumers[slug], private[slug] = destination, native
        checksums[slug] = {name: digest((destination / name).read_bytes()) for name in names}
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
