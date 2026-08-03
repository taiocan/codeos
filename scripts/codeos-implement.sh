#!/usr/bin/env bash
# codeos-implement.sh — out-of-band, opt-in DeepSeek implementer for DBA Stage 4 (Implementation)
# and Stage 5 (Tests). It drafts a CANDIDATE only: output is staged under
# .codeos-state/deepseek-candidates/, never written into modules/ or tests/, and never committed.
# A human promotes the candidate, and the existing Stage 4/5 human gate, advisory review, and
# Stage 7 reconciliation apply unchanged. This tool approves nothing.
#
# Companion to scripts/codeos-review.sh: it mirrors that shim's entry-point discipline (git-repo
# precondition, self-dev-vs-downstream context resolution, fail-closed preconditions, an explicit
# exit-code table). It is off by default — see the activation status file below.
#
# Usage:
#   codeos-implement.sh <feature_id> <stage:4|5> <artifact-path> [more artifact-paths...]
#     <feature_id>     the DBA feature id (used only to name the staging directory)
#     <stage>          4 (implementation) or 5 (tests) — no other value is accepted
#     <artifact-path>  one or more approved artifacts (intent / contract / event schema; plus the
#                      Stage 4 output when stage=5). Each must exist.
#
# Activation (Optional Mechanism Status Convention, UPG-0056): a one-line status file, resolved by
# context the same way codeos-review.sh resolves its writing-discipline file —
#   self-dev    (caller git root == this repo): config/delegated-implementation.yaml
#   downstream  (caller git root != this repo): architecture/delegated-implementation.yaml
# Four outcomes: absent -> disabled; exact "status: disabled" -> disabled; exact "status: enabled"
# -> enabled; anything else -> configuration error. The tool runs ONLY when the value is enabled.
#
# Environment:
#   DEEPSEEK_API_KEY        required when enabled. Read only into the HTTP Authorization header via a
#                           curl config on stdin — never placed in argv, the request body, the
#                           preserved packet, the response dump, or any candidate file.
#   CODEOS_DEEPSEEK_MODEL   optional, default "deepseek-chat".
#   CODEOS_DEEPSEEK_URL     optional, default "https://api.deepseek.com/chat/completions".
#
# Exit codes:
#   0  success — candidate staged
#   1  not inside a git repository
#   2  missing dependency (curl or jq not found on PATH)
#   3  usage error (missing args, or stage not 4/5)
#   4  mechanism disabled or status file absent (refuse to run)
#   5  activation status file malformed (configuration error)
#   6  DEEPSEEK_API_KEY unset or empty (while enabled) — refuse before any network call
#   7  a passed artifact path does not exist
#   8  DeepSeek API / transport error, or an unparseable / unsafe model response
set -euo pipefail

err() { echo "error: $*" >&2; }

# ── 1. git-repo precondition (property of the calling project, checked first) ───────────────────
git rev-parse --show-toplevel >/dev/null 2>&1 || { err "not inside a git repository"; exit 1; }

# ── 3. usage / args ─────────────────────────────────────────────────────────────────────────────
if [[ $# -lt 3 ]]; then
  err "usage: codeos-implement.sh <feature_id> <stage:4|5> <artifact-path> [more...]"
  exit 3
fi
FEATURE="$1"; STAGE="$2"; shift 2
ARTIFACTS=("$@")
if [[ "${STAGE}" != "4" && "${STAGE}" != "5" ]]; then
  err "stage must be 4 (implementation) or 5 (tests); got '${STAGE}'"
  exit 3
fi
# Candidate files must stay in the stage's area (Stage 4 -> modules/, Stage 5 -> tests/). This
# backs the "strict path" contract stated in prompts/codeos-implementer-task.md: the model is
# instructed, and the tool enforces, so a doctrine/config/governance path can never be staged.
case "${STAGE}" in
  4) ALLOWED_PREFIX="modules/";;
  5) ALLOWED_PREFIX="tests/";;
esac

# ── 2. dependencies ─────────────────────────────────────────────────────────────────────────────
for dep in curl jq; do
  command -v "${dep}" >/dev/null 2>&1 || { err "required dependency '${dep}' not found on PATH"; exit 2; }
done

# ── Resolve this script's repo (CODEOS_ROOT) and the caller's repo (CALLER_ROOT) ────────────────
# pwd -P (physical) follows the .codeos symlink when invoked from a downstream project, matching
# scripts/codeos-review.sh. The caller's own git root distinguishes self-dev from downstream.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"
CALLER_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd -P)"

if [[ "${CALLER_ROOT}" == "${CODEOS_ROOT}" ]]; then
  STATUS_FILE="${CODEOS_ROOT}/config/delegated-implementation.yaml"
