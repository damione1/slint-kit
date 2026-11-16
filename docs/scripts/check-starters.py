#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Check complete consumers against workspace sources or the published registry.

Starters that share a dependency graph are checked by one `cargo check` over a synthetic
workspace; errors are attributed to the starter whose package produced them.
"""
import argparse
from dataclasses import dataclass
import json
import os
from pathlib import Path
import re
import subprocess
import sys

from starter_sources import (BuildMode, KIT_PACKAGES, ROOT, dependency_groups,
                             group_workspace, source_directory)


@dataclass(frozen=True)
class GroupResult:
    """Outcome of one grouped `cargo check`: starters with an error and unattributed output."""
    failed: tuple[str, ...]
    unattributed: bool
    diagnostics: tuple[str, ...]

    @property
    def passed(self):
        return not self.failed and not self.unattributed


# A Slint error in a starter's build.rs is a build-script failure: cargo reports it as plain
# stderr text, with no compiler-message record naming the manifest.
BUILD_SCRIPT_FAILURE = re.compile(r'failed to run custom build command for `([^`\s]+) v')


def verify_sources(mode, metadata, owners):
    for package in metadata['packages']:
        if package['name'] in KIT_PACKAGES:
            if mode is BuildMode.RELEASE and not (package.get('source') or '').startswith('registry+'):
                raise ValueError(f'{owners}: release consumers resolve a local {package["name"]}')
            if mode is BuildMode.LOCAL and package.get('source'):
                raise ValueError(f'{owners}: local consumers resolve a registry {package["name"]}')


def attribute(stdout, directory, group, stderr=''):
    """Map each error to a starter: JSON messages by manifest path, build-script failures by package name."""
    by_manifest = {str((directory / slug / 'Cargo.toml').resolve()): slug for slug, _ in group.packages}
    failed = set()
    diagnostics = []
    unattributed = False
    by_package = {package: slug for slug, package in group.packages}
    for package in BUILD_SCRIPT_FAILURE.findall(stderr):
        slug = by_package.get(package)
        if slug is None:
            unattributed = True
        else:
            failed.add(slug)
    for line in stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if not isinstance(message, dict):
            continue
        reason = message.get('reason')
        if reason == 'compiler-message':
            if message['message']['level'] not in ('error', 'error: internal compiler error'):
                continue
            slug = by_manifest.get(str(Path(message.get('manifest_path', '')).resolve()))
            if slug is None:
                unattributed = True
            else:
                failed.add(slug)
            diagnostics.append(message['message'].get('rendered') or message['message']['message'])
        elif reason == 'build-finished' and not message.get('success', False):
            unattributed = unattributed or not failed
    return GroupResult(tuple(sorted(failed)), unattributed, tuple(diagnostics))


def check_group(mode, group, consumers, environment):
    directory = group_workspace(mode, group, consumers)
    command = ['--manifest-path', str(directory / 'Cargo.toml'), '--locked']
    metadata = json.loads(subprocess.check_output(['cargo', 'metadata', *command, '--format-version', '1'],
                                                 cwd=ROOT, env=environment, text=True))
    verify_sources(mode, metadata, ', '.join(slug for slug, _ in group.packages))
    # --keep-going reports every failing starter of the group, not only the first.
    done = subprocess.run(['cargo', 'check', '--workspace', '--keep-going', *command, '--message-format=json'],
                          cwd=ROOT, env=environment, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    print(done.stderr, end='', file=sys.stderr, flush=True)
    result = attribute(done.stdout, directory, group, done.stderr)
    if done.returncode != 0 and result.passed:
        result = GroupResult(result.failed, True, result.diagnostics)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', required=True, choices=[mode.value for mode in BuildMode])
    parser.add_argument('slugs', nargs='*')
    args = parser.parse_args()
    mode = BuildMode(args.mode)
    groups = dependency_groups(mode, tuple(args.slugs))
    consumers = {slug: source_directory(slug, mode) for group in groups for slug, _ in group.packages}
    environment = {**os.environ, 'CARGO_TARGET_DIR': os.environ.get('CARGO_TARGET_DIR', str(ROOT / 'target/docs-consumers'))}
    failures = []
    for group in groups:
        result = check_group(mode, group, consumers, environment)
        for text in result.diagnostics:
            print(text, file=sys.stderr, flush=True)
        slugs = [slug for slug, _ in group.packages]
        failures.extend(result.failed)
        if result.unattributed:
            failures.append(f'unattributed failure in group of {", ".join(slugs)}')
        if result.passed:
            print(f'Checked {len(slugs)} starters against {mode.value} dependencies: {", ".join(slugs)}', flush=True)
    if failures:
        raise SystemExit('Failed starters: ' + ', '.join(failures))


if __name__ == '__main__':
    main()
