#!/usr/bin/env bash
# Verify the registry-free guidance and minimum initializer contract.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../../.." && pwd -P)"
TEST_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/codeos-lean-guidance.XXXXXX")"
trap 'rm -rf "${TEST_ROOT}"' EXIT

fail() {
  printf 'Lean guidance contract failed: %s\n' "$1" >&2
  exit 1
}

expected_guidance=(
  patterns/rust-project-structure.md
  patterns/shared-infrastructure-boundary.md
  patterns/vocabulary-architecture.md
  templates/architecture-scope.md
  templates/contract.md
  templates/event-schema.md
  templates/feature-brief.md
  templates/implementation-profile.yaml
  templates/intent.md
  templates/project-AGENTS.md
  templates/project-CLAUDE.md
  templates/project-root-CLAUDE.md
  templates/refinement.md
  templates/review-file.md
  templates/review-package.md
  templates/reviewer.toml
  terminology.md
)

mapfile -t actual_guidance < <(
  cd "${CODEOS_ROOT}/dba/05-guidance"
  find . -type f -printf '%P\n' | sort
)
[[ "${actual_guidance[*]}" == "${expected_guidance[*]}" ]] || fail "unexpected active guidance inventory"

PROJECT_ROOT="${TEST_ROOT}/project"
mkdir -p "${PROJECT_ROOT}"
(
  cd "${PROJECT_ROOT}"
  bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" example >/dev/null
)

for path in \
  .codeos .codeos/toolkit .codeos/00-project/CLAUDE.md \
  .codeos/01-specification/intents .codeos/01-specification/contracts \
  .codeos/01-specification/event-schemas AGENTS.md CLAUDE.md .gitignore .git; do
  [[ -e "${PROJECT_ROOT}/${path}" || -L "${PROJECT_ROOT}/${path}" ]] || fail "initializer omitted ${path}"
done

[[ -d "${PROJECT_ROOT}/.codeos" && ! -L "${PROJECT_ROOT}/.codeos" ]] || \
  fail ".codeos is not a real project-local directory"
[[ -L "${PROJECT_ROOT}/.codeos/toolkit" ]] || fail "toolkit mount is not a symlink"
[[ -f "${PROJECT_ROOT}/.codeos/toolkit/dba-system.md" ]] || fail "toolkit mount is invalid"
grep -Fxq '/.codeos/toolkit' "${PROJECT_ROOT}/.gitignore" || fail "toolkit mount is not ignored"
grep -Fxq '/.codeos-state/' "${PROJECT_ROOT}/.gitignore" || fail "operational state is not ignored"
cmp -s "${PROJECT_ROOT}/CLAUDE.md" \
  "${CODEOS_ROOT}/dba/05-guidance/templates/project-root-CLAUDE.md" || \
  fail "root CLAUDE.md is not the discovery adapter"

for path in \
  .codeos/00-discovery .codeos/02-architecture .codeos/04-refinement .codeos/05-review \
  .codeos-state features backlog modules tests docs events/runtime_events.jsonl; do
  [[ ! -e "${PROJECT_ROOT}/${path}" ]] || fail "initializer created optional ${path}"
done

# Re-running initialization preserves the canonical instruction file and valid toolkit mount.
printf '\nProject-specific marker.\n' >> "${PROJECT_ROOT}/.codeos/00-project/CLAUDE.md"
(
  cd "${PROJECT_ROOT}"
  bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" example >/dev/null
)
grep -Fq 'Project-specific marker.' "${PROJECT_ROOT}/.codeos/00-project/CLAUDE.md" || \
  fail "initializer rewrote canonical project instructions"

# Existing non-adapter instructions and legacy mounts stop without mutation.
CONFLICT_ROOT="${TEST_ROOT}/instruction-conflict"
mkdir -p "${CONFLICT_ROOT}"
printf 'project-owned instructions\n' > "${CONFLICT_ROOT}/CLAUDE.md"
if (cd "${CONFLICT_ROOT}" && bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" example >/dev/null 2>&1); then
  fail "initializer accepted ambiguous root instructions"
fi
[[ ! -e "${CONFLICT_ROOT}/.codeos" ]] || fail "instruction conflict mutated project"

LEGACY_ROOT="${TEST_ROOT}/legacy-mount"
mkdir -p "${LEGACY_ROOT}"
ln -s "${CODEOS_ROOT}" "${LEGACY_ROOT}/.codeos"
if (cd "${LEGACY_ROOT}" && bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" example >/dev/null 2>&1); then
  fail "initializer accepted a legacy .codeos mount"
fi
[[ -L "${LEGACY_ROOT}/.codeos" ]] || fail "initializer rewrote legacy mount"

BROKEN_ROOT="${TEST_ROOT}/broken-toolkit"
mkdir -p "${BROKEN_ROOT}/.codeos"
ln -s "${BROKEN_ROOT}/missing-toolkit" "${BROKEN_ROOT}/.codeos/toolkit"
if (cd "${BROKEN_ROOT}" && bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" example >/dev/null 2>&1); then
  fail "initializer accepted a broken toolkit mount"
fi
[[ ! -e "${BROKEN_ROOT}/.codeos/00-project" ]] || fail "broken mount mutated project"

SESSION_START="${CODEOS_ROOT}/dba/03-prompts/workflow/00-session-start.md"
INTENT_PROMPT="${CODEOS_ROOT}/dba/03-prompts/workflow/01-intent.md"
rg -q 'partially drafted Specification Package is normal' "${SESSION_START}" || fail "partial packages are not accepted"
rg -q 'architecture-scope membership' "${INTENT_PROMPT}" || \
  fail "feature-id allocation does not scan every identity owner"
rg -q 'incompatible artifacts claim the same identity' "${INTENT_PROMPT}" || \
  fail "identity-conflict boundary is missing"

printf 'Lean guidance contract OK\n'
