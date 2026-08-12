#!/usr/bin/env bash
# Fixture-driven tests for scripts/codeos-implement.sh (UPG-0060 CHG-20260803-001).
#
# Covers the harness corrections and, as regression, every CHG-A property. No network access and no
# API spend: a local stub endpoint (stub-deepseek-server.py) stands in for DeepSeek and echoes the
# run's own nonce, so the delimited protocol is exercised end to end.
#
# Run:  bash scripts/tests/codeos-implement-tests.sh
# Exit: 0 all pass, 1 any failure.
set -uo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
CODEOS_ROOT="$(cd "${HERE}/../.." && pwd -P)"
TOOL="${CODEOS_ROOT}/scripts/codeos-implement.sh"
STUB="${HERE}/stub-deepseek-server.py"
PORT="${CODEOS_STUB_PORT:-8931}"

PASS=0; FAIL=0
ok()   { PASS=$((PASS+1)); printf '  ok   %s\n' "$1"; }
bad()  { FAIL=$((FAIL+1)); printf '  FAIL %s\n     %s\n' "$1" "${2:-}"; }

WORK="$(mktemp -d)"
STUB_PID=""
cleanup() { [[ -n "${STUB_PID}" ]] && kill "${STUB_PID}" 2>/dev/null; rm -rf "${WORK}"; }
trap cleanup EXIT

# A canary value for the leakage assertions. Deliberately not shaped like a credential so that no
# reader mistakes it for one — the assertion only needs a distinctive string to plant and search for.
#
# Note: this does NOT stop the Codeos reviewer redacting it. That redactor keys on the variable NAME
# (`DEEPSEEK_API_KEY=`), not on the value's shape, so any review packet containing this file reports
# SECRET_REDACTION coverage regardless of what the value is. That is accepted, not worked around: a
# suite that tests a tool reading DEEPSEEK_API_KEY must mention DEEPSEEK_API_KEY, and obfuscating the
# name to slip past a security scanner would be a worse habit than living with reduced coverage.
export DEEPSEEK_API_KEY="canary-value-for-leak-assertion-not-a-credential"

# ── a downstream-context repo (caller root != CODEOS_ROOT) ───────────────────────────────────────
REPO="${WORK}/proj"
mkdir -p "${REPO}/architecture" "${REPO}/intents"
git -C "${REPO}" init -q 2>/dev/null || { cd "${REPO}" && git init -q; }
printf 'approved intent\n' > "${REPO}/intents/F-0001.md"
printf 'status: enabled\n' > "${REPO}/architecture/delegated-implementation.yaml"
enable_mech()  { printf 'status: enabled\n'  > "${REPO}/architecture/delegated-implementation.yaml"; }
disable_mech() { printf 'status: disabled\n' > "${REPO}/architecture/delegated-implementation.yaml"; }

fixture() { printf '%s\n' "$1" > "${WORK}/fixture.txt"; export CODEOS_STUB_FIXTURE="${WORK}/fixture.txt"; }

start_stub() {
  [[ -n "${STUB_PID}" ]] && { kill "${STUB_PID}" 2>/dev/null; wait "${STUB_PID}" 2>/dev/null; }
  CODEOS_STUB_PORT="${PORT}" CODEOS_STUB_STATUS="${1:-200}" python3 "${STUB}" >/dev/null 2>&1 &
  STUB_PID=$!
  for _ in $(seq 1 50); do
    (exec 3<>/dev/tcp/127.0.0.1/"${PORT}") 2>/dev/null && { exec 3>&-; return 0; }
    sleep 0.1
  done
  echo "stub failed to start" >&2; return 1
}

# Run the tool inside the downstream repo. Echoes exit code; stdout+stderr land in $OUT.
OUT="${WORK}/out.txt"
run_tool() {
  ( cd "${REPO}" && CODEOS_DEEPSEEK_URL="http://127.0.0.1:${PORT}/chat/completions" \
      bash "${TOOL}" "$@" ) > "${OUT}" 2>&1
  echo $?
}

latest_stage_dir() {
  find "${REPO}/.codeos-state/deepseek-candidates" -mindepth 2 -maxdepth 2 -type d 2>/dev/null \
    | sort | tail -1
}
reset_state() { rm -rf "${REPO}/.codeos-state"; }

