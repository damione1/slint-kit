#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check complete consumers against workspace sources or the published registry."""
import argparse
import json
import os
import subprocess

from starter_sources import BuildMode, KIT_PACKAGES, ROOT, prepare, source_directory


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', required=True, choices=[mode.value for mode in BuildMode])
    parser.add_argument('slugs', nargs='*')
    args = parser.parse_args()
    mode = BuildMode(args.mode)
    consumers = prepare(mode)
    environment = {**os.environ, 'CARGO_TARGET_DIR': os.environ.get('CARGO_TARGET_DIR', str(ROOT / 'target/docs-consumers'))}
    for slug in args.slugs or sorted(consumers):
        directory = source_directory(slug, mode, native=True)
        command = ['--manifest-path', str(directory / 'Cargo.toml'), '--locked']
        metadata = json.loads(subprocess.check_output(['cargo', 'metadata', *command, '--format-version', '1'],
                                                     cwd=ROOT, env=environment, text=True))
        for package in metadata['packages']:
            if package['name'] in KIT_PACKAGES:
                if mode is BuildMode.RELEASE and not (package.get('source') or '').startswith('registry+'):
                    raise ValueError(f'{slug}: release consumer resolves a local {package["name"]}')
                if mode is BuildMode.LOCAL and package.get('source'):
                    raise ValueError(f'{slug}: local consumer resolves a registry {package["name"]}')
        subprocess.run(['cargo', 'check', *command], cwd=ROOT, env=environment, check=True)
        print(f'Checked {slug} against {mode.value} dependencies', flush=True)


if __name__ == '__main__':
    main()
