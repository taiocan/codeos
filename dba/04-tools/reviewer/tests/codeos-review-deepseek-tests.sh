#!/usr/bin/env bash
# Tests for the DeepSeek transport adapter. Every case runs against the local stub endpoint, so no
# network call is made and no API spend occurs.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
ADAPTER="${CODEOS_ROOT}/dba/04-tools/reviewer/codeos-review-deepseek.sh"
STUB="${CODEOS_ROOT}/dba/04-tools/implementer/tests/stub-deepseek-server.py"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-review-deepseek.XXXXXX")"
PORT="${CODEOS_STUB_PORT:-8942}"
STUB_PID=""
cleanup() {
  [[ -n "${STUB_PID}" ]] && { kill "${STUB_PID}" 2>/dev/null || true; }
  rm -rf "${WORK}"
}
trap cleanup EXIT
fail() { printf 'deepseek adapter tests failed: %s\n' "$1" >&2; exit 1; }

PACKET="${WORK}/packet.txt"
printf 'REVIEWER TASK\n\nARTIFACTS TO REVIEW\n  --- tracked.md ---\n' > "${PACKET}"
FIXTURE="${WORK}/reply.txt"
printf 'LOG SUMMARY: CHANGES ADVISED — one blocker\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: none\n' > "${FIXTURE}"

# Each stub gets a fresh port. Reusing one means racing the previous listener's teardown, which is
# a property of the test harness rather than anything the adapter does.
start_stub() {
  PORT=$((PORT + 1))
  if [[ -n "${STUB_PID}" ]]; then
    kill "${STUB_PID}" 2>/dev/null || true
    wait "${STUB_PID}" 2>/dev/null || true
    STUB_PID=""
  fi
  CODEOS_STUB_FIXTURE="${FIXTURE}" CODEOS_STUB_PORT="${PORT}" env "$@" python3 "${STUB}" 2>"${WORK}/stub.err" &
  STUB_PID=$!
  for _ in $(seq 1 50); do
    (exec 3<>"/dev/tcp/127.0.0.1/${PORT}") 2>/dev/null && { exec 3<&-; return 0; }
    sleep 0.1
  done
  fail "stub endpoint did not start on port ${PORT}: $(cat "${WORK}/stub.err")"
}

# `|| status=$?` rather than toggling `set -e`: a nested toggle would re-arm errexit for the caller
# in the middle of a case that deliberately expects a failing exit code.
run_adapter() {
  local status=0
  DEEPSEEK_API_KEY=stub-key \
  CODEOS_DEEPSEEK_URL="http://127.0.0.1:${PORT}/chat/completions" \
    "${ADAPTER}" "$@" > "${WORK}/stdout" 2> "${WORK}/stderr" || status=$?
  return ${status}
}

# --- preconditions, all before any network call -------------------------------------------------
set +e
"${ADAPTER}" "${PACKET}" > /dev/null 2>&1; [[ $? -eq 3 ]] || fail 'wrong argument count must be a usage error'
DEEPSEEK_API_KEY= "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 6 ]] || fail 'missing DEEPSEEK_API_KEY must refuse before the network call'
DEEPSEEK_API_KEY=k "${ADAPTER}" "${WORK}/absent.txt" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 7 ]] || fail 'a missing packet must exit 7'
DEEPSEEK_API_KEY=k CODEOS_DEEPSEEK_MAX_TOKENS=1234 "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 3 ]] || fail 'an unsupported max-tokens bound must fail pre-network'
set -e
[[ ! -e "${WORK}/out.txt" ]] || fail 'a refused run must not write an assessment'

# --- the packet is sent verbatim as the user message, and the reply is written out ---------------
start_stub
OUT="${WORK}/assessment.txt"
run_adapter "${PACKET}" "${OUT}" || fail "adapter failed: $(cat "${WORK}/stderr")"
diff -q "${OUT}" "${FIXTURE}" > /dev/null || fail 'the reply was not written back verbatim'
grep -q 'prompt_tokens=1234' "${OUT}.tokens.txt" || fail 'token accounting was not recorded'
grep -q 'finish_reason=stop' "${OUT}.tokens.txt" || fail 'finish reason was not recorded'
grep -q 'returned_model=deepseek-v4-flash' "${OUT}.tokens.txt" || fail 'returned model was not recorded'
grep -q 'never satisfies a required review round' "${WORK}/stdout" || fail 'adapter must state the advisory limit'
python3 -c "
import json,sys
body=json.load(open('${OUT}.request.json'))
packet=open('${PACKET}').read()
msgs=body['messages']
assert len(msgs)==1 and msgs[0]['role']=='user', msgs
assert msgs[0]['content']==packet, 'packet was not sent verbatim'
assert 'stub-key' not in json.dumps(body), 'the API key must never reach the request body'
" || fail 'request body did not carry the canonical packet'

# --- a truncated reply is accounted but never written as an assessment ---------------------------
start_stub CODEOS_STUB_FINISH_REASON=length
TRUNC="${WORK}/truncated.txt"
status=0
run_adapter "${PACKET}" "${TRUNC}" || status=$?
[[ ${status} -eq 8 ]] || fail 'a non-stop finish reason must exit 8'
[[ ! -e "${TRUNC}" ]] || fail 'a truncated reply must not be written to the assessment path'
grep -q 'finish_reason=length' "${TRUNC}.tokens.txt" || fail 'a truncated reply must still be accounted'
grep -q 'CODEOS_DEEPSEEK_MAX_TOKENS=65536' "${WORK}/stderr" || fail 'a length termination must name the one permitted retry'

# --- a transport failure writes no assessment ----------------------------------------------------
start_stub CODEOS_STUB_STATUS=500
status=0
run_adapter "${PACKET}" "${WORK}/failed.txt" || status=$?
[[ ${status} -eq 8 ]] || fail 'a non-2xx response must exit 8'
[[ ! -e "${WORK}/failed.txt" ]] || fail 'a failed call must not write an assessment'

printf 'deepseek adapter tests: PASS\n'
