# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import json
import gzip
import inspect
import io
import tarfile
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from web_cache import ASSETS

spec = importlib.util.spec_from_file_location('web_components', Path(__file__).with_name('build-web-previews.py'))
web = importlib.util.module_from_spec(spec)
spec.loader.exec_module(web)


class ComponentPreviewTests(unittest.TestCase):
    def test_notices_have_independent_urls_and_shared_bytes_are_deduplicated(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / 'source', root / 'output'
            source.mkdir()
            for name in ASSETS:
                (source / name).write_bytes(name.encode())
            for name in web.EXTRA_ASSETS:
                (source / name).write_bytes(name.encode())
            (source / 'component.json').write_text(json.dumps({'key': 'compiled'}))
            before = web.publish_assets(source, output, 'checkbox', 'kit')
            transport = output / before['base'].lstrip('/') / 'slint_kit_previews_bg.wasm.gz'
            self.assertEqual(gzip.decompress(transport.read_bytes()), b'slint_kit_previews_bg.wasm')
            self.assertEqual((output / before['base'].lstrip('/') / 'slint_kit_previews_bindings.js').read_bytes(), b'slint_kit_previews.js')
            other = web.publish_assets(source, output, 'radio-group', 'kit')
            self.assertEqual(before['sources']['THIRD_PARTY_NOTICES.txt'], other['sources']['THIRD_PARTY_NOTICES.txt'])
            (source / 'NOTICE.txt').write_bytes(b'Corrected copyright notice')
            after = web.publish_assets(source, output, 'checkbox', 'kit')
            self.assertEqual(before['base'], after['base'])
            self.assertEqual(before['assets'], after['assets'])
            self.assertNotEqual(before['sources']['NOTICE.txt'], after['sources']['NOTICE.txt'])
            self.assertEqual(before['sources']['source.tar.gz'], after['sources']['source.tar.gz'])

    def test_fixture_cache_invalidates_changed_host_lock_and_compiler_flags(self):
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            for name in ('build.rs', 'src/lib.rs', 'Cargo.toml', 'Cargo.lock'):
                path = docs / 'preview' / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text('original ' + name)
            with patch.object(web, 'DOCS', docs):
                task = {'sceneKey': 'same UI'}
                def key(environment=None):
                    return web.fixture_code_key(task, 'rustc', 'bindgen', environment or {})
                original = key()
                self.assertEqual(original, key({'CARGO_TARGET_DIR': '/another/cache'}))
                self.assertNotEqual(original, key({'CARGO_PROFILE_RELEASE_OPT_LEVEL': '2'}))
                for name in ('src/lib.rs', 'Cargo.lock'):
                    path = docs / 'preview' / name
                    previous = path.read_text()
                    path.write_text(previous + ' changed')
                    self.assertNotEqual(original, key(), name)
                    path.write_text(previous)
                    self.assertEqual(original, key())

    def test_all_routes_share_one_compilation_and_cached_bundle(self):
        with tempfile.TemporaryDirectory() as directory:
            docs = Path(directory)
            (docs / 'content').mkdir()
            (docs / 'content/starters.json').write_text(json.dumps({'button': {}, 'checkbox': {}}))
            (docs / 'preview').mkdir()
            (docs / 'preview/NOTICE.txt').write_text('license')
            (docs / 'preview/loader.js').write_text('loader')
            def compiled(target, environment, bindgen, destination):
                for name in ASSETS:
                    (destination / name).write_bytes(name.encode())
            def archived(destination):
                (destination / 'source.tar.gz').write_bytes(b'corresponding sources')
                return {'all source files': 'hash'}
            with patch.object(web, 'DOCS', docs), patch('sys.argv', ['build-web-previews.py']), \
                 patch('starter_sources.prepare'), patch.object(web, 'build_inputs', return_value={}), \
                 patch.object(web, 'bundle_key', return_value='bundle-key'), \
                 patch.object(web, 'code_key', side_effect=lambda slug, *args: slug + '-key'), \
                 patch.object(web, 'third_party_notices', return_value='dependency licenses'), \
                 patch.object(web, 'archive_sources', side_effect=archived), \
                 patch.object(web, 'restore_packaging_only', return_value=False), \
                 patch.object(web, 'compile_bundle', side_effect=compiled) as compile_once, \
                 patch.object(web.subprocess, 'check_output', side_effect=lambda command, **kwargs:
                              'wasm-bindgen 0.2.129' if command[-1] == '--version' else 'toolchain or revision'):
                web.build()
                manifest = json.loads((docs / '.generated/web/manifest.json').read_text())
                self.assertEqual(compile_once.call_count, 1)
                self.assertEqual(manifest['entries']['button']['tailwind']['base'], manifest['bundle']['base'])
                self.assertEqual(manifest['entries']['checkbox']['tailwind']['base'], manifest['bundle']['base'])
                web.build()
                self.assertEqual(compile_once.call_count, 1, 'A second run restores the whole bundle without Cargo')
                (docs / '.cache/preview-bundles/bundle-key' / ASSETS[0]).write_bytes(b'corruption')
                web.build()
                self.assertEqual(compile_once.call_count, 2, 'A corrupt bundle must be rebuilt')

    def test_bundle_identity_covers_other_locales_and_shared_flags(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            fixture = root / 'fr.slint'
            fixture.write_text('French fixture')
            inputs = {'entries': {'starter:button:tailwind': {'identity': 'starter'},
                                  'catalogue:button:fr:dark': {'identity': 'localized'}}}
            with patch.object(web, 'ROOT', root), patch.object(web, 'corresponding_sources', return_value=[fixture]), \
                 patch.object(web, 'runtime_inputs', return_value={}):
                def key(environment=None):
                    return web.bundle_key(inputs, 'rustc', 'bindgen', environment or {})
                original = key()
                self.assertEqual(original, key({'CARGO_TARGET_DIR': '/other'}))
                self.assertNotEqual(original, key({'RUSTFLAGS': '-C opt-level=2'}))
                fixture.write_text('Changed French fixture')
                self.assertNotEqual(original, key())
                fixture.write_text('French fixture')
                inputs['entries']['catalogue:button:fr:dark']['identity'] = 'changed dependency'
                self.assertNotEqual(original, key())

    def test_packaging_only_reuse_requires_same_compiler_inputs_flags_and_recipe(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            builder = root / 'docs/scripts/build-web-previews.py'
            builder.parent.mkdir(parents=True)
            builder.write_text(inspect.getsource(web.compile_bundle))
            rust = root / 'app.rs'; rust.write_text('real application')
            source = root / 'compiled'; source.mkdir()
            destination = root / 'packaged'; destination.mkdir()
            cache = root / 'cache'
            for name in ASSETS:
                (source / name).write_bytes(name.encode())
            def archive(folder):
                data = builder.read_bytes()
                with tarfile.open(folder / 'source.tar.gz', 'w:gz') as bundle:
                    info = tarfile.TarInfo('slint-kit/docs/scripts/build-web-previews.py')
                    info.size = len(data); bundle.addfile(info, io.BytesIO(data))
                return {'docs/scripts/build-web-previews.py': web.digest(data)}
            inputs = {'entries': {'starter:button:tailwind': {'identity': 'compiled UI'}}}
            with patch.object(web, 'ROOT', root), patch.object(web, 'corresponding_sources', return_value=[builder, rust]), \
                 patch.object(web, 'runtime_inputs', return_value={}), patch.object(web, 'archive_sources', side_effect=archive):
                original = web.bundle_key(inputs, 'rustc', 'bindgen', {})
                record = {'key': original, 'source_sha256': archive(source)}
                (source / 'component.json').write_text(json.dumps(record))
                web.save(cache, original, source, web.COMPILED_EXTRAS)
                builder.write_text(builder.read_text() + '\n# Packaging changes only\n')
                current = web.bundle_key(inputs, 'rustc', 'bindgen', {})
                def restore(environment=None):
                    return web.restore_packaging_only(cache, current, destination, inputs, 'rustc', 'bindgen', environment or {})
                self.assertFalse(restore({'RUSTFLAGS': '-C opt-level=2'}))
                rust.write_text('changed application')
                self.assertFalse(restore())
                rust.write_text('real application')
                self.assertTrue(restore())
                proof = json.loads((destination / 'component.json').read_text())
                self.assertEqual(proof['reusedCompilation'], original)
                self.assertEqual(proof['key'], current)
                self.assertEqual((destination / ASSETS[0]).read_bytes(), ASSETS[0].encode())

    def test_kit_version_bumps_reuse_modules_but_external_dependencies_and_features_invalidate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / 'docs'
            files = {'quickstart/checkbox/src/main.rs': 'fn main() {}',
                     'quickstart/checkbox/build.rs': 'fn main() {}',
                     'preview/build.rs': 'fn main() {}', 'preview/src/lib.rs': 'pub fn start() {}'}
            for name, content in files.items():
                path = docs / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(content)
            def versions(kit, slint='1.18.1', features='["std"]'):
                (docs / 'preview/Cargo.toml').write_text(f'[dependencies]\nslint-kit = {{ version = "={kit}", features = ["native"] }}\n[build-dependencies]\nslint-kit-build = "={kit}"\n')
                (docs / 'preview/Cargo.lock').write_text(f'version = 4\n[[package]]\nname="slint-kit"\nversion="{kit}"\n[[package]]\nname="slint"\nversion="{slint}"\nsource="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="checksum"\n')
                (root / 'Cargo.toml').write_text(f'[workspace.dependencies]\nslint = {{ version = "={slint}", features = {features} }}\n')
            inputs = {'entries': {'starter:checkbox:kit': {'identity': 'same reachable UI'}}}
            with patch.object(web, 'ROOT', root), patch.object(web, 'DOCS', docs), patch.object(web, 'runtime_inputs', return_value={}), patch.object(web, 'consumer_identity', return_value={}):
                versions('0.4.0'); original = web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {})
                versions('0.4.1'); self.assertEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))
                versions('0.4.1', '1.18.2'); self.assertNotEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))
                versions('0.4.1', features='["std", "accessibility"]'); self.assertNotEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))


if __name__ == '__main__':
    unittest.main()
