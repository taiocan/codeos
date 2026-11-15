#!/usr/bin/env bash
# Resolve the DBA composition governing one project. Output is stable key=value data for tools.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
TOOLKIT_ROOT="$(cd "${HERE}/../../.." && pwd -P)"
PROJECT_ROOT="${1:-$(pwd -P)}"
CONFIG_PATH="${2:-${PROJECT_ROOT}/.codeos/00-project/codeos.yaml}"
fail() { printf 'project configuration resolution failed: %s\n' "$1" >&2; exit 1; }

configuration_id=''
resolution_mode=''
schema=''

if [[ -f "${CONFIG_PATH}" ]]; then
  schema="$(sed -n 's/^codeos_schema:[[:space:]]*\([^[:space:]#]*\).*$/\1/p' "${CONFIG_PATH}" | head -n1)"
  configuration_id="$(sed -n 's/^dba_configuration:[[:space:]]*\([^[:space:]#]*\).*$/\1/p' "${CONFIG_PATH}" | head -n1)"
fi

if [[ -n "${schema}" ]]; then
  [[ "${schema}" == '2' ]] || fail "unsupported codeos_schema '${schema}'"
  [[ -n "${configuration_id}" ]] || fail 'schema 2 requires dba_configuration'
  resolution_mode='explicit'
else
  [[ -z "${configuration_id}" ]] || fail 'dba_configuration requires codeos_schema: 2'
  configuration_id='DBA-7'
  resolution_mode='legacy-dba-7-cutover'
fi

[[ "${configuration_id}" =~ ^DBA-[0-9]+$ ]] || fail "invalid dba_configuration '${configuration_id}'"
configuration_rel="dba/00-entry/configurations/${configuration_id}.yaml"
configuration_path="${TOOLKIT_ROOT}/${configuration_rel}"
[[ -f "${configuration_path}" ]] || fail "unsupported dba_configuration '${configuration_id}'"
grep -Eq '^status:[[:space:]]*approved([[:space:]]|$)' "${configuration_path}" || \
  fail "dba_configuration '${configuration_id}' is not approved"

support_manifest="${TOOLKIT_ROOT}/dba/04-tools/compatibility/support.json"
support_status='unclassified'
if [[ -f "${support_manifest}" ]] && command -v jq >/dev/null 2>&1; then
  support_status="$(jq -r --arg id "${configuration_id}" '.configurations[$id] // "unclassified"' "${support_manifest}")"
fi

printf 'codeos_schema=%s\n' "${schema:-legacy}"
printf 'resolution_mode=%s\n' "${resolution_mode}"
printf 'dba_configuration=%s\n' "${configuration_id}"
printf 'support_status=%s\n' "${support_status}"
printf 'configuration_path=%s\n' "${configuration_path}"
for key in doctrine review_policy architecture_synthesis_policy implementation_profile_policy \
  reviewer_tool_contract codeos_mechanics_policy reader_output_policy user_learning_policy workflow_governance_policy \
  workflow_bootstrap_policy workflow_feature_policy workflow_operation_policy; do
  value="$(awk -v wanted="${key}:" '$1 == wanted { print $2; exit }' "${configuration_path}")"
  [[ -n "${value}" ]] && printf '%s=%s\n' "${key}" "${value}"
done
exit 0
