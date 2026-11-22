#!/usr/bin/env bash
# Behavioral tests for codeos-monitor-snapshot.sh with stub CodeOS binaries (DBAMON WO-03).
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-monitor.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'monitor snapshot tests failed: %s\n' "$1" >&2; exit 1; }

# A toolkit copy without built engines, so the script resolves the stubs on PATH.
TOOLKIT="${WORK}/toolkit"
mkdir -p "${TOOLKIT}/dba/04-tools/monitor" "${WORK}/bin"
cp "${CODEOS_ROOT}/dba/04-tools/monitor/codeos-monitor-snapshot.sh" "${TOOLKIT}/dba/04-tools/monitor/"
SNAP="${TOOLKIT}/dba/04-tools/monitor/codeos-monitor-snapshot.sh"
cat > "${WORK}/bin/codeos-workflow" <<'EOF'
#!/usr/bin/env bash
[[ "$*" == "status --json --history --since all" ]] || exit 7
[[ -n "${STUB_FAIL:-}" ]] && exit 9
printf '{"schema_version":3,"project":{"name":"Demo","configuration":"DBA-36"},"features":[]}\n'
EOF
cat > "${WORK}/bin/codeos-reviewer" <<'EOF'
#!/usr/bin/env bash
[[ "$*" == "review-status --json" ]] || exit 7
printf '{"health":"OK","assessments":[]}\n'
EOF
chmod +x "${WORK}/bin/"*
export PATH="${WORK}/bin:${PATH}"

PROJECT="${WORK}/project"
WF="${PROJECT}/.codeos/06-workflow"
mkdir -p "${PROJECT}/.codeos/00-project" "${WF}/candidates" "${WF}/test-evidence" \
  "${WF}/provider-ledger" "${PROJECT}/events"
printf 'dba_configuration: DBA-36\n' > "${PROJECT}/.codeos/00-project/codeos.yaml"
printf '{"result":"passed"}\n{"result":"failed"}\n' > "${WF}/verifications.jsonl"
printf '{"feature":"F-1","kind":"ordinary","semantic_revision":1,"trigger_commit":null,"governed_inputs":{"secret":"x"}}\n' \
  > "${WF}/candidates/abc.json"
printf '{"schema":1,"verification":"behavior","runs":[{"runner":"libtest","attempt":1,"cases":[{"id":"a","status":"passed"},{"id":"b","status":"failed"}],"duration_ms":42},{"runner":"vitest","attempt":1,"cases":[{"id":"c","status":"passed"}]}]}\n' \
  > "${WF}/test-evidence/e1.json"
printf '{"attempted":3,"successful":2,"failed":1,"unresolved":0,"total":3}\n' \
  > "${WF}/provider-ledger/F-0020-20261019T055319454662796Z-769895.totals.json"
printf '%s\n' '{"event_type":"Presented","timestamp":1789662524981,"payload":{"p":1}}' \
  '{"event_type":"Presented","timestamp":1789662525981}' '{"event_type":"Failed","timestamp":1789662526981}' \
  > "${PROJECT}/events/runtime_events.jsonl"
git -C "${PROJECT}" init -q && git -C "${PROJECT}" add -A && \
  git -C "${PROJECT}" -c user.name=t -c user.email=t@localhost commit -qm fixture
before="$(git -C "${PROJECT}" status --porcelain; find "${PROJECT}" -path '*/.git' -prune -o -type f -print | sort | xargs sha256sum)"

OUT="${WORK}/external"
bash "${SNAP}" "${PROJECT}" "${OUT}" > "${WORK}/run.log" || fail 'snapshot build failed'
grep -q '^PASS: active snapshot' "${WORK}/run.log" || fail 'build did not verify the active snapshot'
after="$(git -C "${PROJECT}" status --porcelain; find "${PROJECT}" -path '*/.git' -prune -o -type f -print | sort | xargs sha256sum)"
[[ "${before}" == "${after}" ]] || fail 'the project was modified'

ID="$(jq -r .snapshot_id "${OUT}/current.json")"
DIR="${OUT}/snapshots/${ID}"
M="${DIR}/manifest.json"
[[ "$(jq -r '.project + " " + .dba_configuration' "${M}")" == 'Demo DBA-36' ]] || fail 'manifest provenance'
[[ "$(jq -r '.files["verifications.jsonl"].rows' "${M}")" == 2 ]] || fail 'verification row count'
[[ "$(jq -r '.files["decisions.jsonl"].rows' "${M}")" == 0 ]] || fail 'absent store is an empty table'
[[ "$(jq -c '[.duration_ms, .cases, .passed, .failed]' "${DIR}/test_runs.jsonl" | tr '\n' ' ')" == '[42,2,1,1] [null,1,1,0] ' ]] \
  || fail 'test runs: duration and case counts, older duration null'
[[ "$(jq -r .subject "${DIR}/provider_runs.jsonl")" == 'F-0020' ]] || fail 'provider subject'
grep -q secret "${DIR}/candidates.jsonl" && fail 'candidate inputs were exported'
grep -q payload "${DIR}/runtime_event_counts.jsonl" && fail 'event payloads were exported'
[[ "$(jq -c '[.event_type, .count]' "${DIR}/runtime_event_counts.jsonl" | tr '\n' ' ')" == '["Failed",1] ["Presented",2] ' ]] \
  || fail 'runtime event counts'
bash "${SNAP}" --verify-active "${OUT}" >/dev/null || fail 'verify-active rejected a valid snapshot'

# Tampering is detected.
printf 'x\n' >> "${DIR}/decisions.jsonl"
bash "${SNAP}" --verify-active "${OUT}" 2>/dev/null && fail 'verify-active accepted a modified file'
: > "${DIR}/decisions.jsonl"
bash "${SNAP}" --verify-active "${OUT}" >/dev/null || fail 'restored snapshot no longer verifies'
touch "${DIR}/extra.jsonl"
bash "${SNAP}" --verify-active "${OUT}" 2>/dev/null && fail 'verify-active accepted an extra file'
rm "${DIR}/extra.jsonl"

# A failed build activates nothing and leaves no staging folder.
current="$(cat "${OUT}/current.json")"
STUB_FAIL=1 bash "${SNAP}" "${PROJECT}" "${OUT}" 2>/dev/null && fail 'a failed projection still succeeded'
[[ "$(cat "${OUT}/current.json")" == "${current}" ]] || fail 'a failed build changed current.json'
compgen -G "${OUT}/.staging-*" >/dev/null && fail 'a failed build left a staging folder'
[[ "$(ls "${OUT}/snapshots" | wc -l)" == 1 ]] || fail 'a failed build left a snapshot'

# The external root may not be inside the project.
bash "${SNAP}" "${PROJECT}" "${PROJECT}/out" 2>/dev/null && fail 'accepted an external root inside the project'
[[ ! -e "${PROJECT}/out" ]] || fail 'a refused external root was still created inside the project'

printf 'monitor snapshot tests: PASS\n'
