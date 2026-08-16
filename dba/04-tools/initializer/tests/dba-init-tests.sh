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

printf 'initializer tests: PASS\n'
