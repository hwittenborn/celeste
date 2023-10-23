# Maintainer: Hunter Wittenborn <hunter@hunterwittenborn.com>
pkgname=celeste
pkgver=0.8.0
pkgrel=1
pkgdesc='Sync your cloud files'
arch=('any')
depends=(
    'libadwaita-1-0'
    'libdbus-1-3'
    'rclone'
)
makedepends=(
    'just'
    'libadwaita-1-dev'
    'libclang-15-dev'
    'libdbus-1-dev'
    'libgtk-4-dev'
    'golang-go>=2:1.17'
    'pkg-config'
    'rustup'
)
license=('GPL-3.0')
url='https://github.com/hwittenborn/celeste'

source=("${url}/archive/refs/tags/v${pkgver}.tar.gz")
sha256sums=('SKIP')

build() {
    cd "${pkgname}-${pkgver}/"
    just build
}

package() {
    cd "${pkgname}-${pkgver}/"
    DESTDIR="${pkgdir}" just install
}

# vim: set sw=4 expandtab:
