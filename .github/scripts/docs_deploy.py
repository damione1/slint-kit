#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Verify published documentation sources, immutable artifacts and release order."""
import argparse
from dataclasses import dataclass
import hashlib
import json
from pathlib import Path
import re
import subprocess
import tomllib
import urllib.error

import release

ROOT = Path(__file__).resolve().parents[2]
DEPLOYMENT = 'deployment.json'
INTEGRITY = 'deployment-integrity.json'
LIBRARY_PATHS = ['Cargo.toml', 'Cargo.lock', 'LICENSE', 'THIRD-PARTY-NOTICES.md', 'crates']


@dataclass(frozen=True, order=True)
class PublishedVersion:
    major: int
    minor: int
    patch: int

    @classmethod
    def parse(cls, value):
        if not isinstance(value, str) or not re.fullmatch(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)', value):
            raise ValueError('Production documentation requires an explicit stable version such as 0.4.0')
        return cls(*(int(part) for part in value.split('.')))

    def __str__(self):
        return f'{self.major}.{self.minor}.{self.patch}'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def source_identity(version):
    version = str(PublishedVersion.parse(version))
    manifest = tomllib.loads((ROOT / 'Cargo.toml').read_text())
    if manifest['workspace']['package']['version'] != version:
        raise ValueError('Documentation version differs from the checked-out library version')
    source = git('rev-parse', 'HEAD')
    tagged = git('rev-parse', f'refs/tags/v{version}^{{commit}}')
    if git('diff', '--name-only', tagged, source, '--', *LIBRARY_PATHS):
        raise ValueError('Documentation source changes published library files; publish a new crate version first')
    if git('status', '--porcelain', '--untracked-files=no'):
        raise ValueError('Documentation publication must start from a clean source revision')
    return {'schemaVersion': 1, 'version': version, 'sourceRevision': source,
            'releaseRevision': tagged}


def verify_registry(identity):
    version, tagged = identity['version'], identity['releaseRevision']
    for name, directory in release.PACKAGES.items():
        metadata = json.loads(release.request(f'https://crates.io/api/v1/crates/{name}/{version}'))['version']
        if metadata['yanked']:
            raise ValueError(f'{name} {version} is yanked')
        data = release.request(f'https://static.crates.io/crates/{name}/{name}-{version}.crate')
        if hashlib.sha256(data).hexdigest() != metadata['checksum']:
            raise ValueError(f'{name}: registry checksum mismatch')
        files = release.archive_files(data, name, version, tagged)
        package = tomllib.loads((ROOT / directory / 'Cargo.toml').read_text())['package']
        required = {'Cargo.toml.orig', 'src/lib.rs'}
        for pattern in package.get('include', []):
            pattern = pattern + '/*' if pattern.endswith('**') else pattern
            required.update(str(path.relative_to(ROOT / directory))
                            for path in (ROOT / directory).glob(pattern) if path.is_file())
        required.discard('Cargo.toml')  # The normalized manifest is checked by archive_files.
        if not required.issubset(files):
            raise ValueError(f'{name}: published archive omits source files: {sorted(required - files.keys())}')
        for relative, content in files.items():
            if relative in {'.cargo_vcs_info.json', 'Cargo.toml'}:
                continue
            original = 'Cargo.toml' if relative == 'Cargo.toml.orig' else relative
            path = ROOT / directory / original
            if not path.is_file() or path.read_bytes() != content:
                raise ValueError(f'{name}: published source differs at {relative}')
    print(f'Verified all six published {version} crates against v{version}', flush=True)


