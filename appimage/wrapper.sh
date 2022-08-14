#!/bin/bash
set -e

# We have to extract the AppImage to make modifications to the files it contains, so do that below, and clean up our work when we're done.
tmpdir="$(mktemp -d)"
cp "${APPDIR}"/* "${tmpdir}" -r
cd "${tmpdir}"

chmod +x usr/bin/{celeste,rclone}
usr/bin/celeste "${@}" || true

rm "${tmpdir}" -r