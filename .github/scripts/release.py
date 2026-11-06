#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Validate a tagged release and safely resume crates.io and GitHub publication."""
import argparse
import hashlib
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tarfile
import time
import tomllib
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
# Explicit allowlist and dependency order; applications/tools are never published.
PACKAGES = {
    'slint-kit-core': 'crates/kit-core',
    'slint-kit-slint': 'crates/kit-slint',
    'slint-kit-native': 'crates/kit-native',
    'slint-kit-build': 'crates/kit-build',
    'slint-kit': 'crates/kit',
}
SEMVER = r'(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)(?:\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?'
USER_AGENT = 'slint-kit-release/0.1 (https://github.com/damione1/slint-kit)'


def run(*command):
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def validate(tag):
    if not re.fullmatch('v' + SEMVER, tag):
        raise ValueError(f'Expected an exact version tag such as v0.1.0; got {tag!r}')
    version = tag[1:]
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']
    if workspace['version'] != version:
        raise ValueError(f'Tag {tag} differs from workspace version {workspace["version"]}')
    for name, directory in PACKAGES.items():
        package = tomllib.loads((ROOT / directory / 'Cargo.toml').read_text())['package']
        resolved = workspace['version'] if package['version'] == {'workspace': True} else package['version']
        if package['name'] != name or resolved != version:
            raise ValueError(f'{directory} must publish {name} at exactly {version}')
    commit = run('git', 'rev-parse', 'HEAD')
    if os.environ.get('GITHUB_SHA', commit) != commit:
        raise ValueError('Checkout does not match the triggering commit')
    if run('git', 'status', '--porcelain'):
        raise ValueError('Release source must be committed and clean')
    print(f'Validated {tag}: {len(PACKAGES)} libraries at commit {commit}', flush=True)
    return version, commit


def archive_files(data, name, version, commit):
    prefix = f'{name}-{version}/'
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        files = {}
        for entry in archive:
            if entry.isdir():
                continue
            if not entry.isfile() or not entry.name.startswith(prefix):
                raise ValueError(f'{name}: unexpected archive member {entry.name}')
            relative = entry.name[len(prefix):]
            if relative in files or '..' in Path(relative).parts:
                raise ValueError(f'{name}: invalid or duplicate archive member {relative}')
            files[relative] = archive.extractfile(entry).read()
    provenance = json.loads(files['.cargo_vcs_info.json'])
    if provenance['git'].get('sha1') != commit or provenance['git'].get('dirty', False):
        raise ValueError(f'{name}: archive is not from the clean tagged commit {commit}')
    if provenance['path_in_vcs'] != PACKAGES[name]:
        raise ValueError(f'{name}: archive source path differs from the expected library')
    manifest = tomllib.loads(files['Cargo.toml'].decode())['package']
    if manifest['name'] != name or manifest['version'] != version:
        raise ValueError(f'{name}: archive manifest has a different name/version')
    if manifest['license'] != 'GPL-3.0-or-later' or files['LICENSE'] != (ROOT / 'LICENSE').read_bytes():
        raise ValueError(f'{name}: archive does not contain the canonical GPL license')
    if not files.get('README.md'):
        raise ValueError(f'{name}: archive README is missing')
    if name == 'slint-kit-slint':
        for required in ['ui/kit.slint', 'ui/shared-contracts.slint']:
            if required not in files:
                raise ValueError(f'{name}: archive is missing {required}')
    # Registry dependency resolution can update Cargo.lock on a later rerun.
    # Every source, manifest, asset and provenance byte must still match.
    files.pop('Cargo.lock', None)
    return files


def request(url):
    req = urllib.request.Request(url, headers={'User-Agent': USER_AGENT})
    with urllib.request.urlopen(req, timeout=30) as response:
        return response.read()


def existing_version(name, version, commit, expected):
    try:
        metadata = json.loads(request(f'https://crates.io/api/v1/crates/{name}/{version}'))['version']
    except urllib.error.HTTPError as error:
        if error.code == 404:
            return False
        raise
    if metadata['yanked']:
        raise ValueError(f'{name} {version} is yanked; refusing to silently skip it')
    data = request(f'https://static.crates.io/crates/{name}/{name}-{version}.crate')
    if hashlib.sha256(data).hexdigest() != metadata['checksum']:
        raise ValueError(f'{name}: registry archive checksum mismatch')
    if archive_files(data, name, version, commit) != expected:
        raise ValueError(f'{name} {version} already exists with different content; refusing publication')
    print(f'{name} {version}: already published from this exact commit; verified', flush=True)
    return True


def check(version, commit):
    archives = {}
    for name in PACKAGES:
        path = ROOT / 'target/package' / f'{name}-{version}.crate'
        archives[name] = archive_files(path.read_bytes(), name, version, commit)
    # Preflight every namespace before uploading anything, including partial reruns.
    published = {name: existing_version(name, version, commit, files) for name, files in archives.items()}
    return archives, published


