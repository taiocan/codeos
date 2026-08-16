#!/usr/bin/env bash
# Verify the intentionally small active guidance inventory and its owned workflow boundaries.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd -P)"
fail() { printf 'guidance contract failed: %s\n' "$1" >&2; exit 1; }

expected_guidance=(
  patterns/rust-project-structure.md
  patterns/shared-infrastructure-boundary.md
  patterns/vocabulary-architecture.md
  templates/architecture-scope.md
  templates/contract.md
  templates/event-schema.md
  templates/feature-brief.md
  templates/full-solution-concept.md
  templates/implementation-profile.yaml
  templates/intent.md
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

SESSION_START="${CODEOS_ROOT}/dba/03-prompts/workflow/00-session-start.md"
INTENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/01-intent.md"
rg -q 'partially drafted Specification Package is normal' "${SESSION_START}" || fail 'partial packages are not accepted'
rg -q 'terminology.md.*exists' "${SESSION_START}" || fail 'optional project terminology is not loaded'
rg -q 'Its absence is valid' "${SESSION_START}" || fail 'project terminology became mandatory'
rg -q 'architecture-scope membership' "${INTENT_PROMPT}" || fail 'feature-id allocation does not scan every identity owner'
rg -q 'incompatible artifacts claim the same identity' "${INTENT_PROMPT}" || fail 'identity-conflict boundary is missing'
rg -q 'Define only feature-local' "${INTENT_PROMPT}" || fail 'Intent terminology ownership is unclear'

printf 'guidance contract: PASS\n'
