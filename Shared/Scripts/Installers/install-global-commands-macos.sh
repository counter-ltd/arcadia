#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
BIN_DIR="${HOME}/.local/bin"
CARGO_BIN="${HOME}/.cargo/bin"

mkdir -p "${BIN_DIR}"

cat > "${BIN_DIR}/arcadia" <<EOF
#!/usr/bin/env bash
set -euo pipefail

export PATH="${CARGO_BIN}:\${PATH}"
PROJECT_ROOT="${ROOT_DIR}/.."
DESKTOP_MANIFEST="Desktop/Cargo.toml"
CONFIG_ROOT="\${HOME}/Arcadia/Configuration"

open_config_file() {
  local config_name="\$1"
  local config_file="\${config_name}"
  if [[ "\${config_file}" != *.toml ]]; then
    config_file="\${config_file}.toml"
  fi

  mkdir -p "\${CONFIG_ROOT}"
  touch "\${CONFIG_ROOT}/\${config_file}"
  open "\${CONFIG_ROOT}/\${config_file}"
}

if [[ "\${1:-}" == "configuration" ]]; then
  if [[ -z "\${2:-}" ]]; then
    echo "Usage: arcadia configuration <config file>"
    exit 1
  fi
  open_config_file "\${2}"
  exit 0
fi

cd "\${PROJECT_ROOT}"
cargo build --manifest-path "\${DESKTOP_MANIFEST}" --target-dir Builds/Desktop/OSX --no-default-features --features headless >/dev/null
exec "\${PROJECT_ROOT}/Builds/Desktop/OSX/debug/arcadia" "\$@"
EOF

cat > "${BIN_DIR}/arcadia-gui" <<EOF
#!/usr/bin/env bash
set -euo pipefail

export PATH="${CARGO_BIN}:\${PATH}"
PROJECT_ROOT="${ROOT_DIR}/.."
DESKTOP_MANIFEST="Desktop/Cargo.toml"
CONFIG_ROOT="\${HOME}/Arcadia/Configuration"

open_config_file() {
  local config_name="\$1"
  local config_file="\${config_name}"
  if [[ "\${config_file}" != *.toml ]]; then
    config_file="\${config_file}.toml"
  fi

  mkdir -p "\${CONFIG_ROOT}"
  touch "\${CONFIG_ROOT}/\${config_file}"
  open "\${CONFIG_ROOT}/\${config_file}"
}

if [[ "\${1:-}" == "configuration" ]]; then
  if [[ -z "\${2:-}" ]]; then
    echo "Usage: arcadia-gui configuration <config file>"
    exit 1
  fi
  open_config_file "\${2}"
  exit 0
fi

cd "\${PROJECT_ROOT}"
cargo build --manifest-path "\${DESKTOP_MANIFEST}" --target-dir Builds/Desktop/OSX --no-default-features --features gui >/dev/null
exec "\${PROJECT_ROOT}/Builds/Desktop/OSX/debug/arcadia" "\$@"
EOF

cat > "${BIN_DIR}/arcadia-ios" <<EOF
#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="${ROOT_DIR}/.."
PROJECT_PATH="Mobile/iOS/ArcadiaApp.xcodeproj"
SHARED_BUILD_SCRIPT="${ROOT_DIR}/Scripts/Builds/build-ios-framework.sh"
DERIVED_DATA_PATH="\${PROJECT_ROOT}/Builds/Mobile/iOS/DerivedData/Device"
BUNDLE_ID="com.stacknode.arcadia"
PREFERRED_DEVICE_NAME="\${ARCADIA_IOS_DEVICE_NAME:-}"
DESTINATIONS=""
DEVICE_UDID=""

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "Error: xcodebuild not found. Install Xcode command line tools."
  exit 1
fi

if [[ ! -d "\${PROJECT_ROOT}/\${PROJECT_PATH}" ]]; then
  echo "Error: iOS project not found at \${PROJECT_ROOT}/\${PROJECT_PATH}"
  exit 1
fi

if [[ ! -f "\${SHARED_BUILD_SCRIPT}" ]]; then
  echo "Error: shared iOS build script not found at \${SHARED_BUILD_SCRIPT}"
  exit 1
fi

cd "\${PROJECT_ROOT}" || exit 1

if ! command -v rg >/dev/null 2>&1; then
  echo "Error: rg (ripgrep) not found. Install via 'brew install ripgrep' and retry."
  exit 1
fi

DESTINATIONS="\$(
  xcodebuild \
    -project "\${PROJECT_PATH}" \
    -scheme "ArcadiaApp" \
    -showdestinations 2>/dev/null
)"

if [[ -n "\${PREFERRED_DEVICE_NAME}" ]]; then
  DEVICE_UDID="\$(
    printf "%s\n" "\${DESTINATIONS}" | \
      rg "platform:iOS, arch:arm64, id:[^,]+, name:\${PREFERRED_DEVICE_NAME}" | \
      rg -o "id:[^,]+" | \
      cut -d: -f2 | \
      head -n 1
  )"
fi

if [[ -z "\${DEVICE_UDID}" ]]; then
  DEVICE_UDID="\$(
    printf "%s\n" "\${DESTINATIONS}" | \
      rg "platform:iOS, arch:arm64, id:" | \
      rg -o "id:[^,]+" | \
      cut -d: -f2 | \
      head -n 1
  )"
fi

if [[ -z "\${DEVICE_UDID}" ]]; then
  echo "Error: no connected physical iOS device found."
  echo "Hint: set ARCADIA_IOS_DEVICE_NAME to your device name and retry."
  exit 1
fi

echo "Building shared iOS artifacts..."
bash "\${SHARED_BUILD_SCRIPT}"

xcodebuild \
  -project "\${PROJECT_PATH}" \
  -scheme "ArcadiaApp" \
  -configuration "Release" \
  -destination "id=\${DEVICE_UDID}" \
  -derivedDataPath "\${DERIVED_DATA_PATH}" \
  build

APP_PATH="\${DERIVED_DATA_PATH}/Build/Products/Release-iphoneos/ArcadiaApp.app"
if [[ ! -d "\${APP_PATH}" ]]; then
  echo "Error: built app not found at \${APP_PATH}"
  exit 1
fi

echo "Installing app on device \${DEVICE_UDID}..."
if [[ "\${ARCADIA_IOS_FORCE_UNINSTALL:-0}" == "1" ]]; then
  xcrun devicectl device uninstall app --device "\${DEVICE_UDID}" "\${BUNDLE_ID}" >/dev/null 2>&1 || true
fi
xcrun devicectl device install app --device "\${DEVICE_UDID}" "\${APP_PATH}"

echo "Launching app (\${BUNDLE_ID}) on device \${DEVICE_UDID}..."
exec xcrun devicectl device process launch --device "\${DEVICE_UDID}" "\${BUNDLE_ID}"
EOF

chmod +x "${BIN_DIR}/arcadia" "${BIN_DIR}/arcadia-gui" "${BIN_DIR}/arcadia-ios"

if [[ ":${PATH}:" != *":${BIN_DIR}:"* ]]; then
  echo "Installed commands, but ${BIN_DIR} is not currently on PATH in this shell."
  echo "Add this to your shell profile (~/.zshrc):"
  echo "  export PATH=\"${BIN_DIR}:\$PATH\""
else
  echo "Installed commands to ${BIN_DIR}"
fi

echo "Use:"
echo "  arcadia <args>"
echo "  arcadia-gui <args>"
echo "  arcadia-ios"
