#!/usr/bin/env bash
set -ex

# Set up the PBMPR.
.drone/scripts/setup-pbmpr.sh

# Install needed dependencies.
sudo apt-get install appimage-builder gh git jq just parse-changelog -y
.drone/scripts/install-pkgbuild-deps.sh

# Create the appimage.
just create-appimage

# Create the release.
pkgver="$(just get-version)"
release_notes="$(parse-changelog CHANGELOG.md "${pkgver}")"
echo "${github_api_key}" | gh auth login --with-token
gh release create "v${pkgver}" --title "v${pkgver}" --target "${DRONE_COMMIT_SHA}" -n "${release_notes}" "celeste.AppImage#celeste-${pkgver}.AppImage"

# vim: set sw=4 expandtab:
