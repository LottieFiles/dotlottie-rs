#!/usr/bin/env bash

TARGET=${TARGET:-all}

echo "Target: ${TARGET}"

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

# Formatting
RED=$(tput setaf 1)
YELLOW=$(tput setaf 3)
GREEN=$(tput setaf 2)
WHITE=$(tput setaf 15)
NC=$(tput sgr0)

# Environment
EMSDK_VERSION=${EMSDK_VERSION:-latest}
UNIFFI_BINDGEN_CPP_VERSION=${UNIFFI_BINDGEN_CPP_VERSION:-"v0.7.2+v0.28.3"}

die() { printf %s "${@+$@$'\n'}" 1>&2 ; exit 1; }

check_for() {
  local -r app=$1

  local install_url=$2
  if [[ -n "${install_url}" ]]; then
    install_url=", ${YELLOW}please install it first${NC}: ${install_url}"
  fi

  echo "Checking for ${GREEN}${app}${NC} ..."
  if ! which "${app}" &>/dev/null; then
    echo "${RED}=>${NC} Could not find ${app}${install_url}"

    local -r instructions=$3
    if [[ -n "$instructions" ]]; then
      echo "${instructions}"
    fi

    exit 1
  fi
}

check_for xcodebuild
check_for brew "https://brew.sh"
check_for rustup "https://rustup.rs" "\
     1. Choose the ${GREEN}default${NC} installation option
     2. Either logout & login after the installation, or execute: ${YELLOW}source \"\$HOME/.cargo/env\""

echo "Installing v1.6.0 of Meson..."
curl https://raw.githubusercontent.com/Homebrew/homebrew-core/8ae7edfa2242b04dc01562dcb4536df60191593c/Formula/m/meson.rb > meson.rb
brew install meson.rb

echo "Installing brew package(s) ..."
brew install cmake \
  nasm \
  ninja \
  pkg-config \
  conan \
  ktlint \
  swiftformat

if [[ "${TARGET}" == "android" || "${TARGET}" == "all" ]]; then
  brew install android-ndk
fi

echo "Checking if Rust nightly is already installed..."
if ! rustup toolchain list | grep -q nightly; then
  echo "Installing Rust nightly..."
  rustup toolchain install nightly
  rustup component add rust-src --toolchain nightly
else
  echo "Rust nightly is already installed. Skipping installation."
fi

rustup component add rust-src --toolchain stable

echo
echo "Installing rust target(s) ..."
case "${TARGET}" in
  android)
    rustup target add aarch64-linux-android \
      armv7-linux-androideabi \
      x86_64-linux-android \
      i686-linux-android
    ;;
  apple)
    rustup target add aarch64-apple-darwin \
      x86_64-apple-darwin \
      aarch64-apple-ios \
      x86_64-apple-ios \
      aarch64-apple-ios-sim \
      x86_64-apple-ios-macabi \
      aarch64-apple-ios-macabi
    ;;
  wasm)
    rustup target add wasm32-unknown-emscripten
    ;;
  all)
    rustup target add aarch64-linux-android \
      armv7-linux-androideabi \
      x86_64-linux-android \
      i686-linux-android \
      aarch64-apple-darwin \
      x86_64-apple-darwin \
      aarch64-apple-ios \
      x86_64-apple-ios \
      aarch64-apple-ios-sim \
      wasm32-unknown-emscripten \
      x86_64-apple-ios-macabi \
      aarch64-apple-ios-macabi
    ;;
  *)
    echo "${RED}Invalid target specified: ${TARGET}${NC}"
    exit 1
    ;;
esac

echo
echo "Setting up project ..."
make deps

if [[ "${TARGET}" == "wasm" || "${TARGET}" == "all" ]]; then
  echo
  echo "Installing cargo dependencies"
  cargo install uniffi-bindgen-cpp \
    --git https://github.com/NordSecurity/uniffi-bindgen-cpp \
    --tag "${UNIFFI_BINDGEN_CPP_VERSION}"

  echo
  echo "Setting up emsdk"
  cd "${SCRIPT_DIR}/deps/modules/emsdk" || die "Could not find Emscripten SDK under ${RED}deps/modules/emsdk${NC}!"
  ./emsdk install "${EMSDK_VERSION}"
  ./emsdk activate "${EMSDK_VERSION}"
  cd "${SCRIPT_DIR}/deps/modules/emsdk/upstream/emscripten" || die "Could not find Emscripten under ${RED}deps/modules/emsdk/upstream/emscripten${NC}!"
  npm install
fi

echo
echo "${WHITE}Setup completed!${NC}"
echo "     1. If your ${GREEN}ANDROID_NDK_HOME${NC} was not installed to ${YELLOW}/opt/homebrew/share/android-ndk${NC}, export it's location in your shell profile"
echo "     2. You can now run ${YELLOW}make${NC} to see information on available build targets, or ${YELLOW}make all${NC} to build everything"
echo "     3. After building everything, all following calls to ${YELLOW}make all${NC} will be incremental, i.e. it will reuse things that have already been built"
echo "     4. If you don't define ${GREEN}APPLE_XCODE_APP_NAME${NC} under the format ${YELLOW}Xcode_[version].app${NC}, it will default to Xcode_13.3.1.app which might not be present on your system in: /Applications/. To use the latest version of Xcode on your system, set to: \"Xcode.app\"."
echo "     5. If you don't define ${GREEN}APPLE_MACOSX_SDK${NC} under the format ${YELLOW}MacOSX[version]${NC}, it will default to MacOSX12.3 which might not be present on your system under: /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/. To use the latest version of Xcode on your system, set to: \"MacOSX\"."
