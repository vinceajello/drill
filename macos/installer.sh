#!/usr/bin/env bash

set -e

# Resolve project root directory (parent of macos/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_DIR"

APP_NAME="Drill.app"
BUNDLE_DIR="target/release/bundle/macos"
APP_BUNDLE="$BUNDLE_DIR/$APP_NAME"

usage() {
    echo "Usage: $0 [build|install|uninstall|run] [options]"
    echo ""
    echo "Commands:"
    echo "  build, -b, --build          Build (if needed) and package macOS Drill.app bundle"
    echo "  install, -i, --install      Build and install Drill.app to /Applications or ~/Applications"
    echo "  run, -r, --run              Build (if needed) and launch Drill.app"
    echo "  uninstall, -u, --uninstall  Remove installed Drill.app and CLI binary"
    echo "  help, -h, --help            Show this help message"
    echo ""
    echo "Options for uninstall:"
    echo "  --purge, -p                 Also remove configuration & log data (~/Library/Application Support/com.drill.drill and ~/.drill)"
}

do_build() {
    echo "=== Drill macOS Application Builder ==="

    # Check for release binary, build if not present
    if [ ! -f "target/release/drill" ]; then
        echo "Release binary not found. Building drill (--release)..."
        cargo build --release
    fi

    echo "Packaging macOS Application Bundle: $APP_BUNDLE"

    local contents_dir="$APP_BUNDLE/Contents"
    local macos_dir="$contents_dir/MacOS"
    local resources_dir="$contents_dir/Resources"

    # Clean old bundle if exists
    rm -rf "$APP_BUNDLE"

    # Create bundle directory structure
    mkdir -p "$macos_dir"
    mkdir -p "$resources_dir"

    # Copy binary
    echo "  Copying binary..."
    cp "target/release/drill" "$macos_dir/drill"
    chmod +x "$macos_dir/drill"

    # Copy Info.plist
    if [ -f "resources/Info.plist" ]; then
        echo "  Copying Info.plist..."
        cp "resources/Info.plist" "$contents_dir/Info.plist"
    else
        echo "⚠️  Warning: resources/Info.plist not found!"
    fi

    # Create PkgInfo
    echo -n "APPL????" > "$contents_dir/PkgInfo"

    # Copy icon
    if [ -f "resources/icon.icns" ]; then
        echo "  Copying icon.icns..."
        cp "resources/icon.icns" "$resources_dir/icon.icns"
    elif [ -f "resources/icon.png" ]; then
        echo "⚠️  resources/icon.icns not found, copying icon.png..."
        cp "resources/icon.png" "$resources_dir/icon.png"
    fi

    # Perform ad-hoc code signing if codesign is available
    if command -v codesign >/dev/null 2>&1; then
        echo "  Applying ad-hoc code signature..."
        codesign --force --deep --sign - "$APP_BUNDLE" 2>/dev/null || true
    fi

    # Remove quarantine extended attributes if xattr is available
    if command -v xattr >/dev/null 2>&1; then
        xattr -cr "$APP_BUNDLE" 2>/dev/null || true
    fi

    echo ""
    echo "✅ Application bundle ready at: $APP_BUNDLE"
}

do_install() {
    # Ensure app bundle is built
    do_build

    echo ""
    echo "=== Drill macOS Installer ==="

    local target_app_dir=""
    local bin_dir=""

    if [ "$EUID" -eq 0 ]; then
        # System-wide installation (root)
        target_app_dir="/Applications/$APP_NAME"
        bin_dir="${PREFIX:-/usr/local/bin}"
    elif [ -w "/Applications" ]; then
        # User has write permissions to /Applications
        target_app_dir="/Applications/$APP_NAME"
        bin_dir="${PREFIX:-/usr/local/bin}"
    else
        # User-level installation
        target_app_dir="${HOME}/Applications/$APP_NAME"
        bin_dir="${HOME}/.local/bin"
    fi

    echo "Installing to:"
    echo "  Application Bundle: $target_app_dir"
    echo "  Command-line tool:  $bin_dir/drill"

    # Create target directory
    mkdir -p "$(dirname "$target_app_dir")"

    # Remove existing app if present
    rm -rf "$target_app_dir"

    # Copy application bundle
    cp -R "$APP_BUNDLE" "$target_app_dir"

    # Install CLI symlink or binary
    if mkdir -p "$bin_dir" 2>/dev/null && [ -w "$bin_dir" ]; then
        ln -sf "$target_app_dir/Contents/MacOS/drill" "$bin_dir/drill" 2>/dev/null || cp "target/release/drill" "$bin_dir/drill"
        chmod +x "$bin_dir/drill"
        echo "  Installed CLI command to $bin_dir/drill"
    else
        # Fallback to ~/.local/bin if /usr/local/bin is not writable
        local user_bin="${HOME}/.local/bin"
        mkdir -p "$user_bin"
        ln -sf "$target_app_dir/Contents/MacOS/drill" "$user_bin/drill" 2>/dev/null || cp "target/release/drill" "$user_bin/drill"
        chmod +x "$user_bin/drill"
        bin_dir="$user_bin"
        echo "  Installed CLI command to $bin_dir/drill"
    fi

    # Register with LaunchServices
    if [ -x "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" ]; then
        /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -f "$target_app_dir" 2>/dev/null || true
    fi

    echo ""
    echo "✅ Installation complete!"

    # Path warning for user installation
    case ":$PATH:" in
        *":$bin_dir:"*) ;;
        *)
            echo "⚠️  Note: '$bin_dir' is not in your PATH."
            echo "   Add it by running: export PATH=\"$bin_dir:\$PATH\""
            ;;
    esac

    echo "You can launch Drill from Spotlight / Launchpad / Applications or run 'drill' from terminal."
}

