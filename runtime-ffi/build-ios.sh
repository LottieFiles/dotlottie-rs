#!/bin/bash
# Genreate Bindings for Swift
echo "Gnerating bindings for Swift"
cargo +nightly run --target aarch64-apple-ios --features=uniffi/cli --bin uniffi-bindgen generate src/dlplayer.udl  --language swift --out-dir uniffi-bindings

ios_target_triples=(
  "x86_64-apple-ios"
  "aarch64-apple-ios-sim"
  "aarch64-apple-ios"
)

#Build IOS targets
for TARGET_TRIPLE in "${ios_target_triples[@]}"; do
  echo "Building ios target $TARGET_TRIPLE"
  cargo build --target $TARGET_TRIPLE --release
done;

echo "Done building for all targets"

