#!/usr/bin/env bash
# codeos-implement.sh — out-of-band, opt-in DeepSeek implementer for DBA Stage 4 (Implementation)
# and Stage 5 (Tests). It drafts a CANDIDATE only: output is staged under
# .codeos-state/deepseek-candidates/, never written into modules/ or tests/, and never committed.
# A human controls candidate promotion. Package authority, advisory review, and delivery-cycle
# verification apply unchanged. This tool approves nothing and adds no approval gate.
#
# Companion to dba/04-tools/reviewer/codeos-review.sh: it mirrors that shim's entry-point discipline (git-repo
# precondition, self-dev-vs-downstream context resolution, fail-closed preconditions, an explicit
# exit-code table). It is off by default — see the activation status file below.
#
# Usage:
#   codeos-implement.sh [options] <feature_id> <stage:4|5> [artifact-path...]
#     <feature_id>     the DBA feature id (used only to name the staging directory)
#     <stage>          4 (implementation) or 5 (tests) — no other value is accepted
#     <artifact-path>  OPTIONAL positional artifacts (intent / contract / event schema; plus the
#                      Stage 4 output when stage=5). Each must exist. A call may pass none of these
#                      and declare every artifact through the role flags below instead — that is the
#                      preferred shape. At least one artifact must arrive by one route or the other.
#
#   Options (each must precede the positional arguments):
#     Artifact ROLE flags — the caller declares each artifact's authority; this tool performs NO
#     inference of role from path, filename, content, headings, or directory. A role a caller does
#     not declare is a role the model is not told about.
#       --contract PATH        BEHAVIORAL CONTRACT   — behavior that must be satisfied
#       --event-schema PATH    EVENT SCHEMA          — events that must be emitted correctly
#       --architecture PATH    PROJECT ARCHITECTURE  — binding architectural constraint
#       --profile PATH         IMPLEMENTATION PROFILE— binding implementation constraint
#     Positional <artifact-path> arguments remain supported for backward compatibility and are
#     labelled APPROVED ARTIFACT (ROLE UNSPECIFIED). They never silently satisfy a declared role.
#
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
#   own reply). dba/04-tools/implementer/tests/codeos-implement-tests.sh scans this script against that list and fails
#   if any external tool outside it appears, so this comment cannot silently drift from the code.
#
# Activation (Optional Mechanism Status Convention, UPG-0056): a one-line status file, resolved by
# context from the caller's repository root —
#   self-dev    (caller git root == this repo): maintenance/config/delegated-implementation.yaml
#   downstream  (caller git root != this repo): .codeos/02-architecture/delegated-implementation.yaml
# Four outcomes: absent -> disabled; exact "status: disabled" -> disabled; exact "status: enabled"
# -> enabled; anything else -> configuration error. The tool runs ONLY when the value is enabled.
#
# Environment:
#   DEEPSEEK_API_KEY        required when enabled. Read only into the HTTP Authorization header via a
#                           curl config on stdin — never placed in argv, the request body, the
#                           preserved packet, the response dump, or any candidate file.
#   CODEOS_DEEPSEEK_MODEL   optional, default "deepseek-v4-flash".
#   CODEOS_DEEPSEEK_URL     optional, default "https://api.deepseek.com/chat/completions".
#   CODEOS_DEEPSEEK_MAX_TOKENS
#                           optional, default 32768. The only other supported value is 65536, for
#                           one explicit pilot retry after finish_reason=length. A retry is a new
#                           invocation so both attempts remain visible and count toward cost.
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
#  12  the same artifact path was declared under two different authority roles (before any network call)
set -euo pipefail

err() { echo "error: $*" >&2; }

# ── 1. git-repo precondition (property of the calling project, checked first) ───────────────────
git rev-parse --show-toplevel >/dev/null 2>&1 || { err "not inside a git repository"; exit 1; }

