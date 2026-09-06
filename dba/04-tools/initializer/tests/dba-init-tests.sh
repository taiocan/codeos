#!/usr/bin/env bash
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-init.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT

fail() { printf 'initializer tests failed: %s\n' "$1" >&2; exit 1; }
run_init() { (cd "$1" && bash "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" "${2:-example}" >/dev/null); }

PROJECT="${WORK}/project"
mkdir -p "${PROJECT}"
run_init "${PROJECT}" 'a&b/c\d'

for path in \
  .codeos .codeos/toolkit .codeos/00-project/CLAUDE.md \
  .codeos/01-specification/intents .codeos/01-specification/contracts \
  .codeos/01-specification/event-schemas AGENTS.md CLAUDE.md .gitignore .git; do
  [[ -e "${PROJECT}/${path}" || -L "${PROJECT}/${path}" ]] || fail "initializer omitted ${path}"
done
[[ -d "${PROJECT}/.codeos" && ! -L "${PROJECT}/.codeos" ]] || fail '.codeos is not project-local'
[[ -L "${PROJECT}/.codeos/toolkit" ]] || fail 'toolkit mount is not a symlink'
[[ "$(cd "${PROJECT}/.codeos/toolkit" && pwd -P)" == "${CODEOS_ROOT}" ]] || fail 'toolkit mount resolves incorrectly'
grep -Fxq '/.codeos/toolkit' "${PROJECT}/.gitignore" || fail 'toolkit mount is not ignored'
grep -Fxq '/.codeos-state/' "${PROJECT}/.gitignore" || fail 'operational state is not ignored'
grep -Fxq '# a&b/c\d' "${PROJECT}/.codeos/00-project/CLAUDE.md" || fail 'project name substitution was unsafe'

for path in \
  .codeos/00-project/terminology.md .codeos/00-discovery .codeos/02-architecture \
  .codeos/04-refinement .codeos/05-review \
  .codeos-state features backlog modules tests docs events/runtime_events.jsonl; do
  [[ ! -e "${PROJECT}/${path}" ]] || fail "initializer created optional ${path}"
done

printf '\nProject-specific marker.\n' >> "${PROJECT}/.codeos/00-project/CLAUDE.md"
run_init "${PROJECT}" example
grep -Fq 'Project-specific marker.' "${PROJECT}/.codeos/00-project/CLAUDE.md" || fail 'rerun rewrote project instructions'

CONFLICT="${WORK}/instruction-conflict"
mkdir -p "${CONFLICT}"
printf 'project-owned instructions\n' > "${CONFLICT}/CLAUDE.md"
if run_init "${CONFLICT}" example 2>/dev/null; then fail 'ambiguous root instructions were accepted'; fi
[[ ! -e "${CONFLICT}/.codeos" ]] || fail 'instruction conflict mutated project'

LEGACY="${WORK}/legacy-mount"
mkdir -p "${LEGACY}"
ln -s "${CODEOS_ROOT}" "${LEGACY}/.codeos"
if run_init "${LEGACY}" example 2>/dev/null; then fail 'legacy .codeos mount was accepted'; fi
[[ -L "${LEGACY}/.codeos" ]] || fail 'legacy mount was rewritten'

BROKEN="${WORK}/broken-toolkit"
mkdir -p "${BROKEN}/.codeos"
ln -s "${BROKEN}/missing-toolkit" "${BROKEN}/.codeos/toolkit"
if run_init "${BROKEN}" example 2>/dev/null; then fail 'broken toolkit mount was accepted'; fi
[[ ! -e "${BROKEN}/.codeos/00-project" ]] || fail 'broken mount mutated project'

PARENT="${WORK}/parent"
mkdir -p "${PARENT}/nested"
git -C "${PARENT}" init -q -b main
if run_init "${PARENT}/nested" example 2>/dev/null; then fail 'nested repository initialization was accepted'; fi
[[ ! -e "${PARENT}/nested/.codeos" ]] || fail 'nested repository rejection mutated project'

SOURCE="${WORK}/worktree-source"
WORKTREE="${WORK}/linked-worktree"
mkdir -p "${SOURCE}"
git -C "${SOURCE}" init -q -b main
git -C "${SOURCE}" config user.email test@example.com
git -C "${SOURCE}" config user.name Test
printf 'seed\n' > "${SOURCE}/seed.txt"
git -C "${SOURCE}" add seed.txt
git -C "${SOURCE}" commit -qm seed
git -C "${SOURCE}" worktree add -q -b linked "${WORKTREE}"
run_init "${WORKTREE}" worktree
[[ -f "${WORKTREE}/.git" ]] || fail 'linked worktree marker was replaced'

