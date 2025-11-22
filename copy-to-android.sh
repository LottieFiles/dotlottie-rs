#!/bin/bash

# Script to copy libdotlottie_player.so files to dotlottie-android project
# Usage: ./copy-to-android.sh

set -e

SOURCE_DIR="release/android-new/jniLibs"
DEST_DIR="$HOME/repos/dotlottie-android/dotlottie/src/main/jniLibs"

# Check if source directory exists
if [ ! -d "$SOURCE_DIR" ]; then
    echo "❌ Error: Source directory not found: $SOURCE_DIR"
    echo "   Please run 'make android-new' first to build the libraries."
    exit 1
fi

# Check if destination directory exists
if [ ! -d "$DEST_DIR" ]; then
    echo "❌ Error: Destination directory not found: $DEST_DIR"
    echo "   Please check that dotlottie-android project exists at: $HOME/repos/dotlottie-android"
    exit 1
fi

echo "📦 Copying libdotlottie_player.so files to dotlottie-android..."
echo ""

# Array of architectures
ARCHS=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

for arch in "${ARCHS[@]}"; do
    SOURCE_FILE="$SOURCE_DIR/$arch/libdotlottie_player.so"
    DEST_ARCH_DIR="$DEST_DIR/$arch"

    if [ ! -f "$SOURCE_FILE" ]; then
        echo "⚠️  Warning: $arch library not found at $SOURCE_FILE"
        continue
    fi

    # Create destination architecture directory if it doesn't exist
    mkdir -p "$DEST_ARCH_DIR"

    # Copy the library
    cp "$SOURCE_FILE" "$DEST_ARCH_DIR/"

    # Get file size
    SIZE=$(ls -lh "$SOURCE_FILE" | awk '{print $5}')

    echo "✓ Copied $arch (${SIZE})"
done

echo ""
echo "✅ All libraries copied successfully to:"
echo "   $DEST_DIR"
echo ""
echo "📝 Note: The libraries include debug logging with tag 'DotLottieRS'"
echo "   Use 'adb logcat -s DotLottieRS' to view logs"
