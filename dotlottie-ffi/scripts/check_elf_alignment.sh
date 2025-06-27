#!/bin/bash
progname="${0##*/}"
progname="${progname%.sh}"

# usage: check_elf_alignment.sh [path to *.so files|path to *.apk]

cleanup_trap() {
  if [ -n "${tmp}" -a -d "${tmp}" ]; then
    rm -rf ${tmp}
  fi
  exit $1
}

usage() {
  echo "Host side script to check the ELF alignment of shared libraries."
  echo "Shared libraries are reported ALIGNED when their ELF regions are"
  echo "16 KB or 64 KB aligned. Otherwise they are reported as UNALIGNED."
  echo
  echo "Usage: ${progname} [input-path|input-APK|input-APEX]"
}

if [ ${#} -ne 1 ]; then
  usage
  exit
fi

case ${1} in
  --help | -h | -\?)
    usage
    exit
    ;;

  *)
    dir="${1}"
    ;;
esac

if ! [ -f "${dir}" -o -d "${dir}" ]; then
  echo "Invalid file: ${dir}" >&2
  exit 1
fi

if [[ "${dir}" == *.apk ]]; then
  trap 'cleanup_trap' EXIT

  echo
  echo "Recursively analyzing $dir"
  echo

  if { zipalign --help 2>&1 | grep -q "\-P <pagesize_kb>"; }; then
    echo "=== APK zip-alignment ==="
    zipalign -v -c -P 16 4 "${dir}" | egrep 'lib/arm64-v8a|lib/x86_64|Verification'
    echo "========================="
  else
    echo "NOTICE: Zip alignment check requires build-tools version 35.0.0-rc3 or higher."
    echo "  You can install the latest build-tools by running the below command"
    echo "  and updating your \$PATH:"
    echo
    echo "    sdkmanager \"build-tools;35.0.0-rc3\""
  fi

  dir_filename=$(basename "${dir}")
  tmp=$(mktemp -d -t "${dir_filename%.apk}_out_XXXXX")
  unzip "${dir}" lib/* -d "${tmp}" >/dev/null 2>&1
  dir="${tmp}"
fi

if [[ "${dir}" == *.apex ]]; then
  trap 'cleanup_trap' EXIT

  echo
  echo "Recursively analyzing $dir"
  echo

  dir_filename=$(basename "${dir}")
  tmp=$(mktemp -d -t "${dir_filename%.apex}_out_XXXXX")
  deapexer extract "${dir}" "${tmp}" || { echo "Failed to deapex." && exit 1; }
  dir="${tmp}"
fi

RED="\033[31m"
GREEN="\033[32m"
YELLOW="\033[33m"
ENDCOLOR="\033[0m"

# Check if we're in a terminal that supports colors
if [ -t 1 ]; then
  USE_COLORS=true
else
  USE_COLORS=false
fi

# Function to print with color support
print_colored() {
  local color="$1"
  local message="$2"
  if [ "$USE_COLORS" = true ]; then
    echo -e "${color}${message}${ENDCOLOR}"
  else
    echo "$message"
  fi
}

unaligned_libs=()
unaligned_64bit_libs=()

echo
echo "=== ELF Alignment Check ==="

matches="$(find "${dir}" -type f)"
IFS=$'\n'
for match in $matches; do
  # We could recursively call this script or rewrite it to though.
  [[ "${match}" == *".apk" ]] && echo "WARNING: doesn't recursively inspect .apk file: ${match}"
  [[ "${match}" == *".apex" ]] && echo "WARNING: doesn't recursively inspect .apex file: ${match}"

  [[ $(file "${match}") == *"ELF"* ]] || continue

  res="$(objdump -p "${match}" | grep LOAD | awk '{ print $NF }' | head -1)"
  lib_name=$(basename "${match}")
  lib_path=$(dirname "${match}" | sed 's|.*/jniLibs/||')
  
  if [[ $res =~ 2\*\*(1[4-9]|[2-9][0-9]|[1-9][0-9]{2,}) ]]; then
    print_colored "$GREEN" "✓ ${lib_path}/${lib_name}: ALIGNED ($res)"
  else
    print_colored "$RED" "✗ ${lib_path}/${lib_name}: UNALIGNED ($res)"
    unaligned_libs+=("${match}")
    
    # Check if this is a 64-bit architecture that requires alignment
    if [[ "${match}" == *"arm64-v8a"* ]] || [[ "${match}" == *"x86_64"* ]]; then
      unaligned_64bit_libs+=("${match}")
    fi
  fi
done

echo "============================"

if [ ${#unaligned_64bit_libs[@]} -gt 0 ]; then
  echo
  print_colored "$RED" "❌ FATAL ERROR: Found ${#unaligned_64bit_libs[@]} unaligned 64-bit library(ies)"
  print_colored "$RED" "64-bit libraries (arm64-v8a/x86_64) MUST be 16KB aligned for Android."
  echo
  print_colored "$RED" "Unaligned 64-bit libraries:"
  for lib in "${unaligned_64bit_libs[@]}"; do
    lib_name=$(basename "${lib}")
    lib_path=$(dirname "${lib}" | sed 's|.*/jniLibs/||')
    print_colored "$RED" "  • ${lib_path}/${lib_name}"
  done
  echo
  print_colored "$RED" "Build FAILED due to unaligned 64-bit libraries."
  echo "============================"
  exit 1
elif [ ${#unaligned_libs[@]} -gt 0 ]; then
  echo
  print_colored "$YELLOW" "⚠️  Found ${#unaligned_libs[@]} unaligned 32-bit library(ies)"
  print_colored "$YELLOW" "32-bit libraries don't require alignment, but 64-bit libraries do."
  echo
  for lib in "${unaligned_libs[@]}"; do
    lib_name=$(basename "${lib}")
    lib_path=$(dirname "${lib}" | sed 's|.*/jniLibs/||')
    print_colored "$YELLOW" "  • ${lib_path}/${lib_name}"
  done
  echo
  print_colored "$GREEN" "✅ All 64-bit libraries are properly aligned."
  echo "============================"
else
  echo
  print_colored "$GREEN" "✅ All libraries are properly aligned!"
  echo "============================"
fi