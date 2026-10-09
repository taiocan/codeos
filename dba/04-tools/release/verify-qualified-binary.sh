#!/usr/bin/env bash
# Verify a release executable against the active installation manifest without requiring sources.
set -euo pipefail
[[ $# -eq 2 ]] || { echo 'usage: verify-qualified-binary.sh <binary-name> <binary-path>' >&2; exit 2; }
NAME="$1"
BINARY="$2"
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "${HERE}/../../.." && pwd -P)"
MANIFEST="${CODEOS_RELEASE_MANIFEST:-${ROOT}/maintenance/release-manifest.json}"
[[ -f "${MANIFEST}" ]] || { echo "release manifest missing: ${MANIFEST}" >&2; exit 1; }
expected="$(jq -er --arg name "${NAME}" '.binaries[$name].sha256' "${MANIFEST}")"
actual="$(sha256sum "${BINARY}" | awk '{print $1}')"
[[ "${actual}" == "${expected}" ]] || {
  echo "${NAME} does not match active CodeOS release manifest" >&2
  exit 1
}
