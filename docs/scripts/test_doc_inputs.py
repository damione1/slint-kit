# SPDX-License-Identifier: GPL-3.0-or-later
from pathlib import Path
import tempfile
import unittest
import json
from unittest.mock import patch
from PIL import Image
from doc_inputs import generated_identity, key_for, write_changed, digest, compiler_environment, relocated_source, local_image_files
from presentation import themed_source
import capture_pipeline as captures


class DocumentationIdentityTests(unittest.TestCase):
    def test_relative_images_resolve_from_authored_source_instead_of_generated_directory(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            authored = root / 'project/ui/app.slint'
            asset = authored.parent / 'assets/profile.jpg'
            asset.parent.mkdir(parents=True)
            asset.write_bytes(b'portrait')
            source = 'Image { source: @image-url("assets/profile.jpg"); }'
            moved = relocated_source(source, authored)
            self.assertIn(json.dumps(str(asset)), moved)
            self.assertEqual(local_image_files(source, authored), [asset])
            self.assertEqual(relocated_source(moved, authored), moved)
            remote = 'Image { source: @image-url("https://example.org/photo.jpg"); }'
            self.assertEqual(relocated_source(remote, authored), remote)
            self.assertEqual(local_image_files(remote, authored), [])

    def test_used_resource_bytes_invalidate_but_checkout_location_does_not(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            a, b = root / 'a', root / 'b'
            a.mkdir(); b.mkdir()
            (a / 'font.ttf').write_bytes(b'font bytes')
            (b / 'font.ttf').write_bytes(b'font bytes')
            def code(folder):
                return f'const FONT: &[u8] = include_bytes ! ("{folder / "font.ttf"}");'
            original = generated_identity(code(a))
            self.assertEqual(original, generated_identity(code(b)))
            (b / 'unrelated.ttf').write_bytes(b'not used by the component')
            self.assertEqual(original, generated_identity(code(b)))
            (b / 'font.ttf').write_bytes(b'changed font bytes')
            self.assertNotEqual(original, generated_identity(code(b)))

    def test_individual_scene_keys_ignore_unrelated_components_and_track_shared_inputs(self):
        inputs = {'entries': {'checkbox': {'identity': 'a'}, 'radio': {'identity': 'b'}}}
        radio = key_for('radio', inputs, {'theme': 'kit'})
        checkbox = key_for('checkbox', inputs, {'theme': 'kit'})
        inputs['entries']['checkbox']['identity'] = 'changed'
        self.assertEqual(radio, key_for('radio', inputs, {'theme': 'kit'}))
        self.assertNotEqual(checkbox, key_for('checkbox', inputs, {'theme': 'kit'}))
        self.assertNotEqual(radio, key_for('radio', inputs, {'theme': 'tailwind'}))
        self.assertNotEqual(radio, key_for('radio', inputs, {'theme': 'kit'}, {'runtime': 'new renderer'}))

    def test_single_presentation_uses_public_font_import_and_is_idempotent(self):
        source = 'import { KitThemes } from "@kit/kit.slint"; export component App inherits Window { background: KitThemes.light.canvas; }'
        presented = themed_source(source, 'tailwind')
        self.assertIn('@kit/fonts/inter.slint', presented)
        self.assertIn('default-font-family: InterFont.family', presented)
        self.assertIn('KitThemes.light', presented)
        self.assertEqual(presented, themed_source(presented, 'tailwind'))
        self.assertNotIn('@kit-tailwind', presented)
        with self.assertRaises(ValueError):
            themed_source(source, 'kit')

    def test_native_debug_and_custom_style_do_not_leak_into_browser_compilation(self):
        environment = compiler_environment({'SLINT_EMIT_DEBUG_INFO': '1', 'SLINT_STYLE': 'native',
                                             'SLINT_SCALE_FACTOR': '3', 'SLINT_WEB_TARGET_DIR': '/cache',
                                             'CARGO_TARGET_DIR': '/build', 'RUSTFLAGS': '-C opt-level=2'})
        self.assertNotIn('SLINT_EMIT_DEBUG_INFO', environment)
        self.assertNotIn('SLINT_SCALE_FACTOR', environment)
        self.assertEqual(environment['SLINT_STYLE'], 'fluent')
        self.assertEqual(environment['SLINT_EMBED_RESOURCES'], 'true')
        self.assertEqual(environment['CARGO_TARGET_DIR'], '/build')
        self.assertEqual(environment['RUSTFLAGS'], '-C opt-level=2')

    def test_unchanged_bytes_keep_the_existing_file(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'artifact'
            self.assertTrue(write_changed(path, b'bytes'))
            before = path.stat().st_mtime_ns
            self.assertFalse(write_changed(path, b'bytes'))
            self.assertEqual(path.stat().st_mtime_ns, before)
            self.assertTrue(write_changed(path, b'changed'))

    def test_derived_image_is_deterministic_and_retains_logical_dimensions(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / 'native@2x.png', root / 'small.png'
            image = Image.new('RGBA', (40, 20), (80, 100, 180, 255))
            image.putpixel((12, 8), (255, 255, 255, 255)); image.save(source)
            captures.downscale(source, output)
            before = output.read_bytes()
            self.assertEqual(captures.dimensions(output), (20, 10))
            captures.downscale(source, output)
            self.assertEqual(before, output.read_bytes())
            Image.new('RGBA', (41, 20)).save(source)
            with self.assertRaises(ValueError):
                captures.downscale(source, output)

    def test_capture_cache_rejects_corruption_and_restores_exact_outputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = [root / 'preview@2x.png', root / 'preview.png']
            for path in paths:
                Image.new('RGBA', (20, 10), (50, 80, 100, 255)).save(path)
            scene = 'scene-key'
            profile = captures.BROWSER_PROFILE
            task = {'sceneKey': scene, 'key': digest({'scene': scene, 'profile': profile}), 'outputs': paths}
            record = {'key': task['key'], 'sceneKey': scene, 'renderProfile': profile,
                      'outputs': {path.name: {'sha256': digest(path.read_bytes())} for path in paths}}
            with patch.object(captures, 'ROOT', root), patch.object(captures, 'DOCS', root):
                self.assertTrue(captures.valid_record(task, record))
                captures.store(task, record)
                paths[0].unlink()
                self.assertFalse(captures.valid_record(task, record))
                self.assertIsNotNone(captures.restore(task, record))
                (captures.cache_directory(task['key']) / '1').write_bytes(b'corrupt')
                paths[0].unlink()
                self.assertIsNone(captures.restore(task, record))
                self.assertFalse(paths[0].exists())

    def test_rejected_cache_never_writes_an_unrelated_file(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            foreign = root / 'unrelated.txt'
            foreign.write_bytes(b'keep this')
            paths = [root / 'preview@2x.png', root / 'preview.png']
            scene = 'scene-key'
            profile = captures.BROWSER_PROFILE
            task = {'sceneKey': scene, 'key': digest({'scene': scene, 'profile': profile}), 'outputs': paths}
            record = {'key': task['key'], 'sceneKey': scene, 'renderProfile': profile,
                      'outputs': {'unrelated.txt': {'sha256': digest(b'overwritten')}}}
            with patch.object(captures, 'ROOT', root), patch.object(captures, 'DOCS', root):
                cache = captures.cache_directory(task['key'])
                cache.mkdir(parents=True)
                (cache / 'manifest.json').write_text(json.dumps(record))
                (cache / '0').write_bytes(b'overwritten')
                self.assertIsNone(captures.restore(task))
                self.assertEqual(foreign.read_bytes(), b'keep this')
                self.assertFalse(any(path.exists() for path in paths))

    def test_interrupted_browser_journal_is_retried_without_resigning_a_proof(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / 'preview@2x.png'
            journal = Path(str(output) + '.environment.json')
            journal.write_text('{')
            task = {'outputs': [output, root / 'preview.png'], 'sceneKey': 'scene', 'key': 'key'}
            manifest = {'entries': {}}
            self.assertFalse(captures.record_browser_capture(task, manifest))
            self.assertEqual(manifest, {'entries': {}})
            self.assertFalse(journal.exists())

    def test_overlay_poster_activation_reaches_the_browser_capture_driver(self):
        task = {'id': 'starter:dialog', 'slug': 'dialog', 'outputs': [captures.ROOT / 'preview@2x.png'],
                'sceneKey': 'scene', 'key': 'key', 'recipe': {'activation': [120, 80]}}
        scene = captures.browser_scene(task, {'base': '/assets/previews/dialog'})
        self.assertEqual(scene['activation'], {'action': 'click', 'point': [120, 80]})
        self.assertEqual(scene['output'], 'preview@2x.png')
        task['recipe']['activation'] = {'action': 'hover', 'point': [90, 42]}
        self.assertEqual(captures.browser_scene(task, {'base': '/assets/previews/tooltip'})['activation'], task['recipe']['activation'])
        task['recipe']['activation'] = {'action': 'focus-first'}
        self.assertEqual(captures.browser_scene(task, {'base': '/assets/previews/tooltip'})['activation'], task['recipe']['activation'])
        task['recipe']['activation'] = None
        self.assertNotIn('activation', captures.browser_scene(task, {'base': '/assets/previews/dialog'}))

    def test_invalid_poster_actions_and_coordinates_fail_before_browser_work(self):
        for value in [{'action': 'focus', 'point': [1, 2]}, {'action': 'focus-first', 'point': [1, 2]}, {'action': 'click', 'point': [False, 2]},
                      [-1, 2], [1], [float('nan'), 2], {'point': [1, 2]}]:
            with self.assertRaises(ValueError):
                captures.poster_activation(value)


if __name__ == '__main__':
    unittest.main()
