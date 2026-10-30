#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Build the same reusable workspace consumer for the interactive product preview."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
from doc_inputs import ROOT, DOCS, compiler_environment, source_hashes, write_changed, write_json


def build():
    environment = compiler_environment()
    subprocess.run(['cargo','build','--manifest-path',str(DOCS/'workspace-preview/Cargo.toml'),'--locked','--release','--target','wasm32-unknown-unknown'],cwd=ROOT,env=environment,check=True)
    bindgen = os.environ.get('WASM_BINDGEN','wasm-bindgen')
    output = DOCS / '.generated/workspace-preview'
    output.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='workspace-preview-') as temporary:
        temporary = Path(temporary)
        subprocess.run([bindgen,'--target','web','--out-dir',str(temporary),'--out-name','workspace',str(DOCS/'workspace-preview/target/wasm32-unknown-unknown/release/slint_kit_workspace_preview.wasm')],check=True)
        names=['workspace.js','workspace_bg.wasm']
        identity=hashlib.sha256(b''.join((temporary/name).read_bytes() for name in names)).hexdigest()[:20]
        for name in names: write_changed(output/identity/name,(temporary/name).read_bytes())
        sources=[DOCS/'src/assets/examples/workspace.slint',DOCS/'src/assets/examples/workspace.rs',DOCS/'workspace-preview/build.rs',DOCS/'workspace-preview/src/lib.rs',DOCS/'workspace-preview/Cargo.toml',DOCS/'workspace-preview/Cargo.lock']
        kit_sources=[path for path in (ROOT/'crates').rglob('*') if path.is_file() and path.suffix in ('.rs','.slint','.toml','.ttf')]
        record={'kitSourceHashes':source_hashes(kit_sources),'schema':1,'module':f'/assets/workspace-preview/{identity}/workspace.js','revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'sources':{str(path.relative_to(ROOT)):hashlib.sha256(path.read_bytes()).hexdigest() for path in sources},'bytes':{name:(temporary/name).stat().st_size for name in names}}
        write_json(output/'manifest.json',record)
        print(json.dumps(record,indent=2))


if __name__=='__main__': build()
