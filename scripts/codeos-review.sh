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
# Secret-key names whose VALUES get redacted. Redaction requires an actual value (8+ chars)
# after the separator, so prose that merely mentions a key name (e.g. docs, this script) is
# left intact and not treated as a secret.
SECRET_KV_KEYS='OPENAI_API_KEY|ANTHROPIC_API_KEY|AWS_SECRET_ACCESS_KEY|[Aa][Pp][Ii][_-]?[Kk][Ee][Yy]|[Pp]assword|[Tt]oken|[Ss]ecret'
redact_secrets() {
  sed -E \
    -e "s/((${SECRET_KV_KEYS})[[:space:]]*[:=][[:space:]]*[\"']?)[A-Za-z0-9._/+-]{8,}/\1[REDACTED]/g" \
    -e 's/-----BEGIN [A-Z ]*PRIVATE KEY-----/[REDACTED PRIVATE KEY]/g'
}

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
    base_sha="$(grep -oE '"base_commit"[^,]*' "${ss}" | sed -E 's/.*"base_commit": *"([^"]*)".*/\1/' || true)"
    # provenance is fail-closed: a stage-start file that exists but has no valid base SHA
    # is malformed state, not "absent" — refuse rather than silently downgrade to empty.
    [[ "${base_sha}" =~ ^[0-9a-fA-F]{7,40}$ ]] || {
      echo "error: ${ss} exists but has no valid base_commit (malformed provenance) — aborting." >&2; exit 4; }
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

  local redaction_count=0 secret_flag=0 path_excluded=0

  # content redaction: blank the value after a secret-like key
  local redacted_diff
  redacted_diff="$(printf '%s\n' "${filtered_diff}" | redact_secrets)"
  if [[ "${redacted_diff}" != "${filtered_diff}" ]]; then
    secret_flag=1
    redaction_count=$(( redaction_count + $(grep -c "\[REDACTED" <<<"${redacted_diff}" || true) ))
    excluded+="(secret-like diff content redacted) "
  fi
  [[ -n "${excluded}" ]] && path_excluded=1   # path/size diff exclusions recorded above

  # --- requested artifacts: secret-redact + size-guard; a requested artifact that cannot
  #     be fully shown is a HARD coverage failure (we were asked to review it). ---
  local artifacts_block="" a raw redacted shown_count=0
  PACKET_ARTIFACT_EXCLUDED=0
  for a in "${artifacts[@]}"; do
    if [[ ! -f "${a}" ]]; then
      artifacts_block+="  --- ${a} (MISSING — not shown) ---"$'\n\n'
      PACKET_ARTIFACT_EXCLUDED=1; excluded+="${a}(missing) "; continue
    fi
    if [[ "$(wc -c < "${a}")" -gt ${SIZE_LIMIT_BYTES} ]]; then
      artifacts_block+="  --- ${a} (EXCLUDED: over size limit — not shown) ---"$'\n\n'
      PACKET_ARTIFACT_EXCLUDED=1; excluded+="${a}(oversize) "; continue
    fi
    raw="$(cat "${a}")"
    redacted="$(printf '%s\n' "${raw}" | redact_secrets)"
    if [[ "${redacted}" != "${raw}" ]]; then
      secret_flag=1
      redaction_count=$(( redaction_count + $(grep -c "\[REDACTED" <<<"${redacted}" || true) ))
      excluded+="${a}(secret redacted) "
    fi
    artifacts_block+="  --- ${a} (sha256: $(sha256_of "${a}")) ---"$'\n'
    artifacts_block+="$(printf '%s\n' "${redacted}" | sed 's/^/    /')"$'\n\n'
    shown_count=$((shown_count + 1))
  done

  # --- explicit coverage state (most severe wins) ---
  #   FULL_COVERAGE | PARTIAL_COVERAGE | SECRET_REDACTION | CRITICAL_OMISSION | EMPTY_PACKET
  local state coverage
  if [[ ${shown_count} -eq 0 && -z "${redacted_diff//[$' \t\n']/}" ]]; then
    state="EMPTY_PACKET"; coverage="empty"
  elif [[ "${PACKET_ARTIFACT_EXCLUDED}" -eq 1 ]]; then
    state="CRITICAL_OMISSION"; coverage="critical"
  elif [[ ${secret_flag} -eq 1 ]]; then
    state="SECRET_REDACTION"; coverage="partial"
  elif [[ ${path_excluded} -eq 1 ]]; then
    state="PARTIAL_COVERAGE"; coverage="partial"
  else
    state="FULL_COVERAGE"; coverage="full"
  fi

  # --- provenance integrity: the reviewed "second state" must be labeled honestly ---
  # A non-empty diff against an identical base/review SHA can only come from uncommitted
  # workspace changes; if the tree is in fact clean, the provenance is self-contradictory.
  local workspace_dirty=0 integrity="OK"
  git diff --quiet HEAD -- . 2>/dev/null || workspace_dirty=1
  local nonempty_diff=0; [[ -n "${redacted_diff//[$' \t\n']/}" ]] && nonempty_diff=1
  if [[ -n "${base_sha}" && "${base_sha}" == "${review_sha}" && ${nonempty_diff} -eq 1 && ${workspace_dirty} -eq 0 ]]; then
    integrity="CONTRADICTION"   # SHAs equal but a committed diff exists — unverifiable
  fi

  PACKET_EXCLUDED="${excluded}"
  PACKET_COVERAGE="${coverage}"
  PACKET_COVERAGE_STATE="${state}"
  PACKET_REDACTION_COUNT="${redaction_count}"
  PACKET_SECRET_FLAG="${secret_flag}"
  PACKET_WORKSPACE_DIRTY="${workspace_dirty}"
  PACKET_INTEGRITY="${integrity}"
  local dirty_suffix=""; [[ ${workspace_dirty} -eq 1 ]] && dirty_suffix=" +workspace"
  PACKET_DIFF_HASH="$(sha256_str "${redacted_diff}")"
  PACKET_BASE_SHA="${base_sha:-(uncommitted artifact)}"
  PACKET_REVIEW_SHA="${review_sha}${dirty_suffix}"
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
    echo "  Review commit:          ${review_sha}$([[ ${workspace_dirty} -eq 1 ]] && echo ' (+ uncommitted workspace changes)')"
    echo "  Current approved stage: ${approved_stage}"
    echo "  Evidence coverage:      ${state}"
    echo "  Provenance integrity:   ${integrity}"
    if [[ "${integrity}" == "CONTRADICTION" ]]; then
      echo "  - PROVENANCE CONTRADICTION: base and review SHA are identical yet the diff is"
      echo "    non-empty and the tree is clean — the reviewed state is not verifiable."
    fi
    echo
    echo "DBA RULES RELEVANT TO THIS STAGE"
    echo "  - Human approval is required for every stage transition; you are advisory only."
    echo "  - Memory is not truth — assess only what is provided, pinned to the review commit."
    echo "  - Implementation must trace to approved artifacts; no behavior beyond intent+contract+schema."
    echo "  - No events outside the approved event schema; no hidden behavior."
    if [[ "${coverage}" != "full" ]]; then
      echo "  - COVERAGE IS PARTIAL: some content was excluded/redacted (see below). You are"
      echo "    seeing an incomplete evidence set — do not issue NO OBJECTION on this basis."
    fi
    echo
    echo "STAGE-SPECIFIC CHECKS"
    echo "${checks}"
    echo
    echo "EXPECTED STAGE OUTPUT"
    echo "  ${expected}"
    echo
    echo "ARTIFACTS TO REVIEW"
    printf '%s' "${artifacts_block}"
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
  local codex_ver; codex_ver="$(codex --version 2>/dev/null | head -1)"
  local session_id=""
  if [[ "${fresh}" == "0" && -f "${sess_file}" ]]; then
    session_id="$(grep -oE '"session_id" *: *"[^"]*"' "${sess_file}" | sed -E 's/.*"([^"]*)"$/\1/' || true)"
    # version drift: a session created under a different codex build may parse/behave
    # differently — force a fresh session rather than resuming across versions.
    # session_id is provenance: present-but-unparseable session state is malformed -> fail closed.
    [[ -n "${session_id}" ]] || { echo "error: ${sess_file} exists but has no session_id (malformed) — delete it or pass --fresh." >&2; exit 4; }
    local stored_ver; stored_ver="$(grep -oE '"codex_version" *: *"[^"]*"' "${sess_file}" | sed -E 's/.*"([^"]*)"$/\1/' || true)"
    if [[ -n "${stored_ver}" && "${stored_ver}" != "${codex_ver}" ]]; then
      echo "note: codex version changed (${stored_ver} -> ${codex_ver}); starting a fresh session." >&2
      session_id=""
    fi
  fi

  local out
  if [[ -n "${session_id}" ]]; then
    # resume: no -s/--cd flags on this subcommand; sandbox is set via config override
    out="$(codex exec resume "${session_id}" -c sandbox_mode="read-only" - < "${packet_file}" 2>&1)" || true
  else
    out="$(codex exec -s read-only --cd "${REPO_ROOT}" - < "${packet_file}" 2>&1)" || true
    session_id="$(printf '%s\n' "${out}" | grep -oE 'session id: [0-9a-fA-F-]+' | head -1 | awk '{print $3}' || true)"
    if [[ -z "${session_id}" ]]; then
      echo "error: could not capture a Codex session id from output — aborting (fail-closed)." >&2
      echo "       review NOT logged. Inspect the codex output and rerun." >&2
      exit 3
    fi
    printf '{ "feature": "%s", "session_id": "%s", "codex_version": "%s", "created_at": "%s" }\n' \
      "${feature}" "${session_id}" "${codex_ver}" "$(now_iso)" > "${sess_file}"
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

  # Effective concern = Codex concern adjusted for evidence coverage. We keep and log BOTH,
  # so a coverage gap can never silently pass as the reviewer's verdict.
  local codex_concern="${concern}" effective_concern="${concern}" coverage_note=""
  case "${PACKET_COVERAGE_STATE:-FULL_COVERAGE}" in
    EMPTY_PACKET)
      effective_concern="DO NOT ADVANCE"
      coverage_note="EMPTY_PACKET — no reviewable content reached Codex" ;;
    CRITICAL_OMISSION)
      effective_concern="DO NOT ADVANCE"
      coverage_note="CRITICAL_OMISSION — a requested artifact could not be shown (missing/oversize)" ;;
    SECRET_REDACTION|PARTIAL_COVERAGE)
      if [[ "${codex_concern}" == "NO OBJECTION" ]]; then
        effective_concern="CHANGES ADVISED"
        coverage_note="${PACKET_COVERAGE_STATE} — partial evidence; NO OBJECTION downgraded to CHANGES ADVISED"
      fi ;;
  esac
  # Provenance integrity failure is fail-closed and wins over any coverage-based verdict.
  if [[ "${PACKET_INTEGRITY:-OK}" == "CONTRADICTION" ]]; then
    effective_concern="DO NOT ADVANCE"
    coverage_note="PROVENANCE CONTRADICTION — base/review SHA identical but diff non-empty; reviewed state unverifiable"
  fi
  concern="${effective_concern}"

  # save full assessment (with metadata header) + the EXACT packet that was reviewed
  local ts outdir assessment_file packet_saved artifact_shas=""
  ts="$(now_iso)"
  if [[ ${scratch} -eq 1 ]]; then outdir="${CODEX_SCRATCH}"; else outdir="${CODEX_DIR}"; fi
  mkdir -p "${outdir}"
  local short_sha; short_sha="$(git rev-parse --short HEAD)"
  assessment_file="${outdir}/${ts//:/}-${feature}-stage-${stage}-${short_sha}.md"
  local packets_dir="${outdir}/packets"; mkdir -p "${packets_dir}"
  packet_saved="${packets_dir}/${ts//:/}-${feature}-stage-${stage}-${short_sha}.packet.txt"
  cp "${PACKET_FILE}" "${packet_saved}"          # exact reviewed bytes — durable, verifiable
  local packet_hash; packet_hash="$(sha256_of "${packet_saved}")"
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
    echo "  coverage_state: ${PACKET_COVERAGE_STATE}"
    echo "  provenance_integrity: ${PACKET_INTEGRITY}"
    echo "  workspace_dirty: $([[ ${PACKET_WORKSPACE_DIRTY} -eq 1 ]] && echo true || echo false)"
    echo "  redaction_count: ${PACKET_REDACTION_COUNT}"
    echo "  secret_redaction: $([[ ${PACKET_SECRET_FLAG} -eq 1 ]] && echo true || echo false)"
    echo "  excluded_paths: \"${PACKET_EXCLUDED}\""
    echo "  reviewed_packet: packets/$(basename "${packet_saved}")"
    echo "  reviewed_packet_sha256: ${packet_hash}"
    echo "  reviewer: \"codex (session ${REVIEW_SESSION})\""
    echo "  codex_concern: ${codex_concern}"
    echo "  effective_concern: ${effective_concern}${coverage_note:+ (${coverage_note})}"
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
    echo "Codex concern: ${codex_concern}"
    echo "Effective concern: ${effective_concern}${coverage_note:+ — ${coverage_note}}"
    echo "Evidence: ${evidence}"
    echo "Coverage: ${PACKET_COVERAGE_STATE} (redactions: ${PACKET_REDACTION_COUNT}); integrity: ${PACKET_INTEGRITY}; workspace_dirty: $([[ ${PACKET_WORKSPACE_DIRTY} -eq 1 ]] && echo true || echo false)"
    echo "Log summary: ${summary_line#LOG SUMMARY: }"
    echo "Full assessment: ${assessment_file} (sha256:${assessment_hash})"
    echo "Reviewed packet: ${packet_saved} (sha256:${packet_hash})"
    if [[ ${PACKET_SECRET_FLAG} -eq 1 || "${PACKET_COVERAGE_STATE}" == CRITICAL_OMISSION || "${PACKET_COVERAGE_STATE}" == EMPTY_PACKET ]]; then
      echo "Coverage gap: ${PACKET_COVERAGE_STATE} — excluded/redacted [${PACKET_EXCLUDED}] — MANUAL SECURITY REVIEW REQUIRED"
    fi
    echo "Human decision: (append with: codeos-review.sh decision ${feature} ${stage} <DECISION> \"<reason>\")"
  } >> "${REVIEW_LOG}"

  # read-only invariant check
  if [[ "${pre_status}" != "${post_status}" ]]; then
    echo "WARNING: working tree changed during review — reviewer should be read-only. Inspect:" >&2
    diff <(printf '%s\n' "${pre_status}") <(printf '%s\n' "${post_status}") >&2 || true
  fi

  echo "review logged: ${REVIEW_LOG}"
  echo "  codex concern: ${codex_concern}   effective concern: ${effective_concern}   evidence: ${evidence}"
  echo "  coverage: ${PACKET_COVERAGE_STATE} (redactions: ${PACKET_REDACTION_COUNT})"
  echo "  assessment: ${assessment_file}"
  echo "  packet: ${packet_saved}"
}

