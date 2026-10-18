#!/usr/bin/env bash
# DQA WO-01A/01D falsifiers: a candidate configuration has no authority; issuance publishes only the
# approved qualified semantic identity; the qualification replay rejects either DBA-27 residual.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-candidate.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'candidate tests failed: %s\n' "$1" >&2; exit 1; }
TK="${WORK}/toolkit"
mkdir -p "${TK}"
git -C "${ROOT}" ls-files -z --cached --others --exclude-standard | (cd "${ROOT}" && xargs -0 cp --parents -t "${TK}")
commit() { git -C "${TK}" add -A && git -C "${TK}" -c user.name=t -c user.email=t@t commit -qm "$1"; }
git -C "${TK}" init -q && commit base
source "${TK}/dba/04-tools/release/candidate-lib.sh"
issue() { CODEOS_ISSUE_ROOT="${TK}" bash "${TK}/dba/04-tools/release/issue-configuration.sh" "$@"; }
next="$(next_configuration_id "${TK}")"

# A draft Feature Workflow policy selected only by a candidate.
mkdir -p "${TK}/maintenance/candidates"
cp "${TK}/dba/02-policies/workflow-feature/v12.md" "${TK}/dba/02-policies/workflow-feature/v99.md"
sed -e 's/^status: approved$/status: candidate/' -e '/^approved_at:/d' \
  -e 's#workflow-feature/v12.md#workflow-feature/v99.md#' \
  "${TK}/dba/00-entry/configurations/DBA-28.yaml" > "${TK}/maintenance/candidates/draft.yaml"
commit 'draft candidate'

# No project can resolve a candidate.
project="${WORK}/project"
mkdir -p "${project}/.codeos/00-project"
ln -s "${TK}" "${project}/.codeos/toolkit"
for id in draft "${next}"; do
  printf 'codeos_schema: 2\ndba_configuration: %s\n' "${id}" > "${project}/.codeos/00-project/codeos.yaml"
  if bash "${TK}/dba/04-tools/configuration/resolve-project-configuration.sh" "${project}" >/dev/null 2>&1; then
    fail "a project resolved the unissued candidate as ${id}"
  fi
done

# Approval binds the exact semantic snapshot: a draft edit after qualification refuses issuance.
approved="$(qualified_candidate_sha256 "${TK}" draft)"
[[ "${approved}" =~ ^[0-9a-f]{64}$ ]] || fail 'no qualified semantic identity'
[[ "$(qualified_candidate_sha256 "${TK}" draft)" == "${approved}" ]] || fail 'the identity is not deterministic'
printf '\nA later edit.\n' >> "${TK}/dba/02-policies/workflow-feature/v99.md"
commit 'edit draft'
if issue draft "${next}" "${approved}" >/dev/null 2>&1; then
  fail 'issuance published a draft changed after approval'
fi
[[ ! -e "${TK}/dba/00-entry/configurations/${next}.yaml" ]] || fail 'a refused issuance wrote a configuration'

# The current identity issues exactly once, freezing the draft with it.
current="$(qualified_candidate_sha256 "${TK}" draft)"
[[ "${current}" != "${approved}" ]] || fail 'a draft edit left the identity unchanged'
issue draft "${next}" "${current}" >/dev/null
grep -q '^status: approved$' "${TK}/dba/00-entry/configurations/${next}.yaml" || fail 'issued configuration is not approved'
grep -q 'workflow-feature/v99.md' "${TK}/dba/00-entry/configurations/closures/${next}.closure" || fail 'the draft is not frozen'
[[ "$(jq -r --arg id "${next}" '.configurations[$id]' "${TK}/dba/04-tools/compatibility/support.json")" == supported ]] \
  || fail 'issued configuration has no support state'
[[ ! -e "${TK}/maintenance/candidates/draft.yaml" ]] || fail 'the issued candidate remains'
commit 'issue'
printf '\nAfter issuance.\n' >> "${TK}/dba/02-policies/workflow-feature/v99.md"
if bash "${TK}/dba/04-tools/configuration/verify-configuration-manifest.sh" "${TK}" >/dev/null 2>&1; then
  fail 'an issued draft stayed mutable'
fi

# Criterion 1: the replay fixtures reject each DBA-27 residual and accept DBA-28.
manifest="${ROOT}/dba/04-tools/workflow/engine/Cargo.toml"
CODEOS_QUALIFICATION_CONFIGURATION=DBA-28 cargo test -q --locked --manifest-path "${manifest}" \
  --test qualification_replay >/dev/null 2>&1 || fail 'DBA-28 fails the qualification replay'
for residual in an_inherited_claim_follows_a_pre_bundle_upstream a_test_runner_configuration_change; do
  if CODEOS_QUALIFICATION_CONFIGURATION=DBA-27 cargo test -q --locked --manifest-path "${manifest}" \
    --test qualification_replay "${residual}" >/dev/null 2>&1; then
    fail "the replay accepts DBA-27's ${residual} residual"
  fi
done

printf 'candidate tests: PASS\n'