else
  STATUS_FILE="${CALLER_ROOT}/architecture/delegated-implementation.yaml"
fi

# ── 4/5. activation status (four-outcome, per UPG-0056; parse identical to codeos-review.sh) ─────
STATUS=""
if [[ ! -f "${STATUS_FILE}" ]]; then
  STATUS="disabled"
else
  NONBLANK=()
  while IFS= read -r _line || [[ -n "${_line}" ]]; do
    [[ -n "${_line}" ]] && NONBLANK+=("${_line}")
  done < <(tr -d '\r' < "${STATUS_FILE}")
  if [[ "${#NONBLANK[@]}" -eq 1 && "${NONBLANK[0]}" == "status: disabled" ]]; then
    STATUS="disabled"
  elif [[ "${#NONBLANK[@]}" -eq 1 && "${NONBLANK[0]}" == "status: enabled" ]]; then
    STATUS="enabled"
  else
    err "invalid activation status file: ${STATUS_FILE}"
    err "       must contain exactly one non-blank line: 'status: enabled' or 'status: disabled'"
    exit 5
  fi
fi
if [[ "${STATUS}" != "enabled" ]]; then
  err "delegated implementation is disabled (status file: ${STATUS_FILE})."
  err "       This mechanism is off by default. Set it to 'status: enabled' to opt in."
  exit 4
fi

# ── 7. artifacts must exist ─────────────────────────────────────────────────────────────────────
for a in "${ARTIFACTS[@]}"; do
  [[ -f "${a}" ]] || { err "artifact path does not exist: ${a}"; exit 7; }
done

# ── 6. API key present (only checked once enabled; before any network call) ─────────────────────
if [[ -z "${DEEPSEEK_API_KEY:-}" ]]; then
  err "DEEPSEEK_API_KEY is unset or empty; refusing to run (no network call made)."
  exit 6
fi

MODEL="${CODEOS_DEEPSEEK_MODEL:-deepseek-chat}"
DS_URL="${CODEOS_DEEPSEEK_URL:-https://api.deepseek.com/chat/completions}"
TASK_PROMPT="${CODEOS_ROOT}/prompts/codeos-implementer-task.md"
[[ -f "${TASK_PROMPT}" ]] || { err "implementer task prompt not found: ${TASK_PROMPT}"; exit 8; }

# ── Staging directory (timestamped -> idempotent across re-runs) ─────────────────────────────────
TS="$(date -u +%Y%m%dT%H%M%SZ)"
FEATURE_SAFE="${FEATURE//\//_}"
STAGE_ROOT="${CALLER_ROOT}/.codeos-state/deepseek-candidates"
mkdir -p "${STAGE_ROOT}/${FEATURE_SAFE}-stage-${STAGE}"
# mktemp guarantees a unique leaf even on rapid same-second re-runs, so a run never overwrites a
# prior run's audit artifacts.
STAGE_DIR="$(mktemp -d "${STAGE_ROOT}/${FEATURE_SAFE}-stage-${STAGE}/${TS}.XXXXXX")"
mkdir -p "${STAGE_DIR}/candidate"
LOG_FILE="${STAGE_ROOT}/implement-log.md"

# ── Build the two message contents (system = role/contract; user = task + artifacts) ────────────
SYS="$(cat "${TASK_PROMPT}")"
{
  printf 'DELEGATED IMPLEMENTATION REQUEST\n'
  printf '  feature_id: %s\n' "${FEATURE}"
  printf '  stage: %s (%s)\n\n' "${STAGE}" "$([[ ${STAGE} == 4 ]] && echo implementation || echo tests)"
  printf 'Produce the Stage %s candidate for this feature, following the STRICT output contract in\n' "${STAGE}"
  printf 'the task above. Approved artifacts follow.\n'
  for a in "${ARTIFACTS[@]}"; do
    printf '\n--- APPROVED ARTIFACT: %s ---\n' "${a}"
    cat "${a}"
  done
} > "${STAGE_DIR}/user_content.txt"
USR="$(cat "${STAGE_DIR}/user_content.txt")"

# Human-readable audit packet (contains NO key — the key is only ever in the Authorization header).
{
  printf '=== SYSTEM ===\n%s\n\n=== USER ===\n%s\n' "${SYS}" "${USR}"
} > "${STAGE_DIR}/packet.txt"

# JSON request body (jq escapes everything; body carries NO key).
REQ_BODY="${STAGE_DIR}/request.json"
jq -n --arg model "${MODEL}" --arg sys "${SYS}" --arg usr "${USR}" \
  '{model:$model,
    messages:[{role:"system",content:$sys},{role:"user",content:$usr}],
    response_format:{type:"json_object"},
    temperature:0,
    stream:false}' > "${REQ_BODY}"