# --- decision (append-only human entry) -------------------------------------
cmd_decision() {
  local feature="$1" stage="$2" decision="$3"; shift 3
  local reason="" force=0
  while [[ $# -gt 0 ]]; do
    case "$1" in --force) force=1; shift ;; *) reason="$1"; shift ;; esac
  done
  case "${decision}" in APPROVE_STAGE|REQUEST_CHANGES|STOP) ;; *)
    echo "decision must be APPROVE_STAGE | REQUEST_CHANGES | STOP" >&2; exit 2 ;; esac
  [[ -f "${REVIEW_LOG}" ]] || init_log
  local sha; sha="$(git rev-parse HEAD)"

  # Re-verify the reviewed artifacts still match what was reviewed, so an approval cannot
  # silently apply to a since-edited artifact (the reviewer's "prove what was reviewed" point).
  local latest verify_lines="" changed=0
  latest="$(ls -1t "${CODEX_DIR}"/*-"${feature}"-stage-"${stage}"-*.md 2>/dev/null | head -1 || true)"
  if [[ -n "${latest}" ]]; then
    local p="" h now
    while IFS= read -r line; do
      case "${line}" in
        *"- path: "*) p="${line#*- path: }" ;;
        *"sha256: "*)
          h="${line#*sha256: }"
          if [[ -n "${p}" ]]; then
            now="(missing)"; [[ -f "${p}" ]] && now="$(sha256_of "${p}")"
            if [[ "${now}" == "${h}" ]]; then verify_lines+="  MATCH   ${p}"$'\n'
            else verify_lines+="  CHANGED ${p} (reviewed ${h:0:12} / now ${now:0:12})"$'\n'; changed=1; fi
            p=""
          fi ;;
      esac
    done < <(sed -n '/^  artifacts:/,/^  diff_hash:/p' "${latest}")
  fi

  # Stale-approval invariant: a CHANGED artifact means the latest review no longer describes
  # the tree. APPROVE_STAGE is refused (nothing logged) unless --force records a stale override.
  if [[ "${decision}" == "APPROVE_STAGE" && ${changed} -eq 1 && ${force} -eq 0 ]]; then
    echo "refused: APPROVE_STAGE against a CHANGED reviewed artifact — run 'review' again first," >&2
    echo "         or pass --force \"<reason>\" to record a stale-override approval." >&2
    printf '%s' "${verify_lines}" >&2
    exit 4
  fi
  local stale_note=""
  [[ "${decision}" == "APPROVE_STAGE" && ${changed} -eq 1 && ${force} -eq 1 ]] && \
    stale_note=" [STALE OVERRIDE — approved against a CHANGED artifact]"

  {
    echo
    echo "## $(now_iso) HUMAN DECISION — ${feature} — Stage ${stage}"
    echo "Commit reviewed: ${sha}"
    echo "Decision: ${decision}${stale_note}"
    echo "Reason/next: ${reason}"
    [[ -n "${latest}" ]] && echo "Verified against: ${latest}"
    [[ -n "${verify_lines}" ]] && { echo "Artifact integrity:"; printf '%s' "${verify_lines}"; }
  } >> "${REVIEW_LOG}"
  echo "decision appended to ${REVIEW_LOG}"
  if [[ ${changed} -eq 1 ]]; then
    echo "WARNING: some reviewed artifacts changed since the review — decision recorded with that flagged." >&2
  fi
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
