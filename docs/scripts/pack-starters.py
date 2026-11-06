#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Package deterministic standalone consumers, retaining unchanged archives."""
import io
import zipfile
import tomllib
from doc_inputs import ROOT, DOCS, write_changed
from starter_sources import source_files, prepare

def readme(slug, kit_version, rust_version, slint_version):
    return f"""# Slint Kit: {slug}

Requires Rust {rust_version}+ and Slint {slint_version}. This example targets Slint Kit **{kit_version}**. Run `cargo run --locked` in this folder to use the matching registry release.

## Optional local source dependencies

Place the example folder beside a `slint-kit` checkout. Replace only the two kit dependency entries in `Cargo.toml` with the following paths, keeping the existing `slint` dependency:

```toml
[dependencies]
slint-kit = {{ path = "../slint-kit/crates/kit" }}

[build-dependencies]
slint-kit-build = {{ path = "../slint-kit/crates/kit-build" }}
```

Run `cargo generate-lockfile` once to resolve the source graph, then `cargo run --locked` in this folder. See https://slint-kit.dev/guides/project-setup/ for complete setup and https://slint-kit.dev/components/{slug}/ for the component.

## Français

Rust {rust_version}+ et Slint {slint_version} sont nécessaires. Cet exemple cible Slint Kit **{kit_version}**. Exécutez `cargo run --locked` dans ce dossier pour utiliser la version correspondante du registre.

Les dépendances sources ci-dessus sont une option pour le développement local.

Placez le dossier de l’exemple à côté du checkout `slint-kit`. Remplacez uniquement les deux entrées de dépendance du kit dans `Cargo.toml` par ces chemins et conservez la dépendance `slint`. Exécutez `cargo generate-lockfile` une fois, puis `cargo run --locked` dans ce dossier.

Configuration complète : https://slint-kit.dev/fr/guides/project-setup/ ; composant : https://slint-kit.dev/fr/components/{slug}/.
""".encode()


def main():
    workspace = tomllib.loads((ROOT / 'Cargo.toml').read_text())['workspace']
    output = DOCS / '.generated/starters'
    output.mkdir(parents=True, exist_ok=True)
    consumers = prepare()
    changed = 0
    for slug, folder in consumers.items():
        files = {name: (folder / name).read_bytes() for name in source_files(folder)}
        dependency = tomllib.loads(files['Cargo.toml'].decode())['dependencies']['slint-kit']
        kit_version = (dependency if isinstance(dependency, str) else dependency['version']).removeprefix('=')
        files['LICENSE'] = (ROOT / 'LICENSE').read_bytes()
        files['README.md'] = readme(slug, kit_version, workspace['package']['rust-version'], workspace['dependencies']['slint']['version'].removeprefix('='))
        stream = io.BytesIO()
        with zipfile.ZipFile(stream, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
            for name, content in sorted(files.items()):
                info = zipfile.ZipInfo(f'{slug}/{name}', date_time=(2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o100644 << 16
                archive.writestr(info, content)
        changed += write_changed(output / f'{slug}.zip', stream.getvalue())
    for path in output.glob('*.zip'):
        if path.stem not in consumers:
            path.unlink()
    print(f'Starter archives: {changed} changed, {len(consumers) - changed} reused')


if __name__ == '__main__':
    main()