echo "== Group 1: the five corrections =="

# ── criterion 2: a build manifest survives end to end, byte-identical ────────────────────────────
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
pub fn hi() -> &'"'"'static str { "hi" }
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:FILE:modules/thing/Cargo.toml>>>
[package]
name = "thing"
version = "0.1.0"
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:SECTION:notes>>>
none
<<<CODEOS:{N}:ENDSECTION>>>'
start_stub
reset_state; enable_mech
rc=$(run_tool F-0001 4 intents/F-0001.md)
SD="$(latest_stage_dir)"
if [[ "${rc}" == "0" && -f "${SD}/candidate/modules/thing/Cargo.toml" ]]; then
  got="$(cat "${SD}/candidate/modules/thing/Cargo.toml")"
  want='[package]
name = "thing"
version = "0.1.0"'
  [[ "${got}" == "${want}" ]] && ok "C2 manifest staged byte-identical" \
                              || bad "C2 manifest content differs" "got: ${got}"
else
  bad "C2 manifest not staged" "rc=${rc} $(tail -3 "${OUT}")"
fi

# ── criterion 6: source emitted verbatim, no JSON escape artifacts ───────────────────────────────
if [[ -f "${SD}/candidate/modules/thing/src/lib.rs" ]]; then
  if grep -q '\\n' "${SD}/candidate/modules/thing/src/lib.rs"; then
    bad "C6 source verbatim" "found literal \\n escape artifact"
  else
    ok "C6 source emitted verbatim (no JSON escape artifacts)"
  fi
else
  bad "C6 source verbatim" "source file missing"
fi

# ── criterion 16: audit set + token instrumentation survive the protocol change ──────────────────
missing=""
for f in packet.txt request.json response.json model_content.txt tokens.txt \
         contract_satisfaction.txt event_emission.txt notes.txt; do
  [[ -f "${SD}/${f}" ]] || missing="${missing} ${f}"
done
grep -q 'prompt_tokens=1234 completion_tokens=567 total_tokens=1801' "${SD}/tokens.txt" 2>/dev/null \
  && toks=1 || toks=0
[[ -z "${missing}" && "${toks}" == 1 ]] && ok "C16 audit set complete + tokens recorded" \
                                        || bad "C16 audit set" "missing:${missing} tokens_ok=${toks}"

# ── criterion 13: secret non-leakage — with a positive control on the checker itself ─────────────
# The checker is asserted to be capable of detecting a planted key BEFORE it is trusted to report
# absence. grep's own exit status is tested directly; no pipeline whose status could be misread.
ctl="${WORK}/positive-control.txt"
printf 'prefix %s suffix\n' "${DEEPSEEK_API_KEY}" > "${ctl}"
if grep -rqF -- "${DEEPSEEK_API_KEY}" "${ctl}"; then
  if grep -rqF -- "${DEEPSEEK_API_KEY}" "${SD}"; then
    bad "C13 secret non-leakage" "key found under ${SD}"
  else
    ok "C13 secret absent from staging tree (checker passed positive control)"
  fi
else
  bad "C13 secret checker is broken" "positive control did not detect a planted key"
fi
if grep -rqF -- "${DEEPSEEK_API_KEY}" "${REPO}/.codeos-state/deepseek-candidates/implement-log.md"; then
  bad "C13 secret in log" "key found in implement-log.md"
else
  ok "C13 secret absent from invocation log"
fi

# ── criterion 3 + 8: exemplar and repair sections labeled distinctly in the packet ───────────────
printf 'exemplar module layout\n' > "${REPO}/exemplar.rs"
printf 'error[E0599]: no method named foo\n' > "${REPO}/build-output.txt"
reset_state
rc=$(run_tool --exemplar exemplar.rs --repair-candidate exemplar.rs --repair-output build-output.txt \
      F-0001 4 intents/F-0001.md)
