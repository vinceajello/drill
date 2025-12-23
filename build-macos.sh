#!/bin/bash

# Build script for Drill macOS app bundle

set -e

echo "🔨 Building Drill for macOS..."

# Check if cargo-packager is installed
if ! command -v cargo-packager &> /dev/null; then
    echo "⚠️  cargo-packager not found. Installing..."
    cargo install cargo-packager --locked
fi

# Build the bundle
echo "📦 Creating .app bundle and .dmg..."
cargo packager --release

echo "✅ Build complete!"
echo ""
echo "📍 Output files:"
echo "   • App Bundle: target/release/bundle/Drill.app"
echo "   • DMG Installer: target/release/bundle/Drill_0.1.0_aarch64.dmg"
echo ""
echo "To open the app:"
echo "   open target/release/bundle/Drill.app"
echo ""
echo "To open the DMG:"
echo "   open target/release/bundle/Drill_0.1.0_aarch64.dmg"
