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
#   codeos-implement.sh [options] <feature_id> <stage:4|5> <artifact-path> [more artifact-paths...]
#     <feature_id>     the DBA feature id (used only to name the staging directory)
#     <stage>          4 (implementation) or 5 (tests) — no other value is accepted
#     <artifact-path>  one or more approved artifacts (intent / contract / event schema; plus the
#                      Stage 4 output when stage=5). Each must exist.
#
#   Options (each repeatable, each must precede the positional arguments):
#     --exemplar PATH          a real file from the target repository shown as a LAYOUT EXEMPLAR:
#                              context demonstrating module naming/placement conventions, explicitly
#                              not a specification to implement. Must exist.
#     --repair-candidate PATH  a file from a previous candidate, shown as a PRIOR ATTEMPT.
#     --repair-output PATH     build/test output produced by that prior attempt, shown as FEEDBACK.
#
#   This tool NEVER runs a build, test, compile, or package-manager command, never eval's, and never
#   shells out to a project-supplied command. Build output is an INPUT the caller supplies; obtaining
#   it is the caller's explicit, external step. The processes this script starts are exactly:
#     git, curl, jq, awk, sed, cat, tr, od, head, date, mkdir, mktemp, rmdir, dirname
#   (awk runs the output-frame parser and sed extracts line ranges — both operate only on the model's
#   own reply). scripts/tests/codeos-implement-tests.sh scans this script against that list and fails
#   if any external tool outside it appears, so this comment cannot silently drift from the code.
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
#   8  DeepSeek API / transport error, or an unsafe candidate path in the model response
#   9  a passed --exemplar path does not exist
#  10  a passed --repair-candidate / --repair-output path does not exist
#  11  the model response violates the delimited output protocol (malformed frame) — nothing staged
set -euo pipefail

err() { echo "error: $*" >&2; }

# ── 1. git-repo precondition (property of the calling project, checked first) ───────────────────
git rev-parse --show-toplevel >/dev/null 2>&1 || { err "not inside a git repository"; exit 1; }