SD2="$(latest_stage_dir)"
if [[ "${rc}" == "0" ]]; then
  p="${SD2}/packet.txt"
  grep -q 'LAYOUT EXEMPLAR (context only' "${p}" && e1=1 || e1=0
  grep -q 'APPROVED ARTIFACT (ROLE UNSPECIFIED): intents/F-0001.md' "${p}" && e2=1 || e2=0
  grep -q 'REPAIR REQUEST — this is a retry' "${p}" && e3=1 || e3=0
  grep -q 'FEEDBACK (build/test output' "${p}" && e4=1 || e4=0
  [[ "${e1}${e2}" == "11" ]] && ok "C3 exemplar labeled distinctly from approved artifacts" \
                             || bad "C3 exemplar labeling" "exemplar=${e1} artifact=${e2}"
  [[ "${e3}${e4}" == "11" ]] && ok "C8 repair input rendered as feedback" \
                             || bad "C8 repair labeling" "repair=${e3} feedback=${e4}"
else
  bad "C3/C8 exemplar+repair run" "rc=${rc} $(tail -3 "${OUT}")"
fi

# ── criterion 9: Option B — no build/test process, and the documented tool set is complete ───────
# Two assertions. The first is the safety property. The second is an allowlist scan: the Step 3 R1
# review caught the change record claiming a process list that omitted awk and sed, so that class of
# documentation drift is now a test failure rather than something a reviewer has to notice.
if grep -nE '^[^#]*\b(cargo|npm|pnpm|yarn|make|mvn|gradle|pytest|rustc|gcc|eval)\b' "${TOOL}" \
     | grep -vE "printf|echo|#" | grep -q .; then
  bad "C9 no local execution" "a build/test invocation appears in the script"
else
  ok "C9 script invokes no build/test/package-manager command"
fi

# Strip comments and the embedded awk program, then look for any external tool outside the set the
# change record documents. Conservative by design: it scans a broad candidate list, so a newly
# introduced tool trips it even if harmless, forcing the documentation to be updated with it.
ALLOWED=" git curl jq awk sed cat tr od head date mkdir mktemp rmdir dirname "
CANDIDATES="cargo npm pnpm yarn make mvn gradle pytest pip python python3 node rustc gcc cc perl ruby
docker ssh scp rsync wget nc socat systemctl sudo su chmod chown find xargs tee sort uniq cut paste
grep egrep fgrep awk sed cat tr od head tail date mkdir mktemp rmdir dirname basename git curl jq"
body="$(sed 's/#.*//' "${TOOL}" | sed "/^[[:space:]]*awk /,/^' /d")"
unexpected=""
for c in ${CANDIDATES}; do
  case "${ALLOWED}" in *" ${c} "*) continue;; esac
  if printf '%s\n' "${body}" | grep -qE "(^|[;|&(\$]|\\\$\()[[:space:]]*${c}[[:space:]]"; then
    unexpected="${unexpected} ${c}"
  fi
done
[[ -z "${unexpected}" ]] \
  && ok "C9 no external tool outside the documented allowlist" \
  || bad "C9 undocumented external tool" "found:${unexpected} — update the change record's process list"

# ── regression: a packet larger than MAX_ARG_STRLEN (128 KiB) must still build ──────────────────
# Latent since CHG-A and only triggered once packets grew past 128 KB: `jq --arg usr "$USR"` passed
# the whole packet as one argv element, and Linux caps a single argument at 128 KiB regardless of
# ARG_MAX. Found by running the tool on a realistic downstream feature, not by review.
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
ok
<<<CODEOS:{N}:ENDFILE>>>'
reset_state; enable_mech
python3 -c "print('# filler line to grow the packet past the 128 KiB single-argument limit\n' * 4000)" \
  > "${REPO}/intents/BIG.md"
big=$(wc -c < "${REPO}/intents/BIG.md")
rc=$(run_tool F-0001 4 intents/F-0001.md intents/BIG.md)
if [[ "${rc}" == "0" && -f "$(latest_stage_dir)/candidate/modules/thing/src/lib.rs" ]]; then
  ok "REG packet of ${big} bytes (>128 KiB) builds and runs"
else
  bad "REG oversized packet" "rc=${rc} packet=${big}B $(tail -2 "${OUT}")"
fi

# ── UPG-0064: caller-declared artifact roles ────────────────────────────────────────────────────
echo "== UPG-0064: artifact authority roles =="
mkdir -p "${REPO}/architecture/scopes" "${REPO}/contracts" "${REPO}/events"
printf 'the contract\n'  > "${REPO}/contracts/F-0001_contract.md"
printf 'the schema\n'    > "${REPO}/events/F-0001_schema.md"
printf 'the architecture\n' > "${REPO}/architecture/scopes/source-intelligence.md"
printf 'the profile\n'   > "${REPO}/architecture/implementation-profile.yaml"

fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
x
<<<CODEOS:{N}:ENDFILE>>>'
reset_state; enable_mech
rc=$(run_tool --contract contracts/F-0001_contract.md --event-schema events/F-0001_schema.md --architecture architecture/scopes/source-intelligence.md --profile architecture/implementation-profile.yaml F-0001 4 intents/F-0001.md)
P="$(latest_stage_dir)/packet.txt"
if [[ "${rc}" == "0" ]]; then
  miss=""
  for lbl in "BEHAVIORAL CONTRACT: contracts/F-0001_contract.md" "EVENT SCHEMA: events/F-0001_schema.md" "PROJECT ARCHITECTURE: architecture/scopes/source-intelligence.md" "IMPLEMENTATION PROFILE: architecture/implementation-profile.yaml"; do
    grep -qF -- "${lbl}" "${P}" || miss="${miss} [${lbl}]"
  done
  if [[ -z "${miss}" ]]; then ok "ROLE each declared artifact is labelled with its authority"
  else bad "ROLE labelling" "missing:${miss}"; fi
  if grep -q 'APPROVED ARTIFACT (ROLE UNSPECIFIED): intents/F-0001.md' "${P}"; then
    ok "ROLE positional stays ROLE UNSPECIFIED alongside declared roles"
  else bad "ROLE positional" "positional not labelled unspecified"; fi
else
  bad "ROLE labelling run" "rc=${rc}"
fi

reset_state
rc=$(run_tool F-0001 4 architecture/scopes/source-intelligence.md)
P2="$(latest_stage_dir)/packet.txt"
inferred=no
grep -q 'PROJECT ARCHITECTURE: architecture/scopes/source-intelligence.md' "${P2}" && inferred=yes
if grep -q 'APPROVED ARTIFACT (ROLE UNSPECIFIED): architecture/scopes/source-intelligence.md' "${P2}" && [[ "${inferred}" == "no" ]]; then
  ok "ROLE no authority inferred from a conventional path"
else
  bad "ROLE inference" "a positional baseline-looking path acquired a role (inferred=${inferred})"
fi

reset_state
rc=$(run_tool --contract contracts/F-0001_contract.md --architecture contracts/F-0001_contract.md F-0001 4 intents/F-0001.md)
staged=$(find "$(latest_stage_dir)/candidate" -type f 2>/dev/null | wc -l)
if [[ "${rc}" == "12" && "${staged}" == "0" ]]; then
  ok "ROLE conflicting roles on one path -> exit 12, nothing staged"
else bad "ROLE conflict" "rc=${rc} staged=${staged}"; fi

reset_state
rc=$(run_tool --contract contracts/F-0001_contract.md --contract contracts/F-0001_contract.md F-0001 4 intents/F-0001.md)
if [[ "${rc}" == "0" ]]; then ok "ROLE same path twice under one role is not a conflict"
else bad "ROLE duplicate same-role" "rc=${rc}"; fi

reset_state
rc=$(run_tool --architecture architecture/scopes/source-intelligence.md F-0001 4 intents/F-0001.md)
D="$(latest_stage_dir)"
lbl='--- PROJECT ARCHITECTURE: architecture/scopes/source-intelligence.md ---'
inreq=no
jq -r '.messages[1].content' "${D}/request.json" | grep -qF -- "${lbl}" && inreq=yes
if grep -qF -- "${lbl}" "${D}/packet.txt" && [[ "${inreq}" == "yes" ]]; then
  ok "ROLE label byte-identical in packet.txt and the request actually sent"
