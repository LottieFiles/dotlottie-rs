#!/bin/bash

ANDROID_NDK_HOME=/opt/homebrew/share/android-ndk

# Genreate Bindings for Kotlin
echo "Gnerating bindings for Kotlin"
cargo +nightly run --features=uniffi/cli --bin uniffi-bindgen generate src/dlplayer.udl  --language kotlin --out-dir uniffi-bindings

android_target_triples=(
  "aarch64-linux-android"
  "armv7-linux-androideabi"
)

export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi19-clang

for TARGET_TRIPLE in "${android_target_triples[@]}"; do
  echo "Building android target $TARGET_TRIPLE"
  cargo build --target $TARGET_TRIPLE --release
done;

echo "Done building for all targets"

