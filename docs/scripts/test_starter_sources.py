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
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.4.0"\nrust-version="1.92"\n[workspace.dependencies]\nslint={version="=1.18.1"}\n')
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
        self.assertEqual(command[:3], ['cargo', 'update', '--workspace'])
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
            runpy.run_path(str(Path(starter_sources.__file__).with_name('pack-starters.py')), run_name='__main__')
        with zipfile.ZipFile(self.root / 'docs/.generated/starters/checkbox.zip') as archive:
            self.assertEqual(archive.read('checkbox/ui/assets/profile.jpg'), b'original image')
            self.assertEqual(archive.read('checkbox/ui/assets/NOTICE.txt'), b'Original artwork license')
        asset.unlink()
        starter_sources.prepare('local')
        self.assertFalse((consumers['checkbox'] / 'ui/assets/profile.jpg').exists())
        self.assertFalse((starter_sources.source_directory('checkbox', 'local', native=True) / 'ui/assets/profile.jpg').exists())

    def test_unreleased_zip_requires_matching_sources_without_patching_its_public_manifest(self):
        entries = self.root / 'docs/content/starters.json'
        entries.write_text(json.dumps({'checkbox': {'registryExample': False}, 'radio-group': {}}))
        consumers = starter_sources.prepare('local')
        (self.root / 'LICENSE').write_text('GPL-3.0-or-later')
        with patch.object(doc_inputs, 'ROOT', self.root), patch.object(doc_inputs, 'DOCS', self.root / 'docs'), patch.object(starter_sources, 'prepare', return_value=consumers):
            runpy.run_path(str(Path(starter_sources.__file__).with_name('pack-starters.py')), run_name='__main__')
        with zipfile.ZipFile(self.root / 'docs/.generated/starters/checkbox.zip') as archive:
            readme = archive.read('checkbox/README.md').decode()
            self.assertIn('Development source example', readme)
            self.assertIn('Required source dependencies', readme)
            self.assertIn('obligatoires', readme)
            self.assertNotIn('use the matching registry release', readme)
            self.assertNotIn('patch', tomllib.loads(archive.read('checkbox/Cargo.toml').decode()))
            self.assertEqual(archive.read('checkbox/src/main.rs'), b'original')

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

    def test_workspace_version_projects_downloads_without_rewriting_templates(self):
        path = self.root / 'docs/quickstart/checkbox/Cargo.toml'
        source = path.read_text().replace('=0.4.0', '=0.3.0')
        source = source.replace('slint-kit = "=0.3.0"',
                                'slint-kit = { version = "=0.3.0", features = ["native"] }')
        path.write_text(source)
        consumers = starter_sources.prepare('local')
        self.assertEqual(path.read_text(), source)
        manifest = tomllib.loads((consumers['checkbox'] / 'Cargo.toml').read_text())
        self.assertEqual(manifest['dependencies']['slint-kit'],
                         {'version': '=0.4.0', 'features': ['native']})
        self.assertEqual(manifest['build-dependencies']['slint-kit-build'], '=0.4.0')

    def test_version_projection_preserves_external_dependencies_and_rejects_source_overrides(self):
        source = (self.root / 'docs/quickstart/checkbox/Cargo.toml').read_text().replace(
            '[dependencies]', '[dependencies]\nslint = { version = "=1.18.1", features = ["std"] }')
        expected = tomllib.loads(source)['dependencies']['slint']
        result = starter_sources.with_kit_version(source, '1.2.3')
        self.assertEqual(tomllib.loads(result)['dependencies']['slint'], expected)
        self.assertEqual(starter_sources.with_kit_version(result, '1.2.3'), result)
        patched = source + '\n[patch.crates-io]\nslint-kit = { path = "../../crates/kit" }\n'
        self.assertTrue(starter_sources.with_kit_version(patched, '1.2.3').endswith(
            '[patch.crates-io]\nslint-kit = { path = "../../crates/kit" }\n'))
        for field in ('path', 'git'):
            invalid = source.replace('slint-kit = "=0.4.0"',
                                     f'slint-kit = {{ version = "=0.4.0", {field} = "source" }}')
            with self.assertRaisesRegex(ValueError, 'registry dependency'):
                starter_sources.with_kit_version(invalid, '1.2.3')

    def test_public_overrides_wrong_versions_and_unknown_components_are_rejected(self):
        path = self.root / 'docs/quickstart/checkbox/Cargo.toml'
        source = path.read_text()
        for manifest in [source.replace('=0.4.0', '=0.3.0'), source + '\n[patch.crates-io]\n']:
            with self.assertRaises(ValueError):
                starter_sources.validate_manifest(manifest, '0.4.0')
        starter_sources.prepare('local')
        with self.assertRaisesRegex(ValueError, 'Unknown starter'):
            starter_sources.source_directory('../outside', 'local')

    def load_checker(self):
        return runpy.run_path(str(Path(starter_sources.__file__).with_name('check-starters.py')))

    def test_starters_with_one_dependency_graph_form_one_group_and_one_workspace(self):
        starter_sources.prepare('local')
        groups = starter_sources.dependency_groups('local')
        self.assertEqual(len(groups), 1)
        self.assertEqual(groups[0].packages, (('checkbox', 'slint-kit-checkbox-example'),
                                              ('radio-group', 'slint-kit-radio-group-example')))
        only = starter_sources.dependency_groups('local', ('radio-group',))
        self.assertEqual(only[0].packages, (('radio-group', 'slint-kit-radio-group-example'),))
        with self.assertRaisesRegex(ValueError, 'Unknown starter'):
            starter_sources.dependency_groups('local', ('missing',))
        consumers = starter_sources._prepared[starter_sources.BuildMode.LOCAL][0]
        directory = starter_sources.group_workspace(starter_sources.BuildMode.LOCAL, groups[0], consumers)
        self.assertEqual(directory.parent.parent, self.root / starter_sources.WORKSPACE_FOLDER)
        root = tomllib.loads((directory / 'Cargo.toml').read_text())
        self.assertEqual(root['workspace']['members'], ['checkbox', 'radio-group'])
        self.assertEqual(set(root['patch']['crates-io']), set(starter_sources.KIT_PACKAGES))
        for slug in ('checkbox', 'radio-group'):
            member = tomllib.loads((directory / slug / 'Cargo.toml').read_text())
            self.assertNotIn('workspace', member)
            self.assertNotIn('patch', member)
            self.assertNotIn('patch', tomllib.loads((consumers[slug] / 'Cargo.toml').read_text()))
            self.assertIn('[workspace]', (consumers[slug] / 'Cargo.toml').read_text())
            self.assertEqual((directory / slug / 'src/main.rs').read_text(), 'original')

    def test_release_workspace_has_no_patch_and_drifting_external_packages_are_rejected(self):
        consumers = starter_sources.prepare('release')
        group = starter_sources.dependency_groups('release')[0]
        mode = starter_sources.BuildMode.RELEASE
        self.cargo.side_effect = lambda command, **kwargs: None  # the release lock already resolves
        directory = starter_sources.group_workspace(mode, group, consumers)
        self.assertNotIn('patch', tomllib.loads((directory / 'Cargo.toml').read_text()))
        def drift(command, **kwargs):
            lock = Path(command[-1]).with_name('Cargo.lock')
            lock.write_text(lock.read_text().replace('version = "0.4.0"', 'version = "0.4.1"'))
        self.cargo.side_effect = drift
        with self.assertRaisesRegex(ValueError, 'drifted'):
            starter_sources.group_workspace(mode, group, consumers)

    def test_cargo_errors_are_attributed_to_the_failing_starter(self):
        checker = self.load_checker()
        starter_sources.prepare('local')
        group = starter_sources.dependency_groups('local')[0]
        directory = self.root / 'workspace'
        def message(slug, level):
            return json.dumps({'reason': 'compiler-message', 'manifest_path': str(directory / slug / 'Cargo.toml'),
                               'message': {'level': level, 'message': 'boom', 'rendered': f'{level} in {slug}'}})
        output = '\n'.join([message('checkbox', 'warning'), 'not json', message('radio-group', 'error'),
                            json.dumps({'reason': 'build-finished', 'success': False})])
        result = checker['attribute'](output, directory, group)
        self.assertEqual(result.failed, ('radio-group',))
        self.assertFalse(result.unattributed)
        self.assertEqual(result.diagnostics, ('error in radio-group',))
        clean = checker['attribute'](message('checkbox', 'warning'), directory, group)
        self.assertTrue(clean.passed)
        foreign = json.dumps({'reason': 'compiler-message', 'manifest_path': '/elsewhere/Cargo.toml',
                              'message': {'level': 'error', 'message': 'dependency broke'}})
        self.assertTrue(checker['attribute'](foreign, directory, group).unattributed)
        silent = checker['attribute'](json.dumps({'reason': 'build-finished', 'success': False}), directory, group)
        self.assertTrue(silent.unattributed)

    def test_build_script_failures_are_attributed_through_stderr(self):
        checker = self.load_checker()
        starter_sources.prepare('local')
        group = starter_sources.dependency_groups('local')[0]
        directory = self.root / 'workspace'
        failure = json.dumps({'reason': 'build-finished', 'success': False})
        stderr = ('error: failed to run custom build command for `slint-kit-radio-group-example v0.1.0 '
                  f'({directory}/radio-group)`\n')
        result = checker['attribute'](failure, directory, group, stderr)
        self.assertEqual(result.failed, ('radio-group',))
        self.assertFalse(result.unattributed)
        both = checker['attribute'](failure, directory, group, stderr + stderr.replace('radio-group', 'checkbox'))
        self.assertEqual(both.failed, ('checkbox', 'radio-group'))
        foreign = 'error: failed to run custom build command for `libfoo-sys v1.0.0 (/x)`\n'
        self.assertTrue(checker['attribute'](failure, directory, group, foreign).unattributed)

    def test_grouped_check_runs_one_cargo_check_for_the_group(self):
        checker = self.load_checker()
        consumers = starter_sources.prepare('local')
        group = starter_sources.dependency_groups('local')[0]
        mode = starter_sources.BuildMode.LOCAL
        calls = []
        def run(command, **kwargs):
            if command[:2] == ['cargo', 'update']:
                return self.resolve(command, **kwargs)
            calls.append(command)
            manifest = command[command.index('--manifest-path') + 1]
            error = json.dumps({'reason': 'compiler-message', 'manifest_path': str(Path(manifest).with_name('checkbox') / 'Cargo.toml'),
                                'message': {'level': 'error', 'message': 'boom'}})
            return type('Done', (), {'returncode': 101, 'stdout': error + '\n', 'stderr': ''})()
        metadata = json.dumps({'packages': [{'name': 'slint-kit', 'source': None}]})
        run_globals = checker['check_group'].__globals__
        with patch.dict(run_globals, {'subprocess': type('Sub', (), {'run': staticmethod(run), 'PIPE': -1,
                                                                    'check_output': staticmethod(lambda *a, **k: metadata)})}):
            result = checker['check_group'](mode, group, consumers, {})
        self.assertEqual(len(calls), 1)
        self.assertIn('--workspace', calls[0])
        self.assertIn('--locked', calls[0])
        self.assertIn('--keep-going', calls[0])
        self.assertEqual(result.failed, ('checkbox',))

    def test_grouped_check_rejects_a_registry_kit_package_in_local_mode(self):
        checker = self.load_checker()
        with self.assertRaisesRegex(ValueError, 'registry slint-kit'):
            checker['verify_sources'](starter_sources.BuildMode.LOCAL,
                                      {'packages': [{'name': 'slint-kit', 'source': 'registry+x'}]}, 'checkbox')
        with self.assertRaisesRegex(ValueError, 'local slint-kit'):
            checker['verify_sources'](starter_sources.BuildMode.RELEASE,
                                      {'packages': [{'name': 'slint-kit', 'source': None}]}, 'checkbox')


if __name__ == '__main__':
    unittest.main()