else bad "ROLE label transport" "packet vs request differ"; fi
# AC-9: the artifact-content region (after the heading and its generated binding note, up to the
# next heading) must be byte-identical to the source file. Stronger than "the bytes appear somewhere".
jq -r '.messages[1].content' "${D}/request.json" > "${WORK}/sent.txt"
awk '/^--- PROJECT ARCHITECTURE: architecture\/scopes\/source-intelligence\.md ---$/{f=1;skip=1;next} f&&skip{skip=0;next} f&&/^--- /{f=0} f{print}' "${WORK}/sent.txt" > "${WORK}/raw.txt"
# The packet separates blocks with a leading newline before each heading, so the extracted region
# carries exactly one trailing blank line belonging to that separator, not to the artifact. Drop
# precisely one, then require byte equality with the file.
awk 'NR>1{print prev} {prev=$0} END{if (prev != "") print prev}' "${WORK}/raw.txt" > "${WORK}/extracted.txt"
if cmp -s "${WORK}/extracted.txt" "${REPO}/architecture/scopes/source-intelligence.md"; then
  ok "ROLE content region byte-identical to source file"
else
  bad "ROLE content mutated" "extracted region differs from the source file"
fi

reset_state
rc=$(run_tool --architecture architecture/nope.md F-0001 4 intents/F-0001.md)
if [[ "${rc}" == "7" ]]; then ok "ROLE missing role artifact -> exit 7"
else bad "ROLE missing" "rc=${rc}"; fi

# ── UPG-0064: deferral_resolution is optional; absence must be frictionless ──────────────────────
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
x
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:SECTION:deferral_resolution>>>
schema validation ordering | first-failure-wins | lib.rs:record | FINAL | -
<<<CODEOS:{N}:ENDSECTION>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
D="$(latest_stage_dir)"
if [[ "${rc}" == "0" ]] && grep -q "first-failure-wins" "${D}/deferral_resolution.txt" 2>/dev/null; then
  ok "DEFERRAL section parses and is staged when present"
else bad "DEFERRAL present" "rc=${rc}"; fi

fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
x
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:SECTION:notes>>>
nothing was deferred
<<<CODEOS:{N}:ENDSECTION>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
D="$(latest_stage_dir)"
sidecar=no
[[ -f "${D}/deferral_resolution.txt" ]] && sidecar=yes
if [[ "${rc}" == "0" && "${sidecar}" == "no" ]]; then
  ok "DEFERRAL absent is a clean success; no empty sidecar fabricated"
else bad "DEFERRAL absent" "rc=${rc} sidecar=${sidecar}"; fi

PR="${CODEOS_ROOT}/prompts/codeos-implementer-task.md"
if grep -q "Omitting this section is the expected outcome" "${PR}" && grep -q "Do not invent a deferral" "${PR}"; then
  ok "DEFERRAL prompt states absence is expected and forbids fabrication"
else bad "DEFERRAL no-pressure wording" "prompt lacks the anti-fabrication instruction"; fi

reset_state
rc=$(run_tool --contract contracts/F-0001_contract.md --architecture architecture/scopes/source-intelligence.md F-0001 4)
if [[ "${rc}" == "0" ]]; then ok "ROLE role-flags-only call needs no positional artifact"
else bad "ROLE role-only call" "rc=${rc} — CHG-B could not avoid the compatibility path"; fi

reset_state
rc=$(run_tool F-0001 4)
if [[ "${rc}" == "3" ]]; then ok "ROLE zero artifacts by any route -> exit 3"
else bad "ROLE zero artifacts" "rc=${rc}"; fi

echo "== Group 1: protocol robustness (criterion 7) =="

# 7a: a marker line bearing the WRONG nonce is ordinary content, written verbatim.
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
// the next line looks like a marker but carries a different nonce
<<<CODEOS:deadbeefdeadbeef:ENDFILE>>>
pub fn after() {}
<<<CODEOS:{N}:ENDFILE>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
SD3="$(latest_stage_dir)"
f="${SD3}/candidate/modules/thing/src/lib.rs"
if [[ "${rc}" == "0" && -f "${f}" ]] && grep -q 'deadbeefdeadbeef' "${f}" && grep -q 'pub fn after' "${f}"; then
  ok "C7a wrong-nonce marker treated as content, file not truncated"
else
  bad "C7a wrong-nonce content" "rc=${rc}; $(tail -2 "${OUT}")"
fi

