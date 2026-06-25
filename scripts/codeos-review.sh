#!/usr/bin/env bash
#
# codeos-review.sh — manual advisory Codex reviewer for the Codeos / DBA pipeline.
#
# The reviewer is INDEPENDENT, READ-ONLY, ADVISORY, and NON-GATEKEEPING. It never edits
# artifacts and never approves stages — it appends a critical assessment to an append-only
# review log so the human can decide faster. See docs/reviewer-pipeline.md for the design,
# the feasibility analysis, the DBA-philosophy scorecard, and the (inert) hook snippets.
#
# This is a PILOT prototype. No Claude Code hooks are wired; you run it manually at a gate.
#
# Subcommands:
#   stage-start <feature> <stage> [--base <sha>]
#       Record the base commit for a stage (feature/stage-scoped state) so a later review
#       diffs base->review rather than trusting HEAD alone.
#
#   review <feature> <stage> <artifact-path> [<artifact-path>...] [--fresh] [--scratch]
#       Build the review packet (secret-filtered), call Codex read-only, save the full
#       assessment, and append a REVIEW entry to the log.
#         --fresh    start a new Codex session instead of resuming the feature's session
#                    (use for safety-sensitive stages, reviewer/human disagreement, or
#                    suspected anchoring). Default is resume-within-feature.
#         --scratch  treat as a pilot/test run: write the assessment under
#                    reviews/codex/_scratch/ (gitignored) so history is not polluted.
#
#   decision <feature> <stage> <APPROVE_STAGE|REQUEST_CHANGES|STOP> "<reason>"
#       Append a HUMAN DECISION entry to the log (never edits prior entries).
#
# Truth model: the reviewed artifacts on disk + the review commit SHA are authoritative.
# A resumed Codex session provides cross-stage CONTEXT only, never evidence — every review
# re-reads the artifacts and pins the SHA.
#
set -euo pipefail

# --- locations ---------------------------------------------------------------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "${REPO_ROOT}" ]]; then
  echo "error: not inside a git repository" >&2
  exit 2
fi
cd "${REPO_ROOT}"

STATE_DIR=".codeos-state"                       # runtime state — gitignored, project-local
SESSIONS_DIR="${STATE_DIR}/codex-sessions"      # one <feature>.json per feature
STAGE_START_DIR="${STATE_DIR}/stage-start"      # <feature>/stage-<N>.json
REVIEW_LOG="reviews/review-log.md"
CODEX_DIR="reviews/codex"
CODEX_SCRATCH="reviews/codex/_scratch"

# Secret/size policy
SIZE_LIMIT_BYTES=$((256 * 1024))
PATH_EXCLUDES=( '*.env' '.env*' '*.pem' '*.key' 'secrets/*' 'credentials/*'
                '*runtime_events*.jsonl' '*.log' )
# Content patterns whose VALUES get redacted before the diff is sent to Codex.
SECRET_PATTERNS='OPENAI_API_KEY|ANTHROPIC_API_KEY|AWS_SECRET_ACCESS_KEY|BEGIN [A-Z ]*PRIVATE KEY|password[[:space:]]*=|token[[:space:]]*=|secret[[:space:]]*='

now_iso() { date -u +%Y-%m-%dT%H:%M:%SZ; }
sha256_of() { sha256sum "$1" | awk '{print $1}'; }
sha256_str() { printf '%s' "$1" | sha256sum | awk '{print $1}'; }

require_codex() {
  command -v codex >/dev/null 2>&1 || { echo "error: codex CLI not found on PATH" >&2; exit 2; }
}

# --- stage-start -------------------------------------------------------------
cmd_stage_start() {
  local feature="$1" stage="$2"; shift 2
  local base=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --base) base="$2"; shift 2 ;;
      *) echo "stage-start: unknown arg $1" >&2; exit 2 ;;
    esac
  done
  [[ -n "${base}" ]] || base="$(git rev-parse HEAD)"
  local branch; branch="$(git rev-parse --abbrev-ref HEAD)"
  local dir="${STAGE_START_DIR}/${feature}"
  mkdir -p "${dir}"
  cat > "${dir}/stage-${stage}.json" <<EOF
{
  "feature": "${feature}",
  "stage": ${stage},
  "base_commit": "${base}",
  "branch": "${branch}",
  "started_at": "$(now_iso)"
}
EOF
  echo "stage-start recorded: ${dir}/stage-${stage}.json (base ${base})"
}

