#!/usr/bin/env bash
# Behavioral tests for codeos-static-analytics.sh with a stub cargo (DBAMON WO-07).
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-static-test.XXXXXX")"
trap 'rm -rf "${WORK}"' EXIT
fail() { printf 'static analytics tests failed: %s\n' "$1" >&2; exit 1; }
commit() { git -C "$1" add -A && git -C "$1" -c user.name=t -c user.email=t@localhost commit -qm "$2"; }

# Stub cargo: canned clippy/audit/deny output keyed on the workspace directory name.
mkdir -p "${WORK}/bin"
cat > "${WORK}/bin/cargo" <<'EOF'
#!/usr/bin/env bash
sub="$1"; shift
[[ "$1" == "--version" ]] && { [[ "${sub}" == deny && -n "${STUB_NO_DENY:-}" ]] && exit 101; echo "cargo-${sub} 9.9.9"; exit 0; }
ws="$(basename "$PWD")"
case "${sub}" in
  clippy)
    [[ "${ws}" == broken ]] && { echo 'error[E0425]: cannot find value' >&2; exit 101; }
    lint() { printf '{"reason":"compiler-message","message":{"level":"warning","code":{"code":"%s"},"spans":[{"file_name":"src/lib.rs","line_start":%s,"column_start":1}]}}\n' "$1" "$2"; }
    lint clippy::needless_return 3; lint clippy::needless_return 3; lint clippy::needless_return 9; lint unused_variables 4
    printf '{"reason":"compiler-message","message":{"level":"note","code":null,"spans":[]}}\n'
    ;;
  audit)
    printf '{"vulnerabilities":{"found":true,"list":[{"advisory":{"id":"RUSTSEC-2026-0001"},"package":{"name":"tokio","version":"1.0.0"}},{"advisory":{"id":"RUSTSEC-2026-0003"},"package":{"name":"ghost","version":"2.0.0"}}]},"warnings":{"unmaintained":[{"advisory":{"id":"RUSTSEC-2026-0002"},"package":{"name":"old","version":"0.1.0"}}]}}\n'
    exit 1 ;;
  tree)
    # Built crate: printed; lockfile-only crate: nothing printed; unknown spec: cargo error.
    case "${!#}" in tokio@*) echo 'tokio v1.0.0' ;; old@*) ;; *) exit 101 ;; esac ;;
  deny)
    printf '%s\n' '{"type":"diagnostic","fields":{"severity":"error","code":"rejected"}}' \
      '{"type":"diagnostic","fields":{"severity":"error","code":"rejected"}}' '{"type":"summary","fields":{}}' >&2 ;;
esac
EOF
chmod +x "${WORK}/bin/cargo"
export PATH="${WORK}/bin:${PATH}"
export CODEOS_MONITOR_TARGET_CACHE="${WORK}/cache"

# A synthetic toolkit with one good workspace and one that fails to build.
TK="${WORK}/toolkit"
mkdir -p "${TK}/dba/04-tools/monitor" "${TK}/good" "${TK}/broken"
cp "${CODEOS_ROOT}/dba/04-tools/monitor/"{codeos-static-analytics.sh,snapshot-lib.sh} "${TK}/dba/04-tools/monitor/"
for ws in good broken; do printf '[package]\nname = "%s"\n' "${ws}" > "${TK}/${ws}/Cargo.toml"; : > "${TK}/${ws}/Cargo.lock"; done
printf '[advisories]\n' > "${TK}/good/deny.toml"
git -C "${TK}" init -q && commit "${TK}" toolkit
# A project with an uncommitted deny.toml: only committed files are analysed.
PJ="${WORK}/project"
mkdir -p "${PJ}/.codeos/00-project" "${PJ}/backend"
# A workspace at the repository root, as in AII.
printf '[package]\nname = "rootws"\n' > "${PJ}/Cargo.toml"; : > "${PJ}/Cargo.lock"
printf 'dba_configuration: DBA-1\n' > "${PJ}/.codeos/00-project/codeos.yaml"
printf '[package]\nname = "backend"\n' > "${PJ}/backend/Cargo.toml"; : > "${PJ}/backend/Cargo.lock"
git -C "${PJ}" init -q && commit "${PJ}" project
printf '[advisories]\n' > "${PJ}/deny.toml"
state() { for r in "${TK}" "${PJ}"; do git -C "${r}" status --porcelain; git -C "${r}" rev-parse HEAD; ls -a "${r}/.git"; done; }
before="$(state)"

