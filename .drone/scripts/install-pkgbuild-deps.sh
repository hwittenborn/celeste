#!/usr/bin/env bash
set -e

cd makedeb/
cp PKGBUILD PKGBUILD.OLD

echo -e '\npackage() { true; }' >> PKGBUILD
sed -i -e 's|source=.*||' -e 's|sha256sums=.*||' PKGBUILD
makedeb -s --no-build --no-check --no-confirm

rm PKGBUILD
mv PKGBUILD.OLD PKGBUILD