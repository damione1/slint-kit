# SPDX-License-Identifier: GPL-3.0-or-later
import contextlib
import importlib.util
import io
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('complete_consumers', Path(__file__).with_name('check-complete-consumers.py'))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class CompleteConsumerChecks(unittest.TestCase):
    def test_omitting_names_selects_all_consumers(self):
        args = checker.parse_arguments(['--mode', 'local'])
        self.assertEqual(args.consumers or checker.CONSUMERS, checker.CONSUMERS)

    def test_explicit_consumer_selection(self):
        args = checker.parse_arguments(['--mode', 'local', 'table-consumer'])
        self.assertEqual(args.consumers, ['table-consumer'])

    def test_unknown_consumer_is_rejected_before_reading_sources(self):
        with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as error:
            checker.parse_arguments(['--mode', 'local', '../outside'])
        self.assertEqual(error.exception.code, 2)

    def test_registry_mode_rejects_local_kit_packages(self):
        for name in checker.KIT_PACKAGES:
            checker.verify_sources([{'name': name, 'source': 'registry+https://github.com/rust-lang/crates.io-index'}], checker.BuildMode.RELEASE)
            with self.assertRaisesRegex(ValueError, 'local dependency'):
                checker.verify_sources([{'name': name, 'source': None}], checker.BuildMode.RELEASE)

    def test_local_mode_rejects_registry_kit_packages(self):
        checker.verify_sources([{'name': 'slint-kit', 'source': None}], checker.BuildMode.LOCAL)
        with self.assertRaisesRegex(ValueError, 'registry dependency'):
            checker.verify_sources([{'name': 'slint-kit', 'source': 'registry+index'}], checker.BuildMode.LOCAL)

    def test_tagged_table_example_is_a_registry_consumer(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / 'table'
            checker.copy_sources(directory, 'table-consumer', 'v0.7.2')
            self.assertIn('slint-kit = "=0.7.2"', (directory / 'Cargo.toml').read_text())
            self.assertNotIn('CellContent', (directory / 'src/main.rs').read_text())
