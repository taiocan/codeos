#!/usr/bin/env bash
# codeos-monitor-snapshot.sh — immutable, hash-verified snapshot of one project's CodeOS records for
# an external read-only consumer such as a Power BI report (DBAMON WO-03).
#
#   codeos-monitor-snapshot.sh <project> <external-root>   build, verify, and activate a snapshot
#   codeos-monitor-snapshot.sh --verify-active <external-root>
#
# Layout (same as the AII report): <external-root>/snapshots/<id>/{manifest.json, data files} and
# <external-root>/current.json naming the active snapshot. A snapshot is built in a staging folder,
# verified there, renamed into place, and only then activated; a failure leaves current.json as it
# was. The project is only read: status and review-status are read-only projections, and every
# other file is derived from records the project already holds. CodeOS owns truth; nothing here is
# workflow, validity, or acceptance evidence. No packet text is exported.
#
# Exit codes: 0 success, 1 usage, 2 failure (nothing activated / verification failed).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd -P)"
# shellcheck source=snapshot-lib.sh
source "${SCRIPT_DIR}/snapshot-lib.sh"
usage() { sed -n '4,5p' "${BASH_SOURCE[0]}" | sed 's/^#  *//' >&2; exit 1; }

if [[ "${1:-}" == "--verify-active" ]]; then
  [[ $# -eq 2 ]] || usage
  verify_active "$2"
  exit 0
fi
[[ $# -eq 2 ]] || usage
PROJECT="$(cd "$1" && pwd -P)" || fail "project not found: $1"
[[ -f "${PROJECT}/.codeos/00-project/codeos.yaml" ]] || fail "not a CodeOS project: ${PROJECT}"
ROOT="$(realpath -m "$2")"
case "${ROOT}/" in "${PROJECT}/"*) fail 'the external root must be outside the project' ;; esac
mkdir -p "${ROOT}"

# The toolkit's own qualified binaries, as the reviewer shim resolves them.
binary() {
  local built="${CODEOS_ROOT}/dba/04-tools/$1/engine/target/release/codeos-$1"
  if [[ -x "${built}" ]]; then printf '%s' "${built}"; else command -v "codeos-$1" || fail "codeos-$1 not found"; fi
}
WORKFLOW="$(binary workflow)"
REVIEWER="$(binary reviewer)"
WF="${PROJECT}/.codeos/06-workflow"

snapshot_stage "${ROOT}"
STAGE="${SNAPSHOT_STAGE}"

# 1. CodeOS projections, verbatim.
(cd "${PROJECT}" && "${WORKFLOW}" status --json --history --since all) > "${STAGE}/status.json" \
  || fail 'codeos-workflow status failed'
(cd "${PROJECT}" && "${REVIEWER}" review-status --json) > "${STAGE}/review_status.json" \
  || fail 'codeos-reviewer review-status failed'

# 2. Append-only records, copied; absent stores become empty tables.
for store in verifications decisions; do
  if [[ -f "${WF}/${store}.jsonl" ]]; then cp "${WF}/${store}.jsonl" "${STAGE}/${store}.jsonl"
  else : > "${STAGE}/${store}.jsonl"; fi
done

# 3. Flat derived tables. Each row states only recorded fields; absent values stay null.
: > "${STAGE}/candidates.jsonl"
for file in "${WF}"/candidates/*.json; do
  [[ -f "${file}" ]] || continue
  jq -c --arg id "$(basename "${file}" .json)" \
    '{candidate_id: $id, feature, kind, semantic_revision, trigger_commit,
      predecessor_bundle, schema_version}' "${file}" >> "${STAGE}/candidates.jsonl"
done
: > "${STAGE}/test_runs.jsonl"
for file in "${WF}"/test-evidence/*.json; do
  [[ -f "${file}" ]] || continue
  jq -c --arg id "$(basename "${file}" .json)" '.verification as $v | .runs[] |
    {evidence_id: $id, verification: $v, runner, attempt, duration_ms,
     cases: (.cases | length),
     passed: ([.cases[] | select(.status == "passed")] | length),
     failed: ([.cases[] | select(.status == "failed")] | length),
     skipped: ([.cases[] | select(.status == "skipped")] | length),
     flaky: ([.cases[] | select(.status == "flaky")] | length)}' "${file}" >> "${STAGE}/test_runs.jsonl"
done
: > "${STAGE}/provider_runs.jsonl"
for file in "${WF}"/provider-ledger/*.totals.json; do
  [[ -f "${file}" ]] || continue
  jq -c --arg run "$(basename "${file}" .totals.json)" \
    '{run: $run, subject: ($run | sub("-[0-9]{8}T[0-9]+Z-[0-9]+$"; "")), attempted, successful, failed, unresolved, total}' \
    "${file}" >> "${STAGE}/provider_runs.jsonl"
done
# Runtime event outcomes per UTC day and event type, never event payloads.
if [[ -f "${PROJECT}/events/runtime_events.jsonl" ]]; then
  jq -c 'select(.event_type and .timestamp) |
         {day: ((.timestamp / 1000 | floor) | strftime("%Y-%m-%d")), event_type}' \
    "${PROJECT}/events/runtime_events.jsonl" \
    | jq -sc 'group_by([.day, .event_type])[] | {day: .[0].day, event_type: .[0].event_type, count: length}' \
    > "${STAGE}/runtime_event_counts.jsonl"
else
  : > "${STAGE}/runtime_event_counts.jsonl"
fi

# 4. Manifest: provenance plus the hash and row count of every file.
files="$(snapshot_files "${STAGE}")"
release="$(jq -r '.release_id // empty' "${CODEOS_ROOT}/maintenance/release-manifest.json" 2>/dev/null || true)"
jq -n --argjson v "${SNAPSHOT_SCHEMA_VERSION}" --arg id "${SNAPSHOT_ID}" --argjson files "${files}" \
  --arg generated "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --arg project_root "${PROJECT}" \
  --slurpfile status "${STAGE}/status.json" --arg release "${release}" \
  --arg workflow_sha "$(sha256sum "${WORKFLOW}" | cut -d' ' -f1)" \
  --arg reviewer_sha "$(sha256sum "${REVIEWER}" | cut -d' ' -f1)" \
  '{schema_version: $v, snapshot_id: $id, status: "complete", generated_at: $generated,
    project: $status[0].project.name, project_root: $project_root,
    dba_configuration: $status[0].project.configuration,
    toolkit: {release_id: (if $release == "" then null else $release end),
              workflow_sha256: $workflow_sha, reviewer_sha256: $reviewer_sha},
    files: $files}' > "${STAGE}/manifest.json"

# 5. Verify in staging, move into place, then activate atomically.
snapshot_activate "${ROOT}"
