#!/usr/bin/env bash
# codeos-static-analytics.sh — immutable, hash-verified snapshot of clippy, cargo-audit, and
# cargo-deny results for the toolkit and the given projects (DBAMON WO-07).
#
#   codeos-static-analytics.sh <external-root> [project ...]
#   codeos-static-analytics.sh --verify-active <external-root>
#
# Every Cargo workspace (each tracked Cargo.lock) is analysed from a `git archive` export of HEAD in
# a temporary folder, so no analysed repository, not even its .git metadata, changes. Builds use
# a reusable cache outside the repositories (CODEOS_MONITOR_TARGET_CACHE, default
# ~/.cache/codeos-monitor/target). cargo-audit fetches the public RustSec database; nothing about a
# project is sent. A tool that is missing, not configured, or fails is recorded as such and its
# findings stay null: absent results are never reported as zero findings.
#
# Tables (one JSON object per line):
#   static_runs.jsonl    one row per workspace and tool: status ok | failed | not_configured |
#                        unavailable, tool version, finding count (null unless ok), note
#   clippy_lints.jsonl   distinct clippy/rustc diagnostics per workspace, by lint and level
#   advisories.jsonl     cargo-audit vulnerabilities and warnings (unmaintained, yanked, unsound),
#                        with `compiled` (true | false | null): whether the crate is in the build graph
#   deny_findings.jsonl  cargo-deny diagnostics per workspace, by code and severity
#
# Exit codes: 0 success, 1 usage, 2 failure (nothing activated / verification failed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd -P)"
# shellcheck source=snapshot-lib.sh
source "${SCRIPT_DIR}/snapshot-lib.sh"
usage() { sed -n '5,6p' "${BASH_SOURCE[0]}" | sed 's/^#  *//' >&2; exit 1; }