do_run() {
    # Build if needed
    if [ ! -d "$APP_BUNDLE" ]; then
        do_build
    fi

    echo "Launching $APP_BUNDLE..."
    open "$APP_BUNDLE"
}

do_uninstall() {
    local purge_data=false
    if [ "$1" = "--purge" ] || [ "$1" = "-p" ] || [ "$2" = "--purge" ] || [ "$2" = "-p" ]; then
        purge_data=true
    fi

    echo "=== Drill macOS Uninstaller ==="

    local found_any=false
    local error_occurred=false

    # Resolve user home directory even if running via sudo
    local user_home="${HOME}"
    if [ -n "$SUDO_USER" ] && [ "$SUDO_USER" != "root" ]; then
        user_home="$(eval echo "~$SUDO_USER")"
    fi

    local sys_app="/Applications/$APP_NAME"
    local user_app="$user_home/Applications/$APP_NAME"
    local sys_bin="${PREFIX:-/usr/local/bin}/drill"
    local user_bin="$user_home/.local/bin/drill"

    # Check and unregister apps
    for app_path in "$sys_app" "$user_app"; do
        if [ -d "$app_path" ]; then
            found_any=true
            echo "Removing $app_path..."
            if [ -x "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" ]; then
                /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister -u "$app_path" 2>/dev/null || true
            fi
            rm -rf "$app_path" || {
                echo "⚠️  Failed to remove '$app_path'. You may need root privileges (sudo)."
                error_occurred=true
            }
        fi
    done

    # Check CLI binaries
    for file in "$sys_bin" "$user_bin"; do
        if [ -f "$file" ] || [ -L "$file" ]; then
            found_any=true
            echo "Removing $file..."
            rm -f "$file" || {
                echo "⚠️  Failed to remove '$file'. You may need root privileges (sudo)."
                error_occurred=true
            }
        fi
    done

    # Purge configuration and data if requested
    local config_dir="$user_home/Library/Application Support/com.drill.drill"
    local legacy_dir="$user_home/.drill"

    if [ "$purge_data" = true ]; then
        if [ -d "$config_dir" ]; then
            found_any=true
            echo "Purging configuration & logs: $config_dir"
            rm -rf "$config_dir" || error_occurred=true
        fi
        if [ -d "$legacy_dir" ]; then
            found_any=true
            echo "Purging legacy configuration: $legacy_dir"
            rm -rf "$legacy_dir" || error_occurred=true
        fi
    else
        if [ -d "$config_dir" ] || [ -d "$legacy_dir" ]; then
            echo "ℹ️  Configuration data was retained."
            echo "   Use './macos/installer.sh uninstall --purge' to remove configuration and log data as well."
        fi
    fi

    if [ "$error_occurred" = true ]; then
        echo ""
        echo "❌ Uninstallation incomplete. Try re-running with sudo:"
        echo "   sudo ./macos/installer.sh uninstall"
        exit 1
    elif [ "$found_any" = false ]; then
        echo "ℹ️  No installed Drill macOS files found."
    else
        echo ""
        echo "✅ Drill uninstalled successfully!"
    fi
}

case "$1" in
    build|-b|--build|bundle)
        do_build
        ;;
    install|-i|--install)
        do_install
        ;;
    run|-r|--run)
        do_run
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
