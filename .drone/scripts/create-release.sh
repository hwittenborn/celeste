#!/usr/bin/env bash
set -ex

# Set up the PBMPR.
.drone/scripts/setup-pbmpr.sh

# Install needed dependencies.
sudo apt-get install gh git jq just parse-changelog -y
.drone/scripts/install-pkgbuild-deps.sh

# Create the release.
pkgver="$(just get-version)"
release_notes="$(parse-changelog CHANGELOG.md "${pkgver}")"
echo "${github_api_key}" | gh auth login --with-token
gh release create "v${pkgver}" --title "v${pkgver}" --target "${DRONE_COMMIT_SHA}" -n "${release_notes}"

# vim: set sw=4 expandtab:
