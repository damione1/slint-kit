# SPDX-License-Identifier: GPL-3.0-or-later
"""Content-verified cache for the compiled preview assets, independent of git mtimes."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import tempfile

ASSETS = ('slint_kit_previews.js', 'slint_kit_previews.d.ts',
          'slint_kit_previews_bg.wasm', 'slint_kit_previews_bg.wasm.d.ts')


def compilation_key(root, files, rustc, bindgen, environment):
    flags = {name: value for name, value in environment.items()
             if name.startswith(('CARGO_PROFILE_RELEASE_', 'CARGO_TARGET_WASM32_', 'SLINT_'))
             and name not in ('SLINT_WEB_TARGET_DIR', 'SLINT_PREVIEW_COMPONENTS')}
    flags.update({name: environment.get(name, '')
                  for name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')})
    digest = hashlib.sha256(json.dumps({'format': 1, 'rustc': rustc, 'bindgen': bindgen,
                                       'flags': flags}, sort_keys=True).encode())
    # These two files are packaged for readers, but are not compiler inputs.
    for file in sorted(set(files) - {root / 'README.md', root / 'LICENSE'}):
        digest.update(str(file.relative_to(root)).encode() + b'\0')
        digest.update(file.read_bytes())
        digest.update(b'\0')
    return digest.hexdigest()


def restore(directory, key, destination):
    entry = directory / key
    try:
        manifest = json.loads((entry / 'manifest.json').read_text())
        if (not isinstance(manifest, dict) or manifest.get('key') != key
                or not isinstance(manifest.get('assets'), dict)
                or set(manifest['assets']) != set(ASSETS)):
            return False
        contents = {name: (entry / name).read_bytes() for name in ASSETS}
        if any(hashlib.sha256(data).hexdigest() != manifest['assets'][name]
               for name, data in contents.items()):
            return False
    except (OSError, ValueError, TypeError):
        return False
    for name, data in contents.items():
        (destination / name).write_bytes(data)
    return True


def save(directory, key, source):
    directory.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.writing-', dir=directory) as temporary:
        staging = Path(temporary) / key
        staging.mkdir()
        checksums = {}
        for name in ASSETS:
            data = (source / name).read_bytes()
            (staging / name).write_bytes(data)
            checksums[name] = hashlib.sha256(data).hexdigest()
        (staging / 'manifest.json').write_text(json.dumps({'key': key, 'assets': checksums}, indent=2))
        entry = directory / key
        if entry.exists():
            shutil.rmtree(entry)
        os.replace(staging, entry)
