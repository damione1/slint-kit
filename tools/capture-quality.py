#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Capture the actual showcase across pages, appearance and text sizes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PAGES = ('gallery', 'profile', 'billing', 'choices', 'schedule', 'files',
         'display', 'navigation', 'data')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--pages', nargs='+', choices=PAGES, default=PAGES)
    parser.add_argument('--positions', nargs='+', choices=('top', 'middle', 'end'),
                        default=('top', 'middle', 'end'))
    parser.add_argument('--revision', help='Source revision of an archived binary')
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=True)
    binary = args.binary.resolve()
    revision = args.revision or subprocess.check_output(
        ['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
    frames = []
    for page in args.pages:
        for mode in ('light', 'dark'):
            for scale in (1, 2):
                for position in args.positions:
                    name = f'{page}-{mode}-{scale}x-{position}.png'
                    path = args.output / name
                    options = ['--page', page, '--position', position]
                    if mode == 'dark':
                        options += ['--dark']
                    if scale == 2:
                        options += ['--text-200']
                    result = subprocess.run(
                        [str(binary), *options, '--snapshot', str(path.resolve())],
                        cwd=ROOT, env={**os.environ, 'SLINT_BACKEND': 'winit-skia'},
                        capture_output=True, text=True, timeout=45)
                    if result.returncode:
                        raise RuntimeError(f'{name}: {result.stdout}\n{result.stderr}')
                    data = path.read_bytes()
                    frames.append({'file': name, 'arguments': options,
                                   'pixels': list(struct.unpack('>II', data[16:24])),
                                   'sha256': hashlib.sha256(data).hexdigest()})
                    print(name, flush=True)
    manifest = {
        'revision': revision,
        'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
        'backend': 'winit', 'renderer': 'Skia', 'system': platform.platform(),
        'device': subprocess.check_output(['sysctl', '-n', 'hw.model'], text=True).strip(),
        'qualification': 'Actual native rendering. Does not qualify VoiceOver, IME or other devices.',
        'frames': frames,
    }
    (args.output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')


if __name__ == '__main__':
    main()