# 7b: a real marker nested inside an open block fails closed, nothing staged.
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/a.rs>>>
content
<<<CODEOS:{N}:FILE:modules/thing/src/b.rs>>>
more
<<<CODEOS:{N}:ENDFILE>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
SD4="$(latest_stage_dir)"
staged=$(find "${SD4}/candidate" -type f 2>/dev/null | wc -l)
[[ "${rc}" == "11" && "${staged}" == "0" ]] \
  && ok "C7b nested marker -> exit 11, nothing staged" \
  || bad "C7b nested marker" "rc=${rc} staged=${staged}"

# 7c: unterminated block fails closed.
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/a.rs>>>
never closed'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
staged=$(find "$(latest_stage_dir)/candidate" -type f 2>/dev/null | wc -l)
[[ "${rc}" == "11" && "${staged}" == "0" ]] \
  && ok "C7c unterminated block -> exit 11, nothing staged" \
  || bad "C7c unterminated block" "rc=${rc} staged=${staged}"

# 7d: duplicate path fails closed.
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/a.rs>>>
one
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:FILE:modules/thing/src/a.rs>>>
two
<<<CODEOS:{N}:ENDFILE>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
staged=$(find "$(latest_stage_dir)/candidate" -type f 2>/dev/null | wc -l)
[[ "${rc}" == "11" && "${staged}" == "0" ]] \
  && ok "C7d duplicate path -> exit 11, nothing staged" \
  || bad "C7d duplicate path" "rc=${rc} staged=${staged}"

# 7e: no file blocks at all fails closed.
fixture 'I could not do this task, sorry.'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
[[ "${rc}" == "11" ]] && ok "C7e no file blocks -> exit 11" || bad "C7e no file blocks" "rc=${rc}"

echo "== Group 2: preserved CHG-A properties =="

# ── criterion 11: path safety — location still constrained, all-or-nothing ───────────────────────
for spec in "/etc/passwd:absolute" "modules/../../escape.rs:traversal" "doctrine/dba-system.md:outside-stage-area"; do
  path="${spec%%:*}"; label="${spec##*:}"
  fixture "<<<CODEOS:{N}:FILE:modules/thing/ok.rs>>>
fine
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:FILE:${path}>>>
bad
<<<CODEOS:{N}:ENDFILE>>>"
  reset_state
  rc=$(run_tool F-0001 4 intents/F-0001.md)
  staged=$(find "$(latest_stage_dir)/candidate" -type f 2>/dev/null | wc -l)
  [[ "${rc}" == "8" && "${staged}" == "0" ]] \
    && ok "C11 ${label} path rejected (exit 8), nothing staged" \
    || bad "C11 ${label}" "rc=${rc} staged=${staged}"
done

# ── criterion 12: CANDIDATE_BLOCKED.md escape hatch still works ──────────────────────────────────
fixture '<<<CODEOS:{N}:FILE:CANDIDATE_BLOCKED.md>>>
The approved contract does not specify the failure taxonomy.
<<<CODEOS:{N}:ENDFILE>>>
<<<CODEOS:{N}:SECTION:notes>>>
blocked
<<<CODEOS:{N}:ENDSECTION>>>'
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
[[ "${rc}" == "0" && -f "$(latest_stage_dir)/candidate/CANDIDATE_BLOCKED.md" ]] \
  && ok "C12 CANDIDATE_BLOCKED.md escape hatch works" \
  || bad "C12 escape hatch" "rc=${rc}"

# ── criterion 15: idempotency — re-runs never overwrite a prior run's audit ──────────────────────
fixture '<<<CODEOS:{N}:FILE:modules/thing/src/lib.rs>>>
x
<<<CODEOS:{N}:ENDFILE>>>'
reset_state
r1=$(run_tool F-0001 4 intents/F-0001.md); r2=$(run_tool F-0001 4 intents/F-0001.md)
dirs=$(find "${REPO}/.codeos-state/deepseek-candidates" -mindepth 2 -maxdepth 2 -type d | wc -l)
[[ "${r1}" == "0" && "${r2}" == "0" && "${dirs}" == "2" ]] \
  && ok "C15 two runs -> two distinct staging dirs" \
  || bad "C15 idempotency" "r1=${r1} r2=${r2} dirs=${dirs}"

