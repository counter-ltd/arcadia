#!/usr/bin/env bash

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
# shellcheck source=/dev/null
source "${SCRIPT_DIR}/../Builds/build-paths.sh"
CARGO_BIN="${HOME}/.cargo/bin"

if [[ ":${PATH}:" != *":${CARGO_BIN}:"* ]]; then
  export PATH="${CARGO_BIN}:${PATH}"
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Error: cargo not found. Install Rust via rustup, then retry."
  echo "Hint: https://rustup.rs"
  exit 1
fi

run_arcadia() {
  local build_mode="$1"
  local features="$2"
  local project_root="${ROOT_DIR}/.."
  local manifest_path="Desktop/Cargo.toml"

  echo
  if [[ -n "${build_mode}" ]]; then
    echo "Running: cargo run --manifest-path ${manifest_path} ${build_mode} --features ${features}"
  else
    echo "Running: cargo run --manifest-path ${manifest_path} --features ${features}"
  fi
  (
    cd "${project_root}" || exit 1
    if [[ -n "${build_mode}" ]]; then
      cargo run --manifest-path "${manifest_path}" "${build_mode}" --features "${features}"
    else
      cargo run --manifest-path "${manifest_path}" --features "${features}"
    fi
  )
}

deploy_ios_device() {
  if ! command -v rg >/dev/null 2>&1; then
    echo "Error: rg (ripgrep) not found. Install via 'brew install ripgrep' and retry."
    return 1
  fi
  local configuration="$1"
  local project_path="${ROOT_DIR}/../Mobile/iOS/ArcadiaApp.xcodeproj"
  local shared_build_script="${ROOT_DIR}/Scripts/Builds/build-ios-framework.sh"
  local derived_data_path="${ROOT_DIR}/../${ARCADIA_IOS_DERIVED_DEVICE_REL}"
  local bundle_id="com.stacknode.arcadia"
  local preferred_device_name="${ARCADIA_IOS_DEVICE_NAME:-}"
  local destinations=""
  local device_udid=""
  local app_path=""

  if ! command -v xcodebuild >/dev/null 2>&1; then
    echo "Error: xcodebuild not found. Install Xcode command line tools."
    return 1
  fi

  if [[ ! -d "${project_path}" ]]; then
    echo "Error: iOS project not found at ${project_path}"
    return 1
  fi

  if [[ ! -f "${shared_build_script}" ]]; then
    echo "Error: shared iOS build script not found at ${shared_build_script}"
    return 1
  fi

  destinations="$(
    xcodebuild \
      -project "${project_path}" \
      -scheme "ArcadiaApp" \
      -showdestinations 2>/dev/null
  )"

  if [[ -n "${preferred_device_name}" ]]; then
    device_udid="$(
      printf "%s\n" "${destinations}" | \
        rg "platform:iOS, arch:arm64, id:[^,]+, name:${preferred_device_name}" | \
        rg -o "id:[^,]+" | \
        cut -d: -f2 | \
        head -n 1
    )"
  fi

  if [[ -z "${device_udid}" ]]; then
    device_udid="$(
      printf "%s\n" "${destinations}" | \
        rg "platform:iOS, arch:arm64, id:" | \
        rg -o "id:[^,]+" | \
        cut -d: -f2 | \
        head -n 1
    )"
  fi

  if [[ -z "${device_udid}" ]]; then
    echo "Error: no connected physical iOS device found."
    echo "Hint: set ARCADIA_IOS_DEVICE_NAME to your device name and retry."
    return 1
  fi

  echo
  echo "Building shared iOS artifacts..."
  bash "${shared_build_script}" || return 1

  echo
  echo "Running: xcodebuild -project Mobile/iOS/ArcadiaApp.xcodeproj -scheme ArcadiaApp -configuration ${configuration} -destination id=${device_udid} build"
  (
    cd "${ROOT_DIR}/.." || exit 1
    xcodebuild \
      -project "Mobile/iOS/ArcadiaApp.xcodeproj" \
      -scheme "ArcadiaApp" \
      -configuration "${configuration}" \
      -destination "id=${device_udid}" \
      -derivedDataPath "${derived_data_path}" \
      build
  )

  app_path="${derived_data_path}/Build/Products/${configuration}-iphoneos/ArcadiaApp.app"
  if [[ ! -d "${app_path}" ]]; then
    echo "Error: built app not found at ${app_path}"
    return 1
  fi

  if [[ "${ARCADIA_IOS_FORCE_UNINSTALL:-0}" == "1" ]]; then
    echo
    echo "Removing existing app (if installed)..."
    xcrun devicectl device uninstall app --device "${device_udid}" "${bundle_id}" >/dev/null 2>&1 || true
  fi

  echo
  echo "Installing app on device ${device_udid}..."
  xcrun devicectl device install app --device "${device_udid}" "${app_path}" || return 1

  echo "Launching app (${bundle_id}) on device ${device_udid}..."
  xcrun devicectl device process launch --device "${device_udid}" "${bundle_id}" || return 1
}

while true; do
  clear
  echo "=================================="
  echo " Arcadia Launcher"
  echo "=================================="
  echo "Choose option (type two keys, no Enter needed):"
  echo "  1A) Launch GUI Release"
  echo "  1B) Launch GUI Debug"
  echo "  2A) Launch Headless Release"
  echo "  2B) Launch Headless Debug"
  echo "  3A) Deploy iOS Release on Device"
  echo "  3B) Deploy iOS Debug on Device"
  echo "  0X) Exit"
  echo
  printf "Enter choice: "

  read -r -s -n 2 raw_choice
  echo
  choice="$(printf '%s' "$raw_choice" | tr '[:lower:]' '[:upper:]')"

  case "$choice" in
    1A)
      run_arcadia "--release" "gui"
      read -r -p "Press Enter to continue..."
      ;;
    1B)
      run_arcadia "" "gui"
      read -r -p "Press Enter to continue..."
      ;;
    2A)
      run_arcadia "--release" "headless"
      read -r -p "Press Enter to continue..."
      ;;
    2B)
      run_arcadia "" "headless"
      read -r -p "Press Enter to continue..."
      ;;
    3A)
      deploy_ios_device "Release"
      read -r -p "Press Enter to continue..."
      ;;
    3B)
      deploy_ios_device "Debug"
      read -r -p "Press Enter to continue..."
      ;;
    0X|00|0A|0B)
      echo "Goodbye."
      exit 0
      ;;
    *)
      echo
      echo "Invalid option. Use 1A, 1B, 2A, 2B, 3A, 3B, or 0X."
      read -r -p "Press Enter to continue..."
      ;;
  esac
done
