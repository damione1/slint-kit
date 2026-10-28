#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Capture the running native example through Slint's embedded MCP server.

Build first with SLINT_EMIT_DEBUG_INFO=1 cargo build --locked
  -p kit-docs-capture --features slint/mcp
No MCP feature is added to the shipped library's default configuration.
"""
import argparse
import base64
import json
import os
from pathlib import Path
import socket
import subprocess
import struct
import time
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / 'docs/src/assets/components'


def call(port, name, **arguments):
    request = urllib.request.Request(
        f'http://127.0.0.1:{port}/mcp',
        data=json.dumps({'jsonrpc': '2.0', 'id': 1, 'method': 'tools/call',
                         'params': {'name': name, 'arguments': arguments}}).encode(),
        headers={'Content-Type': 'application/json',
                 'Accept': 'application/json, text/event-stream'})
    with urllib.request.urlopen(request, timeout=20) as response:
        raw = response.read().decode()
    data = json.loads(next(line[6:] for line in raw.splitlines() if line.startswith('data: '))) if raw.startswith('event:') else json.loads(raw)
    if 'error' in data or data.get('result', {}).get('isError'):
        raise RuntimeError(data)
    return data['result']['content']


def value(port, name, **arguments):
    return json.loads(next(block['text'] for block in call(port, name, **arguments)
                           if block['type'] == 'text'))


def capture(binary, slug, locale, suffix, interaction=None, pixel_density=1):
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    args = [str(binary), slug, '--show', '--locale', locale]
    if '-dark' in suffix:
        args.append('--dark')
    if '-text-200' in suffix:
        args.append('--text-200')
    if pixel_density == 2:
        baseline = ASSETS / f'{slug}{"-fr" if locale == "fr" else ""}{suffix}.png'
        width, height = struct.unpack('>II', baseline.read_bytes()[16:24])
        args += ['--size', f'{width}x{height}']
    env = {**os.environ, 'SLINT_MCP_PORT': str(port), 'SLINT_SCALE_FACTOR': str(pixel_density),
           'SLINT_EMIT_DEBUG_INFO': '1'}
    with subprocess.Popen(args, cwd=ROOT, env=env, stdout=subprocess.DEVNULL,
                          stderr=subprocess.PIPE) as process:
        try:
            deadline = time.monotonic() + 30
            while True:
                if process.poll() is not None:
                    raise RuntimeError(process.stderr.read().decode())
                try:
                    windows = value(port, 'list_windows')['windowHandles']
                    if windows:
                        window = windows[0]
                        break
                except (urllib.error.URLError, TimeoutError):
                    pass
                if time.monotonic() > deadline:
                    raise TimeoutError(f'MCP did not start: {slug}/{locale}')
                time.sleep(0.1)
            time.sleep(0.3)
            properties = value(port, 'get_window_properties', windowHandle=window)
            if properties['scaleFactor'] != pixel_density:
                raise RuntimeError(f'Expected native scale factor {pixel_density}: {properties}')
            if interaction:
                properties = value(port, 'get_window_properties', windowHandle=window)
                root = properties['rootElementHandle']
                if interaction == 'pointer':
                    tree = value(port, 'get_element_tree', elementHandle=root, maxElements=1000)
                    button = next(e for e in tree['elements'] if e.get('accessibleRole') == 'Button')
                    call(port, 'click_element', elementHandle=button['handle'])
                else:
                    call(port, 'dispatch_key_event', windowHandle=window, text='\t')
                    call(port, 'dispatch_key_event', windowHandle=window, text=' ')
                tree = value(port, 'get_element_tree', elementHandle=root, maxElements=1000)
                expected = 'Action demandée' if locale == 'fr' else 'Action requested'
                if not any(e.get('accessibleLabel') == expected for e in tree['elements']):
                    raise RuntimeError(f'{interaction} did not activate {slug}/{locale}')
            blocks = call(port, 'take_screenshot', windowHandle=window)
            image = next(block for block in blocks if block['type'] == 'image')
            output = ASSETS / f'{slug}{"-fr" if locale == "fr" else ""}{suffix}{"@2x" if pixel_density == 2 else ""}.png'
            output.write_bytes(base64.b64decode(image['data'], validate=True))
            print(f'MCP {output.name}' + (f' ({interaction} verified)' if interaction else ''), flush=True)
        finally:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('slugs', nargs='*')
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--locale', choices=['en', 'fr', 'both'], default='both')
    parser.add_argument('--existing-variants', action='store_true')
    parser.add_argument('--pixel-density', type=int, choices=[1, 2], default=1)
    parser.add_argument('--exercise', action='store_true', help='Fresh Button runs: pointer, then Tab/Space')
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    slugs = args.slugs or sorted(p.stem for p in (ROOT / 'docs/examples').glob('*.slint'))
    for locale in (['en', 'fr'] if args.locale == 'both' else [args.locale]):
        for slug in slugs:
            if not (ROOT / 'docs/examples' / (f'fr/{slug}.slint' if locale == 'fr' else f'{slug}.slint')).is_file():
                parser.error(f'Unknown example: {slug}')
            suffixes = ['']
            if args.existing_variants:
                suffixes += [s for s in ['-dark', '-text-200', '-dark-text-200']
                             if (ASSETS / f'{slug}{"-fr" if locale == "fr" else ""}{s}.png').exists()]
            for suffix in suffixes:
                capture(binary, slug, locale, suffix, pixel_density=args.pixel_density)
        if args.exercise:
            # Separate processes ensure a stale status cannot satisfy either check.
            capture(binary, 'button', locale, '-interaction', 'pointer', args.pixel_density)
            capture(binary, 'button', locale, '-interaction', 'keyboard', args.pixel_density)


if __name__ == '__main__':
    main()