# Platform Baseline gating: dba-init.sh must scaffold the skeleton only when the active
# configuration carries codeos_mechanics_policy (DBA-5+), never for DBA-4. A fake toolkit root is
# required because dba-init.sh derives its own toolkit path from its script location, not an env
# var, so the real active pointer (DBA-4) cannot be exercised for the "true" branch in place.
# A Platform-Baseline fake toolkit selecting codeos-mechanics <mechver>. dba-init.sh renders the
# fixed mechanics: block from the selected policy, so the policy and the renderer must be present.
mk_baseline_toolkit() {
  local toolkit="$1" mechver="$2"
  mkdir -p "${toolkit}/dba/00-entry/configurations" \
    "${toolkit}/dba/04-tools/initializer" \
    "${toolkit}/dba/04-tools/configuration" \
    "${toolkit}/dba/02-policies/codeos-mechanics" \
    "${toolkit}/dba/05-guidance/templates"
  cp "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" "${toolkit}/dba/04-tools/initializer/"
  cp -r "${CODEOS_ROOT}/dba/04-tools/initializer/skeleton" "${toolkit}/dba/04-tools/initializer/"
  cp "${CODEOS_ROOT}/dba/04-tools/configuration/render-mechanics-block.sh" \
    "${toolkit}/dba/04-tools/configuration/"
  cp "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/${mechver}.md" \
    "${toolkit}/dba/02-policies/codeos-mechanics/${mechver}.md"
  cp "${CODEOS_ROOT}/dba/05-guidance/templates/project-CLAUDE.md" \
    "${CODEOS_ROOT}/dba/05-guidance/templates/project-root-CLAUDE.md" \
    "${CODEOS_ROOT}/dba/05-guidance/templates/project-AGENTS.md" \
    "${CODEOS_ROOT}/dba/05-guidance/templates/codeos.yaml" \
    "${toolkit}/dba/05-guidance/templates/"
  printf 'codeos_mechanics_policy: dba/02-policies/codeos-mechanics/%s.md\n' "${mechver}" \
    > "${toolkit}/dba/00-entry/configurations/DBA-TEST.yaml"
  printf 'Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-TEST.yaml`\n' \
    > "${toolkit}/dba-system.md"
}

FAKE_TOOLKIT="${WORK}/fake-toolkit"
mk_baseline_toolkit "${FAKE_TOOLKIT}" v2

run_fake_init() { (cd "$1" && bash "${FAKE_TOOLKIT}/dba/04-tools/initializer/dba-init.sh" "${2:-example}" >/dev/null); }

BASELINE_PROJECT="${WORK}/baseline-project"
mkdir -p "${BASELINE_PROJECT}"
run_fake_init "${BASELINE_PROJECT}" baseline-example

for path in backend web docker-compose.yml PLATFORM-BASELINE.md .codeos/00-project/codeos.yaml; do
  [[ -e "${BASELINE_PROJECT}/${path}" ]] || fail "Platform Baseline config omitted ${path}"
done
grep -Fxq '  charter: governed' "${BASELINE_PROJECT}/.codeos/00-project/codeos.yaml" \
  || fail 'copied codeos.yaml is missing locked core-four governance'

# A fresh DBA-6 (v2-selecting) initialization emits the EXACT v2 mechanics set.
V2_EMITTED="$(sed -n '/^mechanics:/,/^[a-z]/p' "${BASELINE_PROJECT}/.codeos/00-project/codeos.yaml" \
  | sed '/^artifacts:/d' | grep -vE '^\s*#|^\s*$')"
V2_EXPECTED="$(bash "${CODEOS_ROOT}/dba/04-tools/configuration/render-mechanics-block.sh" \
  "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/v2.md" | grep -vE '^\s*#|^\s*$')"
[[ -n "${V2_EMITTED}" && "${V2_EMITTED}" == "${V2_EXPECTED}" ]] \
  || fail 'a fresh DBA-6 initialization does not emit the exact v2 mechanics set'
grep -Fxq '    data_integrity: always_when_persistence' "${BASELINE_PROJECT}/.codeos/00-project/codeos.yaml" \
  || fail 'a fresh DBA-6 initialization omits the v2 data_integrity mechanic'

