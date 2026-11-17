# SPDX-License-Identifier: GPL-3.0-or-later
import runpy
from pathlib import Path
import tomllib
import unittest


class ToolchainVersions(unittest.TestCase):
    def test_preserves_consumer_features_and_path_dependencies(self):
        project = runpy.run_path(str(Path(__file__).with_name('sync-doc-versions.py')))
        workspace = {'package': {'rust-version': '1.95'},
                     'dependencies': {'slint': {'version': '~1.19.0'}}}
        source = '''[package]
rust-version = "1.92"
[dependencies]
slint = { version = "=1.18.1", default-features = false, features = ["compat-1-18", "renderer-femtovg"] }
slint-kit = { path = "../../crates/kit" }
[build-dependencies]
slint-build = "=1.18.1"
'''
        result = project['with_toolchain_versions'](source, workspace)
        manifest = tomllib.loads(result)
        self.assertEqual(manifest['package']['rust-version'], '1.95')
        self.assertEqual(manifest['dependencies']['slint'], {
            'version': '=1.19.0', 'default-features': False,
            'features': ['compat-1-18', 'renderer-femtovg']})
        self.assertEqual(manifest['dependencies']['slint-kit'], {'path': '../../crates/kit'})
        self.assertEqual(manifest['build-dependencies']['slint-build'], '=1.19.0')
        self.assertEqual(project['with_toolchain_versions'](result, workspace), result)
