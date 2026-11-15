#!/usr/bin/env bash
# User Learning policy v1 local-state guarantees (LEARNING WO-01 scenarios 1, 5, 6, 7).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
TOOL="${ROOT}/dba/04-tools/learning/codeos-learning.sh"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-learning.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'codeos-learning tests failed: %s\n' "$1" >&2; exit 1; }

git -C "${WORK}" init -q
printf '/.codeos-state/\n' > "${WORK}/.gitignore"
learn() { (cd "${WORK}" && bash "${TOOL}" "$@"); }
STATE="${WORK}/.codeos-state/user-learning.json"
JOURNAL="${WORK}/.codeos-state/learning-journal.md"

# A new project starts guided, with no state file needed.
grep -Fxq 'mode: guided' <<<"$(learn status)" || fail 'default mode is not guided'

# Progress keeps the highest stage reached.
learn record ownership predicted >/dev/null
learn record ownership introduced >/dev/null
[[ "$(jq -r '.concepts.ownership' "${STATE}")" == predicted ]] || fail 'record lowered a stage'
learn record ownership reproduced >/dev/null
[[ "$(jq -r '.concepts.ownership' "${STATE}")" == reproduced ]] || fail 'record did not raise a stage'

# Off and back on keeps progress.
learn set-mode off >/dev/null
grep -Fxq 'mode: off' <<<"$(learn status)" || fail 'set-mode off not applied'
learn set-mode guided >/dev/null
[[ "$(jq -r '.concepts.ownership' "${STATE}")" == reproduced ]] || fail 'switching modes lost progress'

# The journal appends question and answer; the state file holds no answer text.
learn journal ownership --question 'Which module owns X?' --answer 'the API module' \
  --correction 'the domain module owns X' >/dev/null
learn journal ownership --question 'Second question?' --answer 'second answer' >/dev/null
[[ "$(grep -c '^## ' "${JOURNAL}")" -eq 2 ]] || fail 'journal did not append two entries'
grep -Fq -- '- Correction: the domain module owns X' "${JOURNAL}" || fail 'journal lost the correction'
[[ "$(jq -c 'keys' "${STATE}")" == '["concepts","mode","schema"]' ]] || fail 'state has extra keys'
if grep -q 'answer\|mastered' "${STATE}"; then fail 'state holds answer text or a mastered label'; fi

# Neither file is tracked by Git.
[[ -z "$(git -C "${WORK}" status --porcelain --untracked-files=all -- .codeos-state)" ]] \
  || fail 'learning state is visible to Git'

# reset-progress clears progress and deletes the journal, but keeps the mode.
learn set-mode off >/dev/null
learn reset-progress >/dev/null
[[ "$(jq -c '.concepts' "${STATE}")" == '{}' && ! -e "${JOURNAL}" ]] || fail 'reset-progress incomplete'
[[ "$(jq -r '.mode' "${STATE}")" == off ]] || fail 'reset-progress changed the mode'

# Corrupt state never blocks: it reads as guided and exits 0.
printf 'not json' > "${STATE}"
grep -Fxq 'mode: guided' <<<"$(learn status 2>/dev/null)" || fail 'corrupt state did not read as guided'
learn record ownership introduced >/dev/null 2>&1 || fail 'corrupt state blocked record'

# Usage errors are rejected with exit 2.
set +e
learn set-mode mastered >/dev/null 2>&1; [[ $? -eq 2 ]] || fail 'invalid mode accepted'
learn record ownership mastered >/dev/null 2>&1; [[ $? -eq 2 ]] || fail 'mastered stage accepted'
learn journal ownership --question q >/dev/null 2>&1; [[ $? -eq 2 ]] || fail 'journal without answer accepted'
set -e

printf 'codeos-learning tests: PASS\n'
