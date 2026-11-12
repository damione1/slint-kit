#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Move the compiled preview bundle between docs/.cache/preview-bundles and Cloudflare R2.

The cache entry is the directory docs/.cache/preview-bundles/<bundle_key> written by
build-web-previews.py. It travels as one gzip archive named by the same key, so the
object name is the whole cache contract.
"""
import argparse
from collections.abc import Callable
from dataclasses import dataclass
from enum import StrEnum
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from typing import NewType, assert_never

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / 'docs/.cache/preview-bundles'
WRANGLER = 'wrangler@4.154.0'  # Same pin as the production deployment.
OBJECT_PREFIX = 'preview-bundles'
MISSING_OBJECT = re.compile(r'specified key does not exist|NoSuchKey|\b10007\b', re.IGNORECASE)

BundleKey = NewType('BundleKey', str)


class Command(StrEnum):
    PULL = 'pull'
    PUSH = 'push'


class Outcome(StrEnum):
    HIT = 'hit'
    MISS = 'miss'


@dataclass(frozen=True)
class Settings:
    """R2 settings, parsed once from the environment."""
    bucket: str

    @classmethod
    def from_environment(cls, environment: dict[str, str]) -> 'Settings':
        missing = [name for name in ('DOCS_CACHE_BUCKET', 'CLOUDFLARE_API_TOKEN', 'CLOUDFLARE_ACCOUNT_ID')
                   if not environment.get(name)]
        if missing:
            raise ValueError('Missing environment variables: ' + ', '.join(missing))
        return cls(bucket=environment['DOCS_CACHE_BUCKET'])

    def object_path(self, key: BundleKey) -> str:
        return f'{self.bucket}/{OBJECT_PREFIX}/{key}.tar.gz'


# A wrangler runner takes the wrangler arguments and returns the finished process.
WranglerRunner = Callable[[list[str]], subprocess.CompletedProcess[str]]


def run_wrangler(arguments: list[str]) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory(prefix='wrangler-') as folder:
        # A neutral directory keeps wrangler away from docs/wrangler.toml.
        return subprocess.run(['npx', '--yes', WRANGLER, *arguments], cwd=folder, text=True,
                              capture_output=True, check=False)


def parse_key(value: str) -> BundleKey:
    if not re.fullmatch(r'[0-9a-f]{64}', value):
        raise ValueError(f'Invalid bundle key: {value!r}')
    return BundleKey(value)


def archive_entry(cache: Path, key: BundleKey, destination: Path) -> None:
    entry = cache / key
    if not (entry / 'manifest.json').is_file():
        raise ValueError(f'No compiled bundle at {entry}')
    with tarfile.open(destination, 'w:gz') as archive:
        archive.add(entry, arcname=key)


def extract_entry(archive_path: Path, cache: Path, key: BundleKey) -> None:
    """Extract only regular files under <key>/ and replace the entry atomically."""
    cache.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.restoring-', dir=cache) as folder:
        staging = Path(folder)
        with tarfile.open(archive_path, 'r:gz') as archive:
            members = archive.getmembers()
            for member in members:
                parts = Path(member.name).parts
                if '..' in parts or not parts or parts[0] != key or not (member.isfile() or member.isdir()):
                    raise ValueError(f'Unexpected archive member: {member.name}')
            archive.extractall(staging, members=members, filter='data')
        if not (staging / key / 'manifest.json').is_file():
            raise ValueError('Archive does not contain a bundle manifest')
        entry = cache / key
        if entry.exists():
            shutil.rmtree(entry)
        os.replace(staging / key, entry)


def pull(key: BundleKey, settings: Settings, cache: Path, wrangler: WranglerRunner) -> Outcome:
    with tempfile.TemporaryDirectory(prefix='bundle-pull-') as folder:
        archive = Path(folder) / 'bundle.tar.gz'
        result = wrangler(['r2', 'object', 'get', settings.object_path(key), '--file', str(archive), '--remote'])
        if result.returncode:
            if MISSING_OBJECT.search(result.stdout + result.stderr):
                return Outcome.MISS
            raise RuntimeError(f'wrangler r2 object get failed ({result.returncode}): {result.stderr.strip()}')
        extract_entry(archive, cache, key)
    return Outcome.HIT


def push(key: BundleKey, settings: Settings, cache: Path, wrangler: WranglerRunner) -> None:
    with tempfile.TemporaryDirectory(prefix='bundle-push-') as folder:
        archive = Path(folder) / 'bundle.tar.gz'
        archive_entry(cache, key, archive)
        result = wrangler(['r2', 'object', 'put', settings.object_path(key), '--file', str(archive),
                           '--content-type', 'application/gzip', '--remote'])
        if result.returncode:
            raise RuntimeError(f'wrangler r2 object put failed ({result.returncode}): {result.stderr.strip()}')


def write_output(name: str, value: str) -> None:
    path = os.environ.get('GITHUB_OUTPUT')
    if path:
        with open(path, 'a') as output:
            output.write(f'{name}={value}\n')


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', type=Command, choices=list(Command))
    parser.add_argument('--key', required=True, help='Bundle key printed by build-web-previews.py --print-key')
    parser.add_argument('--cache-dir', type=Path, default=CACHE)
    args = parser.parse_args()
    try:
        key = parse_key(args.key)
        settings = Settings.from_environment(dict(os.environ))
        match args.command:
            case Command.PULL:
                outcome = pull(key, settings, args.cache_dir, run_wrangler)
                print(f'Bundle cache {outcome.value}: {key}', flush=True)
                write_output('outcome', outcome.value)
            case Command.PUSH:
                push(key, settings, args.cache_dir, run_wrangler)
                print(f'Bundle cache stored: {key}', flush=True)
            case _:
                assert_never(args.command)
    except (ValueError, RuntimeError, OSError, tarfile.TarError) as error:
        raise SystemExit(f'Bundle cache failed: {error}') from None


if __name__ == '__main__':
    sys.exit(main())
