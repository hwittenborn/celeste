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

    just update-metainfo

update-metainfo:
    #!/usr/bin/env python3
    import sys
    import markdown
    import bs4
    from bs4 import BeautifulSoup, NavigableString

    # Read the changelog and metainfo files into Beautiful Soup.
    changelog_path = "CHANGELOG.md"
    metainfo_path = "assets/com.hunterwittenborn.Celeste.metainfo.xml"

    changelog_text = open(changelog_path).read()
    changelog_html = markdown.markdown(changelog_text)
    metainfo_html = open(metainfo_path).read()

    changelog_soup = BeautifulSoup(changelog_html, features="lxml")
    metainfo_soup = BeautifulSoup(metainfo_html, features="xml")

    # Convert the changelog entries into a dictionary of versions as keys and notes as values.
    releases = {}
    current_release = (None, None)  # A tuple of the version and date of the release (as strings).
    add_notes = lambda notes: releases.setdefault(current_release, []).append(notes)

    for element in changelog_soup.html.body.findChildren(recursive=False):
        # If we encountered an 'h2', that's a version, and we need to add it to the release list.
        if element.name == "h2":
            # This '.split' call and the stuff after gets the version (e.g. '1.0.0') from a changelog version entry (e.g. '[1.0.0] - 1970-01-01').
            version = element.text.split("]")[0][1:]
            date = None

            if version != "Unreleased":
                date = element.text.split("] - ")[1]
            current_release = (version, date)

        # Otherwise if we haven't found a release yet, don't do anything (there's some filler text at the top of the changelog that shouldn't be included).
        elif current_release == (None, None):
            continue
        # Otherwise if the element is an 'h3', add the element to the release's notes as a new 'p' element with custom text.
        elif element.name == "h3":
            section_string = ""
            contents = element.contents[0]

            if contents == "Fixed":
                section_string = "Fixes in this release:"
            elif contents == "Changed":
                section_string = "Changes in this release:"
            elif contents == "Added":
                section_string = "Features added in this release:"
            else:
                raise Exception("Unknown header: " + contents)

            section_tag = metainfo_soup.new_tag("p")
            section_tag.insert(0, NavigableString(section_string))
            add_notes(section_tag)
        # Otherwise just add element as more notes.
        else:
            add_notes(element)

    # Create a new 'releases' tag containing all of the releases.
    releases_tag = metainfo_soup.new_tag("releases")

    for release, notes in releases.items():
        release_tag = metainfo_soup.new_tag("release")

        # If this tag is the unreleased one, make a new 'beta' tag based on the latest release (which will be the second version that was inserted into the releases dictionary).
        if release[0] == "Unreleased":
            release_tag["version"] = list(releases.keys())[1][0] + "-git"
            release_tag["type"] = "development"
        else:
            release_tag["version"] = release[0]
            release_tag["date"] = release[1]

        release_description_tag = metainfo_soup.new_tag("description")

        for note_tag in notes:
            release_description_tag.append(note_tag)

        release_tag.append(release_description_tag)
        releases_tag.append(release_tag)
    
    # Update the releases in the metainfo Beautiful Soup instance and write it out.
    metainfo_soup.component.releases.replace_with(releases_tag)
    prettified_soup = metainfo_soup.prettify(formatter=bs4.formatter.HTMLFormatter(indent=4))
    open(metainfo_path, "w").write(prettified_soup)

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

# vim: set sw=4 expandtab:
