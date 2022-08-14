#!/usr/bin/env bash
set -ex

.drone/scripts/setup-pbmpr.sh
.drone/scripts/install-pkgbuild-deps.sh

cargo fmt --check
cargo clippy -- -D warnings

# vim: set sw=4 expandtab:
