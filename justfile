create-appimage:
    #!/usr/bin/env bash
    set -euo pipefail
    cd "$(git rev-parse --show-toplevel)"
    appimage-builder --recipe appimage/AppImageBuilder.yml

get-version:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo metadata --format-version=1 --no-deps | jq -r '.packages[0].version'

update-versions:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(just get-version)"
    sed -i "s| version: .*| version: ${version}|" appimage/AppImageBuilder.yml
    sed -i "s|pkgver=.*|pkgver=${version}|" makedeb/PKGBUILD