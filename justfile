set positional-arguments

build:
	cargo build --release --bin celeste-tray
	cargo build --release --bin celeste

install:
	install -Dm 755 target/release/celeste "{{ env_var('DESTDIR') }}/usr/bin/celeste"
	install -Dm 644 assets/com.hunterwittenborn.Celeste.desktop "{{ env_var('DESTDIR') }}/usr/share/applications/com.hunterwittenborn.Celeste.desktop"
	install -Dm 644 assets/com.hunterwittenborn.Celeste-regular.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/scalable/apps/com.hunterwittenborn.Celeste.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.Celeste.CelesteTrayLoading-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/symbolic/apps/com.hunterwittenborn.Celeste.CelesteTrayLoading-symbolic.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.Celeste.CelesteTraySyncing-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/symbolic/apps/com.hunterwittenborn.Celeste.CelesteTraySyncing-symbolic.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.Celeste.CelesteTrayWarning-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/symbolic/apps/com.hunterwittenborn.Celeste.CelesteTrayWarning-symbolic.svg"
	install -Dm 644 assets/context/com.hunterwittenborn.Celeste.CelesteTrayDone-symbolic.svg "{{ env_var('DESTDIR') }}/usr/share/icons/hicolor/symbolic/apps/com.hunterwittenborn.Celeste.CelesteTrayDone-symbolic.svg"
	install -Dm 644 assets/com.hunterwittenborn.Celeste.metainfo.xml "{{ env_var('DESTDIR') }}/usr/share/metainfo/com.hunterwittenborn.Celeste.metainfo.xml"

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
    sed -i "s|version: .*|version: '${version}'|" snap/snapcraft.yaml

    date="$(cat CHANGELOG.md | grep "^## \[${version}\]" | grep -o '[^ ]*$')"
    notes="$(parse-changelog CHANGELOG.md "${version}")"
    just update-metainfo "${version}" "${date}" "${notes}"

update-metainfo version date notes:
    #!/usr/bin/env python3
    import sys
    import markdown
    from bs4 import BeautifulSoup

    metainfo_path = "assets/com.hunterwittenborn.Celeste.metainfo.xml"
    version = sys.argv[1]
    date = sys.argv[2]
    notes = markdown.markdown(sys.argv[3])

    text = open(metainfo_path).read()

    soup = BeautifulSoup(text, features="xml")
    release = soup.new_tag("release")
    release["version"] = version
    release["date"] = date
    description = soup.new_tag("description")
    description.append(BeautifulSoup(notes, "html.parser"))
    release.append(description)

    soup.component.releases.findAll()[0].insert_before(release)
    open(metainfo_path, "w").write(soup.prettify())

# Create the Snap using an already build copy of Celeste. This currently requires you to be running on Ubuntu 22.10 or newer.
create-host-snap:
    #!/usr/bin/env bash
    set -euo pipefail

    cd "$(git rev-parse --show-toplevel)"
    host_snapcraft_yml="$(cat snap/snapcraft.yaml | grep -Ev 'source-type: |override-build: |just build')"
    tmpdir="$(mktemp -d)"

    find ./ -mindepth 1 -maxdepth 1 -not -path './target' -exec cp '{}' "${tmpdir}/{}" -R \;
    mkdir -p "${tmpdir}/target/release"
    cp target/debug/celeste "${tmpdir}/target/release"

    cd "${tmpdir}"
    echo "${host_snapcraft_yml}" > snap/snapcraft.yaml
    snapcraft -v

    cd -
    cp "${tmpdir}/"*.snap ./
    rm "${tmpdir}" -rf
