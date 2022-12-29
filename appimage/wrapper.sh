#! bin/bash
set -e

# The 'bin/bash' executable in the AppImage is being modified somehow, presumably by appimage-builder. It's screwing up our `LD_LIBRARY_PATH` calls somehow, so relaunch the binary against the system's copy of Bash.
if [[ "${INNER_BASH:+x}" == '' ]]; then
    # Also through some kind of sorcery, attempting to run either of these isn't working:
    #   - APPDIR="${APPDIR}" /usr/bin/bash ...
    #   - export APPDIR; /usr/bin/bash ...
    # Why is this the case? I have absolutely no clue, but this is working right now so we're gonna keep it.
    /usr/bin/bash -c "INNER_BASH=1 APPDIR='${APPDIR}' bash ${0}"
fi

# We have to extract the AppImage to make modifications to the files it contains, so do that below, and clean up our work when we're done.
tmpdir="$(mktemp -d)"
cp "${APPDIR}"/* "${tmpdir}" -r
cd "${tmpdir}"

chmod +x usr/bin/{celeste,rclone}
LD_LIBRARY_PATH="${PWD}/usr/lib/x86_64-linux-gnu:${PWD}/lib/x86_64-linux-gnu" usr/bin/celeste "${@}" || true

rm "${tmpdir}" -r