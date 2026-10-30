#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Capture the shared workspace through the actual winit/Skia showcase window."""
import hashlib
import json
import os
from pathlib import Path
import platform
import struct
import subprocess
from doc_inputs import ROOT, DOCS, source_hashes, write_json


def capture():
    subprocess.run(['cargo', 'build', '-p', 'kit-showcase', '--locked'], cwd=ROOT, check=True)
    executable = ROOT / 'target/debug/kit-showcase'
    output = DOCS / '.verification/product-workspace/native'
    output.mkdir(parents=True, exist_ok=True)
    environment = {**os.environ, 'SLINT_BACKEND': 'winit-skia'}
    frames = []
    for scene in ('account', 'projects', 'dialog'):
        for family in ('kit', 'tailwind'):
            for mode in ('light', 'dark'):
                for scale in (1, 2):
                    name = f'{scene}-{family}-{mode}-{scale}x.png'
                    arguments = ['--workspace', scene, '--width', '1100', '--height', '850']
                    if family == 'tailwind': arguments += ['--tailwind']
                    if mode == 'dark': arguments += ['--dark']
                    if scale == 2: arguments += ['--text-200']
                    frames.append((name, arguments))
    frames += [
        ('account-error.png', ['--workspace', 'account', '--tailwind', '--invalid']),
        ('dialog-error.png', ['--workspace', 'dialog', '--tailwind', '--invalid']),
        ('dialog-busy.png', ['--workspace', 'busy', '--tailwind']),
        ('dialog-narrow-fr-200.png', ['--workspace', 'dialog', '--tailwind', '--width', '760', '--height', '700', '--text-200', '--text-sample', 'french']),
    ]
    records = []
    for name, arguments in frames:
        path = output / name
        subprocess.run([str(executable), *arguments, '--snapshot', str(path)], cwd=ROOT, env=environment, check=True)
        data = path.read_bytes()
        records.append({'file': name, 'arguments': arguments, 'pixels': list(struct.unpack('>II', data[16:24])), 'sha256': hashlib.sha256(data).hexdigest()})
    sources = [path for directory in ('crates', 'showcase') for path in (ROOT / directory).rglob('*')
               if path.is_file() and path.suffix in ('.rs', '.slint', '.toml', '.ttf')]
    sources += [DOCS / 'src/assets/examples/workspace.slint']
    record = {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'backend': 'winit', 'renderer': 'Skia', 'device': subprocess.check_output(['sysctl', '-n', 'hw.model'], text=True).strip(),
              'system': platform.platform(), 'sourceHashes': source_hashes(sources), 'frames': records,
              'qualification': 'Native rendering only. Keyboard/pointer behavior is checked separately with the testing backend and actual Chrome WASM. IME, VoiceOver and embedded devices are not qualified.'}
    write_json(output / 'manifest.json', record)
    print(f'Captured {len(records)} actual winit/Skia workspace windows.')


if __name__ == '__main__': capture()
