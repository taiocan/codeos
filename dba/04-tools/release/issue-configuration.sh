#!/usr/bin/env bash
# Issue a qualified candidate as the next DBA configuration (DQA WO-01A). Run only after the human
# approves the qualified semantic identity qualification printed. Issuance recomputes it from the
# committed tree and refuses any difference: a changed candidate or draft needs a new
# qualification and a new approval. Leaves the result uncommitted for review.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
ROOT="${CODEOS_ISSUE_ROOT:-$(cd "${HERE}/../../.." && pwd -P)}"
source "${HERE}/candidate-lib.sh"
[[ $# -eq 3 ]] || { echo 'usage: issue-configuration.sh <name> <DBA-N> <qualified_candidate_sha256>' >&2; exit 2; }
NAME="$1" ID="$2" APPROVED="$3"

[[ "${ID}" == "$(next_configuration_id "${ROOT}")" ]] \
  || candidate_fail "${ID} is not the next configuration ($(next_configuration_id "${ROOT}"))"
git -C "${ROOT}" diff --quiet HEAD -- dba dba-system.md maintenance/candidates \
  || candidate_fail 'uncommitted changes to dba/, dba-system.md or maintenance/candidates'
[[ -z "$(git -C "${ROOT}" ls-files --others --exclude-standard -- dba maintenance/candidates)" ]] \
  || candidate_fail 'untracked files under dba/ or maintenance/candidates'
CURRENT="$(qualified_candidate_sha256 "${ROOT}" "${NAME}")"
[[ "${CURRENT}" == "${APPROVED}" ]] \
  || candidate_fail "qualified semantic identity is ${CURRENT}, not the approved ${APPROVED}; requalify"

materialize_candidate "${ROOT}" "${NAME}" "${ID}" "$(date -u +%Y-%m-%d)"
git -C "${ROOT}" rm -q "maintenance/candidates/${NAME}.yaml"
bash "${ROOT}/dba/04-tools/configuration/verify-configuration-manifest.sh" "${ROOT}" >/dev/null
printf 'issued %s from candidate %s (%s); review and commit\n' "${ID}" "${NAME}" "${APPROVED}"
