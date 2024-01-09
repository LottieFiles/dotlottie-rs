#!/usr/bin/env bash

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

UNIFFI_BINDINGS_CPP_DIR="${UNIFFI_BINDINGS_CPP_DIR:-${SCRIPT_DIR}/uniffi-bindings/cpp}"
EMSCRIPTEN_BINDINGS_CPP="${EMSCRIPTEN_BINDINGS_CPP:-${SCRIPT_DIR}/emscripten_bindings.cpp}"

# Copy the emscripten bindings to the uniffi cpp bindings directory
cp "${SCRIPT_DIR}/emscripten_bindings.cpp" "${UNIFFI_BINDINGS_CPP_DIR}/"
