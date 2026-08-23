#!/usr/bin/env bash
# Tests for the gemini branch of the transport adapter's provider switch (UPG-0072). Every case runs
# against the local stub endpoint, so no network call is made and no API spend occurs.
#
# The deepseek branch is covered by codeos-review-deepseek-tests.sh, which runs with no provider
# variable set and is the guard that the default path did not move. This file covers only what the
# gemini branch does differently, and every difference here was measured against the real endpoint
# before it was encoded:
#   * thinking:{type:"enabled"} must NOT be sent — the real API rejects it with HTTP 400.
#   * usage carries prompt/completion/total only, so the residual is derived and recorded as
#     unclassified, never as reasoning; completion_tokens is the final-content figure directly.
#   * the key comes from GEMINI_API_KEY, and its absence still refuses before the network call.
set -euo pipefail

CODEOS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../../.." && pwd -P)"
ADAPTER="${CODEOS_ROOT}/dba/04-tools/reviewer/codeos-review-deepseek.sh"
STUB="${CODEOS_ROOT}/dba/04-tools/implementer/tests/stub-deepseek-server.py"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/codeos-review-gemini.XXXXXX")"
PORT="${CODEOS_STUB_PORT:-8972}"
STUB_PID=""
cleanup() {
  [[ -n "${STUB_PID}" ]] && { kill "${STUB_PID}" 2>/dev/null || true; }
  rm -rf "${WORK}"
}
trap cleanup EXIT
fail() { printf 'gemini adapter tests failed: %s\n' "$1" >&2; exit 1; }

PACKET="${WORK}/packet.txt"
printf 'REVIEWER TASK\n\nARTIFACTS TO REVIEW\n  --- tracked.md ---\n' > "${PACKET}"
FIXTURE="${WORK}/reply.txt"
printf 'LOG SUMMARY: CHANGES ADVISED — one blocker\nEVIDENCE: B\nHIGHEST-IMPACT UNCERTAINTY: none\n' > "${FIXTURE}"
REQ_DUMP="${WORK}/sent-request.json"

start_stub() {
  PORT=$((PORT + 1))
  if [[ -n "${STUB_PID}" ]]; then
    kill "${STUB_PID}" 2>/dev/null || true
    wait "${STUB_PID}" 2>/dev/null || true
    STUB_PID=""
  fi
  CODEOS_STUB_FIXTURE="${FIXTURE}" CODEOS_STUB_PORT="${PORT}" CODEOS_STUB_SHAPE=gemini \
  CODEOS_STUB_REQUEST_DUMP="${REQ_DUMP}" \
    env "$@" python3 "${STUB}" 2>"${WORK}/stub.err" &
  STUB_PID=$!
  for _ in $(seq 1 50); do
    (exec 3<>"/dev/tcp/127.0.0.1/${PORT}") 2>/dev/null && { exec 3<&-; return 0; }
    sleep 0.1
  done
  fail "stub endpoint did not start on port ${PORT}: $(cat "${WORK}/stub.err")"
}

run_adapter() {
  local status=0
  CODEOS_LLM_PROVIDER=gemini \
  GEMINI_API_KEY=stub-key \
  CODEOS_GEMINI_URL="http://127.0.0.1:${PORT}/chat/completions" \
    "${ADAPTER}" "$@" > "${WORK}/stdout" 2> "${WORK}/stderr" || status=$?
  return ${status}
}

# --- preconditions, all before any network call -------------------------------------------------
set +e
CODEOS_LLM_PROVIDER=gemini GEMINI_API_KEY= "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 6 ]] || fail 'missing GEMINI_API_KEY must refuse before the network call'
# DEEPSEEK_API_KEY must not satisfy the gemini branch: a provider takes its own credential.
CODEOS_LLM_PROVIDER=gemini GEMINI_API_KEY= DEEPSEEK_API_KEY=k "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 6 ]] || fail 'the deepseek key must not satisfy the gemini branch'
CODEOS_LLM_PROVIDER=gemini GEMINI_API_KEY=k CODEOS_GEMINI_MAX_TOKENS=1234 "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 3 ]] || fail 'an unsupported max-tokens bound must be rejected before the network call'
CODEOS_LLM_PROVIDER=anthropic GEMINI_API_KEY=k "${ADAPTER}" "${PACKET}" "${WORK}/out.txt" > /dev/null 2>&1
[[ $? -eq 3 ]] || fail 'an unsupported CODEOS_LLM_PROVIDER must be a usage error'
set -e

