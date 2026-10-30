# SPDX-License-Identifier: GPL-3.0-or-later
"""Content identities for reachable Slint code and its rendering inputs."""
import argparse
from dataclasses import dataclass
import hashlib
import inspect
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import tomllib
from presentation import ThemeFamily, themed_source

ROOT = Path(__file__).resolve().parents[2]
DOCS = ROOT / 'docs'
SCHEMA = 1


@dataclass(frozen=True)
class Slug:
    value: str

    def __post_init__(self):
        if not re.fullmatch(r'[a-z]+(?:-[a-z]+)*', self.value):
            raise ValueError(f'Invalid component slug: {self.value}')


def digest(value):
    if not isinstance(value, bytes):
        value = json.dumps(value, sort_keys=True, separators=(',', ':')).encode()
    return hashlib.sha256(value).hexdigest()


def write_changed(path, content):
    path = Path(path)
    if not isinstance(content, bytes):
        content = content.encode()
    if path.is_file() and path.read_bytes() == content:
        return False
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        stream.write(content)
    os.replace(temporary, path)
    return True


def write_json(path, value):
    return write_changed(path, json.dumps(value, indent=2, sort_keys=True) + '\n')


def compiler_environment(environment=None):
    # Compile the same presentation on developer machines and CI. In
    # particular, a native MCP/debug environment must not leak into WASM.
    source = os.environ if environment is None else environment
    result = {name: value for name, value in source.items()
              if not name.startswith('SLINT_') and name != 'DEP_MCU_BOARD_SUPPORT_MCU_EMBED_TEXTURES'}
    result.update({'SLINT_STYLE': 'fluent', 'SLINT_EMBED_RESOURCES': 'true'})
    return result


def source_hashes(files):
    return {str(path.relative_to(ROOT)): digest(path.read_bytes()) for path in sorted(set(files))}


def runtime_inputs():
    # Local package version bumps do not change their implementation. Registry
    # dependencies do: retain every external package's checksum and resolution.
    lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
    external = [package for package in lock['package'] if package.get('source')]
    rust = sorted(path for path in (ROOT / 'crates').rglob('*.rs'))
    manifests = {}
    for path in sorted((ROOT / 'crates').rglob('Cargo.toml')):
        value = tomllib.loads(path.read_text())
        manifests[str(path.relative_to(ROOT))] = value
    return {'external': external, 'rust': source_hashes(rust), 'manifests': manifests}


def consumer_identity(slug):
    folder = DOCS / '.generated/consumers' / slug
    manifest = tomllib.loads((folder / 'Cargo.toml').read_text())
    for section in ['dependencies', 'build-dependencies']:
        for name, value in manifest.get(section, {}).items():
            if name in ('slint-kit', 'slint-kit-build'):
                manifest[section][name] = {**value, 'version': '<resolved-kit-sources>'} if isinstance(value, dict) else '<resolved-kit-sources>'
    packages = tomllib.loads((folder / 'Cargo.lock').read_text())['package']
    packages = [package for package in packages if package.get('source') and not package['name'].startswith('slint-kit')]
    return {'manifest': manifest, 'external': packages}


# Only resource macro paths are normalized. UI text and generated code remain
# byte exact; never strip arbitrary paths, comments, or whitespace from code.
RESOURCE = re.compile(r'include_bytes\s*!\s*\(\s*("(?:[^"\\]|\\.)*"|r(?P<marks>\#*)".*?"(?P=marks))', re.S)
IMAGE_URL = re.compile(r'(@image-url\s*\(\s*)("(?:[^"\\]|\\.)*")(\s*\))')


def local_image_files(source, path):
    paths = []
    for match in IMAGE_URL.finditer(source):
        name = json.loads(match[2])
        if name.startswith('@') or re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:', name):
            continue
        paths.append((Path(path).parent / name).resolve())
    return paths


def relocated_source(source, path):
    # Generated UI moves away from the authored project. Keep the public source
    # portable while resolving its local resources at their original location.
    def replace(match):
        name = json.loads(match[2])
        if name.startswith('@') or re.match(r'[a-zA-Z][a-zA-Z0-9+.-]*:', name):
            return match[0]
        return match[1] + json.dumps(str((Path(path).parent / name).resolve())) + match[3]
    return IMAGE_URL.sub(replace, source)


def generated_identity(source):
    resources = {}
    def resource(match):
        token = match.group(1)
        name = json.loads(token) if token.startswith('"') else token[token.index('"') + 1:token.rindex('"')]
        path = Path(name)
        if not path.is_absolute():
            raise ValueError(f'Generated resource must have an absolute path: {name}')
        content = path.read_bytes()
        checksum = digest(content)
        resources[checksum] = len(content)
        return match.group(0)[:match.group(0).rfind(token)] + json.dumps(f'resource:{checksum}')
    normalized = RESOURCE.sub(resource, source)
    return {'code': digest(normalized.encode()), 'resources': resources}


