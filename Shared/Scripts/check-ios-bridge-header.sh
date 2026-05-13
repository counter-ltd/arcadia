#!/usr/bin/env bash
# Fail when extern "C" exports in Desktop/src/ios_lib.rs drift from ArcadiaBridge.h.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

rust_funcs="$(grep -E 'pub (unsafe )?extern "C" fn arcadia_ios_' Desktop/src/ios_lib.rs \
  | sed -E 's/.*fn (arcadia_ios_[a-z_]+)\(.*/\1/' | sort -u)"
hdr_funcs="$(grep -Eo 'arcadia_ios_[a-z_]+' Mobile/iOS/ArcadiaApp/ArcadiaBridge.h | sort -u)"

if [[ "$rust_funcs" != "$hdr_funcs" ]]; then
  echo "ArcadiaBridge.h out of sync with Desktop/src/ios_lib.rs"
  printf 'Rust (%s lines):\n%s\n\nHeader:\n%s\n' \
    "$(echo "$rust_funcs" | wc -l | tr -d ' ')" "$rust_funcs" "$hdr_funcs"
  exit 1
fi

echo "iOS C ABI: Rust exports match ArcadiaBridge.h"
