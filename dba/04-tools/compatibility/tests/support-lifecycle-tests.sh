#!/usr/bin/env bash
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
WORK="$(mktemp -d)"; trap 'rm -rf "${WORK}"' EXIT
printf '{"configurations":{"DBA-1":"supported"},"lifecycle":{}}\n' > "${WORK}/supported.json"
printf '{"configurations":{"DBA-1":"deprecated"},"lifecycle":{"DBA-1":{"deprecated_in":"R2","migration_target":"DBA-2"}}}\n' > "${WORK}/deprecated.json"
printf '{"configurations":{"DBA-1":"historical-only"},"lifecycle":{"DBA-1":{"deprecated_in":"R2","migration_target":"DBA-2","historical_in":"R3"}}}\n' > "${WORK}/historical.json"
bash "${HERE}/validate-support-transition.sh" "${WORK}/supported.json" "${WORK}/deprecated.json" R2 >/dev/null
bash "${HERE}/validate-support-transition.sh" "${WORK}/deprecated.json" "${WORK}/historical.json" R3 >/dev/null
if bash "${HERE}/validate-support-transition.sh" "${WORK}/supported.json" "${WORK}/historical.json" R3 >/dev/null 2>&1; then
  echo 'support lifecycle allowed supported -> historical-only' >&2; exit 1
fi
if bash "${HERE}/validate-support-transition.sh" "${WORK}/deprecated.json" "${WORK}/historical.json" R2 >/dev/null 2>&1; then
  echo 'support lifecycle allowed retirement in the deprecation release' >&2; exit 1
fi
printf 'compatibility support lifecycle tests: PASS\n'