# --- the request the adapter actually sends -----------------------------------------------------
start_stub
OUT="${WORK}/assessment.txt"
run_adapter "${PACKET}" "${OUT}" || fail "adapter failed: $(cat "${WORK}/stderr")"

jq -e 'has("thinking") | not' "${REQ_DUMP}" > /dev/null \
  || fail 'the gemini request must not carry a thinking field (the real API rejects it with 400)'
jq -e '.model == "gemini-3.7-flash"
       and .reasoning_effort == "high"
       and .max_tokens == 32768
       and (.temperature == null)
       and (.messages | length == 1)
       and (.messages[0].role == "user")' "${REQ_DUMP}" > /dev/null \
  || fail "unexpected gemini request shape: $(jq -c '{model,reasoning_effort,max_tokens}' "${REQ_DUMP}")"

# --- accounting: reported fields as reported, residual named unclassified -----------------------
# Stub usage is prompt=1234 completion=567 total=2101, so the residual is 2101-1234-567 = 300.
TOK="$(cat "${OUT}.tokens.txt")"
grep -q 'provider=gemini' <<< "${TOK}" || fail 'the accounting line must name the provider'
grep -q 'prompt_tokens=1234 completion_tokens=567 total_tokens=2101' <<< "${TOK}" \
  || fail "reported usage fields must be recorded as returned: ${TOK}"
grep -q 'unclassified_tokens_derived=300' <<< "${TOK}" \
  || fail "the residual must be derived as total-prompt-completion: ${TOK}"
grep -q 'final_content_tokens=567' <<< "${TOK}" \
  || fail "final content is completion_tokens directly for this provider: ${TOK}"
if grep -q 'reasoning_tokens' <<< "${TOK}"; then
  fail 'the residual must never be reported as reasoning tokens'
fi
grep -q 'returned_model=gemini-3.7-flash' <<< "${TOK}" || fail 'returned model was not recorded'

diff -q "${OUT}" "${FIXTURE}" > /dev/null || fail 'the reply must be written back byte for byte'

# --- a length termination names the gemini retry variable, and stages nothing --------------------
start_stub CODEOS_STUB_FINISH_REASON=length
set +e
run_adapter "${PACKET}" "${WORK}/out2.txt"
rc=$?
set -e
[[ ${rc} -eq 8 ]] || fail "a non-stop finish must fail with exit 8; got ${rc}"
[[ ! -e "${WORK}/out2.txt" ]] || fail 'a truncated reply must never reach the assessment path'
grep -q 'CODEOS_GEMINI_MAX_TOKENS=65536' "${WORK}/stderr" \
  || fail 'a length termination must name the one permitted retry for this provider'
# The truncated attempt is still costed, and still under the corrected accounting names.
grep -q 'unclassified_tokens_derived=300' "${WORK}/out2.txt.tokens.txt" \
  || fail 'a truncated attempt must still be accounted'

# --- transport failure --------------------------------------------------------------------------
start_stub CODEOS_STUB_STATUS=500
set +e
run_adapter "${PACKET}" "${WORK}/out3.txt"
rc=$?
set -e
[[ ${rc} -eq 8 ]] || fail "an API error must fail with exit 8; got ${rc}"

# --- the key never reaches disk -----------------------------------------------------------------
if grep -rqF -- 'stub-key' "${WORK}"; then
  fail 'the API key must not appear in any written file'
fi

printf 'gemini adapter tests: PASS\n'
