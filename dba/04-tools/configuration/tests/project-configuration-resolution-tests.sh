#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
RESOLVER="${ROOT}/dba/04-tools/configuration/resolve-project-configuration.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-resolve.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'project configuration resolution tests failed: %s\n' "$1" >&2; exit 1; }

mkdir -p "${WORK}/.codeos/00-project"

legacy="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'resolution_mode=legacy-dba-7-cutover' <<<"${legacy}" || fail 'schema-less mode changed'
grep -Fxq 'dba_configuration=DBA-7' <<<"${legacy}" || fail 'schema-less project did not resolve to DBA-7'

printf 'codeos_schema: 2\ndba_configuration: DBA-8\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'resolution_mode=explicit' <<<"${explicit}" || fail 'schema-2 mode is not explicit'
grep -Fxq 'reader_output_policy=dba/02-policies/reader-output/v3.md' <<<"${explicit}" || fail 'DBA-8 did not select Reader Output v3'

printf 'codeos_schema: 2\ndba_configuration: DBA-9\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'doctrine=dba/01-doctrine/v8.md' <<<"${explicit}" || fail 'DBA-9 did not select doctrine v8'
grep -Fxq 'workflow_feature_policy=dba/02-policies/workflow-feature/v2.md' <<<"${explicit}" || fail 'DBA-9 did not select Feature Workflow v2'
grep -Fxq 'codeos_mechanics_policy=dba/02-policies/codeos-mechanics/v4.md' <<<"${explicit}" || fail 'DBA-9 did not select Mechanics v4'

printf 'codeos_schema: 2\ndba_configuration: DBA-10\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'architecture_synthesis_policy=dba/02-policies/architecture-synthesis/v6.md' <<<"${explicit}" || fail 'DBA-10 did not select Architecture Synthesis v6'
grep -Fxq 'workflow_governance_policy=dba/02-policies/workflow-governance/v2.md' <<<"${explicit}" || fail 'DBA-10 did not select Workflow Governance v2'
grep -Fxq 'codeos_mechanics_policy=dba/02-policies/codeos-mechanics/v5.md' <<<"${explicit}" || fail 'DBA-10 did not select Mechanics v5'

printf 'codeos_schema: 2\ndba_configuration: DBA-11\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v4.md' <<<"${explicit}" || fail 'DBA-11 did not select Review Policy v4'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v6.md' <<<"${explicit}" || fail 'DBA-11 did not select Reviewer Contract v6'
grep -Fxq 'workflow_governance_policy=dba/02-policies/workflow-governance/v1.md' <<<"${explicit}" || fail 'DBA-11 did not preserve the DBA-9 workflow policy'
grep -Fxq 'codeos_mechanics_policy=dba/02-policies/codeos-mechanics/v4.md' <<<"${explicit}" || fail 'DBA-11 did not preserve DBA-9 Mechanics v4'

printf 'codeos_schema: 2\ndba_configuration: DBA-12\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v5.md' <<<"${explicit}" || fail 'DBA-12 did not select Review Policy v5'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v7.md' <<<"${explicit}" || fail 'DBA-12 did not select Reviewer Contract v7'
grep -Fxq 'workflow_governance_policy=dba/02-policies/workflow-governance/v1.md' <<<"${explicit}" || fail 'DBA-12 did not preserve the DBA-11 workflow policy'

printf 'codeos_schema: 2\ndba_configuration: DBA-13\n' > "${WORK}/.codeos/00-project/codeos.yaml"
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v6.md' <<<"${explicit}" || fail 'DBA-13 did not select Review Policy v6'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v8.md' <<<"${explicit}" || fail 'DBA-13 did not select Reviewer Contract v8'

cat > "${WORK}/.codeos/00-project/codeos.yaml" <<'YAML'
codeos_schema: 2
dba_configuration: DBA-14
YAML
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v7.md' <<<"${explicit}" || fail 'DBA-14 did not select Review Policy v7'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v9.md' <<<"${explicit}" || fail 'DBA-14 did not select Reviewer Contract v9'
grep -Fxq 'workflow_governance_policy=dba/02-policies/workflow-governance/v1.md' <<<"${explicit}" || fail 'DBA-14 did not preserve the DBA-13 workflow policy'

