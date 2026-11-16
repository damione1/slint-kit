# SPDX-License-Identifier: GPL-3.0-or-later
import io
import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import bundle_cache
from bundle_cache import Command, Outcome, Settings

KEY = bundle_cache.parse_key('a' * 64)
ENVIRONMENT = {'DOCS_CACHE_BUCKET': 'kit-cache', 'CLOUDFLARE_API_TOKEN': 'token', 'CLOUDFLARE_ACCOUNT_ID': 'account'}


def completed(code: int = 0, stdout: str = '', stderr: str = '') -> subprocess.CompletedProcess[str]:
    return subprocess.CompletedProcess([], code, stdout, stderr)


class BundleCacheTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.cache = self.root / 'cache'
        self.settings = Settings.from_environment(ENVIRONMENT)

    def make_entry(self, cache: Path) -> None:
        entry = cache / KEY
        entry.mkdir(parents=True)
        (entry / 'manifest.json').write_text('{"key": "%s"}' % KEY)
        (entry / 'slint_kit_previews_bg.wasm').write_bytes(b'wasm')

    def test_settings_require_every_variable(self):
        for name in ENVIRONMENT:
            with self.subTest(name=name):
                partial = {key: value for key, value in ENVIRONMENT.items() if key != name}
                with self.assertRaisesRegex(ValueError, name):
                    Settings.from_environment(partial)
        self.assertEqual(self.settings.object_path(KEY), f'kit-cache/preview-bundles/{KEY}.tar.gz')

    def test_key_must_be_a_sha256_digest(self):
        for value in ['', 'A' * 64, 'a' * 63, '../' + 'a' * 61]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                bundle_cache.parse_key(value)

    def test_push_then_pull_round_trips_the_entry(self):
        source = self.root / 'source'
        self.make_entry(source)
        uploaded: dict[str, bytes] = {}

        def wrangler(arguments: list[str]) -> subprocess.CompletedProcess[str]:
            file = Path(arguments[arguments.index('--file') + 1])
            self.assertIn('--remote', arguments)
            if arguments[2] == 'put':
                self.assertEqual(arguments[3], self.settings.object_path(KEY))
                uploaded['data'] = file.read_bytes()
            else:
                file.write_bytes(uploaded['data'])
            return completed()

        bundle_cache.push(KEY, self.settings, source, wrangler)
        self.assertEqual(bundle_cache.pull(KEY, self.settings, self.cache, wrangler), Outcome.HIT)
        self.assertEqual((self.cache / KEY / 'slint_kit_previews_bg.wasm').read_bytes(), b'wasm')

    def test_missing_object_is_a_miss_and_other_failures_raise(self):
        miss = lambda arguments: completed(1, stderr='The specified key does not exist. [code: 10007]')
        self.assertEqual(bundle_cache.pull(KEY, self.settings, self.cache, miss), Outcome.MISS)
        self.assertFalse((self.cache / KEY).exists())
        denied = lambda arguments: completed(1, stderr='Authentication error [code: 10000]')
        with self.assertRaisesRegex(RuntimeError, 'Authentication error'):
            bundle_cache.pull(KEY, self.settings, self.cache, denied)

    def test_push_refuses_a_missing_entry(self):
        calls: list[list[str]] = []
        with self.assertRaisesRegex(ValueError, 'No compiled bundle'):
            bundle_cache.push(KEY, self.settings, self.cache, lambda arguments: calls.append(arguments) or completed())
        self.assertEqual(calls, [])

    def test_pull_rejects_foreign_or_unsafe_archive_members(self):
        for name in ['../escape.txt', 'other/manifest.json', f'{KEY}/../../escape.txt']:
            with self.subTest(name=name):
                def wrangler(arguments: list[str], name=name) -> subprocess.CompletedProcess[str]:
                    with tarfile.open(arguments[arguments.index('--file') + 1], 'w:gz') as archive:
                        info = tarfile.TarInfo(name)
                        info.size = 1
                        archive.addfile(info, io.BytesIO(b'x'))
                    return completed()
                with self.assertRaisesRegex(ValueError, 'Unexpected archive member'):
                    bundle_cache.pull(KEY, self.settings, self.cache, wrangler)
        self.assertFalse((self.root / 'escape.txt').exists())

    def test_pull_writes_the_outcome_for_the_workflow(self):
        output = self.root / 'github-output'
        arguments = ['bundle_cache.py', Command.PULL.value, '--key', KEY, '--cache-dir', str(self.cache)]
        miss = lambda arguments: completed(1, stderr='NoSuchKey')
        with patch.dict(os.environ, {**ENVIRONMENT, 'GITHUB_OUTPUT': str(output)}), \
                patch('sys.argv', arguments), patch.object(bundle_cache, 'run_wrangler', miss):
            bundle_cache.main()
        self.assertEqual(output.read_text(), 'outcome=miss\n')

    def test_command_line_failure_exits_non_zero(self):
        arguments = ['bundle_cache.py', Command.PUSH.value, '--key', KEY, '--cache-dir', str(self.cache)]
        with patch.dict(os.environ, ENVIRONMENT), patch('sys.argv', arguments):
            with self.assertRaisesRegex(SystemExit, 'No compiled bundle'):
                bundle_cache.main()


if __name__ == '__main__':
    unittest.main()
