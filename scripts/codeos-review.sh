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

# in_list <value> <pipe|separated|list>  -> 0 if value is exactly one of the list items
in_list() { case "|$2|" in *"|$1|"*) return 0 ;; *) return 1 ;; esac; }

# exc_add <path> <reason> <affected_section>  -> structured excluded-item record
exc_add() { PACKET_EXC_PATH+=("$1"); PACKET_EXC_REASON+=("$2"); PACKET_EXC_SECTION+=("$3"); }

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
  local sha_only_paths=("${PACKET_SHA_ONLY[@]+"${PACKET_SHA_ONLY[@]}"}")
  local delta_mode="${PACKET_DELTA_MODE:-full}"
  local delta_base="${PACKET_DELTA_BASE:-}"
  # guard: missing --sha-only path exits non-zero before Codex (explicit scope guard, not omitted)
  local _so
  for _so in "${sha_only_paths[@]+"${sha_only_paths[@]}"}"; do
    [[ -f "${_so}" ]] || { echo "error: --sha-only path not found: ${_so}" >&2; exit 2; }
  done
  # manifest state — built during artifact processing, written to packet before REVIEW CONTEXT
  local manifest_sha_only="" manifest_full_artifacts="" review_content_bytes=0 diff_bytes=0
  local so_bytes so_sha
  for _so in "${sha_only_paths[@]+"${sha_only_paths[@]}"}"; do
    so_bytes="$(wc -c < "${_so}")"
    so_sha="$(sha256_of "${_so}")"
    PACKET_ARTIFACTS_YAML+="    - path: ${_so}"$'\n'"      sha256: ${so_sha}"$'\n'"      visibility: path_sha_only"$'\n'
    manifest_sha_only+="    - path: ${_so}"$'\n'"      mode: path_sha_only"$'\n'"      bytes: ${so_bytes}"$'\n'"      sha256: ${so_sha}"$'\n'
  done

  local branch review_sha base_sha approved_stage
  branch="$(git rev-parse --abbrev-ref HEAD)"
  review_sha="$(git rev-parse HEAD)"
  # stage is a free token (numeric DBA stages, or labels like selfdev-step-2). Only compute
  # a predecessor for numeric stages; non-numeric stages have no "approved stage N-1".
  if [[ "${stage}" =~ ^[0-9]+$ ]]; then
    approved_stage="$((stage - 1))"
  else
    approved_stage="n/a (non-numeric stage)"
  fi

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
  PACKET_EXC_PATH=(); PACKET_EXC_REASON=(); PACKET_EXC_SECTION=()   # structured exclusion record

  # path exclusion: drop hunks for excluded files (computed from changed-file list)
  local changed_files f keep_pathspec=()
  if [[ "${delta_mode}" == "delta" ]]; then
    # delta mode: diff scoped to positional artifact paths only (--sha-only paths excluded)
    raw_diff="$(git diff "${delta_base}" HEAD -- "${artifacts[@]}" 2>/dev/null || true)"
    mapfile -t changed_files < <(git diff --name-only "${delta_base}" HEAD -- "${artifacts[@]}" ':(exclude)reviews' ':(exclude).codeos-state' 2>/dev/null || true)
  elif [[ -n "${base_sha}" ]]; then
    raw_diff="$(git diff "${base_sha}" -- . 2>/dev/null || true)"
    mapfile -t changed_files < <(git diff --name-only "${base_sha}" -- . ':(exclude)reviews' ':(exclude).codeos-state' 2>/dev/null || true)
  else
    raw_diff="$(git diff HEAD -- . 2>/dev/null || true)"   # working-tree changes vs HEAD
    mapfile -t changed_files < <(git diff --name-only HEAD -- . ':(exclude)reviews' ':(exclude).codeos-state' 2>/dev/null || true)
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
    if [[ ${drop} -eq 1 ]]; then excluded+="${f} "; exc_add "${f}" "path/size excluded" "diff"; else keep_pathspec+=( "${f}" ); fi
  done

  if [[ ${#keep_pathspec[@]} -gt 0 ]]; then
    if [[ "${delta_mode}" == "delta" ]]; then
      filtered_diff="$(git diff "${delta_base}" HEAD -- "${keep_pathspec[@]}" 2>/dev/null || true)"
    elif [[ -n "${base_sha}" ]]; then
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
    exc_add "(diff)" "secret-like content redacted" "diff"
  fi
  [[ -n "${excluded}" ]] && path_excluded=1   # path/size diff exclusions recorded above
  diff_bytes=${#redacted_diff}

  # --- requested artifacts: each is REPRESENTED, never silently dropped —
  #     full mode visibility: shown | shown_redacted | oversize_omitted | missing
  #     delta mode: delta_diff | path_sha_only | omitted_with_reason ---
  local artifacts_block="" a raw redacted shown_count=0 delta_diff_count=0 vis sha reason_note
  PACKET_ARTIFACT_EXCLUDED=0
  PACKET_ARTIFACTS_YAML=""
  local artifact_bytes
  for a in "${artifacts[@]}"; do
    if [[ ! -f "${a}" ]]; then
      artifacts_block+="  --- ${a} (visibility: missing — not shown) ---"$'\n\n'
      PACKET_ARTIFACT_EXCLUDED=1; excluded+="${a}(missing) "; exc_add "${a}" "requested artifact missing" "artifact"
      PACKET_ARTIFACTS_YAML+="    - path: ${a}"$'\n'"      visibility: missing"$'\n'
      manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: omitted_with_reason"$'\n'"      reason: requested artifact missing"$'\n'; continue
    fi
    artifact_bytes="$(wc -c < "${a}")"
    if [[ "${artifact_bytes}" -gt ${SIZE_LIMIT_BYTES} ]]; then
      artifacts_block+="  --- ${a} (visibility: oversize_omitted — over size limit, not shown) ---"$'\n\n'
      PACKET_ARTIFACT_EXCLUDED=1; excluded+="${a}(oversize) "; exc_add "${a}" "requested artifact over size limit" "artifact"
      PACKET_ARTIFACTS_YAML+="    - path: ${a}"$'\n'"      visibility: oversize_omitted"$'\n'
      manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: omitted_with_reason"$'\n'"      reason: over size limit"$'\n'; continue
    fi
    sha="$(sha256_of "${a}")"
    if [[ "${delta_mode}" == "delta" ]]; then
      # delta mode: metadata only in ARTIFACTS TO REVIEW; diff content is in the DIFF section
      if git diff --quiet "${delta_base}" HEAD -- "${a}" 2>/dev/null; then
        artifacts_block+="  --- ${a} (mode: path_sha_only, sha256: ${sha}, bytes: ${artifact_bytes}) ---"$'\n\n'
        manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: path_sha_only"$'\n'"      bytes: ${artifact_bytes}"$'\n'"      sha256: ${sha}"$'\n'
        PACKET_ARTIFACTS_YAML+="    - path: ${a}"$'\n'"      sha256: ${sha}"$'\n'"      visibility: path_sha_only"$'\n'
      else
        reason_note=""
        git cat-file -e "${delta_base}:${a}" 2>/dev/null || reason_note="new_at_base"
        if [[ -n "${reason_note}" ]]; then
          artifacts_block+="  --- ${a} (mode: delta_diff, reason: ${reason_note}, sha256: ${sha}, bytes: ${artifact_bytes}) ---"$'\n\n'
          manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: delta_diff"$'\n'"      bytes: ${artifact_bytes}"$'\n'"      sha256: ${sha}"$'\n'"      reason: ${reason_note}"$'\n'
        else
          artifacts_block+="  --- ${a} (mode: delta_diff, sha256: ${sha}, bytes: ${artifact_bytes}) ---"$'\n\n'
          manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: delta_diff"$'\n'"      bytes: ${artifact_bytes}"$'\n'"      sha256: ${sha}"$'\n'
        fi
        PACKET_ARTIFACTS_YAML+="    - path: ${a}"$'\n'"      sha256: ${sha}"$'\n'"      visibility: delta_diff"$'\n'
        delta_diff_count=$((delta_diff_count + 1))
      fi
      shown_count=$((shown_count + 1))
    else
      # full mode: inline content in ARTIFACTS TO REVIEW
      raw="$(cat "${a}")"; redacted="$(printf '%s\n' "${raw}" | redact_secrets)"
      if [[ "${redacted}" != "${raw}" ]]; then
        vis="shown_redacted"; secret_flag=1
        redaction_count=$(( redaction_count + $(grep -c "\[REDACTED" <<<"${redacted}" || true) ))
        excluded+="${a}(secret redacted) "; exc_add "${a}" "secret value redacted in place" "artifact"
        manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: full_file"$'\n'"      bytes: ${artifact_bytes}"$'\n'"      sha256: ${sha}"$'\n'"      note: secret value redacted in place"$'\n'
      else
        vis="shown"
        manifest_full_artifacts+="    - path: ${a}"$'\n'"      mode: full_file"$'\n'"      bytes: ${artifact_bytes}"$'\n'"      sha256: ${sha}"$'\n'
      fi
      review_content_bytes=$((review_content_bytes + artifact_bytes))
      artifacts_block+="  --- ${a} (sha256: ${sha}, visibility: ${vis}) ---"$'\n'
      artifacts_block+="$(printf '%s\n' "${redacted}" | sed 's/^/    /')"$'\n\n'
      PACKET_ARTIFACTS_YAML+="    - path: ${a}"$'\n'"      sha256: ${sha}"$'\n'"      visibility: ${vis}"$'\n'
      shown_count=$((shown_count + 1))
    fi
  done

  review_content_bytes=$((review_content_bytes + diff_bytes))

  # --- budget check (warning only; never aborts) ---
  local budget_threshold="${CODEOS_PACKET_BUDGET_BYTES:-50000}"
  local estimated_review_tokens=$(( review_content_bytes / 4 ))
  local budget_status="OK"
  if [[ "${review_content_bytes}" -gt "${budget_threshold}" ]]; then
    budget_status="WARNING — ${review_content_bytes} bytes exceeds CODEOS_PACKET_BUDGET_BYTES=${budget_threshold}"
    echo "warning: review content is ${review_content_bytes} bytes, exceeds budget of ${budget_threshold} bytes (CODEOS_PACKET_BUDGET_BYTES)" >&2
  fi

  # --- explicit coverage state (most severe wins) ---
  #   FULL_COVERAGE | PARTIAL_COVERAGE | SECRET_REDACTION | CRITICAL_OMISSION | EMPTY_PACKET
  local state coverage
  if [[ "${delta_mode}" == "delta" && ${delta_diff_count} -eq 0 && -z "${redacted_diff//[$' \t\n']/}" ]] || \
     [[ "${delta_mode}" != "delta" && ${shown_count} -eq 0 && -z "${redacted_diff//[$' \t\n']/}" ]]; then
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

  # --- workspace_dirty: DESCRIPTIVE audit context only ---
  # v0 is a manual advisory logging pilot. We record whether the tree had uncommitted changes at
  # review time so the saved evidence is self-describing. This is NOT an enforcement/binding mode:
  # there is no provenance-integrity matrix and no approval gating. The deeper binding model
  # (COMMIT_BOUND/WORKSPACE_BOUND reverification, rollback) is deferred — see
  # backlog/reviewer-decision-integrity.md.
  # use status --porcelain (not diff --quiet) so untracked files also count as dirty — untracked
  # artifacts are a normal review case in this pilot, so missing them would log false-clean context.
  local workspace_dirty=0
  [[ -n "$(git status --porcelain=v1 --untracked-files=all -- . ':(exclude)reviews' ':(exclude).codeos-state' 2>/dev/null)" ]] && workspace_dirty=1

  PACKET_EXCLUDED="${excluded}"
  PACKET_COVERAGE="${coverage}"
  PACKET_COVERAGE_STATE="${state}"
  PACKET_REDACTION_COUNT="${redaction_count}"
  PACKET_SECRET_FLAG="${secret_flag}"
  PACKET_WORKSPACE_DIRTY="${workspace_dirty}"
  PACKET_DIFF_HASH="$(sha256_str "${redacted_diff}")"
  PACKET_BASE_SHA="${base_sha:-(no base pin)}"   # no stage-start recorded; review pins to review_commit
  PACKET_REVIEW_SHA="${review_sha}"   # machine-pure SHA; the dirty bit lives in workspace_dirty
  PACKET_BRANCH="${branch}"

  # --- stage-specific checks + expected output (lightweight, inline) ---
  local checks expected
  checks="$(stage_checks "${stage}")"
  expected="$(stage_expected "${stage}")"

  local task_prompt="${REPO_ROOT}/prompts/codeos-reviewer-task.md"
  [[ -f "${task_prompt}" ]] || { echo "error: reviewer task template not found: ${task_prompt}" >&2; exit 2; }

  {
    cat "${task_prompt}"
    echo
    echo "PACKET MANIFEST"
    echo "  generated: $(now_iso)"
    echo "  task_prompt: ${task_prompt} ($(wc -c < "${task_prompt}") bytes)"
    echo "  review_content_bytes: ${review_content_bytes}"
    echo "  estimated_review_tokens: ~${estimated_review_tokens}"
    echo "  budget_status: ${budget_status}"
    echo "  packet_mode: ${delta_mode}"
    echo "  delta_base: ${delta_base:-none}"
    echo "  items:"
    printf '%s' "${manifest_sha_only}"
    printf '%s' "${manifest_full_artifacts}"
    echo "    - path: (diff)"
    if [[ "${delta_mode}" == "delta" ]]; then
      echo "      mode: delta_diff"
    else
      echo "      mode: full_file"
    fi
    echo "      bytes: ${diff_bytes}"
    echo
    echo "REVIEW CONTEXT"
    echo "  Feature:                ${feature}"
    echo "  Stage:                  ${stage}"
    echo "  Branch:                 ${branch}"
    echo "  Base commit:            ${PACKET_BASE_SHA}"
    [[ "${delta_mode}" == "delta" ]] && echo "  Delta base:             ${delta_base}"
    echo "  Review commit:          ${review_sha}$([[ ${workspace_dirty} -eq 1 ]] && echo ' (+ uncommitted workspace changes)')"
    echo "  Current approved stage: ${approved_stage}"
    echo "  Evidence coverage:      ${state}"
    echo "  Workspace dirty:        $([[ ${workspace_dirty} -eq 1 ]] && echo 'yes (uncommitted changes at review time)' || echo 'no')"
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
    if [[ "${delta_mode}" == "delta" ]]; then
      echo "DELTA DIFF (${delta_base}->HEAD, artifact paths only, secret/size filtered)"
    else
      echo "DIFF TO REVIEW (base->review, secret/size filtered)"
    fi
    [[ -n "${excluded}" ]] && echo "  [excluded/redacted: ${excluded}] manual security review required"
    printf '%s\n' "${redacted_diff}"
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

# --- local prechecks ---------------------------------------------------------
# Fail-closed checks that run before packet build and Codex invocation.
# Scans only the positional artifact paths and explicit PRECHECK_GUARD_CLEAN paths —
# not the whole repo, not generated packets, not review logs.
#
# Hard fails (exit 2): unfilled template placeholders, forbidden fields, dirty guard paths.
# Warnings (stderr, exit 0): draft markers (TODO/FIXME/TBD/[to be filled]).
run_prechecks() {
  local -a artifacts=("$@")
  local -a guard_clean=("${PRECHECK_GUARD_CLEAN[@]+"${PRECHECK_GUARD_CLEAN[@]}"}")
  local a gc warn_fired=0

  for a in "${artifacts[@]+"${artifacts[@]}"}"; do
    [[ -f "${a}" ]] || continue   # missing artifacts are handled (and represented) by build_packet

    # hard fail: unfilled template placeholders (fixed-string, not a pattern for real IDs)
    if grep -qF 'UPG-####' "${a}"; then
      echo "error: precheck failed — literal placeholder 'UPG-####' found in ${a} (fill in the real UPG id)" >&2; exit 2
    fi
    if grep -qF 'CHG-YYYYMMDD-NNN' "${a}"; then
      echo "error: precheck failed — literal placeholder 'CHG-YYYYMMDD-NNN' found in ${a} (fill in the real CHG id)" >&2; exit 2
    fi

    # hard fail: forbidden field superseded by UPG-0001 (line-anchored to avoid prose matches)
    if grep -qE '^[[:space:]]*latest_review:' "${a}"; then
      echo "error: precheck failed — forbidden field 'latest_review:' found in ${a} (use review_state instead)" >&2; exit 2
    fi

    # warning: unresolved draft markers (do not block)
    if grep -qiE 'TODO|FIXME|\bTBD\b|\[to be filled\]' "${a}"; then
      echo "warning: precheck — unresolved draft marker (TODO/FIXME/TBD/[to be filled]) in ${a}" >&2
      warn_fired=1
    fi
  done

  for gc in "${guard_clean[@]+"${guard_clean[@]}"}"; do
    # hard fail: missing guard path is an error, not a silent pass
    [[ -f "${gc}" ]] || { echo "error: precheck failed — --guard-clean path not found: ${gc}" >&2; exit 2; }
    # hard fail: any staged or unstaged change vs HEAD on the guard path
    git diff --quiet HEAD -- "${gc}" 2>/dev/null || {
      echo "error: precheck failed — --guard-clean path '${gc}' has uncommitted changes (expected clean)" >&2; exit 2; }
  done
}

# --- review ------------------------------------------------------------------
cmd_review() {
  require_codex
  local feature="$1" stage="$2"; shift 2
  local fresh=0 scratch=0 print_only=0 skip_prechecks=0 mode="full" delta_base=""
  local artifacts=() sha_only_paths=() guard_clean_paths=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --fresh) fresh=1; shift ;;
      --scratch) scratch=1; shift ;;
      --print-packet|--dry-run) print_only=1; shift ;;
      --skip-prechecks) skip_prechecks=1; shift ;;
      --mode) [[ $# -ge 2 ]] || { echo "review: --mode requires an argument (full or delta)" >&2; exit 2; }; mode="$2"; shift 2 ;;
      --base) [[ $# -ge 2 ]] || { echo "review: --base requires a SHA argument" >&2; exit 2; }; delta_base="$2"; shift 2 ;;
      --sha-only) [[ $# -ge 2 ]] || { echo "review: --sha-only requires a PATH argument" >&2; exit 2; }; sha_only_paths+=( "$2" ); shift 2 ;;
      --guard-clean) [[ $# -ge 2 ]] || { echo "review: --guard-clean requires a PATH argument" >&2; exit 2; }; guard_clean_paths+=( "$2" ); shift 2 ;;
      *) artifacts+=( "$1" ); shift ;;
    esac
  done
  [[ ${#artifacts[@]} -gt 0 ]] || { echo "review: provide at least one artifact path" >&2; exit 2; }

  # --- --mode / --base validation (all exit 2 before precheck or packet build) ---
  [[ "${mode}" == "full" || "${mode}" == "delta" ]] || { echo "review: --mode must be 'full' or 'delta', got '${mode}'" >&2; exit 2; }
  if [[ "${mode}" == "delta" ]]; then
    [[ -n "${delta_base}" ]] || { echo "review: --mode delta requires --base <sha>" >&2; exit 2; }
    [[ "${delta_base}" =~ ^[0-9a-fA-F]{7,40}$ ]] || { echo "review: --base value is not a valid hex SHA: '${delta_base}'" >&2; exit 2; }
    git rev-parse --verify "${delta_base}^{commit}" >/dev/null 2>&1 || { echo "review: --base '${delta_base}' does not resolve to a valid commit" >&2; exit 2; }
  fi
  # --- --sha-only / positional artifact conflict (exit 2; no silent precedence) ---
  local _so_chk _art_chk
  for _so_chk in "${sha_only_paths[@]+"${sha_only_paths[@]}"}"; do
    for _art_chk in "${artifacts[@]+"${artifacts[@]}"}"; do
      [[ "${_art_chk}" == "${_so_chk}" ]] && { echo "review: path '${_so_chk}' passed both as positional artifact and --sha-only; pass as one or the other" >&2; exit 2; }
    done
  done

  # pilot/test runs must not touch the committed log either
  [[ ${scratch} -eq 1 ]] && { mkdir -p "${CODEX_SCRATCH}"; REVIEW_LOG="${CODEX_SCRATCH}/review-log.md"; }

  if [[ ${skip_prechecks} -eq 1 ]]; then
    echo "warning: prechecks skipped (--skip-prechecks)" >&2
  else
    PRECHECK_GUARD_CLEAN=("${guard_clean_paths[@]+"${guard_clean_paths[@]}"}")
    run_prechecks "${artifacts[@]}"
  fi
  PACKET_FILE="$(mktemp)"; trap 'rm -f "${PACKET_FILE}"' EXIT
  PACKET_SHA_ONLY=("${sha_only_paths[@]+"${sha_only_paths[@]}"}")
  PACKET_DELTA_MODE="${mode}"
  PACKET_DELTA_BASE="${delta_base}"
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

  # normalize to schema enums (the reviewer's free text can be loose)
  in_list "${concern}" "NO OBJECTION|CHANGES ADVISED|DO NOT ADVANCE|UNCLASSIFIED" || concern="UNCLASSIFIED"
  case "${evidence}" in
    A*|B*|C*|D*|E*) evidence="${evidence:0:1}" ;;
    "not reported") ;;
    *) evidence="not reported" ;;
  esac

  # Effective concern = max severity of (Codex concern, the coverage-state MINIMUM floor). We keep
  # and log BOTH so an evidence-coverage gap never silently passes as the verdict. This is the only
  # adjustment the pilot makes — it is about how complete the evidence was, NOT an approval guarantee.
  # Severity rank: NO OBJECTION < CHANGES ADVISED < UNCLASSIFIED < DO NOT ADVANCE.
  local codex_concern="${concern}" effective_concern coverage_note=""
  local floor=0
  case "${PACKET_COVERAGE_STATE:-FULL_COVERAGE}" in
    FULL_COVERAGE)                   floor=0 ;;
    SECRET_REDACTION|PARTIAL_COVERAGE) floor=1 ;;   # min CHANGES ADVISED
    EMPTY_PACKET)                    floor=2 ;;      # min UNCLASSIFIED
    CRITICAL_OMISSION)               floor=3 ;;      # min DO NOT ADVANCE
  esac
  local cr; case "${codex_concern}" in
    "NO OBJECTION") cr=0 ;; "CHANGES ADVISED") cr=1 ;; "UNCLASSIFIED") cr=2 ;; "DO NOT ADVANCE") cr=3 ;; *) cr=2 ;;
  esac
  local eff=${cr}; [[ ${floor} -gt ${eff} ]] && eff=${floor}
  case ${eff} in 0) effective_concern="NO OBJECTION" ;; 1) effective_concern="CHANGES ADVISED" ;; 2) effective_concern="UNCLASSIFIED" ;; 3) effective_concern="DO NOT ADVANCE" ;; esac
  if [[ ${eff} -gt ${cr} ]]; then
    coverage_note="raised from '${codex_concern}' to the coverage floor for ${PACKET_COVERAGE_STATE}"
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

  # --- v0 schema validation, fail-closed (see docs/reviewer-artifact-schemas.md) ---
  local pair _v_err=""
  for pair in "feature=${feature}" "stage=${stage}" "base_commit=${PACKET_BASE_SHA}" \
              "review_commit=${PACKET_REVIEW_SHA}" "diff_hash=${PACKET_DIFF_HASH}" \
              "coverage_state=${PACKET_COVERAGE_STATE}" \
              "reviewed_packet_sha256=${packet_hash}"; do
    [[ -n "${pair#*=}" ]] || _v_err+=" missing:${pair%%=*}"
  done
  in_list "${PACKET_COVERAGE_STATE}" "FULL_COVERAGE|PARTIAL_COVERAGE|SECRET_REDACTION|CRITICAL_OMISSION|EMPTY_PACKET" || _v_err+=" enum:coverage_state"
  in_list "${codex_concern}"     "NO OBJECTION|CHANGES ADVISED|DO NOT ADVANCE|UNCLASSIFIED" || _v_err+=" enum:codex_concern"
  in_list "${effective_concern}" "NO OBJECTION|CHANGES ADVISED|DO NOT ADVANCE|UNCLASSIFIED" || _v_err+=" enum:effective_concern"
  in_list "${evidence}" "A|B|C|D|E|not reported" || _v_err+=" enum:evidence"
  if [[ -z "${PACKET_ARTIFACTS_YAML}" ]] && ! in_list "${PACKET_COVERAGE_STATE}" "CRITICAL_OMISSION|EMPTY_PACKET"; then
    _v_err+=" missing:artifacts"
  fi
  if [[ -n "${_v_err}" ]]; then
    echo "error: v0 schema validation failed (fail-closed):${_v_err}" >&2
    echo "       review NOT logged. See docs/reviewer-artifact-schemas.md" >&2
    exit 4
  fi

  {
    echo "---"
    echo "reviewed:"
    echo "  feature: ${feature}"
    echo "  stage: ${stage}"
    echo "  branch: ${PACKET_BRANCH}"
    echo "  base_commit: ${PACKET_BASE_SHA}"
    echo "  review_commit: ${PACKET_REVIEW_SHA}"
    if [[ -n "${PACKET_ARTIFACTS_YAML}" ]]; then echo "  artifacts:"; printf '%s' "${PACKET_ARTIFACTS_YAML}"; else echo "  artifacts: []"; fi
    echo "  diff_hash: ${PACKET_DIFF_HASH}"
    echo "  coverage_state: ${PACKET_COVERAGE_STATE}"
    echo "  workspace_dirty: $([[ ${PACKET_WORKSPACE_DIRTY} -eq 1 ]] && echo true || echo false)"
    echo "  redaction_count: ${PACKET_REDACTION_COUNT}"
    echo "  secret_redaction: $([[ ${PACKET_SECRET_FLAG} -eq 1 ]] && echo true || echo false)"
    if [[ ${#PACKET_EXC_PATH[@]} -eq 0 ]]; then
      echo "  excluded_paths: []"
    else
      echo "  excluded_paths:"
      local _i
      for _i in "${!PACKET_EXC_PATH[@]}"; do
        echo "    - path: \"${PACKET_EXC_PATH[$_i]}\""
        echo "      reason: \"${PACKET_EXC_REASON[$_i]}\""
        echo "      affected_section: ${PACKET_EXC_SECTION[$_i]}"
      done
    fi
    echo "  reviewed_packet: packets/$(basename "${packet_saved}")"
    echo "  reviewed_packet_sha256: ${packet_hash}"
    echo "  reviewer: \"codex (session ${REVIEW_SESSION})\""
    echo "  codex_concern: ${codex_concern}"
    echo "  effective_concern: ${effective_concern}"
    [[ -n "${coverage_note}" ]] && echo "  effective_concern_note: ${coverage_note}"
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
    echo "Effective concern: ${effective_concern}"
    echo "Evidence: ${evidence}"
    echo "Coverage: ${PACKET_COVERAGE_STATE}; redactions: ${PACKET_REDACTION_COUNT}; workspace_dirty: $([[ ${PACKET_WORKSPACE_DIRTY} -eq 1 ]] && echo true || echo false)${coverage_note:+; note: ${coverage_note}}"
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
# v0 is an ADVISORY logging pilot. This APPENDS the human's decision and, as a best-effort AUDIT
# AID, notes whether the reviewed artifacts still hash-match what the reviewer saw. It does NOT
# enforce approval eligibility, bind approval to a reviewed state, or guarantee rollback — a
# mismatch is flagged, never blocks. Stronger reviewed-state binding / rollback is deferred:
# see backlog/reviewer-decision-integrity.md. APPROVE is the human's word (Rule 1).
cmd_decision() {
  local feature="$1" stage="$2" decision="$3"; shift 3
  local reason=""
  while [[ $# -gt 0 ]]; do reason="$1"; shift; done
  case "${decision}" in APPROVE_STAGE|REQUEST_CHANGES|STOP) ;; *)
    echo "decision must be APPROVE_STAGE | REQUEST_CHANGES | STOP" >&2; exit 2 ;; esac
  [[ -f "${REVIEW_LOG}" ]] || init_log
  local sha; sha="$(git rev-parse HEAD)"

  # Best-effort audit: do the reviewed artifacts still match what the reviewer saw?
  # Informational only — a mismatch is flagged in the log + a warning, never blocks the decision.
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

  if [[ "${decision}" == "APPROVE_STAGE" && -z "${latest}" ]]; then
    echo "note: no review is on record for ${feature} stage ${stage}. The reviewer is advisory;" >&2
    echo "      consider running 'review' first. Recording the human decision anyway." >&2
  fi

  {
    echo
    echo "## $(now_iso) HUMAN DECISION — ${feature} — Stage ${stage}"
    echo "Commit at decision: ${sha}"
    echo "Decision: ${decision}"
    echo "Reason/next: ${reason}"
    [[ -n "${latest}" ]] && echo "Verified against: ${latest}"
    [[ -n "${verify_lines}" ]] && { echo "Artifact integrity (informational audit, not a gate):"; printf '%s' "${verify_lines}"; }
  } >> "${REVIEW_LOG}"
  echo "decision appended to ${REVIEW_LOG}"
  if [[ ${changed} -eq 1 ]]; then
    echo "WARNING: some reviewed artifacts changed since the review — recorded with that flagged (advisory only)." >&2
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
