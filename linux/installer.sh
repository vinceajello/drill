#!/usr/bin/env bash

set -e

# Resolve project root directory (parent of linux/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

usage() {
    echo "Usage: $0 [install|uninstall]"
    echo ""
    echo "Commands:"
    echo "  install, -i, --install      Build (if needed) and install Drill on Linux"
    echo "  uninstall, -u, --uninstall  Remove Drill binary, desktop entry, and icon"
    echo "  help, -h, --help            Show this help message"
}

do_install() {
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

    echo "You can now run 'drill' from terminal or launch it from your application menu!"
}

do_uninstall() {
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
}

case "$1" in
    install|-i|--install)
        do_install
        ;;
    uninstall|-u|--uninstall)
        do_uninstall
        ;;
    help|-h|--help)
        usage
        exit 0
        ;;
    *)
        if [ -z "$1" ]; then
            echo "Error: Missing action parameter." >&2
        else
            echo "Error: Unknown parameter '$1'." >&2
        fi
        usage
        exit 1
        ;;
esac
