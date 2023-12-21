#!/usr/bin/env bash

SCRIPT_DIR="$(dirname "$(readlink -f "${BASH_SOURCE[0]}")")"

UNIFFI_BINDINGS_CPP_DIR="${UNIFFI_BINDINGS_CPP_DIR:-${SCRIPT_DIR}/uniffi-bindings/cpp}"
UNIFFI_UDL="${UNIFFI_UDL:-${SCRIPT_DIR}/src/dotlottie_player.udl}"
EMSCRIPTEN_BINDINGS_CPP="${EMSCRIPTEN_BINDINGS_CPP:-${UNIFFI_BINDINGS_CPP_DIR}/emscripten_bindings.cpp}"

GENERATE_EMSCRIPTEN_BINDINGS=$(cat << 'EOF'
BEGIN {
  print "#include <emscripten/bind.h>"
  print "using namespace emscripten;"
  print
}

/^namespace / { namespace = $2 }

function camelCase(str) {
  while (match(str, /_./)) {
    char = substr(str, RSTART + 1, 1)
    str = substr(str, 1, RSTART - 1) toupper(char) substr(str, RSTART + 2)
  }
  return str
}

/^interface / {
  interface = $2
  printf "EMSCRIPTEN_BINDINGS(%s) {\n", interface
  printf "  class_<%s::%s>(\"%s\")", namespace, interface, interface
}
/^[[:space:]]+[a-z]+/ {
  if (length(interface) > 0) {
    if (index($1, "constructor()")) {
      printf "\n    .constructor(&%s::%s::init)", namespace, interface
    } else {
      split($2, identifier, "(")
      printf "\n    .function(\"%s\", &%s::%s::%s)",
             camelCase(identifier[1]), namespace, interface, identifier[1]
    }
  }
}
/^}/ {
  if (length(interface) > 0) {
    interface = ""
    print ";\n}"
  }
}
EOF
)

# Add header includes
find "${UNIFFI_BINDINGS_CPP_DIR}" \
  -name "*.hpp" \
  -and -not -name "*scaffolding*" \
  -exec sh -c 'file="$1"; printf "#include \"%s\"\n" $(basename "${file}")' shell {} \; > "${EMSCRIPTEN_BINDINGS_CPP}"

# Add emscripten bindings
awk "${GENERATE_EMSCRIPTEN_BINDINGS}" "${UNIFFI_UDL}" >> "${EMSCRIPTEN_BINDINGS_CPP}"
