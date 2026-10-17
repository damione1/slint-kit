#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Record native screenshot inputs and hashes; never generates image content."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import platform
import re
import struct
import subprocess

ROOT = Path(__file__).resolve().parents[2]
ASSETS = ROOT / 'docs/src/assets/components'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dimensions(path):
    with path.open('rb') as stream:
        header = stream.read(24)
    if header[:8] != b'\x89PNG\r\n\x1a\n':
        raise SystemExit(f'Not a PNG capture: {path}')
    return struct.unpack('>II', header[16:24])


def capture_record(example, filename, locale):
    screenshot = ASSETS / filename
    if not screenshot.is_file():
        raise SystemExit(f'Missing native capture: {screenshot}')
    width, height = dimensions(screenshot)
    source = example.read_text()
    logical_width = int(re.search(r'^    preferred-width: (\d+)px;', source, re.M).group(1))
    logical_height = int(re.search(r'^    preferred-height: (\d+)px;', source, re.M).group(1))
    variants = []
    for suffix, theme, scale, interaction in [
        ('', 'light', 1, False), ('-dark', 'dark', 1, False),
        ('-text-200', 'light', 2, False), ('-dark-text-200', 'dark', 2, False),
        ('-interaction', 'light', 1, True),
    ]:
        suffix = ('-fr' if locale == 'fr' else '') + suffix
        image = ASSETS / (example.stem + suffix + '.png')
        if not image.is_file():
            continue
        w, h = dimensions(image)
        variants.append({
            'file': image.name, 'theme': theme, 'textScale': scale,
            'reducedMotion': True, 'width': w, 'height': h,
            'sha256': sha(image),
            'capturedAt': datetime.fromtimestamp(image.stat().st_mtime, timezone.utc).isoformat(),
            'interactionChecked': interaction,
        })
    return {
        'exampleSha256': sha(example), 'screenshotSha256': sha(screenshot),
        'width': width, 'height': height, 'logicalWidth': logical_width,
        'preferredLogicalHeight': logical_height, 'variants': variants,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reviewed-components', type=int, default=0,
                        help='Number of localized component captures visually reviewed (0–246); record only a completed review.')
    args = parser.parse_args()
    if not 0 <= args.reviewed_components <= 246:
        parser.error('--reviewed-components must be between 0 and 246')
    components = {}
    for example in sorted((ROOT / 'docs/examples').glob('*.slint')):
        slug = example.stem
        english = capture_record(example, slug + '.png', 'en')
        french = ROOT / 'docs/examples/fr' / example.name
        if not french.is_file():
            raise SystemExit(f'Missing French example: {french}')
        english['locales'] = {'fr': capture_record(french, slug + '-fr.png', 'fr')}
        components[slug] = english

    kit_ui_digest = hashlib.sha256()
    for source in sorted((ROOT / 'crates/kit-slint/ui').glob('*.slint'), key=lambda path: path.name):
        kit_ui_digest.update(source.name.encode() + b'\0')
        kit_ui_digest.update(source.read_bytes() + b'\0')
    device = subprocess.check_output(['sysctl', '-n', 'machdep.cpu.brand_string'], text=True).strip() if platform.system() == 'Darwin' else platform.machine()
    manifest = {
        'schemaVersion': 1,
        'environment': {
            'backend': 'winit', 'renderer': 'skia', 'snapshotAPI': 'slint::Window::take_snapshot',
            'slintVersion': '1.18.1', 'platform': platform.system() + ' ' + platform.mac_ver()[0],
            'architecture': platform.machine(), 'device': device,
            'display': 'Native desktop display; captured pixels recorded per image',
            'scaleFactor': components['button']['width'] / components['button']['logicalWidth'],
            'captureSourceSha256': sha(ROOT / 'docs/capture/src/main.rs'),
            'kitUiSha256': kit_ui_digest.hexdigest(),
        },
        'qualification': {
            'presentation': 'Native light-theme captures in French and English; representative French and English dark and 200% text variants.',
            'visuallyReviewedLocalizedComponents': args.reviewed_components,
            'interaction': 'French and English Button pointer press/release and keyboard Space callbacks checked through public native window event dispatch.',
            'ime': False, 'screenReader': False, 'otherPlatforms': False,
            'localization': 'Authored fixture strings and exposed labels are localized; hardcoded toolkit defaults retain their shipped language.',
        },
        'components': components,
    }
    output = ROOT / 'docs/content/captures.json'
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    count = sum(len(record['variants']) + len(record['locales']['fr']['variants']) for record in components.values())
    print(f'Recorded {len(components)} components, {count} native localized/variant PNGs')


if __name__ == '__main__':
    main()
