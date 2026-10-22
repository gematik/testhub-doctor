#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE="$ROOT/images/gematik_g_blue_square.png"
EXPECTED_SOURCE="$ROOT/frontend/gematik_g_blue.jpg"
OUTPUT="$ROOT/src-gui/icons"

if [[ ! -f "$SOURCE" ]]; then
  echo "Missing icon source: $SOURCE" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

npx --yes @tauri-apps/cli@2.12.2 icon \
  "$SOURCE" \
  --output "$TMP_DIR/icons"

for icon in icon.png icon.icns icon.ico 32x32.png 128x128.png 128x128@2x.png; do
  cp "$TMP_DIR/icons/$icon" "$OUTPUT/$icon"
done
cp "$ROOT/images/gematik_g_blue.jpg" "$EXPECTED_SOURCE"
