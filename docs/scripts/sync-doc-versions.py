#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Synchronize documentation dependency manifests from workspace.package.version."""
import argparse
import json
import tomllib

from starter_sources import ROOT, with_kit_version, write_changed


def synchronize(check=False):
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']['package']['version']
    starters = json.loads((ROOT / 'docs/content/starters.json').read_text())
    paths = [ROOT / 'docs/quickstart' / slug / 'Cargo.toml' for slug in sorted(starters)]
    paths += [ROOT / 'docs' / folder / 'Cargo.toml' for folder in ('consumer', 'table-consumer', 'preview')]
    stale = []
    for path in paths:
        source = path.read_text()
        projected = with_kit_version(source, version)
        if source != projected:
            stale.append(str(path.relative_to(ROOT)))
            if not check:
                write_changed(path, projected)
    if check and stale:
        raise ValueError('Documentation versions differ from the workspace: ' + ', '.join(stale))
    print(f'Documentation dependency version {version}: {len(paths)} manifests, {len(stale)} changed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    try:
        synchronize(parser.parse_args().check)
    except (ValueError, KeyError) as error:
        raise SystemExit(str(error)) from None