# --- build the secret-filtered diff + packet --------------------------------
# Writes the packet to ${PACKET_FILE}; sets globals PACKET_EXCLUDED / PACKET_DIFF_HASH /
# PACKET_BASE_SHA / PACKET_REVIEW_SHA / PACKET_BRANCH. Must NOT be called in $(...) — the
# globals must persist into the caller's shell.
build_packet() {
  local feature="$1" stage="$2"; shift 2
  local artifacts=( "$@" )

  local branch review_sha base_sha approved_stage
  branch="$(git rev-parse --abbrev-ref HEAD)"
  review_sha="$(git rev-parse HEAD)"
  approved_stage="$((stage - 1))"

  local ss="${STAGE_START_DIR}/${feature}/stage-${stage}.json"
  if [[ -f "${ss}" ]]; then
    base_sha="$(grep -oE '"base_commit"[^,]*' "${ss}" | sed -E 's/.*"base_commit": *"([^"]*)".*/\1/')"
  else
    base_sha=""
  fi

  # --- diff, path-excluded then content-redacted ---
  local raw_diff excluded="" filtered_diff=""
  if [[ -n "${base_sha}" ]]; then
    raw_diff="$(git diff "${base_sha}" -- . 2>/dev/null || true)"
  else
    raw_diff="$(git diff HEAD -- . 2>/dev/null || true)"   # working-tree changes vs HEAD
  fi

  # path exclusion: drop hunks for excluded files (computed from changed-file list)
  local changed_files f keep_pathspec=()
  if [[ -n "${base_sha}" ]]; then
    mapfile -t changed_files < <(git diff --name-only "${base_sha}" -- . 2>/dev/null || true)
  else
    mapfile -t changed_files < <(git diff --name-only HEAD -- . 2>/dev/null || true)
  fi
  for f in "${changed_files[@]}"; do
    local drop=0 pat
    for pat in "${PATH_EXCLUDES[@]}"; do
      # shellcheck disable=SC2053
      [[ "${f}" == ${pat} || "$(basename "${f}")" == ${pat} ]] && drop=1 && break
    done
    if [[ ${drop} -eq 0 ]]; then
      # size guard
      if [[ -f "${f}" ]] && [[ "$(wc -c < "${f}")" -gt ${SIZE_LIMIT_BYTES} ]]; then
        drop=1
      fi
    fi
    if [[ ${drop} -eq 1 ]]; then excluded+="${f} "; else keep_pathspec+=( "${f}" ); fi
  done

  if [[ ${#keep_pathspec[@]} -gt 0 ]]; then
    if [[ -n "${base_sha}" ]]; then
      filtered_diff="$(git diff "${base_sha}" -- "${keep_pathspec[@]}" 2>/dev/null || true)"
    else
      filtered_diff="$(git diff HEAD -- "${keep_pathspec[@]}" 2>/dev/null || true)"
    fi
  fi

  # content redaction: blank the value after a secret-like key
  local redacted_diff
  redacted_diff="$(printf '%s\n' "${filtered_diff}" \
    | sed -E "s/(${SECRET_PATTERNS})([\"'[:space:]:=]*).*/\1\2[REDACTED]/I")"
  if [[ "${redacted_diff}" != "${filtered_diff}" ]]; then
    excluded+="(secret-like content redacted) "
  fi

  PACKET_EXCLUDED="${excluded}"
  PACKET_DIFF_HASH="$(sha256_str "${redacted_diff}")"
  PACKET_BASE_SHA="${base_sha:-(uncommitted artifact)}"
  PACKET_REVIEW_SHA="${review_sha}"
  PACKET_BRANCH="${branch}"

  # --- stage-specific checks + expected output (lightweight, inline) ---
  local checks expected
  checks="$(stage_checks "${stage}")"
  expected="$(stage_expected "${stage}")"

  {
    echo "Critically assess:"
    echo
    echo "REVIEW CONTEXT"
    echo "  Feature:                ${feature}"
    echo "  Stage:                  ${stage}"
    echo "  Branch:                 ${branch}"
    echo "  Base commit:            ${PACKET_BASE_SHA}"
    echo "  Review commit:          ${review_sha}"
    echo "  Current approved stage: ${approved_stage}"
    echo
    echo "DBA RULES RELEVANT TO THIS STAGE"
    echo "  - Human approval is required for every stage transition; you are advisory only."
    echo "  - Memory is not truth — assess only what is provided, pinned to the review commit."
    echo "  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema."
    echo "  - No events outside the approved event schema; no hidden behavior."
    echo
    echo "STAGE-SPECIFIC CHECKS"
    echo "${checks}"
    echo
    echo "EXPECTED STAGE OUTPUT"
    echo "  ${expected}"
    echo
    echo "ARTIFACTS TO REVIEW"
    local a
    for a in "${artifacts[@]}"; do
      if [[ -f "${a}" ]]; then
        echo "  --- ${a} (sha256: $(sha256_of "${a}")) ---"
        sed 's/^/    /' "${a}"
      else
        echo "  --- ${a} (MISSING) ---"
      fi
      echo
    done
    echo "DIFF TO REVIEW (base->review, secret/size filtered)"
    [[ -n "${excluded}" ]] && echo "  [excluded/redacted: ${excluded}] manual security review required"
    printf '%s\n' "${redacted_diff}"
    echo
    echo "INSTRUCTIONS"
    echo "  Give your full critical assessment first (operational, ranked by severity, with"
    echo "  concrete better-designs; separate required fixes from optional ones; end with a"
    echo "  clear judgement). Then on the LAST two lines emit exactly:"
    echo "    LOG SUMMARY: <NO OBJECTION | CHANGES ADVISED | DO NOT ADVANCE> — <single most important point>"
    echo "    EVIDENCE: <A|B|C|D|E>   (optional)"
  } > "${PACKET_FILE}"
}

stage_expected() {
  case "$1" in
    1) echo "Intent — actor+outcome statements, stable guarantees, explicit scope boundary; NO implementation detail." ;;
    2) echo "Behavioral contract — observable Given/When/Then scenarios, named failure modes, invariants; independently testable; no white-box claims." ;;
    3) echo "Event schema — named events with payloads, event flow, coverage map of contract scenarios to events; no speculative telemetry." ;;
    4) echo "Implementation — code satisfying every contract clause; emits only schema events; contract-satisfaction + event-emission tables; nothing untraceable." ;;
    5) echo "Tests — one behavioral test per contract scenario incl. failures; replay tests for schema conformance + chain integrity; coverage table." ;;
    6) echo "Runtime evidence — events in runtime_events.jsonl; correlation chains intact; bounded/sanitized; unexpected/missing events reported." ;;
    7) echo "Reconciliation — Intent->Contract->Schema->Impl->Tests->Runtime with ALIGNED/GAP/MISMATCH/MISSING per item, supported by evidence." ;;
    8) echo "Replay — schema conformance + correlation chain integrity + determinism check; nondeterminism explained; missing fixtures reported." ;;
    9) echo "Refinement — smallest effective change per observed trigger; no redesign disguised as refinement; affected artifacts named." ;;
    *) echo "(no expected-output template for stage $1)" ;;
  esac
}

