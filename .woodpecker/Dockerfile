# SPDX-License-Identifier: GPL-3.0-or-later
FROM rust:1.99.1-bookworm@sha256:4715c3ad743951a3cea90e8b432f2a17e0ef5f3f3c3abf75108879e783b396f0

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        clang libclang-dev pkg-config python3 util-linux time \
        libxcb-shape0-dev libxcb-xfixes0-dev \
        libxkbcommon-dev libxkbcommon-x11-dev \
        libfontconfig1-dev libwayland-dev fonts-dejavu-core \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add rustfmt clippy \
    && rustup target add wasm32-unknown-unknown
