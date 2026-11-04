#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise release identity, archive integrity and partial-publish safeguards."""
import hashlib
import io
import json
from pathlib import Path
import re
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
        patch.object(release, 'request', return_value=b'{"versions": []}').start()

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
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), patch.object(release, 'check', side_effect=ValueError('foreign namespace')), patch.object(release.subprocess, 'run') as upload, patch.object(release, 'github_release') as metadata:
            with self.assertRaisesRegex(ValueError, 'foreign'):
                release.publish('0.1.0', 'abc123')
            upload.assert_not_called()
            metadata.assert_not_called()

    def test_notes_use_exact_version_and_keep_subheadings(self):
        (self.root / 'CHANGELOG.md').write_text(
            '# Changelog\n\n## Unreleased\nFuture\n\n## 0.1.01\nWrong\n\n'
            '## 0.1.0\n\nFix this.\n\n### Native\n- Keep selection.\n\n'
            '## 0.0.9\nOlder\n')
        self.assertEqual(release.release_notes('0.1.0'), 'Fix this.\n\n### Native\n- Keep selection.')
        for content in ['## 0.1.01\nWrong', '## 0.1.0\n\n## Older\nOther',
                        '## 0.1.0\nFirst\n## 0.1.0\nDuplicate']:
            (self.root / 'CHANGELOG.md').write_text(content)
            with self.subTest(content=content), self.assertRaises(ValueError):
                release.release_notes('0.1.0')

    def test_latest_compares_numeric_stable_versions_not_dates(self):
        cases = [
            ('0.6.1', ['v0.5.0', 'v0.6.0'], True, False),
            ('0.6.1', ['v0.10.0'], False, False),
            ('0.6.1', ['v1.0.0-rc.1', 'unrelated', 'v01.0.0'], True, False),
            ('0.6.1-rc.1', ['v0.6.0'], False, True),
            ('0.6.1+build-2', ['v0.6.1+build-1'], True, False),
        ]
        for version, tags, latest, prerelease in cases:
            (self.root / 'CHANGELOG.md').write_text(f'## {version}\nExact notes\n')
            def api(path, payload=None):
                if path.startswith('releases?'):
                    return [{'tag_name': tag, 'draft': False, 'prerelease': False} for tag in tags]
                return {'id': 1}
            with self.subTest(version=version, tags=tags), patch.object(release, 'github_api', side_effect=api) as github:
                release.github_release(version, 'abc123')
                payload = github.call_args.args[1]
                self.assertEqual(payload, {
                    'tag_name': f'v{version}', 'target_commitish': 'abc123',
                    'name': f'v{version}', 'body': 'Exact notes', 'draft': False,
                    'prerelease': prerelease, 'make_latest': 'true' if latest else 'false',
                })

    def test_releases_are_paginated_before_latest_decision(self):
        (self.root / 'CHANGELOG.md').write_text('## 0.6.1\nNotes\n')
        def api(path, payload=None):
            if path == 'releases?per_page=100&page=1':
                return [{'tag_name': 'unversioned', 'draft': False, 'prerelease': False}] * 100
            if path == 'releases?per_page=100&page=2':
                return [{'tag_name': 'v0.7.0', 'draft': False, 'prerelease': False}]
            return {'id': 1}
        with patch.object(release, 'github_api', side_effect=api) as github:
            release.github_release('0.6.1', 'abc123')
            self.assertEqual(github.call_args.args[1]['make_latest'], 'false')
            self.assertIn(unittest.mock.call('releases?per_page=100&page=2'), github.call_args_list)

    def test_latest_uses_published_facade_versions_not_unpublished_tags(self):
        (self.root / 'CHANGELOG.md').write_text('## 0.6.1\nNotes\n')
        cases = [
            ([{'num': '0.6.0', 'yanked': False}], 'true'),
            ([{'num': '0.6.2', 'yanked': False}], 'false'),
            ([{'num': '0.6.2', 'yanked': True}], 'true'),
            ([{'num': '0.6.2-rc.1', 'yanked': False}], 'true'),
        ]
        def api(path, payload=None):
            if path.startswith('tags?'):
                self.fail('An unpublished v0.6.2 tag must not influence latest')
            if path.startswith('releases?'):
                return []
            return {'id': 1}
        for versions, latest in cases:
            with self.subTest(versions=versions), patch.object(release, 'request', return_value=json.dumps({'versions': versions}).encode()) as registry, patch.object(release, 'github_api', side_effect=api) as github:
                release.github_release('0.6.1', 'abc123')
                self.assertEqual(github.call_args.args[1]['make_latest'], latest)
                registry.assert_called_once_with('https://crates.io/api/v1/crates/slint-kit')

    def test_unknown_registry_state_prevents_guessing_latest(self):
        (self.root / 'CHANGELOG.md').write_text('## 0.6.1\nNotes\n')
        error = urllib.error.HTTPError('url', 503, 'unavailable', {}, None)
        with patch.object(release, 'request', side_effect=error), patch.object(release, 'github_api', return_value=[]) as github:
            with self.assertRaises(urllib.error.HTTPError):
                release.github_release('0.6.1', 'abc123')
            github.assert_called_once_with('releases?per_page=100&page=1')

    def test_existing_release_is_preserved_including_notes_assets_and_draft(self):
        for draft in [False, True]:
            existing = {'tag_name': 'v0.1.0', 'body': 'Hand-edited notes',
                        'assets': [{'name': 'installer'}], 'draft': draft}
            with self.subTest(draft=draft), patch.object(release, 'github_api', return_value=[existing]) as github:
                release.github_release('0.1.0', 'abc123')
                github.assert_called_once_with('releases?per_page=100&page=1')

    def test_rerun_after_creation_does_not_publish_or_update_again(self):
        (self.root / 'CHANGELOG.md').write_text('## 0.1.0\nNotes\n')
        releases = []
        def api(path, payload=None):
            if path.startswith('releases?'):
                return releases.copy()
            releases.append(payload)
            return payload
        with patch.object(release, 'github_api', side_effect=api) as github:
            release.github_release('0.1.0', 'abc123')
            release.github_release('0.1.0', 'abc123')
            self.assertEqual(sum(call.args[0] == 'releases' for call in github.call_args_list), 1)

    def test_creation_conflict_only_accepts_the_exact_existing_tag(self):
        (self.root / 'CHANGELOG.md').write_text('## 0.1.0\nNotes\n')
        conflict = urllib.error.HTTPError('url', 422, 'conflict', {}, None)
        with patch.object(release, 'github_api', side_effect=[[], conflict, {'tag_name': 'v0.1.0'}]) as github:
            release.github_release('0.1.0', 'abc123')
            self.assertEqual(github.call_args.args, ('releases/tags/v0.1.0',))
        for status in [401, 403, 429, 503]:
            error = urllib.error.HTTPError('url', status, 'failure', {}, None)
            with self.subTest(status=status), patch.object(release, 'github_api', side_effect=[[], error]), self.assertRaises(urllib.error.HTTPError):
                release.github_release('0.1.0', 'abc123')
        with patch.object(release, 'github_api', side_effect=[[], conflict, {'tag_name': 'v0.0.9'}]), self.assertRaises(urllib.error.HTTPError):
            release.github_release('0.1.0', 'abc123')

    def test_metadata_waits_for_every_registry_archive_and_retries_after_partial_publish(self):
        names = list(release.PACKAGES)
        archives = {name: {} for name in names}
        for all_published in [False, True]:
            published = {name: all_published or name != names[-1] for name in names}
            events = []
            def verified(*args):
                events.append('verified')
                return True
            with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), patch.object(release, 'check', return_value=(archives, published)), patch.object(release.subprocess, 'run') as upload, patch.object(release, 'existing_version', side_effect=verified), patch.object(release, 'github_release', side_effect=lambda *args: events.append('release')):
                release.publish('0.1.0', 'abc123')
                self.assertEqual(events, ['release'] if all_published else ['verified', 'release'])
                self.assertEqual(upload.call_count, 0 if all_published else 1)

    def test_unverified_upload_never_creates_release(self):
        archives = {name: {} for name in release.PACKAGES}
        published = {name: False for name in release.PACKAGES}
        with patch.dict(release.os.environ, {'CARGO_REGISTRY_TOKEN': 'test-only'}), patch.object(release, 'check', return_value=(archives, published)), patch.object(release.subprocess, 'run'), patch.object(release.time, 'sleep'), patch.object(release, 'existing_version', return_value=False), patch.object(release, 'github_release') as metadata:
            with self.assertRaisesRegex(ValueError, 'registry version unavailable'):
                release.publish('0.1.0', 'abc123')
            metadata.assert_not_called()

    def test_github_transport_uses_builtin_token_and_structured_json(self):
        response = io.BytesIO(b'{"id": 1}')
        payload = {'body': 'Literal $() and `code`\nSecond line', 'make_latest': 'false'}
        with patch.dict(release.os.environ, {'GH_TOKEN': 'test-only', 'GITHUB_REPOSITORY': 'owner/kit'}), patch.object(release.urllib.request, 'urlopen', return_value=response) as transport:
            self.assertEqual(release.github_api('releases', payload), {'id': 1})
            request = transport.call_args.args[0]
            self.assertEqual(request.full_url, 'https://api.github.com/repos/owner/kit/releases')
            self.assertEqual(request.get_method(), 'POST')
            self.assertEqual(request.get_header('Authorization'), 'Bearer test-only')
            self.assertEqual(json.loads(request.data), payload)
            self.assertEqual(transport.call_args.kwargs, {'timeout': 30})
        for environment in [{'GH_TOKEN': '', 'GITHUB_REPOSITORY': 'owner/kit'},
                            {'GH_TOKEN': 'test-only', 'GITHUB_REPOSITORY': 'owner/kit/../../elsewhere'}]:
            with self.subTest(environment=environment), patch.dict(release.os.environ, environment), patch.object(release.urllib.request, 'urlopen') as transport:
                with self.assertRaisesRegex(ValueError, 'required'):
                    release.github_api('releases')
                transport.assert_not_called()

    def test_workflow_grants_release_write_only_to_publish_job(self):
        # Inspect the actual workflow, not the temporary crate fixture.
        workflow = (Path(__file__).resolve().parents[1] / 'workflows/release.yml').read_text()
        header, jobs = workflow.split('\njobs:\n', 1)
        match = re.search(r'(?ms)^  publish:\n.*?(?=^  \S|\Z)', jobs)
        self.assertIsNotNone(match)
        publish = match.group(0)
        other_jobs = jobs.replace(publish, '', 1)
        self.assertIn('\npermissions:\n  contents: read\n', header)
        self.assertIn('\n    permissions:\n      contents: write\n', publish)
        self.assertEqual(workflow.count('contents: write'), 1)
        self.assertNotIn('contents: write', other_jobs)
        self.assertIn('GH_TOKEN: ${{ github.token }}', publish)


if __name__ == '__main__':
    unittest.main()