stage_checks() {
  case "$1" in
    1) printf '  - actor/outcome clarity; no implementation detail; scope boundary explicit; stable guarantees clear; ambiguity flagged.\n' ;;
    2) printf '  - every intent outcome has observable contract coverage; failure paths named; invariants testable; no white-box claims.\n' ;;
    3) printf '  - every relevant contract scenario has event coverage; event names stable; required fields clear; no speculative telemetry.\n' ;;
    4) printf '  - code traces to approved contract/schema only; no unapproved events; no hidden behavior; no unrelated files; report complete.\n' ;;
    5) printf '  - behavior tested not private internals; failure paths tested; event/telemetry tests present; replay tests where applicable.\n' ;;
    6) printf '  - runtime evidence captured; event log bounded/sanitized; correlation chains visible; unexpected/missing events reported.\n' ;;
    7) printf '  - ALIGNED/GAP/MISMATCH/MISSING judgments supported; no weak evidence hidden behind fluent summary; gaps routed to right action.\n' ;;
    8) printf '  - replay actually checks event sequence + schema conformance; nondeterminism explained; missing fixtures reported.\n' ;;
    9) printf '  - trigger valid; proposed fix minimal; no redesign disguised as refinement; affected artifacts identified.\n' ;;
    *) printf '  - (no stage-specific checklist for stage %s)\n' "$1" ;;
  esac
}