# ── criterion 14: write-safety — nothing outside .codeos-state, no commit ────────────────────────
tracked_dirty=$(cd "${REPO}" && git status --porcelain -- modules tests 2>/dev/null | wc -l)
commits=$(cd "${REPO}" && git rev-list --all --count 2>/dev/null || echo 0)
if grep -qE '^\s*git (add|commit)' "${TOOL}"; then
  bad "C14 write-safety" "script contains a git add/commit"
elif [[ "${tracked_dirty}" == "0" && "${commits}" == "0" ]]; then
  ok "C14 nothing written under modules/ or tests/, no commit"
else
  bad "C14 write-safety" "dirty=${tracked_dirty} commits=${commits}"
fi

# ── criterion 10: CHG-A fail-closed cases, unchanged codes ───────────────────────────────────────
reset_state
rc=$( (cd "${WORK}" && bash "${TOOL}" F-0001 4 x.md) >/dev/null 2>&1; echo $?)
[[ "${rc}" == "1" ]] && ok "C10 non-git dir -> exit 1" || bad "C10 non-git" "rc=${rc}"

rc=$(run_tool F-0001); [[ "${rc}" == "3" ]] && ok "C10 missing args -> exit 3" || bad "C10 missing args" "rc=${rc}"
rc=$(run_tool F-0001 6 intents/F-0001.md); [[ "${rc}" == "3" ]] && ok "C10 stage=6 -> exit 3" || bad "C10 stage 6" "rc=${rc}"

disable_mech
rc=$(run_tool F-0001 4 intents/F-0001.md); [[ "${rc}" == "4" ]] && ok "C10 status:disabled -> exit 4" || bad "C10 disabled" "rc=${rc}"
rm -f "${REPO}/architecture/delegated-implementation.yaml"
rc=$(run_tool F-0001 4 intents/F-0001.md); [[ "${rc}" == "4" ]] && ok "C10 status file absent -> exit 4" || bad "C10 absent" "rc=${rc}"
printf 'status: sometimes\n' > "${REPO}/architecture/delegated-implementation.yaml"
rc=$(run_tool F-0001 4 intents/F-0001.md); [[ "${rc}" == "5" ]] && ok "C10 malformed status -> exit 5" || bad "C10 malformed" "rc=${rc}"

enable_mech
rc=$( (cd "${REPO}" && DEEPSEEK_API_KEY= CODEOS_DEEPSEEK_URL="http://127.0.0.1:${PORT}/x" \
        bash "${TOOL}" F-0001 4 intents/F-0001.md) >/dev/null 2>&1; echo $?)
[[ "${rc}" == "6" ]] && ok "C10 unset key -> exit 6 (pre-network)" || bad "C10 unset key" "rc=${rc}"
rc=$(run_tool F-0001 4 intents/nope.md); [[ "${rc}" == "7" ]] && ok "C10 missing artifact -> exit 7" || bad "C10 missing artifact" "rc=${rc}"

echo "== Group 2: new exit codes (criterion 18) =="
rc=$(run_tool --exemplar nope.rs F-0001 4 intents/F-0001.md)
[[ "${rc}" == "9" ]] && ok "C18 missing exemplar -> exit 9" || bad "C18 exemplar" "rc=${rc}"
rc=$(run_tool --repair-output nope.txt F-0001 4 intents/F-0001.md)
[[ "${rc}" == "10" ]] && ok "C18 missing repair input -> exit 10" || bad "C18 repair input" "rc=${rc}"
rc=$(run_tool --bogus F-0001 4 intents/F-0001.md)
[[ "${rc}" == "3" ]] && ok "C18 unknown option -> exit 3" || bad "C18 unknown option" "rc=${rc}"

# API/transport error still exit 8.
start_stub 500
reset_state
rc=$(run_tool F-0001 4 intents/F-0001.md)
[[ "${rc}" == "8" ]] && ok "C10 HTTP 500 -> exit 8" || bad "C10 transport error" "rc=${rc}"

# ── criterion 17: mechanism still off by default in the Codeos repo itself ───────────────────────
[[ "$(cat "${CODEOS_ROOT}/config/delegated-implementation.yaml")" == "status: disabled" ]] \
  && ok "C17 self-dev activation file still status: disabled" \
  || bad "C17 off by default" "config changed"

printf '\n%d passed, %d failed\n' "${PASS}" "${FAIL}"
[[ "${FAIL}" -eq 0 ]]