# ── 3. usage / args ─────────────────────────────────────────────────────────────────────────────
EXEMPLARS=()
REPAIR_CANDIDATES=()
REPAIR_OUTPUTS=()
# One array per declared role. Deliberately flat and explicit: the value here is that a reader can
# see exactly which roles exist, not that the plumbing is shared.
ROLE_CONTRACT=(); ROLE_SCHEMA=(); ROLE_ARCH=(); ROLE_PROFILE=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --contract)          [[ $# -ge 2 ]] || { err "--contract requires a path"; exit 3; }
                         ROLE_CONTRACT+=("$2"); shift 2;;
    --event-schema)      [[ $# -ge 2 ]] || { err "--event-schema requires a path"; exit 3; }
                         ROLE_SCHEMA+=("$2"); shift 2;;
    --architecture)      [[ $# -ge 2 ]] || { err "--architecture requires a path"; exit 3; }
                         ROLE_ARCH+=("$2"); shift 2;;
    --profile)           [[ $# -ge 2 ]] || { err "--profile requires a path"; exit 3; }
                         ROLE_PROFILE+=("$2"); shift 2;;
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
# feature_id and stage are always required. Artifacts may arrive positionally OR via role flags —
# a fully role-declared call passes no positional artifact at all, which is the intended shape and
# the only shape CHG-B's precondition permits.
if [[ $# -lt 2 ]]; then
  err "usage: codeos-implement.sh [role flags] [--exemplar PATH] [--repair-candidate PATH]"
  err "                           [--repair-output PATH] <feature_id> <stage:4|5> [artifact-path...]"
  err "       role flags: --contract --event-schema --architecture --profile"
  exit 3
fi
FEATURE="$1"; STAGE="$2"; shift 2
ARTIFACTS=("$@")
if [[ "${STAGE}" != "4" && "${STAGE}" != "5" ]]; then
  err "stage must be 4 (implementation) or 5 (tests); got '${STAGE}'"
  exit 3
fi
# At least one governed artifact must be supplied, by either route. Zero artifacts is a usage error,
# not a silent run against nothing.
if [[ ${#ARTIFACTS[@]} -eq 0 && ${#ROLE_CONTRACT[@]} -eq 0 && ${#ROLE_SCHEMA[@]} -eq 0 \
      && ${#ROLE_ARCH[@]} -eq 0 && ${#ROLE_PROFILE[@]} -eq 0 ]]; then
  err "no artifacts supplied: pass at least one role flag or one positional artifact path"
  exit 3
fi

# Candidate files must stay in the stage's area (Stage 4 -> modules/, Stage 5 -> tests/). This
# backs the "strict path" contract stated in dba/03-prompts/delegation/codeos-implementer-task.md: the model is
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
# pwd -P (physical) follows the .codeos/toolkit symlink when invoked downstream, matching
# dba/04-tools/reviewer/codeos-review.sh. The caller's own git root distinguishes self-dev from downstream.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd -P)"
CALLER_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd -P)"

if [[ "${CALLER_ROOT}" == "${CODEOS_ROOT}" ]]; then
  STATUS_FILE="${CODEOS_ROOT}/maintenance/config/delegated-implementation.yaml"
else
  STATUS_FILE="${CALLER_ROOT}/.codeos/02-architecture/delegated-implementation.yaml"
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
for a in ${ROLE_CONTRACT[@]+"${ROLE_CONTRACT[@]}"} ${ROLE_SCHEMA[@]+"${ROLE_SCHEMA[@]}"} \
         ${ROLE_ARCH[@]+"${ROLE_ARCH[@]}"} \
         ${ROLE_PROFILE[@]+"${ROLE_PROFILE[@]}"}; do
  [[ -f "${a}" ]] || { err "artifact path does not exist: ${a}"; exit 7; }
done

# ── 12. one path may not carry two authority roles ──────────────────────────────────────────────
# Checked before any network call. An artifact declared under two roles is a caller error, never
# something for this tool to arbitrate — it would have to decide which authority wins.
# Pure bash — no external process, so the documented allowlist stays as it is.
declare -A _ROLE_OF=()
_check_role() {
  local role="$1"; shift
  local p
  for p in "$@"; do
    if [[ -n "${_ROLE_OF[${p}]:-}" && "${_ROLE_OF[${p}]}" != "${role}" ]]; then
      err "artifact declared under two authority roles: ${p} (${_ROLE_OF[${p}]} and ${role})"
      err "       a path carries exactly one role; the caller must choose which authority applies."
      exit 12
    fi
    _ROLE_OF["${p}"]="${role}"
  done
}
_check_role "BEHAVIORAL CONTRACT"    ${ROLE_CONTRACT[@]+"${ROLE_CONTRACT[@]}"}
_check_role "EVENT SCHEMA"           ${ROLE_SCHEMA[@]+"${ROLE_SCHEMA[@]}"}
_check_role "PROJECT ARCHITECTURE"   ${ROLE_ARCH[@]+"${ROLE_ARCH[@]}"}
_check_role "IMPLEMENTATION PROFILE" ${ROLE_PROFILE[@]+"${ROLE_PROFILE[@]}"}
_check_role "LAYOUT EXEMPLAR"        ${EXEMPLARS[@]+"${EXEMPLARS[@]}"}

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

MODEL="${CODEOS_DEEPSEEK_MODEL:-deepseek-v4-flash}"
DS_URL="${CODEOS_DEEPSEEK_URL:-https://api.deepseek.com/chat/completions}"
MAX_TOKENS="${CODEOS_DEEPSEEK_MAX_TOKENS:-32768}"
if [[ "${MAX_TOKENS}" != "32768" && "${MAX_TOKENS}" != "65536" ]]; then
  err "CODEOS_DEEPSEEK_MAX_TOKENS must be 32768 or 65536; got '${MAX_TOKENS}'"
  exit 3
fi
TASK_PROMPT="${CODEOS_ROOT}/dba/03-prompts/delegation/codeos-implementer-task.md"
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
  # Declared roles first, in a fixed order, each labelled with the authority it carries. The label
  # is the whole point: it tells the model HOW the artifact binds. Content is emitted byte for byte.
  _emit_role() {
    local label="$1" note="$2"; shift 2
    local a
    for a in "$@"; do
      printf '\n--- %s: %s ---\n' "${label}" "${a}"
      printf '    (%s)\n' "${note}"
      cat "${a}"
    done
  }
  _emit_role "BEHAVIORAL CONTRACT" "binding — the behavior your implementation must satisfy" \
    ${ROLE_CONTRACT[@]+"${ROLE_CONTRACT[@]}"}
  _emit_role "EVENT SCHEMA" "binding — the events you must emit, and only these" \
    ${ROLE_SCHEMA[@]+"${ROLE_SCHEMA[@]}"}
  _emit_role "PROJECT ARCHITECTURE" "binding architectural constraint — follow it; it is not behavior to invent" \
    ${ROLE_ARCH[@]+"${ROLE_ARCH[@]}"}
  _emit_role "IMPLEMENTATION PROFILE" "binding implementation constraint — language and scope" \
    ${ROLE_PROFILE[@]+"${ROLE_PROFILE[@]}"}
  # Positional artifacts: supported, but visibly degraded. They never stand in for a declared role.
  for a in "${ARTIFACTS[@]}"; do
    printf '\n--- APPROVED ARTIFACT (ROLE UNSPECIFIED): %s ---\n' "${a}"
    printf '    (supporting context only — the caller did not declare an authority role for this\n'
    printf '     artifact. It does not replace a Behavioral Contract, Event Schema,\n'
    printf '     Project Architecture, or Implementation Profile.)\n'
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
      --argjson max_tokens "${MAX_TOKENS}" \
      --rawfile sys "${TASK_PROMPT}" \
      --rawfile usr "${STAGE_DIR}/user_content.txt" \
  '{model:$model,
    messages:[{role:"system",content:$sys},{role:"user",content:$usr}],
    thinking:{type:"enabled"},
    reasoning_effort:"high",
    max_tokens:$max_tokens,
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

# Record response accounting before interpreting or parsing the candidate. This keeps a truncated or
# otherwise non-normal response attributable without persisting the model's reasoning content.
PT="$(jq -r '.usage.prompt_tokens // "?"' "${RESP_FILE}")"
CT="$(jq -r '.usage.completion_tokens // "?"' "${RESP_FILE}")"
TT="$(jq -r '.usage.total_tokens // "?"' "${RESP_FILE}")"
PCH="$(jq -r '.usage.prompt_cache_hit_tokens // "?"' "${RESP_FILE}")"
PCM="$(jq -r '.usage.prompt_cache_miss_tokens // "?"' "${RESP_FILE}")"
RT="$(jq -r '.usage.completion_tokens_details.reasoning_tokens // "?"' "${RESP_FILE}")"
RETURNED_MODEL="$(jq -r '.model // "?"' "${RESP_FILE}")"
FINISH_REASON="$(jq -r '.choices[0].finish_reason // "?"' "${RESP_FILE}")"
printf 'prompt_tokens=%s completion_tokens=%s total_tokens=%s prompt_cache_hit_tokens=%s prompt_cache_miss_tokens=%s reasoning_tokens=%s requested_model=%s returned_model=%s finish_reason=%s max_tokens=%s\n' \
  "${PT}" "${CT}" "${TT}" "${PCH}" "${PCM}" "${RT}" "${MODEL}" "${RETURNED_MODEL}" \
  "${FINISH_REASON}" "${MAX_TOKENS}" > "${STAGE_DIR}/tokens.txt"

# Extract the model's plain-text reply verbatim, even for a non-normal finish, so the audit captures
# what actually arrived. Only a natural stop is eligible for candidate parsing and staging.
CONTENT_TXT="${STAGE_DIR}/model_content.txt"
jq -r '.choices[0].message.content // empty' "${RESP_FILE}" > "${CONTENT_TXT}"
if [[ "${FINISH_REASON}" != "stop" ]]; then
  err "model response is not a valid candidate (finish_reason=${FINISH_REASON}); see ${RESP_FILE}"
  if [[ "${FINISH_REASON}" == "length" && "${MAX_TOKENS}" == "32768" ]]; then
    err "       pilot policy permits one explicit retry with CODEOS_DEEPSEEK_MAX_TOKENS=65536"
  fi
  rmdir "${STAGE_DIR}/candidate" 2>/dev/null || true
  exit 8
fi
if [[ ! -s "${CONTENT_TXT}" ]]; then
  err "model returned an empty reply; see ${RESP_FILE}"
  rmdir "${STAGE_DIR}/candidate" 2>/dev/null || true
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
    if (name != "contract_satisfaction" && name != "event_emission" && name != "notes" && name != "deferral_resolution")
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
# deferral_resolution is deliberately NOT in this list: it is optional and usually absent, so the
# file's presence is itself the signal that the model reported a resolved deferral.
for s in contract_satisfaction event_emission notes; do
  [[ -f "${STAGE_DIR}/${s}.txt" ]] || : > "${STAGE_DIR}/${s}.txt"
done

# Append one invocation record to the log.
{
  printf '## %s — %s stage %s\n' "${TS}" "${FEATURE}" "${STAGE}"
  printf -- '- model: requested=%s returned=%s\n' "${MODEL}" "${RETURNED_MODEL}"
  printf -- '- response: finish_reason=%s max_tokens=%s\n' "${FINISH_REASON}" "${MAX_TOKENS}"
  printf -- '- tokens: prompt=%s completion=%s reasoning=%s total=%s cache_hit=%s cache_miss=%s\n' \
    "${PT}" "${CT}" "${RT}" "${TT}" "${PCH}" "${PCM}"
  printf -- '- candidate files: %s\n' "${NFILES}"
  printf -- '- staging: %s\n\n' "${STAGE_DIR}"
} >> "${LOG_FILE}"

echo "candidate staged: ${STAGE_DIR}/candidate  (${NFILES} file(s))"
echo "  tokens: prompt=${PT} completion=${CT} reasoning=${RT} total=${TT}   model=${RETURNED_MODEL}"
echo "  audit:  packet=${STAGE_DIR}/packet.txt  response=${RESP_FILE}  log=${LOG_FILE}"
echo "  NOTE: candidate only — promote manually; the Stage ${STAGE} human gate + advisory review still apply."
