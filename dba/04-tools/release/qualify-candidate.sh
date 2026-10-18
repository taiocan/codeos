#!/usr/bin/env bash
# Qualify a committed candidate configuration without giving it authority (DQA WO-01A, WO-01D). In
# a disposable copy of the committed toolkit, the candidate is materialized as the next DBA-N; the
# full suite, the compatibility matrix and every replay run there; then the copy is deleted. On
# success it prints the qualified semantic identity the human approves. --replay-only skips the
# suites and prints no identity, so its result can never be issued.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="$(cd "${HERE}/../../.." && pwd -P)"
source "${HERE}/candidate-lib.sh"
[[ $# -ge 1 && $# -le 2 ]] || { echo 'usage: qualify-candidate.sh <name> [--replay-only]' >&2; exit 2; }
NAME="$1" REPLAY_ONLY=0
[[ "${2:-}" == '' || "${2}" == '--replay-only' ]] || { echo "unknown option: $2" >&2; exit 2; }
[[ "${2:-}" == '--replay-only' ]] && REPLAY_ONLY=1

candidate_file "${ROOT}" "${NAME}" >/dev/null
ID="$(next_configuration_id "${ROOT}")"
WORK="$(mktemp -d)"
trap 'rm -rf "${WORK}"' EXIT
TOOLKIT="${WORK}/toolkit"
export_head "${ROOT}" "${TOOLKIT}"
materialize_candidate "${TOOLKIT}" "${NAME}" "${ID}" "$(date -u +%Y-%m-%d)"
git -C "${TOOLKIT}" init -q
git -C "${TOOLKIT}" add -A
git -C "${TOOLKIT}" -c user.name=qualification -c user.email=qualification@codeos.invalid commit -qm "${NAME} as ${ID}"
export CODEOS_COMPATIBILITY_LOCAL_ROOT="${CODEOS_COMPATIBILITY_LOCAL_ROOT:-$(dirname "${ROOT}")}"

if [[ "${REPLAY_ONLY}" == 0 ]]; then
  bash "${TOOLKIT}/dba/04-tools/tests/run.sh"
  bash "${TOOLKIT}/dba/04-tools/compatibility/run.sh"
fi
bash "${TOOLKIT}/dba/04-tools/compatibility/replay.sh" "${TOOLKIT}" "${ID}"

if [[ "${REPLAY_ONLY}" == 1 ]]; then
  printf 'replay passed for candidate %s as %s; not qualified (suites skipped)\n' "${NAME}" "${ID}"
else
  printf 'qualified candidate %s as %s\nqualified_candidate_sha256: %s\n' \
    "${NAME}" "${ID}" "$(qualified_candidate_sha256 "${ROOT}" "${NAME}")"
fi
