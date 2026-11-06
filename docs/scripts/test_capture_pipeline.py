# SPDX-License-Identifier: GPL-3.0-or-later
import json
import io
from pathlib import Path
import struct
import tempfile
import tarfile
import unittest
from unittest.mock import patch
import zlib

import capture_pipeline as pipeline
from doc_inputs import archived_runtime_inputs


def png(width, height):
    def chunk(kind, data):
        return (struct.pack('>I', len(data)) + kind + data
                + struct.pack('>I', zlib.crc32(kind + data)))
    return (b'\x89PNG\r\n\x1a\n'
            + chunk(b'IHDR', struct.pack('>IIBBBBB', width, height, 8, 6, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress((b'\0' + b'\xff' * width * 4) * height))
            + chunk(b'IEND', b''))


class CatalogueCaptureTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.docs = Path(temporary.name)
        (self.docs / 'content').mkdir()
        self.path = self.docs / 'content/captures.json'
        self.path.write_text(json.dumps({'components': {}}))
        self.patch = patch.object(pipeline, 'DOCS', self.docs)
        self.patch.start()
        self.addCleanup(self.patch.stop)

    def capture(self, locale):
        source = self.docs / 'examples' / ('fr' if locale == 'fr' else '') / 'chip-button.slint'
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_text('localized fixture: ' + locale)
        stem = 'chip-button' + ('-fr' if locale == 'fr' else '')
        outputs = [self.docs / (stem + '@2x.png'), self.docs / (stem + '.png')]
        outputs[0].write_bytes(png(4, 2))
        outputs[1].write_bytes(png(2, 1))
        pipeline.update_legacy({'kind': pipeline.SceneKind.CATALOGUE, 'slug': 'chip-button',
                                'locale': locale, 'suffix': '', 'outputs': outputs,
                                'recipe': {'theme': 'light', 'textScale': 1}},
                               {'origin': {'capturedAt': '2026-11-04T20:00:00Z'}})
        return source, outputs

    def test_runtime_recipe_keeps_locale_theme_scale_and_viewport_out_of_the_binary_route(self):
        task = {'id': 'catalogue:button:fr:dark:2', 'kind': pipeline.SceneKind.CATALOGUE,
                'slug': 'button', 'sceneKey': 'scene', 'key': 'render',
                'outputs': [pipeline.ROOT / 'example.png'],
                'recipe': {'locale': 'fr', 'theme': 'dark', 'textScale': 2, 'viewport': [1000, 1000]}}
        scene = pipeline.browser_scene(task, {'base': '/shared-bundle'})
        self.assertEqual(scene['recipe'], task['recipe'])
        self.assertEqual(scene['kind'], 'catalogue')
        self.assertEqual(scene['base'], '/shared-bundle')

    def test_build_only_reuse_checks_archived_sources_and_keeps_capture_origin(self):
        workspace = b'[workspace.package]\nversion="0.7.0"\n[workspace.dependencies]\n'
        sources = {'Cargo.toml': workspace, 'Cargo.lock': b'package=[]\n',
                   'crates/kit/Cargo.toml': b'[package]\nname="slint-kit"\nversion="0.7.0"\n',
                   'crates/kit/src/lib.rs': b'pub fn runtime() {}',
                   'crates/kit-build/src/lib.rs': b'pub fn compile() {}'}
        (self.docs / 'Cargo.toml').write_bytes(workspace.replace(b'0.7.0', b'0.7.1'))
        folder = self.docs / '.cache/preview-bundles/previous'
        folder.mkdir(parents=True)
        archive_path = folder / 'source.tar.gz'
        with tarfile.open(archive_path, 'w:gz') as archive:
            for name, content in sources.items():
                entry = tarfile.TarInfo('slint-kit/' + name)
                entry.size = len(content)
                archive.addfile(entry, io.BytesIO(content))
        component = {'source_sha256': {name: pipeline.digest(content) for name, content in sources.items()}}
        (folder / 'component.json').write_text(json.dumps(component))
        (folder / 'manifest.json').write_text(json.dumps({'assets': {
            'source.tar.gz': pipeline.digest(archive_path.read_bytes()),
            'component.json': pipeline.digest((folder / 'component.json').read_bytes())}}))
        image = self.docs / 'capture.png'
        image.write_bytes(png(4, 2))
        before = {'id': 'scene', 'sceneKey': 'old', 'outputs': [image]}
        after = {**before, 'sceneKey': 'new', 'key': pipeline.digest({'scene': 'new', 'profile': pipeline.BROWSER_PROFILE})}
        origin = {'capturedAt': 'original timestamp', 'environment': {'sceneKey': 'old'}}
        record = {'key': pipeline.digest({'scene': 'old', 'profile': pipeline.BROWSER_PROFILE}),
                  'sceneKey': 'old', 'renderProfile': pipeline.BROWSER_PROFILE, 'origin': origin,
                  'outputs': {'capture.png': {'sha256': pipeline.digest(image.read_bytes())}}}
        runtime = archived_runtime_inputs({**sources, 'crates/kit-build/src/lib.rs': b'pub fn fixed_compile() {}'})
        with patch.object(pipeline, 'ROOT', self.docs), patch.object(pipeline, 'runtime_inputs', return_value=runtime), \
             patch.object(pipeline, 'tasks', return_value=[before]), patch.object(pipeline, 'update_legacy'), \
             patch.object(pipeline, 'store'):
            manifest = {'entries': {'scene': record}}
            pipeline.reuse_build_only_changes([after], manifest, {})
            self.assertEqual(manifest['entries']['scene']['key'], after['key'])
            self.assertEqual(manifest['entries']['scene']['origin'], origin)
            self.assertEqual(manifest['entries']['scene']['reuse']['previousKey'], record['key'])
            runtime['rust']['crates/kit/src/lib.rs'] = 'changed runtime'
            manifest = {'entries': {'scene': record}}
            pipeline.reuse_build_only_changes([after], manifest, {})
            self.assertEqual(manifest['entries']['scene'], record)
            runtime['rust']['crates/kit/src/lib.rs'] = pipeline.digest(sources['crates/kit/src/lib.rs'])
            archive_path.write_bytes(b'corrupted archive')
            pipeline.reuse_build_only_changes([after], manifest, {})
            self.assertEqual(manifest['entries']['scene'], record)

    def test_light_and_dark_starter_proofs_do_not_overwrite_each_other(self):
        capture_file = self.docs / 'content/starter-captures.json'
        capture_file.write_text('{}')
        source = self.docs / 'quickstart/button'
        source.mkdir(parents=True)
        (source / 'main.rs').write_text('host')
        for theme in ('dark', 'light', 'dark'):
            suffix = '-dark' if theme == 'dark' else ''
            image = source / f'preview{suffix}@2x.png'
            image.write_bytes(png(4, 2))
            task = {'kind': pipeline.SceneKind.STARTER, 'slug': 'button', 'key': theme,
                    'outputs': [image], 'recipe': {'theme': theme}}
            with patch('starter_sources.source_directory', return_value=source), \
                 patch('starter_sources.source_files', return_value=['main.rs']):
                pipeline.update_legacy(task, {'origin': {'environment': {'platform': 'test'}}})
        proof = json.loads(capture_file.read_text())['button']
        self.assertEqual(proof['theme'], 'light')
        self.assertEqual(proof['image']['file'], 'preview@2x.png')
        self.assertEqual(proof['dark']['theme'], 'dark')
        self.assertEqual(proof['dark']['image']['file'], 'preview-dark@2x.png')

    def test_new_component_registers_both_locales_in_either_order(self):
        for order in [('en', 'fr'), ('fr', 'en')]:
            with self.subTest(order=order):
                self.path.write_text(json.dumps({'components': {}}))
                for locale in order:
                    self.capture(locale)
                component = json.loads(self.path.read_text())['components']['chip-button']
                for locale, entry in [('en', component), ('fr', component['locales']['fr'])]:
                    source = self.docs / 'examples' / ('fr' if locale == 'fr' else '') / 'chip-button.slint'
                    self.assertEqual(entry['exampleSha256'], pipeline.digest(source.read_bytes()))
                    self.assertEqual((entry['width'], entry['height']), (2, 1))
                    self.assertEqual([variant['pixelDensity'] for variant in entry['variants']], [2, 1])

    def test_recapture_replaces_variants_and_preserves_other_locale(self):
        self.capture('en')
        self.capture('fr')
        before = json.loads(self.path.read_text())['components']['chip-button']['locales']['fr']
        self.capture('en')
        component = json.loads(self.path.read_text())['components']['chip-button']
        self.assertEqual(component['locales']['fr'], before)
        self.assertEqual(len(component['variants']), 2)

    def test_existing_component_can_gain_its_first_french_capture(self):
        self.path.write_text(json.dumps({'components': {'chip-button': {'variants': [], 'width': 99}}}))
        self.capture('fr')
        component = json.loads(self.path.read_text())['components']['chip-button']
        self.assertEqual(component['width'], 99)
        self.assertEqual(len(component['locales']['fr']['variants']), 2)


if __name__ == '__main__':
    unittest.main()
