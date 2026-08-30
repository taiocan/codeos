#!/usr/bin/env bash
# Verify the intentionally small active guidance inventory and its owned workflow boundaries.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
fail() { printf 'guidance contract failed: %s\n' "$1" >&2; exit 1; }

expected_guidance=(
  patterns/postgresql-reliability.md
  patterns/rust-domain-modelling.md
  patterns/rust-project-structure.md
  patterns/shared-infrastructure-boundary.md
  patterns/svelte-state-and-components.md
  patterns/vocabulary-architecture.md
  templates/architecture-scope.md
  templates/charter.md
  templates/contract.md
  templates/event-schema.md
  templates/feature-decomposition.md
  templates/implementation-profile.yaml
  templates/intent.md
  templates/learning-register.md
  templates/module-design-note.md
  templates/project-AGENTS.md
  templates/project-CLAUDE.md
  templates/project-root-CLAUDE.md
  templates/project-terminology.md
  templates/refinement.md
  templates/review-file.md
  templates/review-package.md
  templates/reviewer.toml
  terminology.md
)
# LC_ALL=C keeps collation byte-ordered so the expected list is not locale-dependent.
mapfile -t actual_guidance < <(cd "${CODEOS_ROOT}/dba/05-guidance" && find . -type f -printf '%P\n' | LC_ALL=C sort)
[[ "${actual_guidance[*]}" == "${expected_guidance[*]}" ]] || fail 'unexpected active guidance inventory'

SESSION_START="${CODEOS_ROOT}/dba/03-prompts/workflow/support-session-orientation.md"
INTENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/01-intent.md"
FRAMING_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-solution-framing.md"
CHARTER_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-solution-charter.md"
ARCHITECTURE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-architecture-synthesis.md"
IMPLEMENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/04-implement.md"
TEST_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/05-tests.md"
OBSERVE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/06-observe.md"
RECONCILE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/07-reconcile.md"
REPLAY_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/08-replay.md"
rg -q 'partially drafted Specification Package is normal' "${SESSION_START}" || fail 'partial packages are not accepted'
rg -q 'terminology.md.*exists' "${SESSION_START}" || fail 'optional project terminology is not loaded'
rg -q 'Its absence is valid' "${SESSION_START}" || fail 'project terminology became mandatory'
rg -q 'architecture-scope membership' "${INTENT_PROMPT}" || fail 'feature-id allocation does not scan every identity owner'
rg -q 'incompatible artifacts claim the same identity' "${INTENT_PROMPT}" || fail 'identity-conflict boundary is missing'
rg -q 'Define only feature-local' "${INTENT_PROMPT}" || fail 'Intent terminology ownership is unclear'
rg -q 'when promoted into and approved in the Solution Charter' "${FRAMING_PROMPT}" || \
  fail 'Solution Framing promotion boundary is missing'
rg -q 'MUST NOT resolve that concern into components' "${FRAMING_PROMPT}" || \
  fail 'Solution Framing architecture boundary is missing'
rg -q 'only their promotion into an approved Charter makes them' "${CHARTER_PROMPT}" || \
  fail 'Solution Charter promotion boundary is missing'
rg -q 'Stage 4 owns local implementation design' "${ARCHITECTURE_PROMPT}" || \
  fail 'Architecture Synthesis design boundary is missing'
rg -q 'feature-local design decisions inside approved architectural boundaries' "${IMPLEMENT_PROMPT}" || \
  fail 'Stage 4 local design ownership is missing'
rg -q 'presented as acceptance evidence.*boundary named by the acceptance' "${TEST_PROMPT}" || \
  fail 'Stage 5 acceptance-evidence boundary is missing'
rg -q 'Diagnostic and exploratory performance measurements may be collected freely' "${OBSERVE_PROMPT}" || \
  fail 'Stage 6 diagnostic-measurement freedom is missing'
rg -q 'performance measurement is accepted as evidence for a governed requirement' "${OBSERVE_PROMPT}" || \
  fail 'Stage 6 governed-performance evidence boundary is missing'
rg -q 'measured operation exercises the governed behavior' "${OBSERVE_PROMPT}" || \
  fail 'Stage 6 governed-performance evidence boundary is missing'
rg -q 'No reconciliation claim may be stronger than its cited observation' "${RECONCILE_PROMPT}" || \
  fail 'Stage 7 evidence-bounded reconciliation is missing'
rg -q 'separate eligibility status' "${REPLAY_PROMPT}" || \
  fail 'Stage 8 reuses the existing gap route incorrectly'

printf 'guidance contract: PASS\n'
