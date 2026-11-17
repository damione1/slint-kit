#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Synchronize documentation dependency manifests from workspace.package.version."""
import argparse
import json
import re
import tomllib

from starter_sources import ROOT, with_kit_version, write_changed


def synchronize(check=False):
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']
    version = workspace['package']['version']
    starters = json.loads((ROOT / 'docs/content/starters.json').read_text())
    paths = [ROOT / 'docs/quickstart' / slug / 'Cargo.toml' for slug in sorted(starters)]
    paths += [ROOT / 'docs' / folder / 'Cargo.toml' for folder in ('consumer', 'table-consumer', 'dialog-consumer', 'preview')]
    workspace_preview = ROOT / 'docs/workspace-preview/Cargo.toml'
    paths.append(workspace_preview)
    stale = []
    for path in paths:
        source = path.read_text()
        kit_source = source if path == workspace_preview else with_kit_version(source, version)
        projected = with_toolchain_versions(kit_source, workspace)
        if source != projected:
            stale.append(str(path.relative_to(ROOT)))
            if not check:
                write_changed(path, projected)
    if check and stale:
        raise ValueError('Documentation versions differ from the workspace: ' + ', '.join(stale))
    print(f'Documentation dependency version {version}: {len(paths)} manifests, {len(stale)} changed')


def with_toolchain_versions(source, workspace):
    """Keep consumer MSRV and Slint pins aligned without changing features."""
    slint = workspace['dependencies']['slint']['version'].lstrip('=~')
    source = re.sub(r'(?m)^(rust-version\s*=\s*)"[^"]+"',
                    lambda match: match[1] + json.dumps(workspace['package']['rust-version']), source)
    return re.sub(r'(?m)^(slint(?:-build)?\s*=\s*(?:\{\s*version\s*=\s*)?)"[^"]+"',
                  lambda match: match[1] + json.dumps('=' + slint), source)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    try:
        synchronize(parser.parse_args().check)
    except (ValueError, KeyError) as error:
        raise SystemExit(str(error)) from None
