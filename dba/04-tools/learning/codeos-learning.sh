#!/usr/bin/env bash
# codeos-learning.sh — project-local User Learning state (User Learning policy v1). It keeps the
# User Learning Mode and Learning Progress in .codeos-state/user-learning.json and the learning
# journal in .codeos-state/learning-journal.md, both untracked. It approves, verifies and blocks
# nothing: missing or corrupt state reads as guided with no progress.
#
# Usage:
#   codeos-learning.sh status
#   codeos-learning.sh set-mode guided|off
#   codeos-learning.sh record <concept-id> introduced|predicted|observed|reproduced
#   codeos-learning.sh journal <concept-id> --question <text> --answer <text> [--correction <text>]
#   codeos-learning.sh reset-progress
# Exit codes: 0 done (including a warning about unreadable state), 2 usage error.
set -euo pipefail

usage() { sed -n '8,13p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,3\}//' >&2; exit 2; }
warn() { printf 'codeos-learning: %s\n' "$1" >&2; }

ROOT="${CODEOS_LEARNING_ROOT:-$(git rev-parse --show-toplevel 2>/dev/null || pwd -P)}"
STATE_DIR="${ROOT}/.codeos-state"
STATE="${STATE_DIR}/user-learning.json"
JOURNAL="${STATE_DIR}/learning-journal.md"
STAGES=(introduced predicted observed reproduced)
EMPTY='{"schema":1,"mode":"guided","concepts":{}}'

# The current state, or the default when it is missing or unreadable.
load() {
  if [[ ! -f "${STATE}" ]]; then printf '%s\n' "${EMPTY}"; return; fi
  if jq -e '.schema == 1 and (.mode == "guided" or .mode == "off") and (.concepts | type == "object")' \
      "${STATE}" >/dev/null 2>&1; then
    jq -c '{schema, mode, concepts}' "${STATE}"
  else
    warn "unreadable ${STATE#"${ROOT}"/}; using guided with no progress"
    printf '%s\n' "${EMPTY}"
  fi
}

save() {
  mkdir -p "${STATE_DIR}"
  local tmp="${STATE}.tmp.$$"
  printf '%s\n' "$1" | jq '.' > "${tmp}"
  mv "${tmp}" "${STATE}"
}

stage_index() {
  local i
  for i in "${!STAGES[@]}"; do [[ "${STAGES[$i]}" == "$1" ]] && { printf '%s\n' "$i"; return; }; done
  return 1
}

concept_id() { [[ "$1" =~ ^[a-z0-9][a-z0-9._-]*$ ]] || { warn "invalid concept id '$1'"; usage; }; }

[[ $# -ge 1 ]] || usage
command="$1"; shift
case "${command}" in
  status)
    [[ $# -eq 0 ]] || usage
    state="$(load)"
    printf 'mode: %s\n' "$(jq -r '.mode' <<<"${state}")"
    printf 'concepts: %s\n' "$(jq -r '.concepts | length' <<<"${state}")"
    jq -r '.concepts | to_entries[] | "  \(.key): \(.value)"' <<<"${state}"
    if [[ -f "${JOURNAL}" ]]; then printf 'journal: %s\n' "${JOURNAL#"${ROOT}"/}"; fi
    ;;
  set-mode)
    [[ $# -eq 1 && ( "$1" == guided || "$1" == off ) ]] || usage
    save "$(jq -c --arg m "$1" '.mode = $m' <<<"$(load)")"
    printf 'mode: %s\n' "$1"
    ;;
  record)
    [[ $# -eq 2 ]] || usage
    concept_id "$1"
    new="$(stage_index "$2")" || usage
    state="$(load)"
    old_stage="$(jq -r --arg c "$1" '.concepts[$c] // ""' <<<"${state}")"
    if [[ -n "${old_stage}" ]] && old="$(stage_index "${old_stage}")" && (( old >= new )); then
      printf '%s: %s\n' "$1" "${old_stage}"
      exit 0
    fi
    save "$(jq -c --arg c "$1" --arg s "$2" '.concepts[$c] = $s' <<<"${state}")"
    printf '%s: %s\n' "$1" "$2"
    ;;
  journal)
    [[ $# -ge 1 ]] || usage
    concept="$1"; shift
    concept_id "${concept}"
    question='' answer='' correction=''
    while [[ $# -gt 0 ]]; do
      [[ $# -ge 2 ]] || usage
      case "$1" in
        --question) question="$2" ;;
        --answer) answer="$2" ;;
        --correction) correction="$2" ;;
        *) usage ;;
      esac
      shift 2
    done
    [[ -n "${question}" && -n "${answer}" ]] || usage
    mkdir -p "${STATE_DIR}"
    {
      printf '## %s — %s\n\n' "$(date +%Y-%m-%d)" "${concept}"
      printf -- '- Question: %s\n- Answer: %s\n' "${question}" "${answer}"
      [[ -n "${correction}" ]] && printf -- '- Correction: %s\n' "${correction}"
      printf '\n'
    } >> "${JOURNAL}"
    printf 'journal: %s\n' "${JOURNAL#"${ROOT}"/}"
    ;;
  reset-progress)
    [[ $# -eq 0 ]] || usage
    save "$(jq -c '.concepts = {}' <<<"$(load)")"
    rm -f "${JOURNAL}"
    printf 'progress and journal reset\n'
    ;;
  *) usage ;;
esac
