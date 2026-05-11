#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"

cargo build \
    -p arcadia \
    --manifest-path "$REPO_ROOT/Desktop/Cargo.toml" \
    --target aarch64-apple-ios \
    --features ios-gui \
    --lib \
    --release

echo "Built: $REPO_ROOT/Desktop/target/aarch64-apple-ios/release/libarcadia_ios.a"