# ── 3. usage / args ─────────────────────────────────────────────────────────────────────────────
EXEMPLARS=()
REPAIR_CANDIDATES=()
REPAIR_OUTPUTS=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --exemplar)          [[ $# -ge 2 ]] || { err "--exemplar requires a path"; exit 3; }
                         EXEMPLARS+=("$2"); shift 2;;
    --repair-candidate)  [[ $# -ge 2 ]] || { err "--repair-candidate requires a path"; exit 3; }
                         REPAIR_CANDIDATES+=("$2"); shift 2;;
    --repair-output)     [[ $# -ge 2 ]] || { err "--repair-output requires a path"; exit 3; }
                         REPAIR_OUTPUTS+=("$2"); shift 2;;
    --)                  shift; break;;
    -*)                  err "unknown option: $1"; exit 3;;
    *)                   break;;
  esac
done
if [[ $# -lt 3 ]]; then
  err "usage: codeos-implement.sh [--exemplar PATH] [--repair-candidate PATH] [--repair-output PATH]"
  err "                           <feature_id> <stage:4|5> <artifact-path> [more...]"
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

# ── 7/9/10. every supplied input must exist (distinct code per input kind) ──────────────────────
for a in "${ARTIFACTS[@]}"; do
  [[ -f "${a}" ]] || { err "artifact path does not exist: ${a}"; exit 7; }
done
for e in ${EXEMPLARS[@]+"${EXEMPLARS[@]}"}; do
  [[ -f "${e}" ]] || { err "exemplar path does not exist: ${e}"; exit 9; }
done
for r in ${REPAIR_CANDIDATES[@]+"${REPAIR_CANDIDATES[@]}"} ${REPAIR_OUTPUTS[@]+"${REPAIR_OUTPUTS[@]}"}; do
  [[ -f "${r}" ]] || { err "repair-input path does not exist: ${r}"; exit 10; }
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

# Fresh per-run nonce for the output protocol. Random so that candidate content can never collide
# with a marker by accident; the parser accepts a marker ONLY with this exact value.
NONCE="$(head -c 8 /dev/urandom | od -An -tx1 | tr -d ' \n')"
[[ -n "${NONCE}" ]] || { err "could not generate an output nonce"; exit 8; }

{
  printf 'DELEGATED IMPLEMENTATION REQUEST\n'
  printf '  feature_id: %s\n' "${FEATURE}"
  printf '  stage: %s (%s)\n' "${STAGE}" "$([[ ${STAGE} == 4 ]] && echo implementation || echo tests)"
  printf '  output_nonce: %s\n\n' "${NONCE}"
  printf 'Produce the Stage %s candidate for this feature, following the STRICT output contract in\n' "${STAGE}"
  printf 'the task above. Use the output_nonce above verbatim in every marker.\n'
  for a in "${ARTIFACTS[@]}"; do
    printf '\n--- APPROVED ARTIFACT: %s ---\n' "${a}"
    cat "${a}"
  done
  # Layout exemplars are context, never specification. They are labeled distinctly from approved
  # artifacts so they can never be mistaken for something to implement.
  for e in ${EXEMPLARS[@]+"${EXEMPLARS[@]}"}; do
    printf '\n--- LAYOUT EXEMPLAR (context only — shows this repository'"'"'s conventions; do NOT implement,\n'
    printf '    modify, or copy the domain behavior of this file): %s ---\n' "${e}"
    cat "${e}"
  done
  # A prior attempt plus the build/test output it produced. The tool does not run any build — this
  # output was obtained by the caller as an explicit, external step.
  if [[ ${#REPAIR_CANDIDATES[@]} -gt 0 || ${#REPAIR_OUTPUTS[@]} -gt 0 ]]; then
    printf '\n=== REPAIR REQUEST — this is a retry of a previous attempt ===\n'
    printf 'Fix what the feedback below reports, then re-emit the COMPLETE candidate (every file, in\n'
    printf 'full), not a patch. Do not drop a contract invariant to make an error go away.\n'
    for r in ${REPAIR_CANDIDATES[@]+"${REPAIR_CANDIDATES[@]}"}; do
      printf '\n--- PRIOR ATTEMPT (your previous candidate): %s ---\n' "${r}"
      cat "${r}"
    done
    for r in ${REPAIR_OUTPUTS[@]+"${REPAIR_OUTPUTS[@]}"}; do
      printf '\n--- FEEDBACK (build/test output from that prior attempt): %s ---\n' "${r}"
      cat "${r}"
    done
  fi
} > "${STAGE_DIR}/user_content.txt"
USR="$(cat "${STAGE_DIR}/user_content.txt")"

# Human-readable audit packet (contains NO key — the key is only ever in the Authorization header).
{
  printf '=== SYSTEM ===\n%s\n\n=== USER ===\n%s\n' "${SYS}" "${USR}"
} > "${STAGE_DIR}/packet.txt"

# JSON request body (jq escapes everything; body carries NO key).
REQ_BODY="${STAGE_DIR}/request.json"
# No response_format: the candidate is returned as plain text under the delimited protocol, so source
# is never routed through JSON string escaping.
# --rawfile, not --arg: --arg passes the whole packet as a single argv element, and Linux caps one
# argument at 128 KiB (MAX_ARG_STRLEN) regardless of the much larger total ARG_MAX. A realistic
# downstream packet exceeds that — EA-0003 with a layout exemplar and a repair input is ~133 KB —
# and jq then dies with "Argument list too long". Reading from the files avoids argv entirely, so
# packet size is bounded by memory rather than by an exec limit.
jq -n --arg model "${MODEL}" \
      --rawfile sys "${TASK_PROMPT}" \
      --rawfile usr "${STAGE_DIR}/user_content.txt" \
  '{model:$model,
    messages:[{role:"system",content:$sys},{role:"user",content:$usr}],
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

# Extract the model's plain-text reply verbatim.
CONTENT_TXT="${STAGE_DIR}/model_content.txt"
jq -r '.choices[0].message.content // empty' "${RESP_FILE}" > "${CONTENT_TXT}"
if [[ ! -s "${CONTENT_TXT}" ]]; then
  err "model returned an empty reply; see ${RESP_FILE}"
  exit 8
fi

# ── Parse the delimited output protocol ─────────────────────────────────────────────────────────
# Two passes, deliberately: pass 1 validates the whole frame and produces a manifest; pass 2 writes.
# Nothing reaches the candidate directory until the entire response is known to be well formed, so a
# malformed frame can never leave a partial or truncated candidate staged.
MANIFEST="${STAGE_DIR}/.frame-manifest"
FRAME_ERR="${STAGE_DIR}/.frame-error"
: > "${FRAME_ERR}"

awk -v nonce="${NONCE}" -v manifest="${MANIFEST}" -v errfile="${FRAME_ERR}" '
  function fail(msg) { print "line " NR ": " msg > errfile; bad=1; exit }
  BEGIN {
    fpfx  = "<<<CODEOS:" nonce ":FILE:"
    endf  = "<<<CODEOS:" nonce ":ENDFILE>>>"
    spfx  = "<<<CODEOS:" nonce ":SECTION:"
    ends  = "<<<CODEOS:" nonce ":ENDSECTION>>>"
    open  = ""   # "", "file", or "section"
    nfile = 0
  }
  # A line is a marker ONLY if it matches exactly, with this run s nonce, alone on the line.
  index($0, fpfx) == 1 && substr($0, length($0)-2) == ">>>" {
    if (open != "") fail("FILE marker inside an open " open " block")
    path = substr($0, length(fpfx)+1, length($0)-length(fpfx)-3)
    if (path == "") fail("FILE marker with an empty path")
    if (path in seen) fail("duplicate candidate path: " path)
    seen[path] = 1
    open = "file"; start = NR; curpath = path
    next
  }
  $0 == endf {
    if (open != "file") fail("ENDFILE with no open FILE block")
    print "file\t" curpath "\t" (start+1) "\t" (NR-1) > manifest
    nfile++; open = ""
    next
  }
  index($0, spfx) == 1 && substr($0, length($0)-2) == ">>>" {
    if (open != "") fail("SECTION marker inside an open " open " block")
    name = substr($0, length(spfx)+1, length($0)-length(spfx)-3)
    if (name != "contract_satisfaction" && name != "event_emission" && name != "notes")
      fail("unknown section name: " name)
    if (name in seensec) fail("duplicate section: " name)
    seensec[name] = 1
    open = "section"; start = NR; curname = name
    next
  }
  $0 == ends {
    if (open != "section") fail("ENDSECTION with no open SECTION block")
    print "section\t" curname "\t" (start+1) "\t" (NR-1) > manifest
    open = ""
    next
  }
  END {
    if (bad) exit
    if (open != "") { print "unterminated " open " block at end of response" > errfile; exit }
    if (nfile < 1) { print "response contained no candidate file blocks" > errfile; exit }
  }
' "${CONTENT_TXT}"

if [[ -s "${FRAME_ERR}" ]]; then
  err "model response violates the output protocol: $(tr '\n' ';' < "${FRAME_ERR}")"
  err "       nothing was staged. Raw reply preserved at ${CONTENT_TXT}"
  rmdir "${STAGE_DIR}/candidate" 2>/dev/null || true
  exit 11
fi

# Validate every path BEFORE writing any of them, for the same all-or-nothing reason.
while IFS=$'\t' read -r kind name _s _e; do
  [[ "${kind}" == "file" ]] || continue
  case "${name}" in
    /*|*..*) err "unsafe candidate path rejected (absolute or traversal): ${name}"
             rmdir "${STAGE_DIR}/candidate" 2>/dev/null || true; exit 8;;
  esac
  # Enforce the stage-area allowlist, except for the documented CANDIDATE_BLOCKED.md escape hatch
  # the model uses to report insufficient artifacts. This constrains WHERE a file may go; it does
  # not constrain what KIND of file it is (a build manifest inside the stage area is permitted).
  if [[ "${name}" != "CANDIDATE_BLOCKED.md" && "${name}" != "${ALLOWED_PREFIX}"* ]]; then
    err "candidate path outside the Stage ${STAGE} area (must start with '${ALLOWED_PREFIX}'): ${name}"
    rmdir "${STAGE_DIR}/candidate" 2>/dev/null || true
    exit 8
  fi
done < "${MANIFEST}"

# Pass 2 — write. Content is emitted byte for byte between its markers, never re-encoded.
NFILES=0
while IFS=$'\t' read -r kind name s e; do
  if [[ "${kind}" == "file" ]]; then
    outfile="${STAGE_DIR}/candidate/${name}"
    mkdir -p "$(dirname "${outfile}")"
    NFILES=$((NFILES+1))
  else
    outfile="${STAGE_DIR}/${name}.txt"
  fi
  if [[ "${s}" -gt "${e}" ]]; then
    : > "${outfile}"                      # a legitimately empty block
  else
    sed -n "${s},${e}p" "${CONTENT_TXT}" > "${outfile}"
  fi
done < "${MANIFEST}"

# Sidecars the model omitted still exist, empty, so the audit set is uniform across runs.
for s in contract_satisfaction event_emission notes; do
  [[ -f "${STAGE_DIR}/${s}.txt" ]] || : > "${STAGE_DIR}/${s}.txt"
done

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