def collect_requests():
    provenance = json.loads((DOCS / 'content/captures.json').read_text())
    requests = {}
    for source in sorted((DOCS / 'examples').glob('*.slint')):
        slug = Slug(source.stem).value
        for locale in ['en', 'fr']:
            actual = source if locale == 'en' else source.parent / 'fr' / source.name
            record = provenance['components'].get(slug, {})
            if locale == 'fr':
                record = record.get('locales', {}).get('fr', {})
            themes = {'light'} | {v['theme'] for v in record.get('variants', [])}
            for theme in sorted(themes):
                if theme not in ('light', 'dark'):
                    raise ValueError(f'Unknown capture theme: {theme}')
                requests[f'catalogue:{slug}:{locale}:{theme}'] = (actual, theme)
    for slug in sorted(json.loads((DOCS / 'content/starters.json').read_text())):
        Slug(slug)
        for family in ThemeFamily:
            requests[f'starter:{slug}:{family.value}'] = (DOCS / 'quickstart' / slug / 'ui/app.slint', family.value)
    return requests


def build_inputs():
    requests = collect_requests()
    # A coarse frontend cache is safe: a hit skips parsing; a miss only reruns
    # the Slint frontend. Render/link keys below remain individual.
    files = list((ROOT / 'crates/kit-slint/ui').rglob('*'))
    files = [path for path in files if path.is_file()]
    files += [path for path, _ in requests.values()]
    files += [image for path, _ in requests.values() for image in local_image_files(path.read_text(), path)]
    files += [ROOT / 'xtask/src/docs.rs', ROOT / 'Cargo.lock']
    hashes = source_hashes(files)
    target = DOCS / '.generated/inputs'
    manifest_path = target / 'manifest.json'
    try:
        previous = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    except (OSError, ValueError):
        previous = {}
    if (not isinstance(previous, dict) or not isinstance(previous.get('entries'), dict)
            or not isinstance(previous.get('sourceHashes'), dict)):
        previous = {}
    prepared = {}
    for identifier, (source, theme) in requests.items():
        text = source.read_text()
        if identifier.startswith('starter:'):
            text = themed_source(text, theme)
        elif theme == 'dark':
            text = text.replace('KitThemes.light', 'KitThemes.dark')
        prepared[identifier] = relocated_source(text, source)
    previous_hashes = {name: checksum for name, checksum in previous.get('sourceHashes', {}).items()
                       if name not in ('docs/scripts/doc_inputs.py', 'docs/scripts/presentation.py')}
    same_frontend = (previous.get('schema') == SCHEMA and previous_hashes == hashes
                     and set(previous.get('entries', {})) == set(requests))
    same_frontend = same_frontend and all(
        (target / (identifier.replace(':', '--') + '.slint')).is_file()
        and (target / (identifier.replace(':', '--') + '.slint')).read_text() == text
        and (target / (identifier.replace(':', '--') + '.rs')).is_file()
        for identifier, text in prepared.items())
    identity_function = digest({'function': inspect.getsource(generated_identity), 'resourcePattern': RESOURCE.pattern, 'flags': RESOURCE.flags})
    if same_frontend:
        if previous.get('identityFunction') != identity_function:
            for identifier in requests:
                generated = target / (identifier.replace(':', '--') + '.rs')
                identity = generated_identity(generated.read_text())
                previous['entries'][identifier]['identity'] = digest(identity)
                previous['entries'][identifier]['generated'] = identity
        previous.update({'sourceHashes': hashes, 'identityFunction': identity_function})
        write_json(manifest_path, previous)
        return previous
    target.mkdir(parents=True, exist_ok=True)
    compile_requests = []
    for identifier, (source, theme) in requests.items():
        name = identifier.replace(':', '--')
        input_path = target / f'{name}.slint'
        write_changed(input_path, prepared[identifier])
        compile_requests.append({'source': str(input_path), 'output': str(target / f'{name}.rs')})
    request_path = target / 'requests.json'
    write_json(request_path, compile_requests)
    print(f'Slint inputs: examining {len(compile_requests)} component/theme sources', flush=True)
    with (target / 'compiler.log').open('w') as log:
        result = subprocess.run(['cargo', 'run', '-p', 'xtask', '--locked', '--', 'compile-docs', str(request_path)],
                                cwd=ROOT, env=compiler_environment(), stdout=log, stderr=subprocess.STDOUT)
    if result.returncode:
        raise RuntimeError(f'Slint fingerprint compilation failed; see {target / "compiler.log"}')
    entries = {}
    for identifier, (source, theme) in requests.items():
        output = target / f'{identifier.replace(":", "--")}.rs'
        identity = generated_identity(output.read_text())
        entries[identifier] = {'identity': digest(identity), 'sourceSha256': digest(source.read_bytes()),
                               'generated': identity}
    result = {'schema': SCHEMA, 'sourceHashes': hashes, 'identityFunction': identity_function, 'entries': entries}
    write_json(manifest_path, result)
    return result


def key_for(identifier, inputs, recipe, driver=None):
    return digest({'schema': SCHEMA, 'identity': inputs['entries'][identifier]['identity'],
                   'recipe': recipe, 'driver': driver or {}})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    if args.check:
        path = DOCS / '.generated/inputs/manifest.json'
        previous = json.loads(path.read_text()) if path.exists() else None
        current = build_inputs()
        if previous != current:
            raise SystemExit('Documentation inputs changed: run npm run capture:update and npm run build')
    else:
        result = build_inputs()
        print(f'Slint inputs: {len(result["entries"])} individual identities ready')


if __name__ == '__main__':
    main()
