#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Plan and capture only changed scenes through the actual browser previews."""
import argparse
from enum import StrEnum
import json
import math
import os
from pathlib import Path
import re
import struct
import subprocess
import tarfile
import tomllib

from doc_inputs import DOCS, ROOT, build_inputs, digest, key_for, runtime_inputs, write_changed, write_json, consumer_identity, compiler_environment
from doc_inputs import archived_runtime_inputs, rendering_runtime


class SceneKind(StrEnum):
    CATALOGUE = 'catalogue'
    STARTER = 'starter'


class ImageOrigin(StrEnum):
    NATIVE = 'native'
    BROWSER = 'browser'
    DERIVED = 'derived'


class PosterAction(StrEnum):
    CLICK = 'click'
    HOVER = 'hover'
    FOCUS_FIRST = 'focus-first'


def poster_activation(value):
    if value is None:
        return None
    if isinstance(value, list):
        value = {'action': PosterAction.CLICK.value, 'point': value}
    if value == {'action': PosterAction.FOCUS_FIRST.value}:
        return {'action': PosterAction.FOCUS_FIRST.value}
    if not isinstance(value, dict) or set(value) != {'action', 'point'}:
        raise ValueError('Poster activation needs an action and point')
    action = PosterAction(value['action'])
    if action == PosterAction.FOCUS_FIRST:
        raise ValueError('Keyboard poster activation does not take coordinates')
    point = value['point']
    if (not isinstance(point, list) or len(point) != 2
            or any(isinstance(number, bool) or not isinstance(number, (int, float))
                   or not math.isfinite(number) or number < 0 for number in point)):
        raise ValueError('Poster activation needs two finite, nonnegative coordinates')
    return {'action': action.value, 'point': point}


MANIFEST = DOCS / 'content/render-captures.json'
PROFILE = {'slint': '1.18.1', 'density': 2, 'format': 1, 'reducedMotion': True}
BROWSER_PROFILE = {'backend': 'browser', 'renderer': 'femtovg', 'pipeline': 1,
                   'captureDriverSha256': digest((DOCS / 'scripts/capture-browser.mjs').read_bytes()),
                   'bundleBuilder': digest((DOCS / 'scripts/build-web-previews.py').read_bytes()),
                   'browserHost': {name: digest((DOCS / 'preview' / name).read_bytes())
                                   for name in ('build.rs', 'src/lib.rs', 'loader.js')}}
COMPILER_FLAGS = {name: value for name, value in compiler_environment().items()
                  if name in ('RUSTFLAGS', 'CARGO_ENCODED_RUSTFLAGS')
                  or name.startswith(('CARGO_PROFILE_RELEASE_', 'CARGO_TARGET_WASM32_'))}
if COMPILER_FLAGS:
    BROWSER_PROFILE['compilerFlags'] = COMPILER_FLAGS


def dimensions(path):
    data = Path(path).read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError(f'Invalid PNG: {path}')
    return struct.unpack('>II', data[16:24])