SCRIPT="${TK}/dba/04-tools/monitor/codeos-static-analytics.sh"
OUT="${WORK}/external"
bash "${SCRIPT}" "${OUT}" "${PJ}" > "${WORK}/run.log" || fail 'build failed'
grep -q '^PASS: active snapshot' "${WORK}/run.log" || fail 'build did not verify the active snapshot'
[[ "$(state)" == "${before}" ]] || fail 'an analysed repository changed'
D="${OUT}/snapshots/$(jq -r .snapshot_id "${OUT}/current.json")"
row() { jq -c --arg r "$1" --arg w "$2" --arg t "$3" 'select(.repo == $r and .workspace == $w and .tool == $t) | [.status, .findings]' "${D}/static_runs.jsonl"; }

[[ "$(row toolkit good clippy)" == '["ok",3]' ]] || fail 'clippy: duplicates collapse, notes ignored'
[[ "$(jq -c 'select(.workspace == "good") | [.lint, .count]' "${D}/clippy_lints.jsonl" | tr '\n' ' ')" \
  == '["clippy::needless_return",2] ["unused_variables",1] ' ]] || fail 'clippy lint counts'
[[ "$(row toolkit broken clippy)" == '["failed",null]' ]] || fail 'a failed build must be null, not zero'
[[ "$(row toolkit good cargo-audit)" == '["ok",3]' ]] || fail 'audit exit 1 with a report is ok'
[[ "$(jq -c 'select(.workspace == "good") | [.kind, .id, .compiled]' "${D}/advisories.jsonl" | tr '\n' ' ')" \
  == '["vulnerability","RUSTSEC-2026-0001",true] ["vulnerability","RUSTSEC-2026-0003",null] ["unmaintained","RUSTSEC-2026-0002",false] ' ]] \
  || fail 'advisories: compiled is true when built, false when lockfile-only, null when unknown'
[[ "$(row toolkit good cargo-deny)" == '["ok",2]' ]] || fail 'deny with committed config'
[[ "$(jq -c 'select(.workspace == "good") | [.code, .count]' "${D}/deny_findings.jsonl")" == '["rejected",2]' ]] || fail 'deny counts'
[[ "$(row project backend cargo-deny)" == '["not_configured",null]' ]] || fail 'uncommitted deny.toml must not be used'
[[ "$(row project . clippy)" == '["ok",3]' ]] || fail 'a repository-root workspace must be analysed'
[[ "$(jq -r .tools.cargo_deny "${D}/manifest.json")" == 'cargo-deny 9.9.9' ]] || fail 'tool versions in manifest'
bash "${SCRIPT}" --verify-active "${OUT}" >/dev/null || fail 'verify-active rejected a valid snapshot'

# A missing tool is unavailable with null findings, and a temp export is always removed.
STUB_NO_DENY=1 bash "${SCRIPT}" "${OUT}" > /dev/null || fail 'second build failed'
D="${OUT}/snapshots/$(jq -r .snapshot_id "${OUT}/current.json")"
[[ "$(row toolkit good cargo-deny)" == '["unavailable",null]' ]] || fail 'missing tool must be unavailable'
compgen -G "${TMPDIR:-/tmp}/codeos-static.[A-Za-z0-9]*/toolkit" >/dev/null && fail 'temporary exports were left behind'
bash "${SCRIPT}" "${TK}/out" 2>/dev/null && fail 'accepted an external root inside a repository'

printf 'static analytics tests: PASS\n'
