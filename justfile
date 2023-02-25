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
    import bs4
    from bs4 import BeautifulSoup, NavigableString
    
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
    
    for h3 in description.find_all("h3"):
        changed_header_str = ""
    
        if h3.contents[0] == "Fixed":
            changed_header_str = "Fixes in this release:"
        elif h3.contents[0] == "Changed":
            changed_header_str = "Changes made in this release:"
        elif h3.contents[0] == "Added":
            changed_header_str = "Feature additions in this release:"
        else:
            raise Exception("Unknown header: " + str(h3))

        header = soup.new_tag("p")
        header.insert(0, NavigableString(changed_header_str))
        h3.insert_before(header)
        h3.extract()
    
    release.append(description)
    soup.component.releases.findAll()[0].insert_before(release)
    open(metainfo_path, "w").write(soup.prettify(formatter=bs4.formatter.HTMLFormatter(indent=4)))

update-translations:
    xtr celeste/src/main.rs celeste-tray/src/main.rs libceleste/src/lib.rs --copyright-holder 'Hunter Wittenborn <hunter@hunterwittenborn.com>' -o /dev/stdout --package-name 'Celeste' --package-version "$(just get-version)" > po/com.hunterwittenborn.Celeste.pot

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