cat > "${WORK}/.codeos/00-project/codeos.yaml" <<'YAML'
codeos_schema: 2
dba_configuration: DBA-15
YAML
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'doctrine=dba/01-doctrine/v9.md' <<<"${explicit}" || fail 'DBA-15 did not select Doctrine v9'
grep -Fxq 'review_policy=dba/02-policies/review/v8.md' <<<"${explicit}" || fail 'DBA-15 did not select Review Policy v8'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v10.md' <<<"${explicit}" || fail 'DBA-15 did not select Reviewer Contract v10'
grep -Fxq 'codeos_mechanics_policy=dba/02-policies/codeos-mechanics/v6.md' <<<"${explicit}" || fail 'DBA-15 did not select Mechanics v6'
grep -Fxq 'workflow_feature_policy=dba/02-policies/workflow-feature/v3.md' <<<"${explicit}" || fail 'DBA-15 did not select Feature Workflow v3'

cat > "${WORK}/.codeos/00-project/codeos.yaml" <<'YAML'
codeos_schema: 2
dba_configuration: DBA-16
YAML
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v9.md' <<<"${explicit}" || fail 'DBA-16 did not select Review Policy v9'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v11.md' <<<"${explicit}" || fail 'DBA-16 did not select Reviewer Contract v11'
grep -Fxq 'doctrine=dba/01-doctrine/v9.md' <<<"${explicit}" || fail 'DBA-16 did not preserve DBA-15 doctrine'
grep -Fxq 'workflow_feature_policy=dba/02-policies/workflow-feature/v3.md' <<<"${explicit}" || fail 'DBA-16 did not preserve DBA-15 Feature Workflow'

cat > "${WORK}/.codeos/00-project/codeos.yaml" <<'YAML'
codeos_schema: 2
dba_configuration: DBA-17
YAML
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v10.md' <<<"${explicit}" || fail 'DBA-17 did not select Review Policy v10'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v12.md' <<<"${explicit}" || fail 'DBA-17 did not select Reviewer Contract v12'
grep -Fxq 'doctrine=dba/01-doctrine/v9.md' <<<"${explicit}" || fail 'DBA-17 did not preserve DBA-16 doctrine'
grep -Fxq 'workflow_feature_policy=dba/02-policies/workflow-feature/v3.md' <<<"${explicit}" || fail 'DBA-17 did not preserve DBA-16 Feature Workflow'

cat > "${WORK}/.codeos/00-project/codeos.yaml" <<'YAML'
codeos_schema: 2
dba_configuration: DBA-18
YAML
explicit="$(bash "${RESOLVER}" "${WORK}")"
grep -Fxq 'review_policy=dba/02-policies/review/v11.md' <<<"${explicit}" || fail 'DBA-18 did not select Review Policy v11'
grep -Fxq 'reviewer_tool_contract=dba/04-tools/reviewer/contract/v13.md' <<<"${explicit}" || fail 'DBA-18 did not select Reviewer Contract v13'
grep -Fxq 'doctrine=dba/01-doctrine/v9.md' <<<"${explicit}" || fail 'DBA-18 did not preserve DBA-17 doctrine'
grep -Fxq 'workflow_feature_policy=dba/02-policies/workflow-feature/v3.md' <<<"${explicit}" || fail 'DBA-18 did not preserve DBA-17 Feature Workflow'

printf 'codeos_schema: 2\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'missing schema-2 binding was accepted'; fi

printf 'codeos_schema: 2\ndba_configuration: DBA-999\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'unknown binding was accepted'; fi

printf 'dba_configuration: DBA-8\n' > "${WORK}/.codeos/00-project/codeos.yaml"
if bash "${RESOLVER}" "${WORK}" >/dev/null 2>&1; then fail 'binding without schema was accepted'; fi

printf 'project configuration resolution tests: PASS\n'
