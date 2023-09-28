#!/bin/bash

ANDROID_NDK_HOME=/opt/homebrew/share/android-ndk

# export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
# export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc
# cargo build --features=ffi --target x86_64-unknown-linux-gnu --release

# Genreate Bindings for Kotlin
echo "Gnerating bindings for Kotlin"
cargo +nightly run --features=uniffi/cli --bin uniffi-bindgen generate src/dlplayer.udl  --language kotlin --out-dir uniffi-bindings
# cargo +nightly run --features=uniffi/cli generate --library ./target/x86_64-unknown-linux-gnu/release/libdlutils.so --language kotlin --out-dir uniffi-bindings
# cargo +nightly run generate ./src/dlutils.udl --language kotlin --out-dir ./uniffi-bindings --lib-file ./target/x86_64-unknown-linux-gnu/release/libdlutils.so

# Genreate Bindings for Swift
echo "Gnerating bindings for Swift"
cargo +nightly run --features=uniffi/cli --bin uniffi-bindgen generate src/dlplayer.udl  --language swift --out-dir uniffi-bindings
# cargo +nightly run --features=uniffi/cli generate --library ./target/x86_64-unknown-linux-gnu/release/libdlutils.so --language swift --out-dir uniffi-bindings
# cargo +nightly run generate ./src/dlutils.udl  --language swift --out-dir ./uniffi-bindings --lib-file ./target/x86_64-unknown-linux-gnu/release/libdlutils.so

ios_target_triples=(
  "x86_64-apple-ios"
  "aarch64-apple-ios-sim"
  "aarch64-apple-ios"
)
android_target_triples=(
  "aarch64-linux-android"
  "armv7-linux-androideabi"
)

# Build IOS targets
for TARGET_TRIPLE in "${ios_target_triples[@]}"; do
  echo "Building ios target $TARGET_TRIPLE"
  cargo build --target $TARGET_TRIPLE --release
done;


export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang
export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi19-clang
# Build IOS targets
for TARGET_TRIPLE in "${android_target_triples[@]}"; do
  echo "Building android target $TARGET_TRIPLE"
  cargo build --target $TARGET_TRIPLE --release
done;

echo "Done building for all targets"

