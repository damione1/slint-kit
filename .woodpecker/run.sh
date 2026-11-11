#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-or-later
set -euo pipefail

case "${1:-}" in
  check) export CARGO_BUILD_JOBS=1 ;;
  docs) export CARGO_BUILD_JOBS=2 ;;
  *) printf 'Usage: %s check|docs\n' "$0" >&2; exit 2 ;;
esac
kit_workflow=$1

# Agents provide /ci-cache from their own SSD, outside the temporary checkout.
# Separate workflow and checkout paths keep Cargo artifacts compatible while
# allowing both workflows to run concurrently on different workers.
kit_checkout=$(printf '%s' "$PWD" | sha256sum | cut -c 1-16)
kit_cache="/ci-cache/slint-kit/rust-1.99.1-nodebug-v1/$kit_checkout/$kit_workflow"
mkdir -p "$kit_cache"
exec 9>"$kit_cache/build.lock"
flock -n 9 || { printf 'This Cargo cache already has a writer\n' >&2; exit 1; }

export CARGO_HOME=/ci-cache/slint-kit/cargo
export CARGO_TARGET_DIR="$kit_cache/target"
export CARGO_INCREMENTAL=0
export CARGO_TERM_COLOR=always
# Debug symbols greatly enlarge these generated UI crates; they are not needed
# for headless CI assertions. Keep the normal dev/test optimization settings.
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0

kit_report_resources() {
  date -u '+Resources at %Y-%m-%dT%H:%M:%SZ'
  for kit_metric in memory.max memory.current memory.peak memory.events cpu.max; do
    if [[ -r "/sys/fs/cgroup/$kit_metric" ]]; then
      printf '%s: ' "$kit_metric"
      cat "/sys/fs/cgroup/$kit_metric"
    fi
  done
}
kit_report_resources
trap kit_report_resources EXIT
rustc --version
printf 'Cargo jobs: %s; target: %s\n' "$CARGO_BUILD_JOBS" "$CARGO_TARGET_DIR"

case "$kit_workflow" in
  check)
    /usr/bin/time -v cargo run -p xtask --locked -- check
    ;;
  docs)
    python3 -B docs/scripts/sync-doc-versions.py --check
    python3 -B -m unittest discover -s docs/scripts -p 'test_complete_consumers.py'
    python3 -B -m unittest discover -s .github/scripts -p 'test_*.py'
    /usr/bin/time -v python3 docs/scripts/check-starters.py --mode local
    /usr/bin/time -v python3 docs/scripts/check-complete-consumers.py --mode local
    ;;
esac
