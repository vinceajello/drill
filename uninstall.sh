#!/usr/bin/env bash

set -e

echo "=== Drill Linux Uninstaller ==="

if [ "$EUID" -eq 0 ]; then
    BIN_DIR="${PREFIX:-/usr/local/bin}"
    DESKTOP_DIR="${PREFIX:-/usr/local/share}/applications"
    ICON_DIR="${PREFIX:-/usr/local/share}/icons/hicolor/512x512/apps"
else
    BIN_DIR="${HOME}/.local/bin"
    DESKTOP_DIR="${HOME}/.local/share/applications"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
fi

rm -f "$BIN_DIR/drill"
rm -f "$DESKTOP_DIR/drill.desktop"
rm -f "$ICON_DIR/drill.png"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

echo "✅ Drill uninstalled successfully!"