# --- codex session handling --------------------------------------------------
# Runs codex read-only with the packet on stdin. Resumes the feature's session unless --fresh
# or no session exists yet. The session id is captured deterministically from the codex
# startup banner ("session id: <uuid>"); resume reuses it, giving cross-stage continuity.
run_codex() {
  local feature="$1" fresh="$2" packet_file="$3"
  mkdir -p "${SESSIONS_DIR}"
  local sess_file="${SESSIONS_DIR}/${feature}.json"
  local session_id=""
  [[ "${fresh}" == "0" && -f "${sess_file}" ]] && \
    session_id="$(grep -oE '"session_id" *: *"[^"]*"' "${sess_file}" | sed -E 's/.*"([^"]*)"$/\1/')"

  local out
  if [[ -n "${session_id}" ]]; then
    # resume: no -s/--cd flags on this subcommand; sandbox is set via config override
    out="$(codex exec resume "${session_id}" -c sandbox_mode="read-only" - < "${packet_file}" 2>&1)" || true
  else
    out="$(codex exec -s read-only --cd "${REPO_ROOT}" - < "${packet_file}" 2>&1)" || true
    session_id="$(printf '%s\n' "${out}" | grep -oE 'session id: [0-9a-fA-F-]+' | head -1 | awk '{print $3}')"
    if [[ -z "${session_id}" ]]; then
      echo "error: could not capture a Codex session id from output — aborting (fail-closed)." >&2
      echo "       review NOT logged. Inspect the codex output and rerun." >&2
      exit 3
    fi
    printf '{ "feature": "%s", "session_id": "%s", "created_at": "%s" }\n' \
      "${feature}" "${session_id}" "$(now_iso)" > "${sess_file}"
  fi
  REVIEW_SESSION="${session_id}"
  REVIEW_OUTPUT="${out}"
}

