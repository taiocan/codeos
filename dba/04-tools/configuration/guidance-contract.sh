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
  patterns/svelte-gui-verification.md
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
  templates/user-workflow-map.md
  terminology.md
)
# LC_ALL=C keeps collation byte-ordered so the expected list is not locale-dependent.
mapfile -t actual_guidance < <(cd "${CODEOS_ROOT}/dba/05-guidance" && find . -type f -printf '%P\n' | LC_ALL=C sort)
[[ "${actual_guidance[*]}" == "${expected_guidance[*]}" ]] || fail 'unexpected active guidance inventory'

SESSION_START="${CODEOS_ROOT}/dba/03-prompts/workflow/support-session-orientation.md"
INTENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/01-intent.md"
FRAMING_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-solution-framing.md"
CHARTER_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-solution-charter.md"
CHARTER_TEMPLATE="${CODEOS_ROOT}/dba/05-guidance/templates/charter.md"
ARCHITECTURE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/support-architecture-synthesis.md"
IMPLEMENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/04-implement.md"
TEST_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/05-tests.md"
OBSERVE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/06-observe.md"
RECONCILE_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/07-reconcile.md"
REPLAY_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/08-replay.md"
SVELTE_PATTERN="${CODEOS_ROOT}/dba/05-guidance/patterns/svelte-state-and-components.md"
GUI_PATTERN="${CODEOS_ROOT}/dba/05-guidance/patterns/svelte-gui-verification.md"
WORKFLOW_MAP="${CODEOS_ROOT}/dba/05-guidance/templates/user-workflow-map.md"
TERMINOLOGY="${CODEOS_ROOT}/dba/05-guidance/terminology.md"
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
rg -q '^## Decision Supported$' "${CHARTER_TEMPLATE}" || \
  fail 'Solution Charter decision-supported interface is missing'
rg -q 'primary actor and primary decision supported by the solution' "${CHARTER_TEMPLATE}" || \
  fail 'Solution Charter decision-supported content is incomplete'
rg -q 'existing approved Charter remains valid' "${CHARTER_PROMPT}" || \
  fail 'existing Solution Charters are not protected from migration'
rg -q 'new or materially revised Charter' "${CHARTER_PROMPT}" || \
  fail 'Solution Charter decision-supported applicability is missing'
rg -q 'alignment lens, not an' "${CHARTER_PROMPT}" || \
  fail 'Solution Charter decision-supported authority boundary is missing'
rg -q 'acceptance requirement' "${CHARTER_PROMPT}" || \
  fail 'Solution Charter decision-supported acceptance boundary is missing'
rg -q 'primary supported decision' "${TERMINOLOGY}" || \
  fail 'Solution Charter terminology is stale'
rg -q 'Stage 4 owns local implementation design' "${ARCHITECTURE_PROMPT}" || \
  fail 'Architecture Synthesis design boundary is missing'
rg -q 'feature-local design decisions inside approved architectural boundaries' "${IMPLEMENT_PROMPT}" || \
  fail 'Stage 4 local design ownership is missing'
rg -q 'selects a technology covered by an advisory pattern' "${IMPLEMENT_PROMPT}" || \
  fail 'Stage 4 advisory technology-pattern routing is missing'
rg -q 'profile governs' "${IMPLEMENT_PROMPT}" || \
  fail 'Stage 4 Implementation Profile language authority is missing'
rg -q 'architecture governs technology selection' "${IMPLEMENT_PROMPT}" || \
  fail 'Stage 4 language and framework authority are conflated'
rg -q 'svelte-gui-verification.md' "${TEST_PROMPT}" || \
  fail 'Stage 5 Svelte GUI verification guidance is not discoverable'
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
rg -q 'approved architecture selects Svelte' "${SVELTE_PATTERN}" || \
  fail 'Svelte pattern consultation is not architecture-selected'
rg -q 'Vitest Browser Mode with the Playwright provider' "${GUI_PATTERN}" || \
  fail 'Svelte GUI browser-component boundary is missing'
rg -q 'Playwright Test' "${GUI_PATTERN}" || \
  fail 'Svelte GUI end-to-end boundary is missing'
rg -q 'currency is present in a model.*currency is visible' "${GUI_PATTERN}" || \
  fail 'Svelte GUI rendered-field evidence distinction is missing'
rg -q 'criterion appears in a request.*visible matched plots changed' "${GUI_PATTERN}" || \
  fail 'Svelte GUI changed-result evidence distinction is missing'
rg -q 'Before browser timing is accepted.*governed performance' "${GUI_PATTERN}" || \
  fail 'Svelte GUI performance-evidence boundary is missing'
rg -q 'does not install npm packages or download browsers' "${GUI_PATTERN}" || \
  fail 'Svelte GUI guidance introduced a live dependency expectation'
rg -q 'Optional, non-authoritative working aid' "${WORKFLOW_MAP}" || \
  fail 'user workflow map authority boundary is missing'
rg -q 'It has no approval, lifecycle' "${WORKFLOW_MAP}" || \
  fail 'user workflow map creates approval or lifecycle obligations'
rg -q 'review, or traceability role' "${WORKFLOW_MAP}" || \
  fail 'user workflow map creates governance obligations'

printf 'guidance contract: PASS\n'
