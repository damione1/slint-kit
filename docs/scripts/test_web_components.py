# SPDX-License-Identifier: GPL-3.0-or-later
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from web_cache import ASSETS

spec = importlib.util.spec_from_file_location('web_components', Path(__file__).with_name('build-web-previews.py'))
web = importlib.util.module_from_spec(spec)
spec.loader.exec_module(web)


class ComponentPreviewTests(unittest.TestCase):
    def test_notices_have_independent_urls_and_shared_bytes_are_deduplicated(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source, output = root / 'source', root / 'output'
            source.mkdir()
            for name in ASSETS:
                (source / name).write_bytes(name.encode())
            for name in web.EXTRA_ASSETS:
                (source / name).write_bytes(name.encode())
            (source / 'component.json').write_text(json.dumps({'key': 'compiled'}))
            before = web.publish_assets(source, output, 'checkbox', 'kit')
            other = web.publish_assets(source, output, 'radio-group', 'kit')
            self.assertEqual(before['sources']['THIRD_PARTY_NOTICES.txt'], other['sources']['THIRD_PARTY_NOTICES.txt'])
            (source / 'NOTICE.txt').write_bytes(b'Corrected copyright notice')
            after = web.publish_assets(source, output, 'checkbox', 'kit')
            self.assertEqual(before['base'], after['base'])
            self.assertEqual(before['assets'], after['assets'])
            self.assertNotEqual(before['sources']['NOTICE.txt'], after['sources']['NOTICE.txt'])
            self.assertEqual(before['sources']['source.tar.gz'], after['sources']['source.tar.gz'])

    def test_kit_version_bumps_reuse_modules_but_external_dependencies_and_features_invalidate(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            docs = root / 'docs'
            files = {'quickstart/checkbox/src/main.rs': 'fn main() {}',
                     'quickstart/checkbox/build.rs': 'fn main() {}',
                     'preview/build.rs': 'fn main() {}', 'preview/src/lib.rs': 'pub fn start() {}'}
            for name, content in files.items():
                path = docs / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_text(content)
            def versions(kit, slint='1.18.1', features='["std"]'):
                (docs / 'preview/Cargo.toml').write_text(f'[dependencies]\nslint-kit = {{ version = "={kit}", features = ["native"] }}\n[build-dependencies]\nslint-kit-build = "={kit}"\n')
                (docs / 'preview/Cargo.lock').write_text(f'version = 4\n[[package]]\nname="slint-kit"\nversion="{kit}"\n[[package]]\nname="slint"\nversion="{slint}"\nsource="registry+https://github.com/rust-lang/crates.io-index"\nchecksum="checksum"\n')
                (root / 'Cargo.toml').write_text(f'[workspace.dependencies]\nslint = {{ version = "={slint}", features = {features} }}\n')
            inputs = {'entries': {'starter:checkbox:kit': {'identity': 'same reachable UI'}}}
            with patch.object(web, 'ROOT', root), patch.object(web, 'DOCS', docs), patch.object(web, 'runtime_inputs', return_value={}), patch.object(web, 'consumer_identity', return_value={}):
                versions('0.4.0'); original = web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {})
                versions('0.4.1'); self.assertEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))
                versions('0.4.1', '1.18.2'); self.assertNotEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))
                versions('0.4.1', features='["std", "accessibility"]'); self.assertNotEqual(original, web.code_key('checkbox', 'kit', inputs, 'rustc', 'bindgen', {}))


if __name__ == '__main__':
    unittest.main()
