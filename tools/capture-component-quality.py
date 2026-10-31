#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Capture every public example natively with reproducible source hashes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def sources():
    paths = list((ROOT / 'crates/kit-slint/ui').rglob('*.slint'))
    paths += list((ROOT / 'docs/examples').glob('*.slint'))
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted(paths)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--modes', nargs='+', choices=('light', 'dark'), default=('light', 'dark'))
    parser.add_argument('--scales', nargs='+', type=int, choices=(1, 2), default=(1, 2))
    args = parser.parse_args()
    binary = args.binary.resolve()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    hashes = sources()
    frames = []
    expected = len(json.loads((ROOT / 'docs/content/components.json').read_text()))
    for mode in args.modes:
        for scale in args.scales:
            folder = output / f'{mode}-{scale}x'
            folder.mkdir(parents=True, exist_ok=True)
            options = ['--all']
            options += ['--output-dir', str(folder)]
            if mode == 'dark':
                options += ['--dark']
            if scale == 2:
                options += ['--text-200']
            subprocess.run([str(binary), *options], cwd=ROOT,
                           env={**os.environ, 'SLINT_BACKEND': 'winit-skia'}, check=True)
            images = sorted(folder.glob('*.png'))
            if len(images) != expected:
                raise RuntimeError(f'{folder}: expected {expected} frames, got {len(images)}')
            for path in images:
                data = path.read_bytes()
                frames.append({'file': str(path.relative_to(output)), 'mode': mode,
                               'textScale': scale, 'pixels': list(struct.unpack('>II', data[16:24])),
                               'sha256': hashlib.sha256(data).hexdigest()})
    if sources() != hashes:
        raise RuntimeError('UI source changed during capture; repeat with stabilized sources.')
    manifest = {
        'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'binarySha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
        'sourceHashes': hashes,
        'backend': 'winit', 'renderer': 'Skia', 'system': platform.platform(),
        'device': subprocess.check_output(['sysctl', '-n', 'hw.model'], text=True).strip(),
        'qualification': 'Native rendering. Does not qualify VoiceOver, IME, other devices or interactive states absent from the fixtures.',
        'frames': frames,
    }
    (output / 'manifest.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Captured {len(frames)} native frames from {expected} examples.', flush=True)


if __name__ == '__main__':
    main()
