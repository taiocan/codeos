#!/usr/bin/env bash
# Behavioral tests for codeos-toolkit-analytics.sh on synthetic git histories (DBAMON WO-04).
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-analytics.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'toolkit analytics tests failed: %s\n' "$1" >&2; exit 1; }
commit() { git -C "$1" add -A && git -C "$1" -c user.name=t -c user.email=t@localhost commit -qm "$2"; }

# A synthetic toolkit: two configurations issued at different commits.
TK="${WORK}/toolkit"
mkdir -p "${TK}/dba/04-tools/monitor" "${TK}/dba/00-entry/configurations" "${TK}/dba/02-policies/review" \
  "${TK}/dba/05-guidance" "${TK}/src" "${TK}/maintenance"
cp "${CODEOS_ROOT}/dba/04-tools/monitor/"{codeos-toolkit-analytics.sh,snapshot-lib.sh} "${TK}/dba/04-tools/monitor/"
git -C "${TK}" init -q
printf 'status: approved\napproved_at: 2026-01-01\n' > "${TK}/dba/00-entry/configurations/DBA-1.yaml"
printf '# v1\n' > "${TK}/dba/02-policies/review/v1.md"
printf '| **Term** | x |\n' > "${TK}/dba/05-guidance/terminology.md"
printf '#[arg(long)]\nfn a() {}\n#[test]\nfn t() {}\n' > "${TK}/src/main.rs"
commit "${TK}" one
printf 'status: approved\napproved_at: 2026-02-01\n' > "${TK}/dba/00-entry/configurations/DBA-2.yaml"
printf '# v2\n' > "${TK}/dba/02-policies/review/v2.md"
printf '#[arg(long = "x")]\nfn b() {}\n' >> "${TK}/src/main.rs"
printf 'record\n' > "${TK}/maintenance/log.md"
commit "${TK}" two

# A synthetic project that adopts DBA-1, edits unrelated config, then adopts DBA-2.
PJ="${WORK}/project"
mkdir -p "${PJ}/.codeos/00-project" "${PJ}/src" "${PJ}/evidence"
git -C "${PJ}" init -q
printf 'dba_configuration: DBA-1\n' > "${PJ}/.codeos/00-project/codeos.yaml"
printf 'fn main() {}\n' > "${PJ}/src/my file.rs"
commit "${PJ}" adopt-1
printf 'dba_configuration: DBA-1\nplatform: x\n' > "${PJ}/.codeos/00-project/codeos.yaml"
printf '{"big": 1}\n' > "${PJ}/evidence/run.json"
commit "${PJ}" unrelated
printf 'dba_configuration: DBA-2\n' > "${PJ}/.codeos/00-project/codeos.yaml"
printf 'fn main() { x() }\n' > "${PJ}/src/my file.rs"
commit "${PJ}" adopt-2
git -C "${PJ}" mv "src/my file.rs" src/renamed.rs && commit "${PJ}" rename
snapshot_state() { for r in "${TK}" "${PJ}"; do git -C "${r}" status --porcelain; git -C "${r}" rev-parse HEAD; done; }
before="$(snapshot_state)"

SCRIPT="${TK}/dba/04-tools/monitor/codeos-toolkit-analytics.sh"
OUT="${WORK}/external"
bash "${SCRIPT}" "${OUT}" "${PJ}" > "${WORK}/run.log" || fail 'analytics build failed'
grep -q '^PASS: active snapshot' "${WORK}/run.log" || fail 'build did not verify the active snapshot'
[[ "$(snapshot_state)" == "${before}" ]] || fail 'an analysed repository was modified'
D="${OUT}/snapshots/$(jq -r .snapshot_id "${OUT}/current.json")"

[[ "$(jq -c '[.configuration, .approved_at, .corrective]' "${D}/configurations.jsonl" | tr '\n' ' ')" \
  == '["DBA-1","2026-01-01",null] ["DBA-2","2026-02-01",null] ' ]] || fail 'configurations'
[[ "$(jq -c '[.as_of, .policy_versions, .configurations, .cli_flags, .rust_tests, .glossary_terms, .codeos_attributable_defects]' \
  "${D}/governance_growth.jsonl" | tr '\n' ' ')" == '["DBA-1",1,1,1,1,1,null] ["DBA-2",2,2,2,1,1,null] ["HEAD",2,2,2,1,1,null] ' ]] \
  || fail 'governance growth counts per issue commit'
[[ "$(jq -r .doctrine_versions "${D}/governance_growth.jsonl" | sort -u)" == null ]] \
  || fail 'an absent source must be null, not 0'
[[ "$(jq -c '[.project, .configuration]' "${D}/adoptions.jsonl" | tr '\n' ' ')" \
  == '["project","DBA-1"] ["project","DBA-2"] ' ]] || fail 'adoptions: only configuration changes'
jq -e -s 'map(select(.path | test("^(maintenance|\\.codeos|evidence)/"))) | length == 0' "${D}/churn.jsonl" >/dev/null \
  || fail 'bookkeeping paths were counted as churn'
jq -e -s 'map(select(.path | test("=>"))) | length == 0' "${D}/churn.jsonl" >/dev/null || fail 'rename notation in churn'
jq -e -s 'map(select(.repo == "project" and .path == "src/my file.rs")) | length == 3' "${D}/churn.jsonl" >/dev/null \
  || fail 'a path with a space: add, change, and delete-on-rename'
[[ "$(jq -c 'select(.repo == "project") | [.path, .lines]' "${D}/file_sizes.jsonl")" == '["src/renamed.rs",1]' ]] \
  || fail 'current file sizes exclude bookkeeping'
bash "${SCRIPT}" --verify-active "${OUT}" >/dev/null || fail 'verify-active rejected a valid snapshot'
printf '{}\n' >> "${D}/churn.jsonl"
bash "${SCRIPT}" --verify-active "${OUT}" 2>/dev/null && fail 'verify-active accepted a modified file'

bash "${SCRIPT}" "${PJ}/out" "${PJ}" 2>/dev/null && fail 'accepted an external root inside a repository'
[[ ! -e "${PJ}/out" ]] || fail 'a refused external root was still created'
bash "${SCRIPT}" "${OUT}" "${WORK}" 2>/dev/null && fail 'accepted a non-CodeOS project'

printf 'toolkit analytics tests: PASS\n'
