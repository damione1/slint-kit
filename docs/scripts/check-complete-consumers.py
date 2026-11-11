#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Compile the complete downloadable applications against local or registry crates."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

from starter_sources import BuildMode, KIT_PACKAGES, ROOT, local_patches

CONSUMERS = ('consumer', 'table-consumer', 'dialog-consumer')


def verify_sources(packages, mode):
    for package in packages:
        if package['name'] not in KIT_PACKAGES:
            continue
        source = package.get('source')
        if mode is BuildMode.RELEASE and not (source or '').startswith('registry+'):
            raise ValueError(f'{package["name"]}: release consumer resolved a local dependency')
        if mode is BuildMode.LOCAL and source:
            raise ValueError(f'{package["name"]}: local consumer resolved a registry dependency')


def copy_sources(destination, name, revision):
    if revision is None:
        shutil.copytree(ROOT / 'docs' / name, destination)
        return
    prefix = f'docs/{name}/'
    files = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', revision, '--', prefix], cwd=ROOT, text=True).splitlines()
    if not files:
        raise ValueError(f'{name}: no consumer at {revision}')
    for file in files:
        relative = Path(file).relative_to(prefix)
        path = destination / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(subprocess.check_output(['git', 'show', f'{revision}:{file}'], cwd=ROOT))


def parse_consumer(value):
    if value not in CONSUMERS:
        raise argparse.ArgumentTypeError(f'unknown consumer {value!r}; choose from {", ".join(CONSUMERS)}')
    return value


def parse_arguments(arguments=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', required=True, choices=[mode.value for mode in BuildMode])
    parser.add_argument('--revision', help='Verify an immutable release example instead of the current source')
    # Python 3.11 checks an empty nargs='*' list against choices as one value.
    # Validate each supplied name instead, preserving the all-consumers default.
    parser.add_argument('consumers', nargs='*', type=parse_consumer, metavar='CONSUMER')
    return parser.parse_args(arguments)


def main():
    args = parse_arguments()
    mode = BuildMode(args.mode)
    if args.revision and (args.revision.startswith('-') or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._/-' for c in args.revision)):
        raise ValueError('Invalid source revision')
    environment = {**os.environ, 'CARGO_TARGET_DIR': os.environ.get('CARGO_TARGET_DIR', str(ROOT / 'target/docs-consumers'))}
    with tempfile.TemporaryDirectory(prefix='kit-complete-consumers-') as temporary:
        for name in args.consumers or CONSUMERS:
            directory = Path(temporary) / name
            copy_sources(directory, name, args.revision)
            manifest = directory / 'Cargo.toml'
            if mode is BuildMode.LOCAL:
                manifest.write_text(manifest.read_text() + local_patches())
            command = ['--manifest-path', str(manifest)]
            metadata = json.loads(subprocess.check_output(['cargo', 'metadata', *command, '--format-version', '1'], cwd=ROOT, env=environment, text=True))
            verify_sources(metadata['packages'], mode)
            subprocess.run(['cargo', 'check', *command, '--locked'], cwd=ROOT, env=environment, check=True)
            print(f'Checked complete {name} against {mode.value} crates' + (f' from {args.revision}' if args.revision else ''), flush=True)


if __name__ == '__main__':
    main()
