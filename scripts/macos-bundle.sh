#!/usr/bin/env bash
# Assembles Rekall.app and the disk image you drag it into /Applications from.
#
#   scripts/macos-bundle.sh native   the GraalVM binary, ~90 MB, starts instantly
#   scripts/macos-bundle.sh jvm      a jlink runtime plus the jar, ~200 MB, no GraalVM needed
#
# This script only packages: it expects the payload to already be built, which is what the
# `dmg-native` and `dmg-jvm` targets in the Makefile do before calling it. It is macOS only by
# construction (swiftc, iconutil, hdiutil), and deliberately separate from `build` and `native`,
# which have to keep working unchanged on Windows and Linux.
set -euo pipefail

FLAVOUR="${1:-}"
case "$FLAVOUR" in
    native|jvm) ;;
    *) echo "usage: $(basename "$0") native|jvm" >&2; exit 2 ;;
esac

if [ "$(uname -s)" != "Darwin" ]; then
    echo "This packages a macOS .app and only runs on macOS." >&2
    exit 1
fi

cd "$(dirname "$0")/.."

# The reactor version, read from the pom rather than from a Maven invocation: this runs after a
# build that took minutes and should not spend three more seconds starting a JVM to learn a
# string that is one line of the file. CFBundleShortVersionString has to be numeric, so the
# qualifier is dropped from what goes into Info.plist.
VERSION="$(awk '/<artifactId>rekall-parent<\/artifactId>/ { found = 1; next }
                found && /<version>/ { gsub(/.*<version>|<\/version>.*/, ""); print; exit }' pom.xml)"
SHORT_VERSION="${VERSION%%-*}"
ARCH="$(uname -m)"

# dist/, not target/: the native build runs `mvn clean install` over the reactor, and the reactor
# root owns target/. Building one flavour there deleted the disk image the other had just
# produced. dist/ is gitignored and nothing in the Maven build ever looks at it.
OUT="dist/macos/$FLAVOUR"
APP="$OUT/Rekall.app"
DMG="dist/Rekall-${SHORT_VERSION}-${FLAVOUR}-${ARCH}.dmg"

rm -rf "$OUT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

# --- payload -----------------------------------------------------------------------------

case "$FLAVOUR" in
native)
    BINARY="rekall-app/target/rekall-app"
    if [ ! -x "$BINARY" ]; then
        echo "$BINARY not found. Run 'make native' first (it needs GraalVM as JAVA_HOME)." >&2
        exit 1
    fi
    echo "==> Native binary ($(du -h "$BINARY" | cut -f1))"
    cp "$BINARY" "$APP/Contents/Resources/rekall-app"
    ;;
jvm)
    JAR="rekall-app/target/rekall-app-${VERSION}.jar"
    if [ ! -f "$JAR" ]; then
        echo "$JAR not found. Run 'make build' first." >&2
        exit 1
    fi
    JLINK="${JAVA_HOME:+$JAVA_HOME/bin/}jlink"
    command -v "$JLINK" >/dev/null 2>&1 || { echo "jlink not found. Set JAVA_HOME to a JDK 25." >&2; exit 1; }

    JDK_MAJOR="$("${JLINK}" --version 2>&1 | cut -d. -f1)"
    if [ "$JDK_MAJOR" -lt 25 ] 2>/dev/null; then
        echo "jlink is from JDK $JDK_MAJOR, the jar is built for 25. Set JAVA_HOME to a JDK 25." >&2
        exit 1
    fi

    echo "==> jlink runtime"
    # java.se is the whole standard platform: trimming it further with jdeps is not worth doing
    # against a Spring application, where half of what is loaded is decided reflectively at
    # runtime and a missing module surfaces as a failure on some screen nobody opened while
    # testing. jdk.unsupported is sun.misc.Unsafe, which Spring and Hibernate both still reach
    # for; jdk.crypto.ec and jdk.zipfs cover TLS to anything the notes link to and the zip
    # filesystem the export writes through.
    "${JLINK}" \
        --add-modules java.se,jdk.unsupported,jdk.crypto.ec,jdk.zipfs,jdk.management \
        --strip-debug --no-header-files --no-man-pages --compress=zip-6 \
        --output "$APP/Contents/runtime"
    cp "$JAR" "$APP/Contents/Resources/rekall-app.jar"
    ;;
esac

# --- launcher ----------------------------------------------------------------------------

echo "==> Launcher (swiftc, $ARCH)"
# -parse-as-library because a .swift file not named main.swift is otherwise compiled as
# a script, and @main is rejected in a module that has top-level code.
swiftc -O -parse-as-library -target "${ARCH}-apple-macos13.0" \
    packaging/macos/Launcher.swift packaging/macos/FolderPicker.swift \
    packaging/macos/ClaudeCodeLauncher.swift \
    -o "$APP/Contents/MacOS/Rekall"

# --- icon --------------------------------------------------------------------------------

# Built from the PWA icon the UI already ships, so the dock icon and the browser tab cannot
# drift apart. 512 is the largest source there is, so the 1024 slot is an upscale: replace
# icons/icon-512.png with a larger master and this picks it up unchanged.
echo "==> Icon"
ICONSET="$OUT/AppIcon.iconset"
mkdir -p "$ICONSET"
ICON_SOURCE="rekall-ui/public/icons/icon-512.png"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" "$ICON_SOURCE" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
    sips -z "$((size * 2))" "$((size * 2))" "$ICON_SOURCE" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
rm -rf "$ICONSET"

# --- metadata ----------------------------------------------------------------------------

sed "s/@VERSION@/${SHORT_VERSION}/g" packaging/macos/Info.plist.in > "$APP/Contents/Info.plist"
printf 'APPL????' > "$APP/Contents/PkgInfo"

# An unsigned bundle is refused outright by macOS on Apple silicon, so this is not optional
# polish: without at least an ad-hoc signature the app does not launch at all on the machine
# that built it. It is still not a Developer ID signature, so a copy that travels through a
# browser download arrives quarantined - see the README for the one command that clears it.
echo "==> Ad-hoc signature"
codesign --force --deep --sign - "$APP" 2>/dev/null

# --- disk image --------------------------------------------------------------------------

echo "==> Disk image"
# The window you drag Rekall out of needs an /Applications symlink beside the bundle, and to
# anything walking the tree that symlink is the whole of /Applications: staged inside the
# repository, it makes the IDE index every application on the machine. So the staging folder
# lives outside the project and the bundle is moved through it, leaving dist/ with the .app and
# the disk image and no link at all. The move is a rename when TMPDIR is on the same volume,
# which on macOS it is.
STAGE="$(mktemp -d "${TMPDIR:-/tmp}/rekall-dmg.XXXXXX")"
unstage() {
    if [ -d "$STAGE/Rekall.app" ]; then
        mv "$STAGE/Rekall.app" "$APP"
    fi
    rm -rf "$STAGE"
}
trap unstage EXIT

mv "$APP" "$STAGE/Rekall.app"
ln -s /Applications "$STAGE/Applications"
rm -f "$DMG"
hdiutil create -quiet -volname "Rekall" -srcfolder "$STAGE" -ov -format UDZO "$DMG"
unstage
trap - EXIT

echo
echo "    $DMG ($(du -h "$DMG" | cut -f1))"
echo "    open $(dirname "$DMG"), then drag Rekall onto Applications"
