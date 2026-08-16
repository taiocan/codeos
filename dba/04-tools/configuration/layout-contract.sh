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
  --glob '!downstream-upgrade.md' \
  '\.codeos/(dba-system\.md|dba/|(prompts|scripts|templates|patterns|tools)(/|`)|terminology\.md)|dba/(configurations|doctrine|policies|tools)/' \
  "${active_paths[@]}"; then
  fail "an active file references a legacy toolkit path"
fi

# Retired mechanisms must not re-enter the supported runtime/configuration surface. Historical
# records remain valid outside this list; this is an active-layout invariant, not a history scan.
supported_runtime_paths=(
  "${CODEOS_ROOT}/dba-system.md"
  "${CODEOS_ROOT}/dba/00-entry/configurations/DBA-3.yaml"
  "${CODEOS_ROOT}/dba/03-prompts"
  "${CODEOS_ROOT}/dba/04-tools/initializer"
  "${CODEOS_ROOT}/dba/04-tools/reviewer"
  "${CODEOS_ROOT}/dba/05-guidance/templates"
)
if rg -n -S \
  --glob '!tests/**' \
  'Controlled Plain English|controlled-plain-english|controlled_plain_english|writing-discipline|codeos-cpe-status|CPE_STATUS|CPE_CONFIG|Layer [ABCD][12]?' \
  "${supported_runtime_paths[@]}"; then
  fail "a retired mechanism is referenced by the supported runtime/configuration surface"
fi
for retired_path in \
  maintenance/config/writing-discipline.yaml \
  dba/05-guidance/patterns/controlled-plain-english.md \
  dba/02-policies/controlled-plain-english/v1.md \
  dba/03-prompts/workflow/00-full-solution-concept.md \
  dba/05-guidance/templates/full-solution-concept.md; do
  [[ ! -e "${CODEOS_ROOT}/${retired_path}" ]] || fail "retired active artifact remains: ${retired_path}"
done

for stage in {01..09}; do
  mapfile -t stage_prompts < <(find "${CODEOS_ROOT}/dba/03-prompts/workflow" -maxdepth 1 \
    -type f -name "${stage}-*.md" -printf '%p\n')
  [[ "${#stage_prompts[@]}" -eq 1 ]] || \
    fail "governed Stage ${stage#0} must have exactly one ${stage}- prefixed prompt"
  grep -q "^# Stage ${stage#0}:" "${stage_prompts[0]}" || \
    fail "${stage_prompts[0]##*/} does not declare governed Stage ${stage#0}"
done

expected_support_prompts=(
  support-architecture-synthesis.md
  support-existing-codebase-intake.md
  support-feature-decomposition.md
  support-session-handoff.md
  support-session-orientation.md
  support-solution-charter.md
  support-solution-framing.md
)
mapfile -t actual_support_prompts < <(find "${CODEOS_ROOT}/dba/03-prompts/workflow" -maxdepth 1 \
  -type f -name 'support-*.md' -printf '%f\n' | LC_ALL=C sort)
[[ "${actual_support_prompts[*]}" == "${expected_support_prompts[*]}" ]] || \
  fail 'unexpected support workflow inventory'

while IFS= read -r support_prompt; do
  [[ "${support_prompt##*/}" =~ ^0[1-9]- ]] && continue
  [[ "${support_prompt##*/}" =~ ^support-[a-z0-9]+(-[a-z0-9]+)*\.md$ ]] || \
    fail "non-stage workflow prompt lacks a descriptive support- filename: ${support_prompt##*/}"
  if grep -q '^# Stage [0-9]' "${support_prompt}"; then
    fail "support prompt masquerades as a governed stage: ${support_prompt##*/}"
  fi
done < <(find "${CODEOS_ROOT}/dba/03-prompts/workflow" -maxdepth 1 -type f -name '*.md' -printf '%p\n')

