#!/usr/bin/env bash

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

# Check if we're in a terminal that supports colors
if [ -t 1 ]; then
    RED=$(tput setaf 1 2>/dev/null || echo '')
    YELLOW=$(tput setaf 3 2>/dev/null || echo '')
    GREEN=$(tput setaf 2 2>/dev/null || echo '')
    WHITE=$(tput setaf 15 2>/dev/null || echo '')
    NC=$(tput sgr0 2>/dev/null || echo '')
else
    RED=''
    YELLOW=''
    GREEN=''
    WHITE=''
    NC=''
fi

EMSDK_VERSION=${EMSDK_VERSION:-latest}
UNIFFI_BINDGEN_CPP_VERSION=${UNIFFI_BINDGEN_CPP_VERSION:-"v0.6.3+v0.25.0"}

die() { printf %s "${@+$@$'\n'}" 1>&2 ; exit 1; }

check_for() {
    local -r app=$1
    local install_cmd=$2
    
    echo "Checking for ${GREEN}${app}${NC} ..."
    if ! which "${app}" &>/dev/null; then
        echo "${RED}=>${NC} Could not find ${app}, installing..."
        if [ -z "$install_cmd" ]; then
            echo "${RED}No installation command provided for ${app}${NC}"
            exit 1
        fi
        eval "$install_cmd" || die "Failed to install ${app}"
    fi
}

if [ -f /etc/debian_version ]; then
    PKG_MANAGER="apt-get"
    PKG_UPDATE="sudo apt-get update"
    INSTALL_CMD="sudo apt-get install -y"
elif [ -f /etc/fedora-release ]; then
    PKG_MANAGER="dnf"
    PKG_UPDATE="sudo dnf check-update"
    INSTALL_CMD="sudo dnf install -y"
else
    echo "${RED}Unsupported Linux distribution${NC}"
    exit 1
fi

echo "Updating package manager..."
eval "$PKG_UPDATE"

echo "Installing basic dependencies..."
$INSTALL_CMD \
    build-essential \
    cmake \
    pkg-config \
    ninja-build \
    python3-pip \
    nasm \
    git \
    curl \
    wget

echo "Installing Meson 1.6.0..."
pip3 install 'meson==1.6.0'

check_for rustup "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

source "$HOME/.cargo/env"

echo
echo "Installing rust target ..."
rustup target add wasm32-unknown-emscripten

echo
echo "Setting up project ..."
make deps

echo "Installing nightly toolchain"
rustup install nightly
rustup component add rust-src --toolchain nightly
rustup target add wasm32-unknown-emscripten --toolchain nightly

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
source ./emsdk_env.sh
cd "${SCRIPT_DIR}/deps/modules/emsdk/upstream/emscripten" || die "Could not find Emscripten under ${RED}deps/modules/emsdk/upstream/emscripten${NC}!"
npm install
cd "${SCRIPT_DIR}" || die "Could not find project root directory!"

echo
echo "Disabling unneeded webp features"
cd "${SCRIPT_DIR}/deps/modules/libwebp" || die "Could not find libwebp under ${RED}deps/modules/libwep${NC}!"
file_path="${SCRIPT_DIR}/deps/modules/libwebp/CMakeLists.txt"
sed -i 's/option(WEBP_BUILD_ANIM_UTILS "Build animation utilities." ON)/option(WEBP_BUILD_ANIM_UTILS "Build animation utilities." OFF)/' "$file_path"
sed -i 's/option(WEBP_BUILD_GIF2WEBP "Build the gif2webp conversion tool." ON)/option(WEBP_BUILD_GIF2WEBP "Build the gif2webp conversion tool." OFF)/' "$file_path"
