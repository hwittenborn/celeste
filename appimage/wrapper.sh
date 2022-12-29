#!/bin/bash
set -e

# We have to extract the AppImage to make modifications to the files it contains, so do that below, and clean up our work when we're done.
tmpdir="$(mktemp -d)"
cp "${APPDIR}"/* "${tmpdir}" -r
cd "${tmpdir}"

chmod +x usr/bin/{celeste,rclone}
LD_LIBRARY_PATH="${PWD}/usr/lib/x86_64-linux-gnu:${PWD}/lib/x86_64-linux-gnu" usr/bin/celeste "${@}" || true

rm "${tmpdir}" -r