def downscale(source, destination):
    from PIL import Image
    with Image.open(source) as image:
        if image.width % 2 or image.height % 2:
            raise ValueError('A ×2 capture must have even dimensions')
        image = image.convert('RGBA').resize((image.width // 2, image.height // 2), Image.Resampling.LANCZOS)
        import io
        stream = io.BytesIO()
        image.save(stream, format='PNG', optimize=False, compress_level=9)
        write_changed(destination, stream.getvalue())


def expected_outputs(task):
    return [str(path.relative_to(ROOT)) for path in task['outputs']]


def valid_metadata(task, record):
    if not isinstance(record, dict) or record.get('sceneKey') != task['sceneKey']:
        return False
    if not isinstance(record.get('renderProfile'), dict):
        return False
    if record.get('key') != digest({'scene': task['sceneKey'], 'profile': record.get('renderProfile')}):
        return False
    outputs = record.get('outputs')
    if not isinstance(outputs, dict):
        return False
    expected = set(expected_outputs(task))
    if record.get('legacyNative'):
        if (not set(outputs) <= expected or str(task['outputs'][1].relative_to(ROOT)) not in outputs
                or not isinstance(record.get('origin'), dict)
                or record['origin'].get('method') != 'imported-native-proof'
                or record.get('renderProfile', {}).get('backend') != 'winit'):
            return False
    elif set(outputs) != expected or record.get('renderProfile') != BROWSER_PROFILE:
        return False
    return all(isinstance(info, dict) and isinstance(info.get('sha256'), str) and re.fullmatch('[a-f0-9]{64}', info.get('sha256', '')) for info in outputs.values())


def valid_record(task, record):
    if not valid_metadata(task, record):
        return False
    for name, info in record['outputs'].items():
        path = ROOT / name
        if not path.is_file() or digest(path.read_bytes()) != info['sha256']:
            return False
    return True


def tasks(inputs, runtime=None):
    old = json.loads((DOCS / 'content/captures.json').read_text())
    shared = {'runtime': digest(runtime_inputs() if runtime is None else runtime), 'driver': 'documentation-scene-v1'}
    result = []
    for source in sorted((DOCS / 'examples').glob('*.slint')):
        slug = source.stem
        for locale in ['en', 'fr']:
            actual = source if locale == 'en' else source.parent / 'fr' / source.name
            text = actual.read_text()
            width = int(re.search(r'preferred-width:\s*(\d+)px', text)[1])
            height = int(re.search(r'preferred-height:\s*(\d+)px', text)[1])
            record = old['components'].get(slug, {})
            if locale == 'fr':
                record = record.get('locales', {}).get('fr', {})
            variants = {('light', 1)} | {(v['theme'], v['textScale']) for v in record.get('variants', []) if not v.get('interactionChecked')}
            for theme, scale in sorted(variants):
                suffix = ('-dark' if theme == 'dark' else '') + ('-text-200' if scale == 2 else '')
                stem = slug + ('-fr' if locale == 'fr' else '') + suffix
                viewport = [1000, 1000] if scale == 2 else [width, height]
                recipe = {**PROFILE, 'locale': locale, 'theme': theme, 'textScale': scale, 'viewport': viewport}
                reference = f'catalogue:{slug}:{locale}:{theme}'
                identifier = reference + f':{scale}'
                outputs = [DOCS / 'src/assets/components' / (stem + '@2x.png'),
                           DOCS / 'src/assets/components' / (stem + '.png')]
                result.append({'id': identifier, 'kind': SceneKind.CATALOGUE, 'slug': slug, 'locale': locale,
                               'suffix': suffix, 'recipe': recipe, 'outputs': outputs,
                               'key': key_for(reference, inputs, recipe, shared)})
    entries = json.loads((DOCS / 'content/starters.json').read_text())
    for slug in sorted(entries):
        source = DOCS / 'quickstart' / slug
        ui = (source / 'ui/app.slint').read_text()
        activation = poster_activation(entries[slug].get('posterActivation'))
        driver = {**shared, 'consumer': consumer_identity(slug), 'host': digest((source / 'src/main.rs').read_bytes()),
                  'build': digest((source / 'build.rs').read_bytes())}
        for theme in ('light', 'dark'):
            recipe = {**PROFILE, 'reducedMotion': False, 'theme': theme, 'locale': 'en', 'textScale': 1,
                      'activation': activation, 'viewport': [int(re.search(r'preferred-width:\s*(\d+)px', ui)[1]),
                                                             int(re.search(r'preferred-height:\s*(\d+)px', ui)[1])]}
            suffix = '-dark' if theme == 'dark' else ''
            result.append({'id': f'starter:{slug}' + (':dark' if theme == 'dark' else ''),
                           'kind': SceneKind.STARTER, 'slug': slug, 'recipe': recipe,
                           'outputs': [source / f'preview{suffix}@2x.png', source / f'preview{suffix}.png'],
                           'key': key_for(f'starter:{slug}:tailwind', inputs, recipe, driver)})
    for task in result:
        task['sceneKey'] = task['key']
        task['key'] = digest({'scene': task['sceneKey'], 'profile': BROWSER_PROFILE})
    return result


def reuse_build_only_changes(work, manifest, inputs):
    """Retain original capture provenance after proving rendering inputs equal."""
    current = runtime_inputs()
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())

    def build_independent_workspace(value):
        value = json.loads(json.dumps(value))
        value['workspace']['package'].pop('version', None)
        for name, dependency in value['workspace']['dependencies'].items():
            if name.startswith('kit-') or name == 'slint-kit':
                dependency.pop('version', None)
        return value

    for folder in sorted((DOCS / '.cache/preview-bundles').glob('*')):
        try:
            proof = json.loads((folder / 'manifest.json').read_text())
            archive_path = folder / 'source.tar.gz'
            if proof['assets']['source.tar.gz'] != digest(archive_path.read_bytes()):
                continue
            if proof['assets']['component.json'] != digest((folder / 'component.json').read_bytes()):
                continue
            component = json.loads((folder / 'component.json').read_text())
            checksums = component['source_sha256']
            with tarfile.open(archive_path, 'r:gz') as archive:
                sources = {entry.name.removeprefix('slint-kit/'): archive.extractfile(entry).read()
                           for entry in archive if entry.isfile()}
            if digest(sources['Cargo.toml']) != checksums['Cargo.toml']:
                # The source packager removes applications absent from the tar.
                # Restore that exact projection and verify its original hash.
                suffix = b', "showcase", "tests", "xtask", "docs/capture", "docs/integration"'
                sources['Cargo.toml'] = re.sub(rb'(?m)^(members = \[.*)(\])$',
                                               lambda match: match[1] + suffix + match[2],
                                               sources['Cargo.toml'], count=1)
            if set(sources) != set(checksums) or any(digest(content) != checksums[name] for name, content in sources.items()):
                continue
            previous = archived_runtime_inputs(sources)
            if rendering_runtime(previous) != rendering_runtime(current):
                continue
            # A package-version-sensitive runtime must be rebuilt and captured.
            if any(b'CARGO_PKG_VERSION' in content for name, content in sources.items()
                   if name in rendering_runtime(previous)['rust']):
                continue
            if build_independent_workspace(tomllib.loads(sources['Cargo.toml'].decode())) != build_independent_workspace(workspace):
                continue
        except (OSError, ValueError, KeyError, TypeError, tarfile.TarError):
            continue
        before = {task['id']: task for task in tasks(inputs, previous)}
        reused = 0
        for task in work:
            record = manifest['entries'].get(task['id'])
            if valid_record(task, record) or not valid_record(before[task['id']], record):
                continue
            record = {**record, 'key': task['key'], 'sceneKey': task['sceneKey'],
                      'reuse': {'method': 'unchanged-rendering-inputs', 'previousKey': record['key'],
                                'sourceArchiveSha256': proof['assets']['source.tar.gz']}}
            manifest['entries'][task['id']] = record
            update_legacy(task, record)
            store(task, record)
            reused += 1
        if reused:
            print(f'Capture cache: {reused} scenes reused after verified build-only/version changes; original provenance retained', flush=True)


def read_manifest():
    return json.loads(MANIFEST.read_text()) if MANIFEST.exists() else {'schema': 2, 'entries': {}}


def browser_scene(task, entry):
    scene = {'id': task['id'], 'slug': task['slug'], 'base': entry['base'],
             'output': str(task['outputs'][0].relative_to(ROOT)),
             'sceneKey': task['sceneKey'], 'key': task['key'],
             'kind': task['kind'], 'recipe': task['recipe']}
    activation = poster_activation(task['recipe'].get('activation'))
    if activation is not None:
        scene['activation'] = activation
    return scene


def output_record(path, native):
    width, height = dimensions(path)
    return {'sha256': digest(path.read_bytes()), 'width': width, 'height': height,
            'kind': ImageOrigin.BROWSER if native else ImageOrigin.DERIVED, 'density': 2 if native else 1}


def migrate_catalogue(work, manifest):
    # Import only the exact, previously attested source revision. This does not
    # recapture an image or rewrite its capture timestamp/environment.
    old = json.loads((DOCS / 'content/captures.json').read_text())
    ui_hash = __import__('hashlib').sha256()
    for file in sorted((ROOT / 'crates/kit-slint/ui').glob('*.slint')):
        ui_hash.update(file.name.encode() + b'\0' + file.read_bytes() + b'\0')
    if old['environment'].get('kitUiSha256') != ui_hash.hexdigest():
        return
    if old['environment'].get('captureSourceSha256') != digest((DOCS / 'capture/src/main.rs').read_bytes()):
        return
    revision = subprocess.check_output(['git', 'log', '-1', '--format=%H', '--', 'docs/content/captures.json'], cwd=ROOT, text=True).strip()
    unchanged = subprocess.run(['git', 'diff', '--quiet', revision, '--', 'crates', 'docs/examples', 'docs/capture/src/main.rs'], cwd=ROOT)
    if unchanged.returncode:
        return
    old_manifest = subprocess.check_output(['git', 'show', f'{revision}:Cargo.toml'], cwd=ROOT)
    if old_manifest != (ROOT / 'Cargo.toml').read_bytes():
        return
    import tomllib
    old_lock = tomllib.loads(subprocess.check_output(['git', 'show', f'{revision}:Cargo.lock'], cwd=ROOT, text=True))
    new_lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    if ([p for p in old_lock['package'] if p.get('source')]
            != [p for p in new_lock['package'] if p.get('source')]):
        return
    for task in work:
        if task['kind'] != SceneKind.CATALOGUE or task['id'] in manifest['entries']:
            continue
        slug, locale = task['slug'], task['locale']
        original = old['components'][slug]
        if locale == 'fr':
            original = original['locales']['fr']
        source = DOCS / 'examples' / ('fr' if locale == 'fr' else '') / f'{slug}.slint'
        if original['exampleSha256'] != digest(source.read_bytes()):
            continue
        verified = {variant['file']: variant for variant in original['variants']}
        # Preserve genuine legacy ×1 variants until their scene changes.
        # Never upscale them or reclassify them as native ×2.
        available = [path for path in task['outputs'] if path.name in verified]
        if (not available or any(verified[path.name].get('kind') == ImageOrigin.BROWSER for path in available)
                or not all(path.exists() and digest(path.read_bytes()) == verified[path.name]['sha256'] for path in available)):
            continue
        outputs = {str(path.relative_to(ROOT)): {**output_record(path, True),
                    'density': verified[path.name]['pixelDensity'], 'kind': verified[path.name].get('kind', ImageOrigin.NATIVE)} for path in available}
        profile = {'backend': 'winit', 'renderer': 'skia', 'slint': old['environment']['slintVersion']}
        manifest['entries'][task['id']] = {'key': digest({'scene': task['sceneKey'], 'profile': profile}),
                    'sceneKey': task['sceneKey'], 'renderProfile': profile, 'legacyNative': True, 'outputs': outputs,
                    'origin': {'revision': revision, 'environment': old['environment'],
                               'capturedAt': verified[available[0].name]['capturedAt'],
                               'method': 'imported-native-proof'}, 'recipe': task['recipe']}


def cache_directory(key):
    return DOCS / '.cache/captures' / key


def restore(task, previous=None):
    key = previous['key'] if previous and previous.get('sceneKey') == task['sceneKey'] else task['key']
    folder = cache_directory(key)
    try:
        record = json.loads((folder / 'manifest.json').read_text())
        if not valid_metadata(task, record) or record['key'] != key:
            return None
        contents = [(ROOT / name, (folder / str(index)).read_bytes()) for index, name in enumerate(sorted(record['outputs']))]
        if any(digest(data) != record['outputs'][str(path.relative_to(ROOT))]['sha256'] for path, data in contents):
            return None
    except (OSError, ValueError, KeyError, TypeError):
        return None
    for path, data in contents:
        write_changed(path, data)
    return record if valid_record(task, record) else None


def store(task, record):
    folder = cache_directory(record['key'])
    folder.mkdir(parents=True, exist_ok=True)
    for index, name in enumerate(sorted(record['outputs'])):
        write_changed(folder / str(index), (ROOT / name).read_bytes())
    write_json(folder / 'manifest.json', record)


def update_legacy(task, record):
    if task['kind'] == SceneKind.STARTER:
        path = DOCS / 'content/starter-captures.json'
        captures = json.loads(path.read_text())
        from starter_sources import source_directory, source_files
        source = source_directory(task['slug'])
        width, height = dimensions(task['outputs'][0])
        proof = {'environment': {'platform': record['origin']['environment'].get('platform'), 'backend': 'browser', 'renderer': 'femtovg', 'scale_factor': 2},
                 'source_sha256': {name: digest((source / name).read_bytes()) for name in source_files(source)},
                 'image': {'file': task['outputs'][0].name, 'sha256': digest(task['outputs'][0].read_bytes()), 'width': width, 'height': height},
                 'renderKey': task['key'], 'theme': task['recipe']['theme']}
        current = captures.setdefault(task['slug'], {})
        if task['recipe']['theme'] == 'dark':
            current['dark'] = proof
        else:
            captures[task['slug']] = {**proof, **({'dark': current['dark']} if 'dark' in current else {})}
        write_changed(path, json.dumps(captures, indent=2, ensure_ascii=False) + '\n')
        return
    path = DOCS / 'content/captures.json'
    old = json.loads(path.read_text())
    capture = old['components'].setdefault(task['slug'], {'variants': [], 'locales': {}})
    if task['locale'] == 'fr':
        capture = capture.setdefault('locales', {}).setdefault('fr', {'variants': []})
    names = {path.name for path in task['outputs']}
    capture['variants'] = [variant for variant in capture['variants'] if variant['file'] not in names]
    for index, output in enumerate(task['outputs']):
        width, height = dimensions(output)
        capture['variants'].append({'file': output.name, 'theme': task['recipe']['theme'],
            'textScale': task['recipe']['textScale'], 'pixelDensity': 2 if index == 0 else 1,
            'reducedMotion': True, 'width': width, 'height': height, 'sha256': digest(output.read_bytes()),
            'capturedAt': record['origin']['capturedAt'], 'interactionChecked': False,
            'kind': ImageOrigin.BROWSER if index == 0 else ImageOrigin.DERIVED})
    if not task['suffix']:
        width, height = dimensions(task['outputs'][1])
        source = DOCS / 'examples' / ('fr' if task['locale'] == 'fr' else '') / f'{task["slug"]}.slint'
        capture.update({'width': width, 'height': height, 'screenshotSha256': digest(task['outputs'][1].read_bytes()), 'exampleSha256': digest(source.read_bytes())})
    write_changed(path, json.dumps(old, indent=2, ensure_ascii=False) + '\n')


def repair_derivative(task, record):
    if not valid_metadata(task, record):
        return False
    source, destination = task['outputs']
    info = record.get('outputs', {}).get(str(source.relative_to(ROOT)), {})
    if not source.is_file() or digest(source.read_bytes()) != info.get('sha256'):
        return False
    downscale(source, destination)
    from PIL import __version__ as pillow_version
    derived = output_record(destination, False)
    derived['derivation'] = {'source': str(source.relative_to(ROOT)), 'filter': 'LANCZOS', 'pillow': pillow_version}
    record['outputs'][str(destination.relative_to(ROOT))] = derived
    update_legacy(task, record)
    store(task, record)
    return valid_record(task, record)


def record_browser_capture(task, manifest):
    environment_path = Path(str(task['outputs'][0]) + '.environment.json')
    if not environment_path.is_file():
        return False
    try:
        environment = json.loads(environment_path.read_text())
    except (OSError, ValueError):
        environment_path.unlink(missing_ok=True)
        return False
    if not isinstance(environment, dict):
        environment_path.unlink(missing_ok=True)
        return False
    if environment.get('sceneKey') != task['sceneKey'] or environment.get('key') != task['key']:
        return False
    downscale(*task['outputs'])
    outputs = {str(path.relative_to(ROOT)): output_record(path, index == 0) for index, path in enumerate(task['outputs'])}
    from PIL import __version__ as pillow_version
    outputs[str(task['outputs'][1].relative_to(ROOT))]['derivation'] = {'source': str(task['outputs'][0].relative_to(ROOT)), 'filter': 'LANCZOS', 'pillow': pillow_version}
    record = {'key': task['key'], 'sceneKey': task['sceneKey'], 'renderProfile': BROWSER_PROFILE,
              'outputs': outputs, 'recipe': task['recipe'],
              'origin': {'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                         'capturedAt': environment['capturedAt'], 'environment': environment,
                         'method': 'chrome-headless-wasm', 'dirty': bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT))}}
    manifest['entries'][task['id']] = record
    update_legacy(task, record)
    store(task, record)
    write_json(MANIFEST, manifest)
    environment_path.unlink()
    return True


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('action', choices=['plan', 'update', 'check'])
    parser.add_argument('slugs', nargs='*')
    parser.add_argument('--reuse-bundle', action='store_true', help='Require an existing verified WASM bundle; never compile one')
    args = parser.parse_args()
    from starter_sources import prepare
    prepare()
    inputs = build_inputs()
    work = tasks(inputs)
    if args.slugs:
        known = {task['slug'] for task in work}
        if not set(args.slugs) <= known:
            parser.error('Unknown component slug')
        work = [task for task in work if task['slug'] in args.slugs]
    manifest = read_manifest()
    if args.action == 'update':
        migrate_catalogue(work, manifest)
        reuse_build_only_changes(work, manifest, inputs)
        for task in work:
            if not valid_record(task, manifest['entries'].get(task['id'])):
                if record_browser_capture(task, manifest):
                    continue
                previous = manifest['entries'].get(task['id'])
                if repair_derivative(task, previous):
                    continue
                cached = restore(task, previous)
                if cached:
                    manifest['entries'][task['id']] = cached
                    update_legacy(task, cached)
    pending = [task for task in work if not valid_record(task, manifest['entries'].get(task['id']))]
    print(f'Documentation captures: {len(work) - len(pending)} reused, {len(pending)} scenes need capture', flush=True)
    for task in pending:
        previous = manifest['entries'].get(task['id'])
        reason = 'inputs changed' if previous and previous.get('key') != task['key'] else 'missing or damaged proof/output'
        print(f'  {task["id"]}: {reason}', flush=True)
    if args.action == 'check':
        if pending:
            raise SystemExit('Stale documentation captures: run npm run capture:update')
        return
    if args.action == 'plan':
        write_json(DOCS / '.generated/capture-plan.json', {'pending': [{'id': task['id'], 'key': task['key']} for task in pending]})
        return
    if pending:
        # One bundle serves both real starters and every localized catalogue
        # fixture. Theme, text scale and viewport are runtime scene parameters.
        subprocess.run(['python3', str(DOCS / 'scripts/build-web-previews.py'),
                        *(['--package-only'] if args.reuse_bundle else [])], cwd=ROOT, check=True)
        bundle = json.loads((DOCS / '.generated/web/manifest.json').read_text())['bundle']
        browser_plan = [browser_scene(task, bundle) for task in pending]
        plan_path = DOCS / '.generated/browser-capture-plan.json'
        write_json(plan_path, browser_plan)
        process = subprocess.run(['node', str(DOCS / 'scripts/capture-browser.mjs'), str(plan_path)], cwd=ROOT)
        for task in pending:
            record_browser_capture(task, manifest)
        if process.returncode:
            raise SystemExit('Browser capture failed; completed scenes were saved and will be reused')
    # Imported proofs are also useful to a fresh CI checkout.
    for task in work:
        record = manifest['entries'].get(task['id'])
        if valid_record(task, record):
            store(task, record)
    write_json(MANIFEST, manifest)
    print(f'Documentation captures ready: {len(work)} verified scenes', flush=True)


if __name__ == '__main__':
    main()
