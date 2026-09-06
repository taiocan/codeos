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
  "${CODEOS_ROOT}/dba/00-entry/configurations/DBA-6.yaml"
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
  support-solution-bootstrap.md
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
  '.codeos/00-project/codeos.yaml'
  '.codeos/00-project/charter.md'
  '.codeos/00-project/learnings.md'
  '.codeos/00-project/terminology.md'
  '.codeos/01-specification/intents/<feature-id>.md'
  '.codeos/01-specification/contracts/<feature-id>_contract.md'
  '.codeos/01-specification/event-schemas/<feature-id>_schema.md'
  '.codeos/02-architecture/scopes/<scope-id>.md'
  '.codeos/02-architecture/implementation-profile.yaml'
  '.codeos/03-design/<module-slug>.md'
  '.codeos/00-discovery/<topic-slug>.md'
  '.codeos/04-refinement/<feature-id>-<slug>.md'
  '.codeos/05-review/reviewer.toml'
  '.codeos/05-review/reviews/'
  '.codeos/05-review/measurements/<name>.md'
  '.codeos/06-workflow/decisions.jsonl'
  '.codeos/06-workflow/verifications.jsonl'
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
  [support-solution-bootstrap.md]='.codeos/06-workflow/decisions.jsonl'
  [01-intent.md]='.codeos/01-specification/intents/<feature-id>.md'
  [02-contract.md]='.codeos/01-specification/contracts/<feature-id>_contract.md'
  [03-event-schema.md]='.codeos/01-specification/event-schemas/<feature-id>_schema.md'
  [support-architecture-synthesis.md]='.codeos/02-architecture/scopes/<scope-id>.md'
  [09-refine.md]='.codeos/04-refinement/<feature-id>-<slug>.md'
)

# This map is kept deliberately. Which prompt produces which artifact is not derivable from the
# layout: the relation is many-to-many and partial — two prompts produce a discovery note, two
# produce an Intent, and several canonical locations have no producing prompt at all. Deriving the
# expectation from the prompts themselves would make the check assert only that a prompt contains
# what it contains. The map is an independent guard over an otherwise unowned relation.
#
# The map tracks the primary canonical artifact a workflow prompt's step brings into being. A
# support prompt whose step records a decision receipt is mapped to `.codeos/06-workflow/decisions.jsonl`
# (support-solution-bootstrap.md → the Initial Product Preview receipt). It is NOT mapped to
# `.codeos/06-workflow/verifications.jsonl`: verification records are mechanical evidence the shared
# workflow checker writes for every workflow, not a distinctive output of one prompt — the same
# reason the reviewer tool's own records have no producer entry.
for prompt in "${!producer_outputs[@]}"; do
  path="${CODEOS_ROOT}/dba/03-prompts/workflow/${prompt}"
  grep -Fq "${producer_outputs[${prompt}]}" "$path" || \
    fail "artifact-producing prompt omits canonical output: ${prompt}"
  # The path itself is owned by the layout contract. Requiring each value to be a canonical path
  # keeps this map a guard rather than a second place a path may be edited into existence.
  printf '%s\n' "${canonical_paths[@]}" | grep -Fxq "${producer_outputs[${prompt}]}" || \
    fail "producer output is not a canonical path: ${producer_outputs[${prompt}]}"
done

# One placeholder notation for canonical paths. A second spelling of the same governed fact is what
# previously forced this check to carry translation entries, so it is refused rather than bridged.
# Content placeholders inside template bodies are a different thing and are not matched here: this
# looks only inside a .codeos path token.
mixed_notation="$(grep -rn '\.codeos/[^ `"]*\[[a-z_-]*\]' "${CODEOS_ROOT}/dba" "${CODEOS_ROOT}/dba-system.md" 2>/dev/null || true)"
[[ -z "${mixed_notation}" ]] || \
  fail "canonical path uses a non-canonical placeholder notation (use <kebab-case>): ${mixed_notation}"

# Doctrine adapters: the selected doctrine owns which boundaries exist, so this check derives the
# expected set from it rather than restating it. A list here would be a second, manually
# synchronized authority — and the one that previously decided membership in practice, because
# nothing else named the set.
ACTIVE_CONFIG="${CODEOS_ROOT}/dba/00-entry/configurations/DBA-6.yaml"
doctrine_rel="$(awk '$1 == "doctrine:" { print $2 }' "${ACTIVE_CONFIG}")"
[[ -n "${doctrine_rel}" ]] || fail 'active configuration names no doctrine'
DOCTRINE="${CODEOS_ROOT}/${doctrine_rel}"
[[ -f "${DOCTRINE}" ]] || fail "selected doctrine is missing: ${doctrine_rel}"

# Membership as the doctrine declares it: one backticked name per list item under its adapter
# boundary heading.
mapfile -t doctrine_adapters < <(
  awk '/^### Doctrine Adapter Boundaries$/ { inside = 1; next }
       inside && /^## / { inside = 0 }
       inside && /^- `[a-z-]+`/ { gsub(/^- `|`.*$/, ""); print }' "${DOCTRINE}" | LC_ALL=C sort
)
[[ "${#doctrine_adapters[@]}" -gt 0 ]] || \
  fail 'selected doctrine declares no adapter boundaries'

# Membership as the prompts declare it.
mapfile -t declared_adapters < <(
  grep -rhoE 'DOCTRINE ADAPTER: [a-z-]+' "${CODEOS_ROOT}/dba/03-prompts" \
    | sed 's/^DOCTRINE ADAPTER: //' | LC_ALL=C sort
)
[[ "${declared_adapters[*]}" == "$(printf '%s\n' "${declared_adapters[@]}" | LC_ALL=C sort -u | tr '\n' ' ' | sed 's/ $//')" ]] || \
  fail 'a doctrine adapter is declared by more than one prompt'
[[ "${doctrine_adapters[*]}" == "${declared_adapters[*]}" ]] || \
  fail "adapter boundaries disagree: doctrine [${doctrine_adapters[*]}] vs prompts [${declared_adapters[*]}]"

bash "${CODEOS_ROOT}/dba/04-tools/configuration/dba-config-boundaries.sh" \
  dba/00-entry/configurations/DBA-6.yaml >/dev/null

printf 'Codeos layout contract OK\n'