# --- review ------------------------------------------------------------------
cmd_review() {
  require_codex
  local feature="$1" stage="$2"; shift 2
  local fresh=0 scratch=0 print_only=0 artifacts=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --fresh) fresh=1; shift ;;
      --scratch) scratch=1; shift ;;
      --print-packet|--dry-run) print_only=1; shift ;;
      *) artifacts+=( "$1" ); shift ;;
    esac
  done
  [[ ${#artifacts[@]} -gt 0 ]] || { echo "review: provide at least one artifact path" >&2; exit 2; }

  # pilot/test runs must not touch the committed log either
  [[ ${scratch} -eq 1 ]] && { mkdir -p "${CODEX_SCRATCH}"; REVIEW_LOG="${CODEX_SCRATCH}/review-log.md"; }

  PACKET_FILE="$(mktemp)"; trap 'rm -f "${PACKET_FILE}"' EXIT
  build_packet "${feature}" "${stage}" "${artifacts[@]}"   # sets PACKET_* globals (no subshell)

  # safety: inspect exactly what would be sent to Codex, without calling it (no cost, no session)
  if [[ ${print_only} -eq 1 ]]; then
    cat "${PACKET_FILE}"
    [[ -n "${PACKET_EXCLUDED}" ]] && echo "# [excluded/redacted: ${PACKET_EXCLUDED}]" >&2
    exit 0
  fi

  local pre_status; pre_status="$(git status --porcelain)"
  run_codex "${feature}" "${fresh}" "${PACKET_FILE}"
  local post_status; post_status="$(git status --porcelain)"

  # parse LOG SUMMARY / EVIDENCE
  local summary_line concern evidence
  summary_line="$(printf '%s\n' "${REVIEW_OUTPUT}" | grep -E '^LOG SUMMARY:' | tail -1 || true)"
  evidence="$(printf '%s\n' "${REVIEW_OUTPUT}" | grep -E '^EVIDENCE:' | tail -1 | sed -E 's/^EVIDENCE:[[:space:]]*//' || true)"
  if [[ -z "${summary_line}" ]]; then
    concern="UNCLASSIFIED"
    summary_line="LOG SUMMARY: UNCLASSIFIED — no parseable summary; HIGH attention, manual review required"
  else
    concern="$(printf '%s' "${summary_line}" | sed -E 's/^LOG SUMMARY:[[:space:]]*([A-Z ]*[A-Z]).*/\1/' | sed -E 's/[[:space:]]+$//')"
  fi
  [[ -n "${evidence}" ]] || evidence="not reported"

  # save full assessment (with metadata header)
  local ts outdir assessment_file artifact_shas=""
  ts="$(now_iso)"
  if [[ ${scratch} -eq 1 ]]; then outdir="${CODEX_SCRATCH}"; else outdir="${CODEX_DIR}"; fi
  mkdir -p "${outdir}"
  local short_sha; short_sha="$(git rev-parse --short HEAD)"
  assessment_file="${outdir}/${ts//:/}-${feature}-stage-${stage}-${short_sha}.md"
  local a
  for a in "${artifacts[@]}"; do
    [[ -f "${a}" ]] && artifact_shas+=$'\n'"    - path: ${a}"$'\n'"      sha256: $(sha256_of "${a}")"
  done
  {
    echo "---"
    echo "reviewed:"
    echo "  feature: ${feature}"
    echo "  stage: ${stage}"
    echo "  branch: ${PACKET_BRANCH}"
    echo "  base_commit: ${PACKET_BASE_SHA}"
    echo "  review_commit: ${PACKET_REVIEW_SHA}"
    echo "  artifacts:${artifact_shas}"
    echo "  diff_hash: ${PACKET_DIFF_HASH}"
    echo "  excluded_paths: \"${PACKET_EXCLUDED}\""
    echo "  reviewer: \"codex (session ${REVIEW_SESSION})\""
    echo "  concern: ${concern}"
    echo "  evidence: ${evidence}"
    echo "---"
    echo
    printf '%s\n' "${REVIEW_OUTPUT}"
  } > "${assessment_file}"
  local assessment_hash; assessment_hash="$(sha256_of "${assessment_file}")"

  # append REVIEW entry to the append-only log
  mkdir -p "$(dirname "${REVIEW_LOG}")"
  [[ -f "${REVIEW_LOG}" ]] || init_log
  {
    echo
    echo "## ${ts} REVIEW — ${feature} — Stage ${stage}"
    echo "Base: ${PACKET_BASE_SHA}  Review: ${PACKET_REVIEW_SHA}  Branch: ${PACKET_BRANCH}"
    echo "Diff-hash: ${PACKET_DIFF_HASH}"
    echo "Reviewer: codex default-model (session ${REVIEW_SESSION})"
    echo "Concern: ${concern}"
    echo "Evidence: ${evidence}"
    echo "Log summary: ${summary_line#LOG SUMMARY: }"
    echo "Full assessment: ${assessment_file} (sha256:${assessment_hash})"
    [[ -n "${PACKET_EXCLUDED}" ]] && echo "Coverage gap: excluded/redacted [${PACKET_EXCLUDED}] — manual security review required"
    echo "Human decision: (append with: codeos-review.sh decision ${feature} ${stage} <DECISION> \"<reason>\")"
  } >> "${REVIEW_LOG}"

  # read-only invariant check
  if [[ "${pre_status}" != "${post_status}" ]]; then
    echo "WARNING: working tree changed during review — reviewer should be read-only. Inspect:" >&2
    diff <(printf '%s\n' "${pre_status}") <(printf '%s\n' "${post_status}") >&2 || true
  fi

  echo "review logged: ${REVIEW_LOG}"
  echo "  concern: ${concern}   evidence: ${evidence}"
  echo "  assessment: ${assessment_file}"
}

# --- decision (append-only human entry) -------------------------------------
cmd_decision() {
  local feature="$1" stage="$2" decision="$3" reason="${4:-}"
  case "${decision}" in APPROVE_STAGE|REQUEST_CHANGES|STOP) ;; *)
    echo "decision must be APPROVE_STAGE | REQUEST_CHANGES | STOP" >&2; exit 2 ;; esac
  [[ -f "${REVIEW_LOG}" ]] || init_log
  local sha; sha="$(git rev-parse HEAD)"
  {
    echo
    echo "## $(now_iso) HUMAN DECISION — ${feature} — Stage ${stage}"
    echo "Commit reviewed: ${sha}"
    echo "Decision: ${decision}"
    echo "Reason/next: ${reason}"
  } >> "${REVIEW_LOG}"
  echo "decision appended to ${REVIEW_LOG}"
}

init_log() {
  mkdir -p "$(dirname "${REVIEW_LOG}")"
  cat > "${REVIEW_LOG}" <<'EOF'
# Codeos Review Log (append-only, v0)

Append-only record of automated advisory reviews and the human decisions that follow them.
Entries are NEVER edited — a human decision is a separately appended entry. The reviewer is
advisory and read-only; APPROVE belongs to the human. See docs/reviewer-pipeline.md.

(v0 layout: one global log. Per-feature logs are a documented future layout.)
EOF
}

# --- dispatch ----------------------------------------------------------------
usage() {
  sed -n '3,40p' "$0" | sed 's/^# \{0,1\}//'
  exit "${1:-0}"
}

[[ $# -ge 1 ]] || usage 2
sub="$1"; shift || true
case "${sub}" in
  stage-start) [[ $# -ge 2 ]] || usage 2; cmd_stage_start "$@" ;;
  review)      [[ $# -ge 3 ]] || usage 2; cmd_review "$@" ;;
  decision)    [[ $# -ge 3 ]] || usage 2; cmd_decision "$@" ;;
  -h|--help|help) usage 0 ;;
  *) echo "unknown subcommand: ${sub}" >&2; usage 2 ;;
esac
