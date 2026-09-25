#!/usr/bin/env bash
# Render every raster icon from rekall-ui/public/favicon.svg, the mark's one source.
#
#   ./scripts/render-icons.sh      (or `make icons`)
#
# macOS only (sips) and needs Google Chrome, which rasterises the SVG's gradients, blurs and
# grain faithfully where qlmanage and sips do not. The PNGs it writes are committed, so neither
# the release workflow nor `make dmg` needs Chrome: run this after editing the SVG, and
# commit what it produces.
#
#   rekall-ui/public/icons/icon-{192,512}.png   PWA "any": the rounded plate, corners transparent
#   rekall-ui/public/icons/icon-512-maskable.png PWA "maskable": square, the platform crops it
#   rekall-ui/public/apple-touch-icon.png        square 180, iOS rounds it itself
#   rekall-app/desktop/icons/icon@2x.png         1024 dock master on Apple's icon grid (Tauri);
#                                                 named @2x so the bundler reads it as retina —
#                                                 1024px only resolves to an ICNS type at density 2.
set -euo pipefail

cd "$(dirname "$0")/.."

CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
SOURCE="rekall-ui/public/favicon.svg"

if [[ ! -x "$CHROME" ]]; then
    echo "Google Chrome not found at $CHROME - set CHROME to its binary" >&2
    exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
cp "$SOURCE" "$WORK/icon.svg"

# One page per layout, each 1024 square; Chrome screenshots it with a transparent background.
#
# plate:    the SVG as drawn, full bleed.
# square:   the plate enlarged until its rounded corners and rim fall outside the frame (at
#           1180 each corner arc only touches the frame's corner), so a platform mask can cut
#           any shape out of it. The anchor bead, the outermost element that matters, stays
#           inside the maskable safe zone, the central 80% circle.
# dock:     Apple's grid: the plate at 824 in the middle of 1024, with the soft shadow system
#           icons carry baked into the empty margin.
page() {
    local name="$1" body_style="$2" img_style="$3"
    cat > "$WORK/$name.html" <<EOF
<!doctype html><html><head><style>
  html, body { margin: 0; width: 1024px; height: 1024px; overflow: hidden; }
  body { $body_style }
  img { display: block; $img_style }
</style></head><body><img src="icon.svg"></body></html>
EOF
}

page plate  "background: transparent;" "width: 1024px; height: 1024px;"
page square "background: #1a1b1d;" "width: 1180px; height: 1180px; margin: -78px;"
page dock   "background: transparent;" \
    "width: 824px; height: 824px; margin: 100px; filter: drop-shadow(0 14px 44px rgba(0, 0, 0, 0.38));"

render() {
    "$CHROME" --headless --disable-gpu --hide-scrollbars --force-device-scale-factor=1 \
        --default-background-color=00000000 --window-size=1024,1024 \
        --screenshot="$WORK/$1.png" "file://$WORK/$1.html" >/dev/null 2>&1
    if [[ ! -s "$WORK/$1.png" ]]; then
        echo "Chrome produced no screenshot for $1" >&2
        exit 1
    fi
}

echo "==> Rendering $SOURCE"
render plate
render square
render dock

resize() {
    sips -z "$2" "$2" "$WORK/$1.png" --out "$3" >/dev/null
    echo "    $3"
}

resize plate 512 rekall-ui/public/icons/icon-512.png
resize plate 192 rekall-ui/public/icons/icon-192.png
resize square 512 rekall-ui/public/icons/icon-512-maskable.png
resize square 180 rekall-ui/public/apple-touch-icon.png
cp "$WORK/dock.png" rekall-app/desktop/icons/icon@2x.png
echo "    rekall-app/desktop/icons/icon@2x.png"