if rg -n '"(discovery|brief|onboarding)"|`(discovery|brief|onboarding)`' \
  "${CODEOS_ROOT}/dba/03-prompts/review" \
  "${CODEOS_ROOT}/dba/04-tools/reviewer/contract" \
  "${CODEOS_ROOT}/dba/04-tools/reviewer/engine/src" \
  "${CODEOS_ROOT}/dba/04-tools/reviewer/engine/tests"; then
  fail 'an active reviewer surface uses a retired support-workflow identifier'
fi

canonical_paths=(
  '.codeos/00-project/CLAUDE.md'
  '.codeos/00-project/charter.md'
  '.codeos/00-project/learnings.md'
  '.codeos/00-project/terminology.md'
  '.codeos/01-specification/intents/<feature-id>.md'
  '.codeos/01-specification/contracts/<feature-id>_contract.md'
  '.codeos/01-specification/event-schemas/<feature-id>_schema.md'
  '.codeos/02-architecture/scopes/<scope-id>.md'
  '.codeos/02-architecture/implementation-profile.yaml'
  '.codeos/00-discovery/<topic-slug>.md'
  '.codeos/04-refinement/<feature-id>-<slug>.md'
  '.codeos/05-review/reviewer.toml'
  '.codeos/05-review/reviews/'
  '.codeos/05-review/measurements/<name>.md'
  '.codeos/toolkit'
  '.codeos-state/'
  'events/runtime_events.jsonl'
)

for path in "${canonical_paths[@]}"; do
  grep -Fq "$path" "${CODEOS_ROOT}/dba-system.md" || \
    fail "downstream layout owner omits canonical path: ${path}"
done

declare -A producer_outputs=(
  [support-solution-charter.md]='.codeos/00-project/charter.md'
  [support-solution-framing.md]='.codeos/00-discovery/<topic-slug>.md'
  [support-feature-decomposition.md]='.codeos/00-discovery/<topic-slug>.md'
  [support-existing-codebase-intake.md]='.codeos/01-specification/intents/<feature-id>.md'
  [01-intent.md]='.codeos/01-specification/intents/[feature_id].md'
  [02-contract.md]='.codeos/01-specification/contracts/[feature_id]_contract.md'
  [03-event-schema.md]='.codeos/01-specification/event-schemas/[feature_id]_schema.md'
  [support-architecture-synthesis.md]='.codeos/02-architecture/scopes/<scope-id>.md'
  [09-refine.md]='.codeos/04-refinement/<feature-id>-<slug>.md'
)

for prompt in "${!producer_outputs[@]}"; do
  path="${CODEOS_ROOT}/dba/03-prompts/workflow/${prompt}"
  grep -Fq "${producer_outputs[${prompt}]}" "$path" || \
    fail "artifact-producing prompt omits canonical output: ${prompt}"
done

declare -A adapters=(
  [purpose-approval]="dba/03-prompts/workflow/support-solution-charter.md"
  [specification-approval]="dba/03-prompts/workflow/03-event-schema.md"
  [delivery-entry]="dba/03-prompts/workflow/04-implement.md"
  [final-acceptance]="dba/03-prompts/workflow/08-replay.md"
  [architecture-entry]="dba/03-prompts/workflow/support-architecture-synthesis.md"
)

for adapter in "${!adapters[@]}"; do
  path="${CODEOS_ROOT}/${adapters[${adapter}]}"
  [[ -f "${path}" ]] || fail "adapter owner is missing: ${adapters[${adapter}]}"
  grep -q "DOCTRINE ADAPTER: ${adapter}" "${path}" || \
    fail "${adapter} is not declared by ${adapters[${adapter}]}"
done

adapter_count="$(rg -l 'DOCTRINE ADAPTER: [a-z-]+' "${CODEOS_ROOT}/dba/03-prompts" | wc -l)"
[[ "${adapter_count}" -eq 5 ]] || fail "expected 5 doctrine adapter owners, found ${adapter_count}"

bash "${CODEOS_ROOT}/dba/04-tools/configuration/dba-config-boundaries.sh" \
  dba/00-entry/configurations/DBA-3.yaml >/dev/null

printf 'Codeos layout contract OK\n'
