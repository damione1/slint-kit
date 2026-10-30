# SPDX-License-Identifier: GPL-3.0-or-later
"""Verify independent public sources, lock reuse and explicit publication modes."""
import json
import runpy
from pathlib import Path
import tempfile
import tomllib
import unittest
import zipfile
from unittest.mock import patch

import starter_sources
import doc_inputs


class StarterSourceTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.addCleanup(patch.stopall)
        patch.object(starter_sources, 'ROOT', self.root).start()
        starter_sources._prepared.clear()
        self.addCleanup(starter_sources._prepared.clear)
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.4.0"\n[workspace.dependencies]\n')
        for package, directory in starter_sources.KIT_PACKAGES.items():
            folder = self.root / directory
            folder.mkdir(parents=True)
            (folder / 'Cargo.toml').write_text(f'[package]\nname="{package}"\nversion="0.4.0"\n')
        content = self.root / 'docs/content'
        content.mkdir(parents=True)
        (content / 'starters.json').write_text(json.dumps({'checkbox': {}, 'radio-group': {}}))
        for slug in ['checkbox', 'radio-group']:
            folder = self.root / 'docs/quickstart' / slug
            (folder / 'src').mkdir(parents=True)
            (folder / 'ui').mkdir()
            (folder / 'Cargo.toml').write_text(f'''[package]
name = "slint-kit-{slug}-example"
version = "0.1.0"
edition = "2024"
[dependencies]
slint-kit = "=0.4.0"
[build-dependencies]
slint-kit-build = "=0.4.0"
[workspace]
''')
            (folder / 'Cargo.lock').write_text(f'''version = 4
[[package]]
name = "slint-kit-{slug}-example"
version = "0.1.0"
dependencies = ["slint-kit", "slint-kit-build"]
[[package]]
name = "slint-kit"
version = "0.3.0"
''')
            for name in ['build.rs', 'src/main.rs', 'ui/app.slint']:
                (folder / name).write_text('original')
        self.cargo = patch.object(starter_sources.subprocess, 'run', side_effect=self.resolve).start()
        patch.object(starter_sources.subprocess, 'check_output', return_value='cargo 1.99.1').start()

    def resolve(self, command, **kwargs):
        self.assertEqual(command[:2], ['cargo', 'generate-lockfile'])
        manifest = Path(command[-1])
        parsed = tomllib.loads(manifest.read_text())
        lock = manifest.with_name('Cargo.lock')
        source = lock.read_text().replace('version = "0.3.0"', 'version = "0.4.0"')
        if 'patch' not in parsed:
            source += 'source = "registry+https://github.com/rust-lang/crates.io-index"\n'
        lock.write_text(source)

    def test_local_builds_are_patched_but_public_manifests_are_registry_consumers(self):
        consumers = starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 1)
        for slug, directory in consumers.items():
            public = tomllib.loads((directory / 'Cargo.toml').read_text())
            self.assertNotIn('patch', public)
            native = starter_sources.source_directory(slug, 'local', native=True)
            private = tomllib.loads((native / 'Cargo.toml').read_text())
            self.assertEqual(set(private['patch']['crates-io']), set(starter_sources.KIT_PACKAGES))
            self.assertEqual(public['dependencies']['slint-kit'], '=0.4.0')
            packages = tomllib.loads((directory / 'Cargo.lock').read_text())['package']
            self.assertEqual(packages[0]['name'], f'slint-kit-{slug}-example')
            self.assertEqual(packages[1]['version'], '0.4.0')
        manifest = json.loads((self.root / 'docs/.generated/consumers/manifest.json').read_text())
        self.assertEqual((manifest['mode'], manifest['version']), ('local', '0.4.0'))
        self.assertEqual(manifest['slugs'], ['checkbox', 'radio-group'])

    def test_repeated_prepare_reuses_locks_and_does_not_rewrite_identical_files(self):
        consumers = starter_sources.prepare('local')
        before = (consumers['checkbox'] / 'Cargo.lock').stat().st_mtime_ns
        starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 1)
        self.assertEqual((consumers['checkbox'] / 'Cargo.lock').stat().st_mtime_ns, before)
        (self.root / 'docs/quickstart/checkbox/ui/app.slint').write_text('new UI')
        starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 1)
        self.assertEqual((consumers['checkbox'] / 'ui/app.slint').read_text(), 'new UI')
        self.assertEqual((consumers['radio-group'] / 'ui/app.slint').read_text(), 'original')

    def test_bundled_images_and_notices_reach_native_sources_hashes_and_standalone_zip(self):
        asset = self.root / 'docs/quickstart/checkbox/ui/assets/profile.jpg'
        asset.parent.mkdir()
        asset.write_bytes(b'original image')
        asset.with_name('NOTICE.txt').write_text('Original artwork license')
        consumers = starter_sources.prepare('local')
        for directory in [consumers['checkbox'], starter_sources.source_directory('checkbox', 'local', native=True)]:
            self.assertEqual((directory / 'ui/assets/profile.jpg').read_bytes(), b'original image')
        manifest = json.loads((self.root / 'docs/.generated/consumers/manifest.json').read_text())
        self.assertIn('ui/assets/profile.jpg', manifest['source_sha256']['checkbox'])
        (self.root / 'LICENSE').write_text('GPL-3.0-or-later')
        with patch.object(doc_inputs, 'ROOT', self.root), patch.object(doc_inputs, 'DOCS', self.root / 'docs'), patch.object(starter_sources, 'prepare', return_value=consumers):
            runpy.run_path(str(Path(starter_sources.__file__).with_name('pack-starters.py')))
        with zipfile.ZipFile(self.root / 'docs/.generated/starters/checkbox.zip') as archive:
            self.assertEqual(archive.read('checkbox/ui/assets/profile.jpg'), b'original image')
            self.assertEqual(archive.read('checkbox/ui/assets/NOTICE.txt'), b'Original artwork license')
        asset.unlink()
        starter_sources.prepare('local')
        self.assertFalse((consumers['checkbox'] / 'ui/assets/profile.jpg').exists())
        self.assertFalse((starter_sources.source_directory('checkbox', 'local', native=True) / 'ui/assets/profile.jpg').exists())

    def test_release_generates_a_separate_registry_lock_without_any_path_patch(self):
        starter_sources.prepare('local')
        consumers = starter_sources.prepare('release')
        self.assertEqual(self.cargo.call_count, 2)
        for slug, directory in consumers.items():
            self.assertEqual(starter_sources.source_directory(slug, 'release', native=True), directory)
            self.assertNotIn('patch', tomllib.loads((directory / 'Cargo.toml').read_text()))
            self.assertIn('registry+', (directory / 'Cargo.lock').read_text())
        # Switching back must not return public files belonging to the other mode.
        starter_sources.source_directory('checkbox', 'local')
        manifest = json.loads((self.root / 'docs/.generated/consumers/manifest.json').read_text())
        self.assertEqual(manifest['mode'], 'local')

    def test_corrupt_lock_or_cache_manifest_is_regenerated(self):
        starter_sources.prepare('local')
        cache = next((self.root / 'docs/.cache/consumer-locks/local').iterdir())
        (cache / 'Cargo.lock').write_text('corrupt')
        starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 2)
        (cache / 'manifest.json').write_text('{broken')
        starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 3)

    def test_changed_dependency_graph_invalidates_only_its_lock_resolution(self):
        starter_sources.prepare('local')
        manifest = self.root / 'docs/quickstart/checkbox/Cargo.toml'
        manifest.write_text(manifest.read_text().replace('slint-kit = "=0.4.0"', 'slint-kit = { version = "=0.4.0", features = ["native"] }'))
        starter_sources.prepare('local')
        self.assertEqual(self.cargo.call_count, 2)

    def test_public_overrides_wrong_versions_and_unknown_components_are_rejected(self):
        path = self.root / 'docs/quickstart/checkbox/Cargo.toml'
        source = path.read_text()
        for manifest in [source.replace('=0.4.0', '=0.3.0'), source + '\n[patch.crates-io]\n']:
            with self.assertRaises(ValueError):
                starter_sources.validate_manifest(manifest, '0.4.0')
        starter_sources.prepare('local')
        with self.assertRaisesRegex(ValueError, 'Unknown starter'):
            starter_sources.source_directory('../outside', 'local')


if __name__ == '__main__':
    unittest.main()
