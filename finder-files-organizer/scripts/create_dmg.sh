#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   ./scripts/create_dmg.sh [--target aarch64-apple-darwin|x86_64-apple-darwin] [--name finder-files-organizer]
#
# Notes:
# - Must be run on macOS (requires: hdiutil).
# - Produces ./dist/<name>-<target>.dmg

TARGET=""
APP_NAME="finder-files-organizer"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --target)
      TARGET="${2:-}"
      shift 2
      ;;
    --name)
      APP_NAME="${2:-}"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [--target aarch64-apple-darwin|x86_64-apple-darwin] [--name finder-files-organizer]";
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if ! command -v hdiutil >/dev/null 2>&1; then
  echo "Error: hdiutil not found. Run this script on macOS." >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"
STAGE_DIR="$DIST_DIR/stage"

mkdir -p "$DIST_DIR"
rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

CARGO_ARGS=(build --release)
DMG_SUFFIX="native"

if [[ -n "$TARGET" ]]; then
  CARGO_ARGS+=(--target "$TARGET")
  DMG_SUFFIX="$TARGET"
fi

( cd "$REPO_ROOT" && cargo "${CARGO_ARGS[@]}" )

BIN_PATH=""
if [[ -n "$TARGET" ]]; then
  BIN_PATH="$REPO_ROOT/target/$TARGET/release/finder-files-organizer"
else
  BIN_PATH="$REPO_ROOT/target/release/finder-files-organizer"
fi

if [[ ! -f "$BIN_PATH" ]]; then
  echo "Error: built binary not found at: $BIN_PATH" >&2
  exit 1
fi

# DMG layout: a folder with the binary + Applications symlink
cp -f "$BIN_PATH" "$STAGE_DIR/$APP_NAME"
chmod +x "$STAGE_DIR/$APP_NAME"
ln -s /Applications "$STAGE_DIR/Applications"

DMG_PATH="$DIST_DIR/${APP_NAME}-${DMG_SUFFIX}.dmg"
TMP_DMG_PATH="$DIST_DIR/${APP_NAME}-${DMG_SUFFIX}-tmp.dmg"

rm -f "$DMG_PATH" "$TMP_DMG_PATH"

# Create a writable temp DMG first, then convert to compressed read-only
hdiutil create -ov -volname "$APP_NAME" -srcfolder "$STAGE_DIR" -format UDRW "$TMP_DMG_PATH" >/dev/null
hdiutil convert "$TMP_DMG_PATH" -format UDZO -o "$DMG_PATH" >/dev/null
rm -f "$TMP_DMG_PATH"

echo "Created DMG: $DMG_PATH"
