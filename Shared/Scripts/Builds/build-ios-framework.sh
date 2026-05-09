#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SOURCE_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"
REPO_ROOT="$(cd "${SOURCE_DIR}/.." && pwd)"
CARGO_TARGET_DIR="${REPO_ROOT}/Builds/Shared"
OUT_DIR="${SOURCE_DIR}/../Mobile/iOS/ArcadiaCore"
LIB_NAME="libarcadia_core.a"
DEVICE_TARGET="aarch64-apple-ios"
SIM_TARGET="aarch64-apple-ios-sim"

DEVICE_LIB="${CARGO_TARGET_DIR}/${DEVICE_TARGET}/release/${LIB_NAME}"
SIM_LIB="${CARGO_TARGET_DIR}/${SIM_TARGET}/release/${LIB_NAME}"
XCFRAMEWORK_DIR="${OUT_DIR}/ArcadiaCore.xcframework"
BINDGEN_OUT="${OUT_DIR}/Generated"
DEVICE_DIR=""
SIM_DIR=""
trap '[[ -n "${DEVICE_DIR}" ]] && rm -rf "${DEVICE_DIR}"; [[ -n "${SIM_DIR}" ]] && rm -rf "${SIM_DIR}"' EXIT

# ── 0. Install targets ────────────────────────────────────────────────────────
echo "==> Installing Rust targets"
rustup target add "${DEVICE_TARGET}" "${SIM_TARGET}"

# ── 1. Build for device + simulator ──────────────────────────────────────────
echo "==> Building for ${DEVICE_TARGET}"
(cd "${SOURCE_DIR}" && cargo build -p arcadia-core --release --target "${DEVICE_TARGET}" --target-dir "${CARGO_TARGET_DIR}")

echo "==> Building for ${SIM_TARGET}"
(cd "${SOURCE_DIR}" && cargo build -p arcadia-core --release --target "${SIM_TARGET}" --target-dir "${CARGO_TARGET_DIR}")

# ── 2. Generate Swift bindings ────────────────────────────────────────────────
echo "==> Generating Swift bindings"
mkdir -p "${BINDGEN_OUT}"
(cd "${SOURCE_DIR}" && cargo run --target-dir "${CARGO_TARGET_DIR}" -p uniffi-bindgen -- \
    generate \
    --library "${DEVICE_LIB}" \
    --language swift \
    --out-dir "${BINDGEN_OUT}")

# ── 3. Build xcframework ──────────────────────────────────────────────────────
echo "==> Creating xcframework"
rm -rf "${XCFRAMEWORK_DIR}"
mkdir -p "${OUT_DIR}"

DEVICE_DIR="${OUT_DIR}/_device"
SIM_DIR="${OUT_DIR}/_sim"
rm -rf "${DEVICE_DIR}" "${SIM_DIR}"

for DIR in "${DEVICE_DIR}" "${SIM_DIR}"; do
    mkdir -p "${DIR}/Headers" "${DIR}/Modules"
done

HEADER=""
MODULEMAP=""
for candidate in \
    "${BINDGEN_OUT}/arcadia_coreFFI.h" \
    "${BINDGEN_OUT}/arcadia_coreCFFI.h"; do
    if [[ -f "${candidate}" ]]; then
        HEADER="${candidate}"
        break
    fi
done

for candidate in \
    "${BINDGEN_OUT}/arcadia_coreFFI.modulemap" \
    "${BINDGEN_OUT}/arcadia_coreCFFI.modulemap"; do
    if [[ -f "${candidate}" ]]; then
        MODULEMAP="${candidate}"
        break
    fi
done

if [[ -z "${HEADER}" || -z "${MODULEMAP}" ]]; then
    echo "Error: expected UniFFI header/modulemap not found in ${BINDGEN_OUT}"
    echo "       looked for arcadia_coreFFI.{h,modulemap} and arcadia_coreCFFI.{h,modulemap}"
    exit 1
fi

for DIR in "${DEVICE_DIR}" "${SIM_DIR}"; do
    cp "${HEADER}"    "${DIR}/Headers/$(basename "${HEADER}")"
    cp "${MODULEMAP}" "${DIR}/Modules/module.modulemap"
done

cp "${DEVICE_LIB}" "${DEVICE_DIR}/libarcadia_core.a"
cp "${SIM_LIB}"    "${SIM_DIR}/libarcadia_core.a"

xcodebuild -create-xcframework \
    -library "${DEVICE_DIR}/libarcadia_core.a" \
    -headers "${DEVICE_DIR}/Headers"           \
    -library "${SIM_DIR}/libarcadia_core.a"    \
    -headers "${SIM_DIR}/Headers"              \
    -output  "${XCFRAMEWORK_DIR}"

rm -rf "${DEVICE_DIR}" "${SIM_DIR}"

echo ""
echo "Swift bindings : ${BINDGEN_OUT}/arcadia_core.swift"
echo "xcframework    : ${XCFRAMEWORK_DIR}"
echo "Done."