def release_notes(version):
    """Read only the exact version section, retaining its Markdown subsections."""
    text = (ROOT / 'CHANGELOG.md').read_text()
    headings = list(re.finditer(r'^## ([^\n]+)\s*$', text, re.MULTILINE))
    matches = [index for index, heading in enumerate(headings) if heading[1].strip() == version]
    if len(matches) != 1:
        raise ValueError(f'CHANGELOG.md must contain exactly one section for {version}')
    index = matches[0]
    end = headings[index + 1].start() if index + 1 < len(headings) else len(text)
    notes = text[headings[index].end():end].strip()
    if not notes:
        raise ValueError(f'CHANGELOG.md section for {version} is empty')
    return notes


def github_api(path, payload=None):
    token = os.environ.get('GH_TOKEN')
    repository = os.environ.get('GITHUB_REPOSITORY', '')
    if not token or not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('GH_TOKEN and GITHUB_REPOSITORY are required for GitHub publication')
    req = urllib.request.Request(
        f'https://api.github.com/repos/{repository}/{path}',
        data=None if payload is None else json.dumps(payload).encode(),
        headers={'User-Agent': USER_AGENT, 'Accept': 'application/vnd.github+json',
                 'Authorization': f'Bearer {token}', 'X-GitHub-Api-Version': '2022-11-28',
                 'Content-Type': 'application/json'},
    )
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.load(response)


def github_pages(resource):
    page = 1
    while True:
        entries = github_api(f'{resource}?per_page=100&page={page}')
        yield from entries
        if len(entries) < 100:
            return
        page += 1


def stable_version(tag):
    """Stable SemVer precedence; build metadata never changes precedence."""
    match = re.fullmatch('v' + SEMVER, tag)
    if match is None or '-' in tag.split('+', 1)[0]:
        return None
    return tuple(int(part) for part in match.groups())


def github_release(version, commit):
    tag = f'v{version}'
    releases = list(github_pages('releases'))
    if any(entry['tag_name'] == tag for entry in releases):
        print(f'GitHub Release {tag} already exists; preserving its metadata and assets', flush=True)
        return
    notes = release_notes(version)
    stable = stable_version(tag)
    latest = False
    if stable is not None:
        # The facade is published last. Its registry versions also cover releases
        # missing a GitHub object, without mistaking an unbuilt tag for a release.
        candidates = [entry['tag_name'] for entry in releases if not entry['draft'] and not entry['prerelease']]
        registry = json.loads(request('https://crates.io/api/v1/crates/slint-kit'))
        candidates.extend('v' + entry['num'] for entry in registry['versions'] if not entry['yanked'])
        latest = not any(other > stable for candidate in candidates
                         if (other := stable_version(candidate)) is not None)
    payload = {'tag_name': tag, 'target_commitish': commit, 'name': tag, 'body': notes,
               'draft': False, 'prerelease': stable is None,
               'make_latest': 'true' if latest else 'false'}
    try:
        github_api('releases', payload)
    except urllib.error.HTTPError as error:
        if error.code != 422:
            raise
        # Another publisher may have created the same Release after our read.
        # Only that exact conflict is idempotent; all other failures remain fatal.
        existing = github_api('releases/tags/' + urllib.parse.quote(tag, safe=''))
        if existing['tag_name'] != tag:
            raise error
        print(f'GitHub Release {tag} appeared concurrently; preserving it', flush=True)
        return
    print(f'Created GitHub Release {tag} (latest={latest})', flush=True)


def publish(version, commit):
    if not os.environ.get('CARGO_REGISTRY_TOKEN'):
        raise ValueError('CARGO_REGISTRY_TOKEN repository secret is missing or empty')
    archives, published = check(version, commit)
    for name in PACKAGES:
        if published[name]:
            continue
        result = subprocess.run(['cargo', 'publish', '--locked', '--registry', 'crates-io', '-p', name], cwd=ROOT)
        # A successful upload may still report an index-polling timeout. Check the
        # registry archive before deciding whether the dependency can proceed.
        for attempt in range(24):
            if existing_version(name, version, commit, archives[name]):
                break
            if attempt < 23:
                time.sleep(5)
        else:
            raise ValueError(f'{name}: upload exit {result.returncode}; registry version unavailable. Rerun this same tag after resolving the failure; verified libraries will be skipped.')
    print(f'Published and verified all {len(PACKAGES)} Slint Kit {version} libraries', flush=True)
    github_release(version, commit)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=['validate', 'check', 'publish'])
    parser.add_argument('--tag', default=os.environ.get('GITHUB_REF_NAME', ''))
    args = parser.parse_args()
    try:
        version, commit = validate(args.tag)
        if args.command == 'check':
            check(version, commit)
        elif args.command == 'publish':
            publish(version, commit)
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError, tarfile.TarError) as error:
        raise SystemExit(f'Release failed: {error}') from None


if __name__ == '__main__':
    main()
