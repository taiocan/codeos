#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
WORK="$(mktemp -d)"; trap 'rm -rf "${WORK}"' EXIT
printf 'qualified binary\n' > "${WORK}/tool"
sha="$(sha256sum "${WORK}/tool" | awk '{print $1}')"
printf '{"schema":1,"binaries":{"demo":{"sha256":"%s","build_id":"source"}}}\n' "${sha}" > "${WORK}/manifest.json"
CODEOS_RELEASE_MANIFEST="${WORK}/manifest.json" bash "${HERE}/verify-qualified-binary.sh" demo "${WORK}/tool"
printf 'stale\n' >> "${WORK}/tool"
if CODEOS_RELEASE_MANIFEST="${WORK}/manifest.json" bash "${HERE}/verify-qualified-binary.sh" demo "${WORK}/tool" >/dev/null 2>&1; then
  echo 'stale binary was accepted' >&2; exit 1
fi
printf 'release identity tests: PASS\n'
