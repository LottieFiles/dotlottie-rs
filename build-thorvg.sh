#!/bin/bash

# Clone ThorVG repo if it doesn't exist
if [ ! -d "thorvg" ]; then
  git clone git@github.com:thorvg/thorvg.git
  git checkout hermet/ios
fi

ANDROID_NDK_HOME=/opt/homebrew/share/android-ndk

# List of target triples
targets=(
  "aarch64-apple-darwin"
  "x86_64-apple-ios"
  "aarch64-apple-ios-sim"
  "aarch64-apple-ios"
  "aarch64-linux-android"
  "armv7-linux-androideabi"
)

BASE_PATH=$(pwd)
# Path to the ThorVG source code
thorvg_path=$BASE_PATH/thorvg
cross_file=$BASE_PATH/cross-file.txt

rm -rf "$BASE_PATH/build"

# Navigating to ThorVG repo
cd $thorvg_path

echo "Building pwd: $(pwd)"

# Loop over each target
for target in "${targets[@]}"; do

  # Set up the cross-compiler environment variables
  # This will depend on your specific cross-compiler setup
  # For example, for Android targets, you might do something like this:
  if [[ $target == *"android"* ]]; then
    if [[ $target == "aarch64-linux-android" ]]; then
      target_name="aarch64-linux-android21"
    elif [[ $target == "armv7-linux-androideabi" ]]; then
      target_name="armv7a-linux-androideabi21"
    fi
    # aarch64-linux-android21-clang
    # armv7a-linux-androideabi21-clang
    export CC="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/$target_name-clang"
    export CXX="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/$target_name-clang++"
  fi

  # Creating crossfile
  # For iOS targets
  if [[ $target == *"ios"* ]]; then
    SYSROOT=$(xcrun --sdk iphoneos --show-sdk-path)
    SYSTEM="darwin"
    if [[ $target == *"x86_64"* ]]; then
      ARCH="x86_64"
      CPU_FAMILY="x86_64"
      CPU="x86_64"
    elif [[ $target == *"aarch64"* ]]; then
      ARCH="arm64"
      CPU_FAMILY="aarch64"
      CPU="aarch64"
    fi
  elif [[ $target == "aarch64-apple-darwin" ]]; then
    SYSROOT=$(xcrun --sdk macosx --show-sdk-path)
    SYSTEM="darwin"
    CPU_FAMILY="arm"
    CPU="aarch64"
  # For Android targets
  elif [[ $target == *"android"* ]]; then
    if [[ $target == *"aarch64"* ]]; then
      SYSROOT="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/aarch64-linux-android/21"
      CPP="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android21-clang++"
      AR="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
      STRIP="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
      CPU_FAMILY="aarch64"
      CPU="aarch64"
    elif [[ $target == *"armv7"* ]]; then
      SYSROOT="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/sysroot/usr/lib/arm-linux-androideabi/21"
      CPP="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi21-clang++"
      AR="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
      STRIP="/opt/homebrew/share/android-ndk/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-strip"
      CPU_FAMILY="arm"
      CPU="armv7"
    fi
  fi

  if [[ $target == *"ios"* ]]; then
        -e "s|CPU_FAMILY:|$CPU_FAMILY|g" \
        -e "s|CPU:|$CPU|g" $BASE_PATH/ios_cross.txt > "/tmp/.$target-cross.txt"
  else
    sed -e "s|SYSROOT:|$SYSROOT|g" \
        -e "s|CPP:|$CPP|g" \
        -e "s|AR:|$AR|g" \
        -e "s|STRIP:|$STRIP|g" \
        -e "s|CPU_FAMILY:|$CPU_FAMILY|g" \
        -e "s|CPU:|$CPU|g" $BASE_PATH/android-cross.txt > "/tmp/.$target-cross.txt"
  fi


  # Check the crossfile
  echo "File: /tmp/.$target-cross.txt"
  cat "/tmp/.$target-cross.txt"
  echo ""

  build_dir="$BASE_PATH/build/$target"
  mkdir -p $build_dir

  rm -rf builddir
  # Configure and build ThorVG for this target

  if [[ $target == *"android"* ]]; then
    # meson setup --prefix=/ -Ddefault_library=static -Dstatic=true -Dbindings=capi --cross-file "/tmp/.$target-cross.txt" builddir
    meson setup --backend=ninja builddir --prefix=/ -Dlog=true -Dloaders="all" -Ddefault_library=static -Dstatic=true -Dsavers="all" -Dbindings="capi" --cross-file "/tmp/.$target-cross.txt"
  else
    # meson setup --prefix=/ -Dbindings=capi --cross-file "/tmp/.$target-cross.txt" builddir
    meson setup --backend=ninja builddir --prefix=/ -Dlog=true -Dloaders="all" -Dstatic=true -Dsavers="all" -Dbindings="capi" --cross-file ./ios_cross.txt
  fi

  DESTDIR=$build_dir ninja -C builddir install

done