if [[ "${1:-}" == "--verify-active" ]]; then
  [[ $# -eq 2 ]] || usage
  verify_active "$2"
  exit 0
fi
[[ $# -ge 1 ]] || usage
ROOT="$(realpath -m "$1")"; shift
REPOS=("${CODEOS_ROOT}")
for project in "$@"; do
  path="$(cd "${project}" && pwd -P)" || fail "project not found: ${project}"
  [[ -f "${path}/.codeos/00-project/codeos.yaml" ]] || fail "not a CodeOS project: ${path}"
  git -C "${path}" rev-parse --git-dir >/dev/null 2>&1 || fail "not a git repository: ${path}"
  REPOS+=("${path}")
done
for repo in "${REPOS[@]}"; do
  case "${ROOT}/" in "${repo}/"*) fail 'the external root must be outside every analysed repository' ;; esac
done
CACHE="${CODEOS_MONITOR_TARGET_CACHE:-${XDG_CACHE_HOME:-${HOME}/.cache}/codeos-monitor/target}"
mkdir -p "${ROOT}" "${CACHE}"
snapshot_stage "${ROOT}"
STAGE="${SNAPSHOT_STAGE}"
EXPORTS="$(mktemp -d "${TMPDIR:-/tmp}/codeos-static.XXXXXX")"
trap 'rm -rf "${STAGE}" "${EXPORTS}"' EXIT
: > "${STAGE}/static_runs.jsonl"; : > "${STAGE}/clippy_lints.jsonl"
: > "${STAGE}/advisories.jsonl"; : > "${STAGE}/deny_findings.jsonl"

# The installed version of a cargo subcommand, or empty when it is not installed.
tool_version() { cargo "$1" --version 2>/dev/null | head -1 || true; }
CLIPPY_VERSION="$(tool_version clippy)"
AUDIT_VERSION="$(tool_version audit)"
DENY_VERSION="$(tool_version deny)"

run_row() { # <repo> <workspace> <tool> <status> <version> <findings|null> <note>
  jq -nc --arg repo "$1" --arg ws "$2" --arg tool "$3" --arg status "$4" --arg version "$5" \
    --argjson findings "$6" --arg note "$7" \
    '{repo: $repo, workspace: $ws, tool: $tool, status: $status,
      tool_version: (if $version == "" then null else $version end), findings: $findings,
      note: (if $note == "" then null else $note end)}' >> "${STAGE}/static_runs.jsonl"
}

for repo in "${REPOS[@]}"; do
  name="$(basename "${repo}")"
  export_dir="${EXPORTS}/${name}"
  mkdir -p "${export_dir}"
  git -C "${repo}" archive --format=tar HEAD | tar -x -C "${export_dir}"
  while IFS= read -r lock; do
    ws="$(dirname "${lock}")"
    dir="${export_dir}/${ws}"
    [[ -f "${dir}/Cargo.toml" ]] || continue
    slug="$(printf '%s' "${ws}" | tr '/' '_')"; [[ "${ws}" == . ]] && slug=root
    target="${CACHE}/${name}/${slug}"
    log="${EXPORTS}/log"

    # clippy: distinct diagnostics (code, file, line, column) from workspace crates.
    if [[ -z "${CLIPPY_VERSION}" ]]; then
      run_row "${name}" "${ws}" clippy unavailable "" null 'cargo clippy is not installed'
    elif (cd "${dir}" && CARGO_TARGET_DIR="${target}" cargo clippy --workspace --all-targets \
            --locked --message-format=json 2>"${log}") > "${EXPORTS}/clippy.json"; then
      jq -c --arg repo "${name}" --arg ws "${ws}" 'select(.reason == "compiler-message")
          | .message | select(.level == "warning" or .level == "error") | select(.code != null)
          | {lint: .code.code, level, at: ((.spans[0] // {}) | "\(.file_name):\(.line_start):\(.column_start)")}' \
          "${EXPORTS}/clippy.json" | sort -u \
        | jq -sc --arg repo "${name}" --arg ws "${ws}" 'group_by([.lint, .level])[]
            | {repo: $repo, workspace: $ws, lint: .[0].lint, level: .[0].level, count: length}' \
        > "${EXPORTS}/lints.jsonl"
      cat "${EXPORTS}/lints.jsonl" >> "${STAGE}/clippy_lints.jsonl"
      run_row "${name}" "${ws}" clippy ok "${CLIPPY_VERSION}" \
        "$(jq -s 'map(.count) | add // 0' "${EXPORTS}/lints.jsonl")" ''
    else
      run_row "${name}" "${ws}" clippy failed "${CLIPPY_VERSION}" null \
        "$(grep -m1 -E '^error' "${log}" | cut -c1-200 || echo 'cargo clippy failed')"
    fi

    # cargo-audit: exit status 1 only means vulnerabilities were found; valid JSON decides success.
    if [[ -z "${AUDIT_VERSION}" ]]; then
      run_row "${name}" "${ws}" cargo-audit unavailable "" null 'cargo-audit is not installed'
    else
      (cd "${dir}" && cargo audit --json 2>"${log}") > "${EXPORTS}/audit.json" || true
      if jq -e '.vulnerabilities' "${EXPORTS}/audit.json" >/dev/null 2>&1; then
        jq -c --arg repo "${name}" --arg ws "${ws}" '
            ([.vulnerabilities.list[] | {kind: "vulnerability", id: .advisory.id,
               package: .package.name, version: .package.version}]
             + [(.warnings // {}) | to_entries[] | .key as $kind | .value[]
               | {kind: $kind, id: (.advisory.id // null), package: .package.name,
                  version: .package.version}])[]
            | {repo: $repo, workspace: $ws} + .' "${EXPORTS}/audit.json" > "${EXPORTS}/adv.jsonl"
        # cargo-audit reads Cargo.lock, which also lists optional crates that are never built.
        # `compiled` says whether the crate is in the build graph; null when that is unknown.
        while IFS= read -r row; do
          spec="$(jq -r '"\(.package)@\(.version)"' <<<"${row}")"
          if tree="$(cd "${dir}" && CARGO_TARGET_DIR="${target}" cargo tree --locked --target all \
                -e normal,build,dev -i "${spec}" 2>/dev/null)"; then
            compiled="$([[ -n "${tree}" ]] && echo true || echo false)"
          else
            compiled=null
          fi
          jq -c --argjson c "${compiled}" '. + {compiled: $c}' <<<"${row}" >> "${STAGE}/advisories.jsonl"
        done < "${EXPORTS}/adv.jsonl"
        run_row "${name}" "${ws}" cargo-audit ok "${AUDIT_VERSION}" "$(grep -c '' "${EXPORTS}/adv.jsonl" || true)" ''
      else
        run_row "${name}" "${ws}" cargo-audit failed "${AUDIT_VERSION}" null \
          "$(grep -m1 -i 'error' "${log}" | cut -c1-200 || echo 'cargo audit produced no report')"
      fi
    fi

    # cargo-deny: only with a committed deny.toml; default policy would reject every license.
    config=''
    for candidate in "${dir}/deny.toml" "${export_dir}/deny.toml"; do
      [[ -f "${candidate}" ]] && { config="${candidate}"; break; }
    done
    if [[ -z "${DENY_VERSION}" ]]; then
      run_row "${name}" "${ws}" cargo-deny unavailable "" null 'cargo-deny is not installed'
    elif [[ -z "${config}" ]]; then
      run_row "${name}" "${ws}" cargo-deny not_configured "${DENY_VERSION}" null 'no committed deny.toml'
    else
      (cd "${dir}" && cargo deny --format json check --config "${config}" 2>"${EXPORTS}/deny.json" >/dev/null) || true
      if jq -e -s 'any(.[]; .type == "summary")' "${EXPORTS}/deny.json" >/dev/null 2>&1; then
        jq -c 'select(.type == "diagnostic") | .fields | {code: (.code // "unknown"), severity}' \
            "${EXPORTS}/deny.json" \
          | jq -sc --arg repo "${name}" --arg ws "${ws}" 'group_by([.code, .severity])[]
              | {repo: $repo, workspace: $ws, code: .[0].code, severity: .[0].severity, count: length}' \
          > "${EXPORTS}/deny.jsonl"
        cat "${EXPORTS}/deny.jsonl" >> "${STAGE}/deny_findings.jsonl"
        run_row "${name}" "${ws}" cargo-deny ok "${DENY_VERSION}" \
          "$(jq -s 'map(.count) | add // 0' "${EXPORTS}/deny.jsonl")" ''
      else
        run_row "${name}" "${ws}" cargo-deny failed "${DENY_VERSION}" null 'cargo deny produced no summary'
      fi
    fi
  done < <(git -C "${repo}" ls-files | grep -E '(^|/)Cargo\.lock$')
done

repos="$(for repo in "${REPOS[@]}"; do
  jq -nc --arg n "$(basename "${repo}")" --arg h "$(git -C "${repo}" rev-parse HEAD)" '{name: $n, head: $h}'
done | jq -sc .)"
jq -n --argjson v "${SNAPSHOT_SCHEMA_VERSION}" --arg id "${SNAPSHOT_ID}" \
  --arg generated "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --argjson repos "${repos}" \
  --argjson files "$(snapshot_files "${STAGE}")" \
  --arg clippy "${CLIPPY_VERSION}" --arg audit "${AUDIT_VERSION}" --arg deny "${DENY_VERSION}" \
  '{schema_version: $v, snapshot_id: $id, status: "complete", kind: "static-analytics",
    generated_at: $generated, repositories: $repos,
    tools: {clippy: (if $clippy == "" then null else $clippy end),
            cargo_audit: (if $audit == "" then null else $audit end),
            cargo_deny: (if $deny == "" then null else $deny end)},
    files: $files}' > "${STAGE}/manifest.json"
snapshot_activate "${ROOT}"
rm -rf "${EXPORTS}"