def files(directory):
    return {str(path.relative_to(directory)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(directory.rglob('*'))
            if path.is_file() and path != directory / INTEGRITY}


def stamp(directory, identity):
    (directory / 'deployment-revision.txt').write_text(identity['sourceRevision'] + '\n')
    (directory / DEPLOYMENT).write_text(json.dumps(identity, sort_keys=True, indent=2) + '\n')
    (directory / INTEGRITY).write_text(json.dumps({'schemaVersion': 1, 'files': files(directory)},
                                               sort_keys=True, indent=2) + '\n')


def require_release_consumers(version):
    manifest = json.loads((ROOT / 'docs/.generated/consumers/manifest.json').read_text())
    if (manifest.get('schemaVersion') != 1 or manifest.get('mode') != 'release'
            or manifest.get('version') != version or not manifest.get('slugs')):
        raise ValueError('Production documentation cannot contain local or unprepared consumers')


def verify_artifact(directory, version, source):
    identity = json.loads((directory / DEPLOYMENT).read_text())
    manifest = json.loads((directory / INTEGRITY).read_text())
    if (identity.get('schemaVersion') != 1 or identity.get('version') != str(PublishedVersion.parse(version))
            or identity.get('sourceRevision') != source
            or not re.fullmatch(r'[a-f0-9]{40}', identity.get('releaseRevision', ''))):
        raise ValueError('Documentation artifact has a different version or source revision')
    if manifest.get('schemaVersion') != 1 or manifest.get('files') != files(directory):
        raise ValueError('Documentation artifact is missing, changed or contains unexpected files')
    if (directory / 'deployment-revision.txt').read_text().strip() != source:
        raise ValueError('Documentation revision marker differs from the artifact')
    return identity


def newer_published(version):
    candidate = PublishedVersion.parse(version)
    metadata = json.loads(release.request('https://crates.io/api/v1/crates/slint-kit'))
    for entry in metadata['versions']:
        if entry['yanked']:
            continue
        try:
            other = PublishedVersion.parse(entry['num'])
        except ValueError:
            continue  # Prerelease versions do not replace stable production documentation.
        if other > candidate:
            raise ValueError(f'Refusing outdated documentation {version}; {other} is already published')


def guard(identity, origin):
    newer_published(identity['version'])
    try:
        deployed = json.loads(release.request(origin.rstrip('/') + '/' + DEPLOYMENT))
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return  # First deployment of the versioned manifest (the previous site may be legacy).
        raise
    if deployed.get('schemaVersion') != 1:
        raise ValueError('Production has an unsupported deployment manifest')
    current, candidate = PublishedVersion.parse(deployed['version']), PublishedVersion.parse(identity['version'])
    if current > candidate:
        raise ValueError(f'Refusing to replace production {current} with {candidate}')
    if current == candidate and deployed['sourceRevision'] != identity['sourceRevision']:
        previous = deployed['sourceRevision']
        if not re.fullmatch(r'[a-f0-9]{40}', previous):
            raise ValueError('Production has an invalid source revision')
        result = subprocess.run(['git', 'merge-base', '--is-ancestor', previous,
                                 identity['sourceRevision']], cwd=ROOT, check=False)
        if result.returncode != 0:
            raise ValueError('Refusing an older or unrelated documentation correction for this release')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['verify-source', 'stamp', 'verify-artifact', 'guard'])
    parser.add_argument('--version', required=True)
    parser.add_argument('--source')
    parser.add_argument('--directory', type=Path, default=ROOT / 'docs/_site')
    parser.add_argument('--identity-file', type=Path, default=ROOT / 'docs/.verification/release-source.json')
    parser.add_argument('--origin', default='https://slint-kit.dev')
    args = parser.parse_args()
    try:
        if args.command == 'verify-source':
            identity = source_identity(args.version)
            verify_registry(identity)
            args.identity_file.parent.mkdir(parents=True, exist_ok=True)
            args.identity_file.write_text(json.dumps(identity, sort_keys=True, indent=2) + '\n')
        elif args.command == 'stamp':
            identity = json.loads(args.identity_file.read_text())
            if identity['version'] != str(PublishedVersion.parse(args.version)) or identity['sourceRevision'] != git('rev-parse', 'HEAD'):
                raise ValueError('Build identity differs from the verified release source')
            if git('diff', '--name-only', identity['releaseRevision'], '--', *LIBRARY_PATHS):
                raise ValueError('The documentation build changed published library sources')
            require_release_consumers(identity['version'])
            stamp(args.directory, identity)
        else:
            if not args.source:
                raise ValueError('--source is required to verify the tested artifact')
            identity = verify_artifact(args.directory, args.version, args.source)
            if args.command == 'guard':
                if git('rev-parse', 'HEAD') != args.source:
                    raise ValueError('Deploy checkout differs from the tested artifact')
                if git('rev-parse', f'refs/tags/v{args.version}^{{commit}}') != identity['releaseRevision']:
                    raise ValueError('Release tag differs from the tested artifact')
                guard(identity, args.origin)
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        raise SystemExit(f'Documentation publication failed: {error}') from None


if __name__ == '__main__':
    main()
