#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Capture clean native starter posters in fresh processes, before any input test."""
import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import socket
import struct
import subprocess
import sys
import time

DOCS = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(DOCS / 'capture'))
from mcp import call, value  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('slugs', nargs='+')
    parser.add_argument('--activate', help='Open a dialog/drawer with this button before capture; never enter test text.')
    parser.add_argument('--target-dir', type=Path, required=True,
                        help='Shared native Cargo target; never create one target per starter.')
    args = parser.parse_args()
    entries = json.loads((DOCS / 'content/starters.json').read_text())
    captures_path = DOCS / 'content/starter-captures.json'
    captures = json.loads(captures_path.read_text())
    environment = {**os.environ, 'CARGO_TARGET_DIR': str(args.target_dir.resolve()),
                   'SLINT_EMIT_DEBUG_INFO': '1'}
    evidence = DOCS / '.verification/starter-posters'
    evidence.mkdir(parents=True, exist_ok=True)
    for slug in args.slugs:
        if slug not in entries:
            parser.error(f'Unknown component: {slug}')
        starter = DOCS / 'quickstart' / slug
        with (evidence / f'{slug}-build.log').open('w') as log:
            subprocess.run(['cargo', 'build', '--manifest-path', str(starter / 'Cargo.toml'),
                            '--locked', '--features', 'slint/mcp'], env=environment,
                           stdout=log, stderr=log, check=True)
        with socket.socket() as listener:
            listener.bind(('127.0.0.1', 0))
            port = listener.getsockname()[1]
        binary = args.target_dir / 'debug' / f'slint-kit-{slug}-example'
        with (evidence / f'{slug}-runtime.log').open('w') as log, subprocess.Popen(
                [str(binary)], env={**environment, 'SLINT_MCP_PORT': str(port),
                                   'SLINT_BACKEND': 'winit-skia', 'SLINT_SCALE_FACTOR': '2'},
                stdout=log, stderr=log) as process:
            try:
                for _ in range(300):
                    if process.poll() is not None:
                        raise RuntimeError(f'{slug} exited before capture')
                    try:
                        windows = value(port, 'list_windows')['windowHandles']
                        if windows:
                            break
                    except (OSError, RuntimeError):
                        pass
                    time.sleep(.1)
                else:
                    raise TimeoutError(f'{slug}: native window did not start')
                window = windows[0]
                properties = value(port, 'get_window_properties', windowHandle=window)
                assert properties['scaleFactor'] == 2
                if args.activate:
                    tree = value(port, 'get_element_tree',
                                 elementHandle=properties['rootElementHandle'], maxElements=2000)
                    opener = next(element for element in tree['elements']
                                  if element.get('accessibleRole') == 'Button'
                                  and element.get('accessibleLabel') == args.activate)
                    value(port, 'click_element', elementHandle=opener['handle'])
                    time.sleep(.3)
                tree = value(port, 'get_element_tree',
                             elementHandle=properties['rootElementHandle'], maxElements=2000)
                (evidence / f'{slug}-initial.json').write_text(json.dumps(tree, indent=2) + '\n')
                # A poster uses a fresh window, optionally opening its overlay.
                # Input/interaction tests must never overwrite this public file.
                image = next(block for block in call(port, 'take_screenshot', windowHandle=window)
                             if block['type'] == 'image')
                pixels = base64.b64decode(image['data'])
                (starter / 'preview@2x.png').write_bytes(pixels)
                width, height = struct.unpack('>II', pixels[16:24])
                captures[slug] = {
                    'environment': {'platform': 'macOS arm64', 'backend': 'winit',
                                    'renderer': 'skia', 'scale_factor': properties['scaleFactor']},
                    'source_sha256': {name: hashlib.sha256((starter / name).read_bytes()).hexdigest()
                                      for name in ['Cargo.toml', 'Cargo.lock', 'build.rs',
                                                   'src/main.rs', 'ui/app.slint']},
                    'image': {'file': 'preview@2x.png', 'sha256': hashlib.sha256(pixels).hexdigest(),
                              'width': width, 'height': height},
                }
                captures_path.write_text(json.dumps(captures, ensure_ascii=False, indent=2) + '\n')
                print(f'{slug}: clean native poster {width}×{height}', flush=True)
            finally:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()


if __name__ == '__main__':
    main()
