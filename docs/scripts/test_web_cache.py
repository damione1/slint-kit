# SPDX-License-Identifier: GPL-3.0-or-later
import os
from pathlib import Path
import tempfile
import unittest

from web_cache import ASSETS, compilation_key, restore, save


class PreviewCacheTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.source = self.root / 'app.slint'
        self.source.write_text('export component Demo inherits Window {}')
        self.cache = self.root / '.cache'
        self.output = self.root / 'output'
        self.output.mkdir()

    def key(self, **kwargs):
        return compilation_key(self.root, [self.source], kwargs.get('rustc', 'rustc 1'),
                               kwargs.get('bindgen', 'wasm-bindgen 1'), kwargs.get('env', {}))

    def test_fresh_checkout_timestamps_reuse_but_changed_source_invalidates(self):
        key = self.key()
        os.utime(self.source, (1, 1))
        self.assertEqual(self.key(), key)
        self.source.write_text('export component AnotherDemo inherits Window {}')
        self.assertNotEqual(self.key(), key)

    def test_toolchain_and_codegen_changes_invalidate(self):
        key = self.key()
        self.assertNotEqual(self.key(rustc='rustc 2'), key)
        self.assertNotEqual(self.key(bindgen='wasm-bindgen 2'), key)
        self.assertNotEqual(self.key(env={'CARGO_PROFILE_RELEASE_LTO': 'thin'}), key)
        self.assertNotEqual(self.key(env={'SLINT_EMIT_DEBUG_INFO': '1'}), key)
        self.assertEqual(self.key(env={'SLINT_WEB_TARGET_DIR': '/different/target'}), key)

    def test_cache_restores_exact_assets_and_rejects_corruption(self):
        key = self.key()
        self.assertFalse(restore(self.cache, key, self.output))
        for name in ASSETS:
            (self.output / name).write_bytes(name.encode())
        save(self.cache, key, self.output)
        for file in self.output.iterdir():
            file.unlink()
        self.assertTrue(restore(self.cache, key, self.output))
        self.assertEqual({f.name: f.read_bytes() for f in self.output.iterdir()},
                         {name: name.encode() for name in ASSETS})
        (self.cache / key / ASSETS[0]).write_bytes(b'corrupt')
        self.assertFalse(restore(self.cache, key, self.output))

    def test_documentation_only_edits_do_not_invalidate_compilation(self):
        readme = self.root / 'README.md'
        readme.write_text('Before')
        key = compilation_key(self.root, [self.source, readme], 'rustc', 'bindgen', {})
        readme.write_text('After')
        self.assertEqual(compilation_key(self.root, [self.source, readme], 'rustc', 'bindgen', {}), key)


if __name__ == '__main__':
    unittest.main()
