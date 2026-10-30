# SPDX-License-Identifier: GPL-3.0-or-later
"""Exercise release gating, archive identity, artifact integrity and rollback guards."""
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import urllib.error

import docs_deploy


SOURCE = 'a' * 40
TAGGED = 'b' * 40
PREVIOUS = 'c' * 40


class DocumentationDeploymentTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.addCleanup(patch.stopall)
        patch.object(docs_deploy, 'ROOT', self.root).start()
        patch.object(docs_deploy.release, 'ROOT', self.root).start()
        (self.root / 'Cargo.toml').write_text('[workspace.package]\nversion="0.4.0"\n')
        (self.root / 'LICENSE').write_text('GPL fixture')
        self.site = self.root / 'site'
        self.site.mkdir()
        (self.site / 'index.html').write_text('<h1>Slint Kit</h1>')
        (self.site / '.well-known').mkdir()
        (self.site / '.well-known/example.json').write_text('{}')
        self.identity = {'schemaVersion': 1, 'version': '0.4.0',
                         'sourceRevision': SOURCE, 'releaseRevision': TAGGED}

    def test_only_explicit_stable_versions_can_reach_production(self):
        self.assertLess(docs_deploy.PublishedVersion.parse('0.9.0'), docs_deploy.PublishedVersion.parse('0.10.0'))
        for value in ['', 'v0.4.0', '00.4.0', '0.4', '0.4.0-rc.1', '0.4.0+build', '../main', None]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                docs_deploy.PublishedVersion.parse(value)

    def test_local_consumers_can_never_be_stamped_for_production(self):
        path = self.root / 'docs/.generated/consumers/manifest.json'
        path.parent.mkdir(parents=True)
        manifest = {'schemaVersion': 1, 'version': '0.4.0', 'slugs': ['checkbox'], 'mode': 'local'}
        path.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError, 'local or unprepared'):
            docs_deploy.require_release_consumers('0.4.0')
        manifest['mode'] = 'release'
        path.write_text(json.dumps(manifest))
        docs_deploy.require_release_consumers('0.4.0')
        with self.assertRaises(ValueError):
            docs_deploy.require_release_consumers('0.3.0')

    def test_documentation_correction_requires_unchanged_released_libraries(self):
        with patch.object(docs_deploy, 'git', side_effect=[SOURCE, TAGGED, '', '']):
            self.assertEqual(docs_deploy.source_identity('0.4.0'), self.identity)
        with patch.object(docs_deploy, 'git', side_effect=[SOURCE, TAGGED, 'crates/kit/src/lib.rs']):
            with self.assertRaisesRegex(ValueError, 'publish a new crate'):
                docs_deploy.source_identity('0.4.0')
        with patch.object(docs_deploy, 'git', side_effect=[SOURCE, TAGGED, '', ' M docs/src/page.md']):
            with self.assertRaisesRegex(ValueError, 'clean'):
                docs_deploy.source_identity('0.4.0')
        with self.assertRaisesRegex(ValueError, 'library version'):
            docs_deploy.source_identity('0.3.0')

    def test_integrity_covers_every_file_including_hidden_assets(self):
        docs_deploy.stamp(self.site, self.identity)
        self.assertEqual(docs_deploy.verify_artifact(self.site, '0.4.0', SOURCE), self.identity)
        (self.site / '.well-known/example.json').write_text('changed')
        with self.assertRaisesRegex(ValueError, 'changed'):
            docs_deploy.verify_artifact(self.site, '0.4.0', SOURCE)

    def test_missing_extra_or_different_revision_artifacts_are_rejected(self):
        for mutation in ['missing', 'extra', 'version', 'source']:
            with self.subTest(mutation=mutation):
                docs_deploy.stamp(self.site, self.identity)
                if mutation == 'missing':
                    (self.site / 'index.html').unlink()
                elif mutation == 'extra':
                    (self.site / 'unexpected.js').write_text('alert(1)')
                with self.assertRaises(ValueError):
                    docs_deploy.verify_artifact(self.site, '0.3.0' if mutation == 'version' else '0.4.0',
                                                PREVIOUS if mutation == 'source' else SOURCE)
                (self.site / 'index.html').write_text('<h1>Slint Kit</h1>')
                (self.site / 'unexpected.js').unlink(missing_ok=True)

    def archive(self, source='library'):
        manifest = '[package]\nname="slint-kit"\nversion="0.4.0"\nlicense="GPL-3.0-or-later"\n'
        files = {'Cargo.toml': manifest, 'Cargo.toml.orig': manifest, 'src/lib.rs': source,
                 '.cargo_vcs_info.json': json.dumps({'git': {'sha1': TAGGED}, 'path_in_vcs': 'crates/kit'}),
                 'LICENSE': 'GPL fixture', 'README.md': 'Usage'}
        data = io.BytesIO()
        with tarfile.open(fileobj=data, mode='w:gz') as archive:
            for name, text in files.items():
                content = text.encode()
                member = tarfile.TarInfo('slint-kit-0.4.0/' + name)
                member.size = len(content)
                archive.addfile(member, io.BytesIO(content))
        return data.getvalue(), files

    def test_published_archive_must_match_tag_checksum_and_local_library_source(self):
        original, files = self.archive()
        for name, content in files.items():
            path = self.root / 'crates/kit' / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content)
        with patch.dict(docs_deploy.release.PACKAGES, {'slint-kit': 'crates/kit'}, clear=True):
            for foreign_source in [False, True]:
                data = self.archive(source='foreign')[0] if foreign_source else original
                metadata = json.dumps({'version': {'yanked': False, 'checksum': hashlib.sha256(data).hexdigest()}}).encode()
                with patch.object(docs_deploy.release, 'request', side_effect=[metadata, data]):
                    if foreign_source:
                        with self.assertRaisesRegex(ValueError, 'source differs'):
                            docs_deploy.verify_registry(self.identity)
                    else:
                        docs_deploy.verify_registry(self.identity)

    def test_registry_newer_stable_version_prevents_rollback(self):
        versions = {'versions': [{'num': '0.4.0', 'yanked': False}, {'num': '0.5.0-rc.1', 'yanked': False},
                                 {'num': '0.6.0', 'yanked': True}]}
        with patch.object(docs_deploy.release, 'request', return_value=json.dumps(versions).encode()):
            docs_deploy.newer_published('0.4.0')
        versions['versions'].append({'num': '0.5.0', 'yanked': False})
        with patch.object(docs_deploy.release, 'request', return_value=json.dumps(versions).encode()):
            with self.assertRaisesRegex(ValueError, 'already published'):
                docs_deploy.newer_published('0.4.0')

    def test_initial_legacy_site_is_allowed_but_network_errors_fail_closed(self):
        with patch.object(docs_deploy, 'newer_published'):
            for code in [404, 429, 503]:
                with patch.object(docs_deploy.release, 'request', side_effect=urllib.error.HTTPError('url', code, '', {}, None)):
                    if code == 404:
                        docs_deploy.guard(self.identity, 'https://example.test')
                    else:
                        with self.assertRaises(urllib.error.HTTPError):
                            docs_deploy.guard(self.identity, 'https://example.test')

    def test_same_version_correction_must_descend_from_deployed_revision(self):
        deployed = {**self.identity, 'sourceRevision': PREVIOUS}
        with patch.object(docs_deploy, 'newer_published'), patch.object(docs_deploy.release, 'request', return_value=json.dumps(deployed).encode()):
            for returncode in [0, 1, 128]:
                with patch.object(docs_deploy.subprocess, 'run', return_value=subprocess.CompletedProcess([], returncode)):
                    if returncode == 0:
                        docs_deploy.guard(self.identity, 'https://example.test')
                    else:
                        with self.assertRaisesRegex(ValueError, 'older or unrelated'):
                            docs_deploy.guard(self.identity, 'https://example.test')

    def test_newer_production_version_is_never_replaced(self):
        deployed = {**self.identity, 'version': '0.5.0'}
        with patch.object(docs_deploy, 'newer_published'), patch.object(docs_deploy.release, 'request', return_value=json.dumps(deployed).encode()):
            with self.assertRaisesRegex(ValueError, 'replace production'):
                docs_deploy.guard(self.identity, 'https://example.test')


if __name__ == '__main__':
    unittest.main()
