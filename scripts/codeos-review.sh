#!/usr/bin/env bash
# codeos-review.sh — thin shim delegating to the compiled Rust binary, with automatic
# Controlled Plain English status injection for the two packet-building subcommands.
# Subcommands: review / plan / decision / diagnose / stage-start / check-drift / generate-*
# (see: codeos-reviewer --help)
# To build: cargo build --release --manifest-path tools/reviewer/Cargo.toml
#
# Exit codes: 1 = not a git repo, 2 = binary not found, 7 = Controlled Plain English
# status file is malformed (see "Controlled Plain English automatic status injection"
# below). Codes 0/3/4/5/6 belong to the Rust binary itself (tools/reviewer/src/main.rs)
# and are passed through unchanged.
set -euo pipefail
# Preserve the original precondition: the shim requires the CALLER to be inside some git
# repository (a property of the project being reviewed) — unrelated to where the binary
# itself lives, so this check is deliberately kept separate from binary-path resolution
# below, running first, exactly as before this change.
git rev-parse --show-toplevel >/dev/null 2>&1 || { echo "error: not inside a git repository" >&2; exit 1; }

# Resolve the binary relative to this script's own physical location (following the
# .codeos symlink when invoked from a downstream project), not the calling repo's git
# root — git rev-parse --show-toplevel from within a downstream project resolves to that
# project's own root, not through .codeos to Codeos, which is where the binary actually
# lives. pwd -P (physical) is required, not plain pwd, to resolve through the symlink
# rather than preserve its logical name.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd -P)"
BINARY="${CODEOS_ROOT}/tools/reviewer/target/release/codeos-reviewer"
if [[ ! -x "${BINARY}" ]]; then
  command -v codeos-reviewer >/dev/null 2>&1 && BINARY="codeos-reviewer" || {
    echo "error: binary not found at ${BINARY} and not on PATH" >&2
    echo "       Build: cargo build --release --manifest-path ${CODEOS_ROOT}/tools/reviewer/Cargo.toml" >&2
    exit 2
  }
fi

SUBCOMMAND="${1:-}"

# ── Controlled Plain English automatic status injection ────────────────────────────────
# Only "review" and "plan" build a reviewer packet (see tools/reviewer/src/main.rs); every
# other subcommand (decision, diagnose, stage-start, check-drift, generate-*) is passed
# through unchanged below, exactly as before this change.
if [[ "${SUBCOMMAND}" == "review" || "${SUBCOMMAND}" == "plan" ]]; then
  # Context resolution: this one shared script is reached two ways — directly as
  # scripts/codeos-review.sh from within this repo (a self-development review), or as
  # .codeos/scripts/codeos-review.sh through the downstream symlink (a downstream
  # project's review). The caller's own git root (already required to exist by the
  # precondition above) tells them apart: it equals CODEOS_ROOT only in the former case.
  CALLER_ROOT="$(cd "$(git rev-parse --show-toplevel)" && pwd -P)"
  STAGE_ARG="${3:-}"

  if [[ "${CALLER_ROOT}" == "${CODEOS_ROOT}" ]]; then
    CPE_CONFIG="${CODEOS_ROOT}/config/writing-discipline.yaml"
  else
    CPE_CONFIG="${CALLER_ROOT}/architecture/controlled-plain-english.yaml"
  fi

  # Four-outcome resolution, per UPG-0056's Optional Mechanism Status Convention
  # (templates/conventions.md): absent -> disabled; exact "status: disabled" -> disabled;
  # exact "status: enabled" -> enabled; anything else -> configuration error. Leading/
  # trailing blank lines are allowed and line endings are normalized (CRLF -> LF) before
  # comparison; internal whitespace, case, tabs, and comments are NOT normalized and make
  # the file invalid, exactly like every other consumer of this convention.
  CPE_STATUS=""
  if [[ ! -f "${CPE_CONFIG}" ]]; then
    CPE_STATUS="disabled"
  else
    CPE_NONBLANK=()
    while IFS= read -r _cpe_line || [[ -n "${_cpe_line}" ]]; do
      [[ -n "${_cpe_line}" ]] && CPE_NONBLANK+=("${_cpe_line}")
    done < <(tr -d '\r' < "${CPE_CONFIG}")

    if [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: disabled" ]]; then
      CPE_STATUS="disabled"
    elif [[ "${#CPE_NONBLANK[@]}" -eq 1 && "${CPE_NONBLANK[0]}" == "status: enabled" ]]; then
      CPE_STATUS="enabled"
    else
      # Malformed or contradictory configuration: fail BEFORE invoking the reviewer. This
      # is an invocation precondition failure (same class as "binary not found" above),
      # never a reviewer finding — ordinary style non-compliance in generated prose is
      # always and only a reviewer finding, never a packet-generation failure.
      echo "error: invalid Controlled Plain English status file: ${CPE_CONFIG}" >&2
      echo "       must contain exactly one non-blank line: 'status: enabled' or 'status: disabled'" >&2
      exit 7
    fi
  fi

  # The generated artifact is explicitly synthetic: a recognizable filename
  # (codeos-cpe-status.*), a deterministic three-line body, and safe temp-file handling
  # (mktemp, quoted paths, trap cleanup covering both the success and failure paths).
  # NOTE: this branch cannot end in `exec` (see below) — a bash EXIT trap never fires
  # across exec, since exec replaces the process image instead of letting the shell
  # return to run its traps. To still guarantee cleanup, the binary is invoked as an
  # ordinary subprocess here and this script exits with its exact exit code afterward.
  CPE_STATUS_FILE="$(mktemp "${TMPDIR:-/tmp}/codeos-cpe-status.XXXXXX")"
  trap 'rm -f "${CPE_STATUS_FILE}"' EXIT
  {
    printf 'Controlled Plain English status for this review: %s\n' "${CPE_STATUS}"
    printf 'Source: %s\n' "${CPE_CONFIG}"
    printf 'Applicable scope: %s\n' "${STAGE_ARG}"
  } > "${CPE_STATUS_FILE}"

  "${BINARY}" "$@" "${CPE_STATUS_FILE}"
  exit $?
fi

exec "${BINARY}" "$@"
