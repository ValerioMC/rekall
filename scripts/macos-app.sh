#!/usr/bin/env bash
# Mounts the disk image `make dmg` just built and runs Rekall.app straight from it.
#
#   scripts/macos-app.sh [path/to/Rekall.dmg]
#
# Nothing is copied into /Applications: the build you are trying is the one on the disk image,
# exactly what another machine would get, and an installed copy (if there is one) is left alone.
# A Rekall that is already running is quit first and a disk image from an earlier build is
# ejected, so the one that opens is always the one just built.
set -euo pipefail

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This mounts a macOS disk image and only runs on macOS." >&2
    exit 1
fi

cd "$(dirname "$0")/.."

DMG="${1:-$(ls -t target/release/bundle/dmg/*.dmg 2>/dev/null | head -1 || true)}"
if [ -z "$DMG" ] || [ ! -f "$DMG" ]; then
    echo "No disk image under target/release/bundle/dmg. Run 'make dmg' first." >&2
    exit 1
fi

if pgrep -xq rekall-desktop || pgrep -xq Rekall; then
    echo "==> Quitting the running Rekall"
    osascript -e 'quit app "Rekall"' >/dev/null 2>&1 || true
    for _ in $(seq 1 40); do
        { pgrep -xq rekall-desktop || pgrep -xq Rekall; } || break
        sleep 0.25
    done
fi

# Every volume a Rekall disk image was mounted on earlier ("Rekall", "Rekall 1", ...).
for volume in /Volumes/Rekall*; do
    [ -d "$volume/Rekall.app" ] || continue
    echo "==> Ejecting $volume"
    hdiutil detach "$volume" -quiet || hdiutil detach "$volume" -force -quiet || true
done

echo "==> Mounting $(basename "$DMG")"
MOUNT="$(hdiutil attach -noverify -noautoopen "$DMG" | awk -F'\t' '/\/Volumes\// { print $NF }' | tail -1)"
if [ -z "$MOUNT" ] || [ ! -d "$MOUNT/Rekall.app" ]; then
    echo "The disk image mounted, but no Rekall.app was found on it." >&2
    exit 1
fi

echo "==> Running $MOUNT/Rekall.app"
open "$MOUNT/Rekall.app"
echo
echo "    Running from the disk image; eject \"$(basename "$MOUNT")\" in Finder when you are done."