# ── 8. Call DeepSeek. Key is passed via a curl config on stdin (never argv / body / files). ──────
RESP_FILE="${STAGE_DIR}/response.json"
HTTP_CODE="$(
  printf 'header = "Authorization: Bearer %s"\n' "${DEEPSEEK_API_KEY}" \
    | curl -sS -K - \
        -H 'Content-Type: application/json' \
        -X POST "${DS_URL}" \
        --data-binary @"${REQ_BODY}" \
        -o "${RESP_FILE}" \
        -w '%{http_code}' \
    || true
)"
if [[ ! "${HTTP_CODE}" =~ ^2[0-9][0-9]$ ]]; then
  err "DeepSeek API call failed (http_code=${HTTP_CODE:-none}); response saved to ${RESP_FILE}"
  exit 8
fi

# Extract the model's JSON object (json_object mode guarantees the content is valid JSON).
CONTENT_JSON="${STAGE_DIR}/model_content.json"
jq -r '.choices[0].message.content // empty' "${RESP_FILE}" > "${CONTENT_JSON}"
if [[ ! -s "${CONTENT_JSON}" ]] || ! jq empty "${CONTENT_JSON}" >/dev/null 2>&1; then
  err "model did not return a parseable JSON object; see ${RESP_FILE}"
  exit 8
fi

NFILES="$(jq '(.files // []) | length' "${CONTENT_JSON}")"
if [[ "${NFILES}" -lt 1 ]]; then
  err "model response contained no candidate files; see ${CONTENT_JSON}"
  exit 8
fi

# Write candidate files into the staging area only, with path-traversal protection.
for ((i=0; i<NFILES; i++)); do
  fpath="$(jq -r ".files[${i}].path // empty" "${CONTENT_JSON}")"
  if [[ -z "${fpath}" ]]; then err "candidate file ${i} has no path"; exit 8; fi
  case "${fpath}" in
    /*|*..*) err "unsafe candidate path rejected (absolute or traversal): ${fpath}"; exit 8;;
  esac
  # Enforce the stage-area allowlist, except for the documented CANDIDATE_BLOCKED.md escape hatch
  # the model uses to report insufficient artifacts.
  if [[ "${fpath}" != "CANDIDATE_BLOCKED.md" && "${fpath}" != "${ALLOWED_PREFIX}"* ]]; then
    err "candidate path outside the Stage ${STAGE} area (must start with '${ALLOWED_PREFIX}'): ${fpath}"
    exit 8
  fi
  outfile="${STAGE_DIR}/candidate/${fpath}"
  mkdir -p "$(dirname "${outfile}")"
  jq -r ".files[${i}].content // \"\"" "${CONTENT_JSON}" > "${outfile}"
done

# Sidecar summaries for the human reviewer.
jq -r '.contract_satisfaction // ""' "${CONTENT_JSON}" > "${STAGE_DIR}/contract_satisfaction.txt" || true
jq -r '.event_emission // ""'        "${CONTENT_JSON}" > "${STAGE_DIR}/event_emission.txt" || true
jq -r '.notes // ""'                 "${CONTENT_JSON}" > "${STAGE_DIR}/notes.txt" || true

# Token usage.
PT="$(jq -r '.usage.prompt_tokens // "?"' "${RESP_FILE}")"
CT="$(jq -r '.usage.completion_tokens // "?"' "${RESP_FILE}")"
TT="$(jq -r '.usage.total_tokens // "?"' "${RESP_FILE}")"
printf 'prompt_tokens=%s completion_tokens=%s total_tokens=%s model=%s\n' "${PT}" "${CT}" "${TT}" "${MODEL}" \
  > "${STAGE_DIR}/tokens.txt"

# Append one invocation record to the log.
{
  printf '## %s — %s stage %s\n' "${TS}" "${FEATURE}" "${STAGE}"
  printf -- '- model: %s\n' "${MODEL}"
  printf -- '- tokens: prompt=%s completion=%s total=%s\n' "${PT}" "${CT}" "${TT}"
  printf -- '- candidate files: %s\n' "${NFILES}"
  printf -- '- staging: %s\n\n' "${STAGE_DIR}"
} >> "${LOG_FILE}"

echo "candidate staged: ${STAGE_DIR}/candidate  (${NFILES} file(s))"
echo "  tokens: prompt=${PT} completion=${CT} total=${TT}   model=${MODEL}"
echo "  audit:  packet=${STAGE_DIR}/packet.txt  response=${RESP_FILE}  log=${LOG_FILE}"
echo "  NOTE: candidate only — promote manually; the Stage ${STAGE} human gate + advisory review still apply."
