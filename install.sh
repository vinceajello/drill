#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== Drill Linux Installer ==="

# Check for release binary, build if not present
if [ ! -f "target/release/drill" ]; then
    echo "Release binary not found. Building drill (--release)..."
    cargo build --release
fi

# Determine install directories
if [ "$EUID" -eq 0 ]; then
    # System-wide installation (root)
    BIN_DIR="${PREFIX:-/usr/local/bin}"
    DESKTOP_DIR="${PREFIX:-/usr/local/share}/applications"
    ICON_DIR="${PREFIX:-/usr/local/share}/icons/hicolor/512x512/apps"
else
    # User-level installation
    BIN_DIR="${HOME}/.local/bin"
    DESKTOP_DIR="${HOME}/.local/share/applications"
    ICON_DIR="${HOME}/.local/share/icons/hicolor/512x512/apps"
fi

echo "Installing to:"
echo "  Executable:       $BIN_DIR/drill"
echo "  Desktop entry:    $DESKTOP_DIR/drill.desktop"
echo "  Application Icon: $ICON_DIR/drill.png"

# Create directories
mkdir -p "$BIN_DIR"
mkdir -p "$DESKTOP_DIR"
mkdir -p "$ICON_DIR"

# Copy executable
cp "target/release/drill" "$BIN_DIR/drill"
chmod +x "$BIN_DIR/drill"

# Copy icon
if [ -f "resources/icon.png" ]; then
    cp "resources/icon.png" "$ICON_DIR/drill.png"
fi

# Copy desktop file
if [ -f "resources/drill.desktop" ]; then
    cp "resources/drill.desktop" "$DESKTOP_DIR/drill.desktop"
fi

# Update desktop & icon caches if tools are available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$(dirname "$ICON_DIR")" 2>/dev/null || true
fi

echo ""
echo "✅ Installation complete!"

# Path warning for user installation
if [ "$EUID" -ne 0 ]; then
    case ":$PATH:" in
        *":$BIN_DIR:"*) ;;
        *)
            echo "⚠️  Note: '$BIN_DIR' is not in your PATH."
            echo "   Add it by running: export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
    esac
fi

echo "You can now run 'drill' from terminal or launch it from your application menu/launchpad!"
