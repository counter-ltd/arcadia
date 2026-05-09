#!/usr/bin/env bash
# shellcheck shell=bash
# Shared path helpers for the repo-root Builds/ tree:
#   Builds/Desktop/{OSX,Linux,Windows}/  — Desktop crate (cargo --target-dir)
#   Builds/Shared/                       — Shared workspace (arcadia-core, uniffi)
#   Builds/Mobile/iOS/DerivedData/       — Xcode -derivedDataPath

arcadia_desktop_target_rel() {
  case "$(uname -s)" in
    Darwin) echo "Builds/Desktop/OSX" ;;
    Linux) echo "Builds/Desktop/Linux" ;;
    MINGW*|MSYS*|CYGWIN*) echo "Builds/Desktop/Windows" ;;
    *) echo "Builds/Desktop/$(uname -s)" ;;
  esac
}

ARCADIA_SHARED_TARGET_REL="Builds/Shared"
ARCADIA_IOS_DERIVED_SIM_REL="Builds/Mobile/iOS/DerivedData/Simulator"
ARCADIA_IOS_DERIVED_DEVICE_REL="Builds/Mobile/iOS/DerivedData/Device"
