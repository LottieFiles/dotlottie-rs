#!/usr/bin/env bash

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

# Formatting
RED=$(tput setaf 1)
YELLOW=$(tput setaf 3)
GREEN=$(tput setaf 2)
WHITE=$(tput setaf 15)
NC=$(tput sgr0)

# Environment
EMSDK_VERSION=${EMSDK_VERSION:-latest}
UNIFFI_BINDGEN_CPP_VERSION=${UNIFFI_BINDGEN_CPP_VERSION:-"v0.6.0+v0.25.0"}

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

echo "Installing brew package(s) ..."
brew install android-ndk \
  cmake \
  nasm \
  meson \
  ninja \
  pkg-config \
  ktlint \
  swiftformat

rustup component add rust-src

echo
echo "Installing rust target(s) ..."
rustup target add aarch64-linux-android \
  armv7-linux-androideabi \
  x86_64-linux-android \
  aarch64-apple-darwin \
  x86_64-apple-darwin \
  aarch64-apple-ios \
  aarch64-apple-ios-sim \
  wasm32-unknown-emscripten

echo
echo "Install cargo dependencies"
cargo install uniffi-bindgen-cpp \
  --git https://github.com/NordSecurity/uniffi-bindgen-cpp \
  --tag "${UNIFFI_BINDGEN_CPP_VERSION}"


echo
echo "Setting up project ..."
make deps

echo
echo "Setting up emsdk"
cd "${SCRIPT_DIR}/deps/modules/emsdk" || die "Could not find Emscripten SDK under ${RED}deps/modules/emsdk${NC}!"
./emsdk install "${EMSDK_VERSION}"
./emsdk activate "${EMSDK_VERSION}"
cd "${SCRIPT_DIR}/deps/modules/emsdk/upstream/emscripten" || die "Could not find Emscripten under ${RED}deps/modules/emsdk/upstream/emscripten${NC}!"
npm install


echo
echo "Disabling unneeded webp features"
cd "${SCRIPT_DIR}/deps/modules/libwebp" || die "Could not find libwebp under ${RED}deps/modules/libwep${NC}!"
file_path="${SCRIPT_DIR}/deps/modules/libwebp/CMakeLists.txt"
# Use sed to replace the specified lines
sed -i -e 's/option(WEBP_BUILD_ANIM_UTILS "Build animation utilities." ON)/option(WEBP_BUILD_ANIM_UTILS "Build animation utilities." OFF)/' "$file_path"
sed -i -e 's/option(WEBP_BUILD_GIF2WEBP "Build the gif2webp conversion tool." ON)/option(WEBP_BUILD_GIF2WEBP "Build the gif2webp conversion tool." OFF)/' "$file_path"

echo
echo "${WHITE}Setup completed!${NC}"
echo "     1. If your ${GREEN}ANDROID_NDK_HOME${NC} was not installed to ${YELLOW}/opt/homebrew/share/android-ndk${NC}, export it's location in your shell profile"
echo "     2. You can now run ${YELLOW}make${NC} to see information on available build targets, or ${YELLOW}make all${NC} to build everything"
echo "     3. After building everything, all following calls to ${YELLOW}make all${NC} will be incremental, i.e. it will reuse things that have already been built"