# An explicit DBA-5 (v1-selecting) initialization emits the EXACT v1 set — no data_integrity.
V1_TOOLKIT="${WORK}/fake-toolkit-v1"
mk_baseline_toolkit "${V1_TOOLKIT}" v1
V1_PROJECT="${WORK}/v1-project"
mkdir -p "${V1_PROJECT}"
(cd "${V1_PROJECT}" && bash "${V1_TOOLKIT}/dba/04-tools/initializer/dba-init.sh" v1-example >/dev/null)
V1_EMITTED="$(sed -n '/^mechanics:/,/^[a-z]/p' "${V1_PROJECT}/.codeos/00-project/codeos.yaml" \
  | sed '/^artifacts:/d' | grep -vE '^\s*#|^\s*$')"
V1_EXPECTED="$(bash "${CODEOS_ROOT}/dba/04-tools/configuration/render-mechanics-block.sh" \
  "${CODEOS_ROOT}/dba/02-policies/codeos-mechanics/v1.md" | grep -vE '^\s*#|^\s*$')"
[[ -n "${V1_EMITTED}" && "${V1_EMITTED}" == "${V1_EXPECTED}" ]] \
  || fail 'an explicit DBA-5 initialization does not emit the exact v1 mechanics set'
grep -Fq 'data_integrity' "${V1_PROJECT}/.codeos/00-project/codeos.yaml" \
  && fail 'an explicit DBA-5 initialization emitted the v2-only data_integrity mechanic'

printf 'local edit\n' >> "${BASELINE_PROJECT}/backend/README.md"
run_fake_init "${BASELINE_PROJECT}" baseline-example
grep -Fq 'local edit' "${BASELINE_PROJECT}/backend/README.md" \
  || fail 'rerun overwrote an existing Platform Baseline skeleton'

# A DBA-4-shaped fixture (no codeos_mechanics_policy key) must not scaffold the Platform Baseline
# — mirrors BASELINE_PROJECT's fixture above rather than relying on whichever configuration this
# repo's own active pointer happens to name, which changes over time.
FAKE_TOOLKIT_DBA4="${WORK}/fake-toolkit-dba4"
mkdir -p "${FAKE_TOOLKIT_DBA4}/dba/00-entry/configurations" \
  "${FAKE_TOOLKIT_DBA4}/dba/04-tools/initializer" \
  "${FAKE_TOOLKIT_DBA4}/dba/05-guidance/templates"
cp "${CODEOS_ROOT}/dba/04-tools/initializer/dba-init.sh" "${FAKE_TOOLKIT_DBA4}/dba/04-tools/initializer/"
cp -r "${CODEOS_ROOT}/dba/04-tools/initializer/skeleton" "${FAKE_TOOLKIT_DBA4}/dba/04-tools/initializer/"
cp "${CODEOS_ROOT}/dba/05-guidance/templates/project-CLAUDE.md" \
  "${CODEOS_ROOT}/dba/05-guidance/templates/project-root-CLAUDE.md" \
  "${CODEOS_ROOT}/dba/05-guidance/templates/project-AGENTS.md" \
  "${CODEOS_ROOT}/dba/05-guidance/templates/codeos.yaml" \
  "${FAKE_TOOLKIT_DBA4}/dba/05-guidance/templates/"
printf 'doctrine: dba/01-doctrine/v4.md\n' \
  > "${FAKE_TOOLKIT_DBA4}/dba/00-entry/configurations/DBA-4-TEST.yaml"
printf 'Active configuration: `.codeos/toolkit/dba/00-entry/configurations/DBA-4-TEST.yaml`\n' \
  > "${FAKE_TOOLKIT_DBA4}/dba-system.md"

DBA4_PROJECT="${WORK}/dba4-project"
mkdir -p "${DBA4_PROJECT}"
(cd "${DBA4_PROJECT}" && bash "${FAKE_TOOLKIT_DBA4}/dba/04-tools/initializer/dba-init.sh" dba4-example >/dev/null)
for path in backend web docker-compose.yml PLATFORM-BASELINE.md .codeos/00-project/codeos.yaml; do
  [[ ! -e "${DBA4_PROJECT}/${path}" ]] || fail "DBA-4 initialization created Platform Baseline ${path}"
done

printf 'initializer tests: PASS\n'
