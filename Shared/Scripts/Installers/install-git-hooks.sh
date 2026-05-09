#!/usr/bin/env bash
# Point this repo at .githooks/ so optional hooks (e.g. iOS FFI refresh) run.
# Safe to run multiple times (idempotent).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
cd "${REPO_ROOT}"

HOOK_DIR="${REPO_ROOT}/.githooks"
if [[ ! -d "${HOOK_DIR}" ]]; then
	echo "Error: missing ${HOOK_DIR}" >&2
	exit 1
fi

chmod +x "${HOOK_DIR}/pre-commit" 2>/dev/null || true

git config core.hooksPath .githooks

echo "Configured git hooksPath -> .githooks for ${REPO_ROOT}"
echo "Active hooks:" && ls -1 "${HOOK_DIR}"
