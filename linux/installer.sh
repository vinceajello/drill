#!/usr/bin/env bash

set -e

# Resolve project root directory (parent of linux/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

usage() {
    echo "Usage: $0 [install|uninstall] [options]"
    echo ""
    echo "Commands:"
    echo "  install, -i, --install      Build (if needed) and install Drill on Linux"
    echo "  uninstall, -u, --uninstall  Remove Drill binary, desktop entry, and icon"
    echo "  help, -h, --help            Show this help message"
    echo ""
    echo "Options for uninstall:"
    echo "  --purge, -p                 Also remove configuration & log data (~/.drill)"
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
    local purge_data=false
    if [ "$1" = "--purge" ] || [ "$1" = "-p" ] || [ "$2" = "--purge" ] || [ "$2" = "-p" ]; then
        purge_data=true
    fi

    echo "=== Drill Linux Uninstaller ==="

    local found_any=false
    local error_occurred=false

    # Resolve user home directory even if running via sudo
    local user_home="${HOME}"
    if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
        user_home="$(eval echo "~$SUDO_USER")"
    fi

    local user_bin="$user_home/.local/bin/drill"
    local user_desktop="$user_home/.local/share/applications/drill.desktop"
    local user_icon="$user_home/.local/share/icons/hicolor/512x512/apps/drill.png"

    local sys_prefix="${PREFIX:-/usr/local}"
    local sys_bin="$sys_prefix/bin/drill"
    local sys_desktop="$sys_prefix/share/applications/drill.desktop"
    local sys_icon="$sys_prefix/share/icons/hicolor/512x512/apps/drill.png"

    # Check system files
    for file in "$sys_bin" "$sys_desktop" "$sys_icon"; do
        if [ -f "$file" ]; then
            found_any=true
            if [ "$EUID" -ne 0 ]; then
                echo "⚠️  Found system file '$file' but root privileges (sudo) are required to remove it."
                error_occurred=true
            else
                echo "Removing $file..."
                rm -f "$file" || error_occurred=true
            fi
        fi
    done

    # Check user files
    for file in "$user_bin" "$user_desktop" "$user_icon"; do
        if [ -f "$file" ]; then
            found_any=true
            if [ "$EUID" -eq 0 ] && [ -n "$SUDO_USER" ]; then
                echo "Removing $file..."
                rm -f "$file" || error_occurred=true
            elif [ "$EUID" -ne 0 ]; then
                echo "Removing $file..."
                rm -f "$file" || error_occurred=true
            fi
        fi
    done

    # Purge configuration directory if requested or notify user
    local drill_config_dir="$user_home/.drill"
    if [ "$purge_data" = true ]; then
        if [ -d "$drill_config_dir" ]; then
            found_any=true
            echo "Purging configuration directory: $drill_config_dir"
            rm -rf "$drill_config_dir" || error_occurred=true
        fi
    elif [ -d "$drill_config_dir" ]; then
        echo "ℹ️  Configuration directory '$drill_config_dir' was retained."
        echo "   Use './linux/installer.sh uninstall --purge' to remove configuration data as well."
    fi

    # Update desktop and icon caches
    if command -v update-desktop-database >/dev/null 2>&1; then
        update-desktop-database "$sys_prefix/share/applications" 2>/dev/null || true
        update-desktop-database "$user_home/.local/share/applications" 2>/dev/null || true
    fi

    if command -v gtk-update-icon-cache >/dev/null 2>&1; then
        gtk-update-icon-cache -f -t "$sys_prefix/share/icons/hicolor" 2>/dev/null || true
        gtk-update-icon-cache -f -t "$user_home/.local/share/icons/hicolor" 2>/dev/null || true
    fi

    if [ "$error_occurred" = true ]; then
        if [ "$EUID" -ne 0 ]; then
            echo ""
            echo "❌ Uninstallation incomplete. Please re-run with sudo:"
            echo "   sudo ./linux/installer.sh uninstall"
            exit 1
        else
            echo ""
            echo "❌ Some files could not be removed."
            exit 1
        fi
    elif [ "$found_any" = false ]; then
        echo "ℹ️  No installed Drill files found."
    else
        echo ""
        echo "✅ Drill uninstalled successfully!"
    fi
}

case "$1" in
    install|-i|--install)
        do_install
        ;;
    uninstall|-u|--uninstall)
        do_uninstall "$@"
        ;;
    purge)
        do_uninstall --purge
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
