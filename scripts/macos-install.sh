#!/usr/bin/env bash
# Installs the bundle scripts/macos-bundle.sh just built into /Applications.
#
#   scripts/macos-install.sh native
#   scripts/macos-install.sh jvm
#
# The disk image is still what travels to another machine. This is for the machine that built
# it: the copy you are iterating on should be the one in the dock, and mounting a disk image to
# drag a bundle across is a manual step that gets skipped exactly when the build mattered.
#
# An existing /Applications/Rekall.app is replaced outright, without asking. REKALL_INSTALL=0
# skips the install and leaves the disk image as the only output.
set -euo pipefail

FLAVOUR="${1:-}"
case "$FLAVOUR" in
    native|jvm) ;;
    *) echo "usage: $(basename "$0") native|jvm" >&2; exit 2 ;;
esac

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This installs a macOS .app and only runs on macOS." >&2
    exit 1
fi

if [ "${REKALL_INSTALL:-1}" = "0" ]; then
    echo "==> Not installing (REKALL_INSTALL=0)"
    exit 0
fi

cd "$(dirname "$0")/.."

SOURCE="dist/macos/$FLAVOUR/Rekall.app"
TARGET="/Applications/Rekall.app"

if [ ! -d "$SOURCE" ]; then
    echo "$SOURCE not found. Run 'make dmg-$FLAVOUR' rather than this script on its own." >&2
    exit 1
fi

# /Applications is group-writable by admins on a stock macOS. Where it is not, this stops rather
# than escalating to sudo on its own: what a build target may write outside the project is not
# something to decide silently.
if [ ! -w /Applications ]; then
    echo "/Applications is not writable by $(whoami). Open $SOURCE's disk image and drag it across instead." >&2
    exit 1
fi

# A running copy has the bundle open, and the JVM flavour keeps a child process inside it. A
# replacement underneath either one leaves a process running code that is no longer on disk, so
# the app comes down first and goes back up afterwards only if it was up to begin with.
RUNNING=0
if pgrep -f "$TARGET/" >/dev/null 2>&1; then
    RUNNING=1
    echo "==> Quitting the running Rekall"
    osascript -e 'quit app "Rekall"' >/dev/null 2>&1 || true
    for _ in $(seq 1 40); do
        pgrep -f "$TARGET/" >/dev/null 2>&1 || break
        sleep 0.25
    done
    # It ignored the quit, or a child outlived it. Nothing here is unsaved: the console writes
    # on a pause and the database is a file the next start reopens.
    if pgrep -f "$TARGET/" >/dev/null 2>&1; then
        pkill -f "$TARGET/" >/dev/null 2>&1 || true
        sleep 1
    fi
fi

echo "==> Installing into /Applications"
# ditto rather than cp -R: it carries the extended attributes the ad-hoc signature is stored in,
# and a bundle that arrives without them is refused at launch on Apple silicon.
rm -rf "$TARGET"
ditto "$SOURCE" "$TARGET"

if [ "$RUNNING" = "1" ]; then
    echo "==> Starting the new one"
    open "$TARGET"
fi

echo
echo "    installed $TARGET ($FLAVOUR, $(du -sh "$TARGET" | cut -f1 | tr -d ' '))"
