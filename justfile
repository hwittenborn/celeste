build:
	cargo build --release --bin celeste-tray
	cargo build --release --bin celeste

install:
	install -Dm 755 target/release/celeste "{{ env_var('DESTDIR') }}/usr/bin/celeste"
	install -Dm 644 assets/com.hunterwittenborn.Celeste.desktop "{{ env_var('DESTDIR') }}/usr/share/applications/com.hunterwittenborn.Celeste.desktop"
	install -Dm 644 assets/com.hunterwittenborn.Celeste-regular.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/scalable/apps/com.hunterwittenborn.Celeste.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.CelesteTrayLoading-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/scalable/status/com.hunterwittenborn.CelesteTrayLoading-symbolic.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.CelesteTraySyncing-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/scalable/status/com.hunterwittenborn.CelesteTraySyncing-symbolic.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.CelesteTrayDone-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/scalable/status/com.hunterwittenborn.CelesteTrayDone-symbolic.svg"

clippy:
	cargo build --bin celeste-tray
	cargo clippy -- -D warnings

get-version:
    #!/usr/bin/env bash
    source makedeb/PKGBUILD
    echo "${pkgver}"

update-versions:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(just get-version)"
    sed -i "s|version = .*|version = \"${version}\"|" celeste/Cargo.toml celeste-tray/Cargo.toml libceleste/Cargo.toml

# Create the Snap using an already build copy of Celeste. This currently requires you to be running on Ubuntu 22.10 or newer.
create-host-snap:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "$(git rev-parse --show-toplevel)"
    host_snapcraft_yml="$(cat snap/snapcraft.yaml | grep -Ev 'source-type: |override-build: |just build')"
    tmpdir="$(mktemp -d)"

    find ./ -mindepth 1 -maxdepth 1 -not -path './target' -exec cp '{}' "${tmpdir}/{}" -R \;
    mkdir -p "${tmpdir}/target/release"
    cp target/release/celeste "${tmpdir}/target/release"

    cd "${tmpdir}"
    echo "${host_snapcraft_yml}" > snap/snapcraft.yaml
    snapcraft -v

    cd -
    cp "${tmpdir}/"*.snap ./
    rm "${tmpdir}" -rf
