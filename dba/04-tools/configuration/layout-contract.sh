#!/usr/bin/env bash
# Verify the supported two-surface Codeos repository layout.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../../.." && pwd -P)"

fail() {
  printf 'Codeos layout contract failed: %s\n' "$1" >&2
  exit 1
}

for file in README.md AGENTS.md CLAUDE.md dba-system.md; do
  [[ -f "${CODEOS_ROOT}/${file}" ]] || fail "missing root discovery file: ${file}"
done

for dir in \
  dba/00-entry dba/01-doctrine dba/02-policies dba/03-prompts \
  dba/04-tools dba/05-guidance dba/06-reference \
  maintenance/backlog maintenance/reviews maintenance/config maintenance/archive; do
  [[ -d "${CODEOS_ROOT}/${dir}" ]] || fail "missing layout directory: ${dir}"
done

for dir in Archive backlog config docs patterns prompts reviews scripts templates tools \
  dba/configurations dba/doctrine dba/policies dba/tools; do
  [[ ! -e "${CODEOS_ROOT}/${dir}" ]] || fail "legacy active directory remains: ${dir}"
done

active_paths=(
  "${CODEOS_ROOT}/README.md"
  "${CODEOS_ROOT}/CLAUDE.md"
  "${CODEOS_ROOT}/dba-system.md"
  "${CODEOS_ROOT}/dba"
  "${CODEOS_ROOT}/maintenance/backlog"
  "${CODEOS_ROOT}/maintenance/config"
  "${CODEOS_ROOT}/maintenance/reviews"
)

if rg -n -S \
  --glob '!path-migration.md' \
  '\.codeos/(prompts|scripts|templates|patterns|tools)(/|`)|\.codeos/terminology\.md|dba/(configurations|doctrine|policies|tools)/' \
  "${active_paths[@]}"; then
  fail "an active file references a legacy toolkit path"
fi

declare -A adapters=(
  [specification-approval]="dba/03-prompts/workflow/03-event-schema.md"
  [delivery-entry]="dba/03-prompts/workflow/04-implement.md"
  [final-acceptance]="dba/03-prompts/workflow/08-replay.md"
  [architecture-entry]="dba/03-prompts/workflow/03b-architecture-synthesis.md"
)

for adapter in "${!adapters[@]}"; do
  path="${CODEOS_ROOT}/${adapters[${adapter}]}"
  [[ -f "${path}" ]] || fail "adapter owner is missing: ${adapters[${adapter}]}"
  grep -q "DOCTRINE ADAPTER: ${adapter}" "${path}" || \
    fail "${adapter} is not declared by ${adapters[${adapter}]}"
done

adapter_count="$(rg -l 'DOCTRINE ADAPTER: [a-z-]+' "${CODEOS_ROOT}/dba/03-prompts" | wc -l)"
[[ "${adapter_count}" -eq 4 ]] || fail "expected 4 doctrine adapter owners, found ${adapter_count}"

bash "${CODEOS_ROOT}/dba/04-tools/configuration/dba-config-boundaries.sh" \
  dba/00-entry/configurations/DBA-2.yaml >/dev/null

printf 'Codeos layout contract OK\n'
