#!/usr/bin/env bash
# Assemble FlyOnTheWall.app around an already-built release binary.
# Usage: packaging/macos-app.sh <path-to-binary> <version> [out-dir]
#
# The bundle carries the assets folder BESIDE the binary, the way Divus Factus
# does, because Bevy resolves its asset root from the executable's own
# directory. That is not a detail worth rediscovering: run this game's binary
# directly rather than through `cargo run` and, without this line, the glTF
# fails with "Path not found" while `cargo run` works perfectly — `cargo run`
# sets `CARGO_MANIFEST_DIR` and a shipped build has no such thing.
#
# The bundle is named without a space on purpose: Finder shows the spaced name
# from CFBundleDisplayName anyway, and a spaced path is one more thing for tar,
# codesign and the launcher to get right.
set -euo pipefail
BIN="${1:?usage: macos-app.sh <binary> <version> [out-dir]}"
VERSION="${2:?need a version}"
OUT="${3:-dist}"
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"

APP="$OUT/FlyOnTheWall.app"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

# The executable's name must match CFBundleExecutable in Info.plist, or macOS
# refuses to launch the bundle.
cp "$BIN" "$APP/Contents/MacOS/fly-on-the-wall"
chmod +x "$APP/Contents/MacOS/fly-on-the-wall"
strip "$APP/Contents/MacOS/fly-on-the-wall" 2>/dev/null || true

cp -R "$ROOT/assets" "$APP/Contents/MacOS/assets"

# CFBundleIconFile names it without the extension.
cp "$HERE/FlyOnTheWall.icns" "$APP/Contents/Resources/FlyOnTheWall.icns"
sed "s/__VERSION__/$VERSION/g" "$HERE/Info.plist" > "$APP/Contents/Info.plist"

# Ad-hoc sign so macOS runs it without a "damaged" error; the launcher also
# strips the download quarantine on install.
codesign --force --deep --sign - "$APP" 2>/dev/null || true

echo "built $APP ($(du -sh "$APP" | cut -f1))"
