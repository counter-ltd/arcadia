#!/usr/bin/env bash
# shellcheck shell=bash
# Shared path helpers for the repo-root Builds/ tree:
#   Builds/workspace/          — unified Cargo target-dir (see /.cargo/config.toml)
#   Builds/Mobile/iOS/DerivedData/ — Xcode -derivedDataPath

# Relative to repository root. Matches [build] target-dir in /.cargo/config.toml.
ARCADIA_CARGO_TARGET_REL="Builds/workspace"

# Deprecated alias: all platforms share Builds/workspace via Cargo config.
arcadia_desktop_target_rel() {
  printf '%s\n' "${ARCADIA_CARGO_TARGET_REL}"
}

# Legacy name kept for scripts that still source ARCADIA_SHARED_TARGET_REL.
ARCADIA_SHARED_TARGET_REL="${ARCADIA_CARGO_TARGET_REL}"
ARCADIA_IOS_DERIVED_SIM_REL="Builds/Mobile/iOS/DerivedData/Simulator"
ARCADIA_IOS_DERIVED_DEVICE_REL="Builds/Mobile/iOS/DerivedData/Device"
