#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise release identity, archive integrity and partial-publish safeguards."""
import hashlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

import release


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / 'LICENSE').write_text('GPL fixture')
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion = "0.1.0"\n')
        for name, directory in release.PACKAGES.items():
            folder = self.root / directory
            folder.mkdir(parents=True)
            (folder / 'Cargo.toml').write_text(f'[package]\nname = "{name}"\nversion.workspace = true\n')
        self.addCleanup(patch.stopall)
        patch.object(release, 'ROOT', self.root).start()
        patch.object(release, 'run', side_effect=lambda *cmd: 'abc123' if cmd[1] == 'rev-parse' else '').start()
        patch.dict(release.os.environ, {'GITHUB_SHA': 'abc123'}).start()

    def archive(self, *, commit='abc123', dirty=False, source='original', lock='first', path='crates/kit'):
        files = {
            '.cargo_vcs_info.json': json.dumps({'git': {'sha1': commit, 'dirty': dirty}, 'path_in_vcs': path}),
            'Cargo.toml': '[package]\nname = "slint-kit"\nversion = "0.1.0"\nlicense = "GPL-3.0-or-later"\n',
            'LICENSE': 'GPL fixture', 'README.md': 'Public usage',
            'src/lib.rs': source, 'Cargo.lock': lock,
        }
        output = io.BytesIO()
        with tarfile.open(fileobj=output, mode='w:gz') as archive:
            for name, text in files.items():
                data = text.encode()
                entry = tarfile.TarInfo('slint-kit-0.1.0/' + name)
                entry.size = len(data)
                archive.addfile(entry, io.BytesIO(data))
        return output.getvalue()

    def test_tag_is_strict_and_all_versions_match(self):
        self.assertEqual(release.validate('v0.1.0'), ('0.1.0', 'abc123'))
        for tag in ['0.1.0', 'v01.1.0', 'v0.1', 'v0.1.0-01', 'v0.1.1', 'v0.1.0;echo nope']:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                release.validate(tag)

    def test_individual_package_drift_fails(self):
        manifest = self.root / 'crates/kit-slint/Cargo.toml'
        manifest.write_text('[package]\nname = "slint-kit-slint"\nversion = "0.2.0"\n')
        with self.assertRaisesRegex(ValueError, 'exactly'):
            release.validate('v0.1.0')

    def test_checkout_must_match_event_and_be_clean(self):
        with patch.dict(release.os.environ, {'GITHUB_SHA': 'other'}), self.assertRaisesRegex(ValueError, 'triggering'):
            release.validate('v0.1.0')
        with patch.object(release, 'run', side_effect=['abc123', ' M source.rs']), self.assertRaisesRegex(ValueError, 'clean'):
            release.validate('v0.1.0')

    def test_archive_provenance_and_source_identity(self):
        baseline = release.archive_files(self.archive(), 'slint-kit', '0.1.0', 'abc123')
        changed_lock = release.archive_files(self.archive(lock='later resolution'), 'slint-kit', '0.1.0', 'abc123')
        self.assertEqual(baseline, changed_lock)
        changed_source = release.archive_files(self.archive(source='foreign source'), 'slint-kit', '0.1.0', 'abc123')
        self.assertNotEqual(baseline, changed_source)
        for arguments in [{'commit': 'other'}, {'dirty': True}, {'path': 'foreign/path'}]:
            with self.subTest(arguments=arguments), self.assertRaises(ValueError):
                release.archive_files(self.archive(**arguments), 'slint-kit', '0.1.0', 'abc123')

    def test_registry_absence_is_only_404(self):
        for status in [404, 429, 503]:
            error = urllib.error.HTTPError('url', status, 'error', {}, None)
            with patch.object(release, 'request', side_effect=error):
                if status == 404:
                    self.assertFalse(release.existing_version('slint-kit', '0.1.0', 'abc123', {}))
                else:
                    with self.assertRaises(urllib.error.HTTPError):
                        release.existing_version('slint-kit', '0.1.0', 'abc123', {})

    def test_rerun_only_skips_identical_verified_content(self):
        expected = release.archive_files(self.archive(), 'slint-kit', '0.1.0', 'abc123')
        for arguments, valid in [({}, True), ({'source': 'foreign'}, False), ({'commit': 'other'}, False)]:
            data = self.archive(**arguments)
            metadata = json.dumps({'version': {'yanked': False, 'checksum': hashlib.sha256(data).hexdigest()}}).encode()
            with patch.object(release, 'request', side_effect=[metadata, data]):
                if valid:
                    self.assertTrue(release.existing_version('slint-kit', '0.1.0', 'abc123', expected))
                else:
                    with self.assertRaises(ValueError):
                        release.existing_version('slint-kit', '0.1.0', 'abc123', expected)

    def test_foreign_namespace_preflight_prevents_upload(self):
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), patch.object(release, 'check', side_effect=ValueError('foreign namespace')), patch.object(release.subprocess, 'run') as upload:
            with self.assertRaisesRegex(ValueError, 'foreign'):
                release.publish('0.1.0', 'abc123')
            upload.assert_not_called()


if __name__ == '__main__':
    unittest.main()
