#!/usr/bin/env bash
# Mounts the disk image `make dmg` just built, copies Rekall.app into /Applications, and ejects
# the disk image again.
#
#   scripts/macos-install.sh [path/to/Rekall.dmg]
#
# Unlike scripts/macos-app.sh, this replaces whatever is at /Applications/Rekall.app. A Rekall
# that is already running (from /Applications or from a mounted disk image) is quit first, so the
# copy is never in use while it is replaced.
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

echo "==> Installing into /Applications"
rm -rf /Applications/Rekall.app
cp -R "$MOUNT/Rekall.app" /Applications/Rekall.app
xattr -dr com.apple.quarantine /Applications/Rekall.app 2>/dev/null || true

echo "==> Ejecting $MOUNT"
hdiutil detach "$MOUNT" -quiet || hdiutil detach "$MOUNT" -force -quiet || true

echo
echo "    Installed at /Applications/Rekall.app"
