#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-or-later
"""Provision the missing native build tools only inside Workers Builds."""
import hashlib
import io
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import urllib.request

RUST = '1.98.1'
BINDGEN = '0.2.129'
ARCHIVES = {
    'x86_64': ('x86_64-unknown-linux-musl', '82d12bb940e2d4e72e0d5605387fc1b8ca179044e012b620f0ce4e7440e8320e'),
    'aarch64': ('aarch64-unknown-linux-musl', '2ed4351c35dd9440308bbb02767d47ea278efe851a52465300f3c94f5b6c2a87'),
}


def download(url):
    with urllib.request.urlopen(url, timeout=120) as response:
        return response.read()


def prepare():
    if os.environ.get('WORKERS_CI') != '1':
        return
    if platform.system() != 'Linux' or platform.machine() not in ARCHIVES:
        raise SystemExit('Workers web build requires Linux x86_64 or aarch64')
    tools = Path(__file__).resolve().parents[1] / '.generated/tools'
    tools.mkdir(parents=True, exist_ok=True)
    target, checksum = ARCHIVES[platform.machine()]
    bindgen = tools / f'wasm-bindgen-{BINDGEN}-{target}/wasm-bindgen'
    if not bindgen.is_file():
        url = f'https://github.com/wasm-bindgen/wasm-bindgen/releases/download/{BINDGEN}/wasm-bindgen-{BINDGEN}-{target}.tar.gz'
        content = download(url)
        if hashlib.sha256(content).hexdigest() != checksum:
            raise SystemExit('wasm-bindgen release checksum mismatch')
        with tarfile.open(fileobj=io.BytesIO(content), mode='r:gz') as archive:
            archive.extractall(tools, filter='data')
    os.environ['WASM_BINDGEN'] = str(bindgen)
    # Keep provisioning inside ignored build output; do not alter a user's Rust setup.
    cargo_home = tools / 'cargo'
    rustup_home = tools / 'rustup'
    os.environ.update(CARGO_HOME=str(cargo_home), RUSTUP_HOME=str(rustup_home),
                      RUSTUP_TOOLCHAIN=RUST,
                      PATH=str(cargo_home / 'bin') + os.pathsep + os.environ['PATH'])
    rustup = cargo_home / 'bin/rustup'
    if not rustup.is_file():
        host = target.replace('-musl', '-gnu')
        url = f'https://static.rust-lang.org/rustup/dist/{host}/rustup-init'
        content = download(url)
        checksum = download(url + '.sha256').decode().split()[0]
        if hashlib.sha256(content).hexdigest() != checksum:
            raise SystemExit('rustup installer checksum mismatch')
        installer = tools / 'rustup-init'
        installer.write_bytes(content)
        installer.chmod(0o755)
        subprocess.run([str(installer), '-y', '--no-modify-path', '--profile', 'minimal',
                        '--default-toolchain', RUST], check=True)
    subprocess.run([str(rustup), 'toolchain', 'install', RUST, '--profile', 'minimal'], check=True)
    subprocess.run([str(rustup), 'target', 'add', '--toolchain', RUST,
                    'wasm32-unknown-unknown'], check=True)
    os.environ.setdefault('CARGO_BUILD_JOBS', '2')
    os.environ.setdefault('CARGO_INCREMENTAL', '0')
    print(f'Workers tools ready: Rust {RUST}, wasm-bindgen {BINDGEN}', flush=True)
