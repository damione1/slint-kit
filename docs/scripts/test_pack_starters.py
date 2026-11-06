# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('pack_starters', Path(__file__).with_name('pack-starters.py'))
pack = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pack)


class ArchiveReadmeTests(unittest.TestCase):
    def test_readme_uses_supplied_release_and_toolchain_in_both_languages(self):
        source = pack.readme('button', '1.2.3', '1.99', '1.20.0').decode()
        self.assertEqual(source.count('**1.2.3**'), 2)
        self.assertEqual(source.count('Rust 1.99+'), 2)
        self.assertEqual(source.count('Slint 1.20.0'), 2)
        self.assertIn('cargo run --locked', source)
        self.assertIn('Optional local source dependencies', source)
        self.assertIn('option pour le développement local', source)
        self.assertNotIn('unreleased', source)
        self.assertNotIn('non publiée', source)


if __name__ == '__main__':
    unittest.main()
