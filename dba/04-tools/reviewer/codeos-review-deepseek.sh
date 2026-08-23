#!/usr/bin/env bash
# codeos-review-deepseek.sh — transport only. Sends a canonical reviewer packet to DeepSeek and
# writes the raw reply back out. It parses nothing, decides nothing, and writes no reviewer record.
#
# This is the model-transport half of the external-assessment path:
#
#   codeos-review.sh plan <feature> <stage> --emit-packet PACKET <artifacts...>
#   codeos-review-deepseek.sh PACKET ASSESSMENT
#   codeos-review.sh review <feature> <stage> --assessment ASSESSMENT --packet PACKET \
#       --reviewer-label deepseek-v4-flash <artifacts...>
#
# --packet is required: the record is bound to the exact packet bytes the model read, which is why
# PACKET must be the same file exported above and must keep its .meta.json sidecar beside it.
#
# The packet already carries the reviewer task prompt, so it is sent as the user message and no
# system message is added: the packet is what a Codex-backed review would have received, verbatim.
#
# The result is an EXTERNAL ASSESSMENT. It supplies advisory findings and never satisfies a required
# review round — see dba/02-policies/review/v2.md. DeepSeek-specific knowledge stops in this file;
# the reviewer engine holds no credentials and no provider configuration.
#
# Usage:
#   codeos-review-deepseek.sh <packet-file> <assessment-file>
#
# Environment:
#   CODEOS_LLM_PROVIDER  optional, default "deepseek". The only other value is "gemini", added for
#                        the UPG-0072 qualification experiment. The gemini branch is experiment
#                        support with a disposition rule in that brief — it is removed if every role
#                        fails, and retaining it on a pass is a separate decision. It is NOT a
#                        supported extension point, and nothing downstream references it.
#
#   provider=deepseek (default — request bytes and accounting line unchanged since UPG-0071):
#     DEEPSEEK_API_KEY  required. Read only into the HTTP Authorization header via a curl config on
#                       stdin — never in argv, the request body, or any written file.
#     CODEOS_DEEPSEEK_MODEL       optional, default "deepseek-v4-flash".
#     CODEOS_DEEPSEEK_URL         optional, default "https://api.deepseek.com/chat/completions".
#     CODEOS_DEEPSEEK_MAX_TOKENS  optional, default 32768. The only other supported value is 65536,
#                               for one explicit retry after finish_reason=length. A retry is a new
#                               invocation so both attempts remain visible and count toward cost.
#     CODEOS_DEEPSEEK_REASONING_EFFORT
#                               optional, default "high". Sent verbatim as the request's
#                               reasoning_effort. The default is what every recorded assessment so
#                               far was produced with; set it only to run a deliberately different
#                               configuration, and record which value was used with the result.
#
#   provider=gemini:
#     GEMINI_API_KEY  required, handled identically to DEEPSEEK_API_KEY above.
#     CODEOS_GEMINI_MODEL       optional, default "gemini-3.7-flash".
#     CODEOS_GEMINI_URL         optional, default the OpenAI-compatible endpoint at
#                               generativelanguage.googleapis.com.
#     CODEOS_GEMINI_MAX_TOKENS  optional, default 32768; same 65536 single-retry rule. Note this
#                               bound INCLUDES the model's thinking, so a reply can terminate with
#                               finish_reason=length having produced almost no content.
#     CODEOS_GEMINI_REASONING_EFFORT
#                               optional, default "high" — the maximum this API accepts. "max" is
#                               rejected with HTTP 400.
#
#   Two provider differences are not reachable by configuration, which is why this switch exists at
#   all rather than a set of env vars:
#     * thinking:{type:"enabled"} is sent only to deepseek; gemini rejects the field with HTTP 400.
#     * gemini's usage carries prompt/completion/total only — no reasoning field, no cache fields.
#       Its accounting line reports those three as returned and records total-prompt-completion as
#       unclassified_tokens_derived, never as reasoning: nothing authoritative says what the
#       residual contains. Its completion_tokens EXCLUDES that residual and so is the final-content
#       figure directly, the opposite of deepseek, where completion includes reported reasoning.
#
# Exit codes:
#   0  success — assessment written
#   2  missing dependency (curl or jq not found on PATH)
#   3  usage error, an unsupported max-tokens bound, or an unsupported CODEOS_LLM_PROVIDER
#      (all checked before any network call)
#   6  the provider's API key is unset or empty — refuse before any network call
#   7  the packet file does not exist or is empty
#   8  provider API / transport error, or a response that did not terminate normally
set -euo pipefail

err() { echo "error: $*" >&2; }

if [[ $# -ne 2 ]]; then
  err "usage: codeos-review-deepseek.sh <packet-file> <assessment-file>"
  exit 3
fi
PACKET="$1"
OUT="$2"

for dep in curl jq; do
  command -v "${dep}" >/dev/null 2>&1 || { err "required dependency '${dep}' not found on PATH"; exit 2; }
done

[[ -s "${PACKET}" ]] || { err "packet file is missing or empty: ${PACKET}"; exit 7; }

# Provider resolution. Default deepseek, so the DeepSeek request bytes and accounting line are
# unchanged and the existing suites are themselves the guard that this path did not move. The gemini
# branch is UPG-0072 experiment support with a disposition rule, not a supported extension point.
PROVIDER="${CODEOS_LLM_PROVIDER:-deepseek}"
case "${PROVIDER}" in
  deepseek)
    KEY_VAR=DEEPSEEK_API_KEY
    MODEL="${CODEOS_DEEPSEEK_MODEL:-deepseek-v4-flash}"
    REASONING_EFFORT="${CODEOS_DEEPSEEK_REASONING_EFFORT:-high}"
    API_URL="${CODEOS_DEEPSEEK_URL:-https://api.deepseek.com/chat/completions}"
    MAX_TOKENS="${CODEOS_DEEPSEEK_MAX_TOKENS:-32768}"
    SEND_THINKING=1
    ;;
  gemini)
    KEY_VAR=GEMINI_API_KEY
    MODEL="${CODEOS_GEMINI_MODEL:-gemini-3.7-flash}"
    REASONING_EFFORT="${CODEOS_GEMINI_REASONING_EFFORT:-high}"
    API_URL="${CODEOS_GEMINI_URL:-https://generativelanguage.googleapis.com/v1beta/openai/chat/completions}"
    MAX_TOKENS="${CODEOS_GEMINI_MAX_TOKENS:-32768}"
    SEND_THINKING=0
    ;;
  *)
    err "unsupported CODEOS_LLM_PROVIDER: '${PROVIDER}' (supported: deepseek, gemini)"
    exit 3
    ;;
esac
if [[ "${MAX_TOKENS}" != "32768" && "${MAX_TOKENS}" != "65536" ]]; then
  err "max-tokens bound must be 32768 or 65536; got '${MAX_TOKENS}'"
  exit 3
fi

if [[ -z "${!KEY_VAR:-}" ]]; then
  err "${KEY_VAR} is unset or empty; refusing to run (no network call made)."
  exit 6
fi

OUT_DIR="$(dirname "${OUT}")"
mkdir -p "${OUT_DIR}"
RESP_FILE="${OUT}.response.json"
TOKENS_FILE="${OUT}.tokens.txt"

# --rawfile, not --arg: a single argv element is capped at 128 KiB on Linux (MAX_ARG_STRLEN) and a
# realistic reviewer packet exceeds that. Reading from the file avoids argv entirely.
REQ_BODY="${OUT}.request.json"
# thinking is added only where the provider accepts it: Gemini's OpenAI-compatible endpoint rejects
# the field outright with 400 Unknown name "thinking". The DeepSeek body is unchanged.
if [[ "${SEND_THINKING}" == 1 ]]; then THINKING_ARG='{"type":"enabled"}'; else THINKING_ARG='null'; fi
jq -n --arg model "${MODEL}" \
      --arg effort "${REASONING_EFFORT}" \
      --argjson max_tokens "${MAX_TOKENS}" \
      --argjson thinking "${THINKING_ARG}" \
      --rawfile packet "${PACKET}" \
  '{model:$model,
    messages:[{role:"user",content:$packet}],}
   + (if $thinking == null then {} else {thinking:$thinking} end)
   + {reasoning_effort:$effort,
      max_tokens:$max_tokens,
      stream:false}' > "${REQ_BODY}"

HTTP_CODE="$(
  printf 'header = "Authorization: Bearer %s"\n' "${!KEY_VAR}" \
    | curl -sS -K - \
        -H 'Content-Type: application/json' \
        -X POST "${API_URL}" \
        --data-binary @"${REQ_BODY}" \
        -o "${RESP_FILE}" \
        -w '%{http_code}' \
    || true
)"
if [[ ! "${HTTP_CODE}" =~ ^2[0-9][0-9]$ ]]; then
  err "${PROVIDER} API call failed (http_code=${HTTP_CODE:-none}); response saved to ${RESP_FILE}"
  exit 8
fi

# Accounting is recorded before the reply is interpreted, so a truncated or otherwise non-normal
# response stays attributable and costed. Note that the preserved response file is the whole
# envelope: when the model returns reasoning_content, that content is on disk too.
PT="$(jq -r '.usage.prompt_tokens // "?"' "${RESP_FILE}")"
CT="$(jq -r '.usage.completion_tokens // "?"' "${RESP_FILE}")"
TT="$(jq -r '.usage.total_tokens // "?"' "${RESP_FILE}")"
PCH="$(jq -r '.usage.prompt_cache_hit_tokens // "?"' "${RESP_FILE}")"
PCM="$(jq -r '.usage.prompt_cache_miss_tokens // "?"' "${RESP_FILE}")"
RT="$(jq -r '.usage.completion_tokens_details.reasoning_tokens // "?"' "${RESP_FILE}")"
RETURNED_MODEL="$(jq -r '.model // "?"' "${RESP_FILE}")"
FINISH_REASON="$(jq -r '.choices[0].finish_reason // "?"' "${RESP_FILE}")"
if [[ "${PROVIDER}" == "gemini" ]]; then
  # This API returns prompt/completion/total only: no reasoning field and no cache fields. The three
  # returned numbers are recorded as returned. total-prompt-completion is a residual, and no
  # authoritative source states what it contains, so it is named unclassified rather than reasoning.
  # completion_tokens here excludes that residual, so it IS the final-content figure — the opposite
  # of DeepSeek, where completion includes the reported reasoning.
  UNCL="?"
  if [[ "${PT}" =~ ^[0-9]+$ && "${CT}" =~ ^[0-9]+$ && "${TT}" =~ ^[0-9]+$ ]]; then
    UNCL=$(( TT - PT - CT ))
  fi
  printf 'provider=gemini prompt_tokens=%s completion_tokens=%s total_tokens=%s unclassified_tokens_derived=%s final_content_tokens=%s requested_model=%s returned_model=%s finish_reason=%s max_tokens=%s\n' \
    "${PT}" "${CT}" "${TT}" "${UNCL}" "${CT}" "${MODEL}" "${RETURNED_MODEL}" \
    "${FINISH_REASON}" "${MAX_TOKENS}" > "${TOKENS_FILE}"
else
  printf 'prompt_tokens=%s completion_tokens=%s total_tokens=%s prompt_cache_hit_tokens=%s prompt_cache_miss_tokens=%s reasoning_tokens=%s requested_model=%s returned_model=%s finish_reason=%s max_tokens=%s\n' \
    "${PT}" "${CT}" "${TT}" "${PCH}" "${PCM}" "${RT}" "${MODEL}" "${RETURNED_MODEL}" \
    "${FINISH_REASON}" "${MAX_TOKENS}" > "${TOKENS_FILE}"
fi

# Only a natural stop is a complete assessment. A truncated reply is preserved and accounted, but is
# never written to the assessment path where it would be imported as if it were whole.
if [[ "${FINISH_REASON}" != "stop" ]]; then
  err "response did not terminate normally (finish_reason=${FINISH_REASON}); see ${RESP_FILE}"
  if [[ "${FINISH_REASON}" == "length" && "${MAX_TOKENS}" == "32768" ]]; then
    if [[ "${PROVIDER}" == "gemini" ]]; then
      err "       retry once with CODEOS_GEMINI_MAX_TOKENS=65536"
    else
      err "       retry once with CODEOS_DEEPSEEK_MAX_TOKENS=65536"
    fi
  fi
  exit 8
fi

# -j, not -r: the reply is written byte for byte rather than gaining a newline jq would append.
# A single trailing newline is then guaranteed, so the file is a well-formed text file either way.
jq -j '.choices[0].message.content // empty' "${RESP_FILE}" > "${OUT}"
if [[ -s "${OUT}" && -n "$(tail -c1 "${OUT}")" ]]; then printf '\n' >> "${OUT}"; fi
if [[ ! -s "${OUT}" ]]; then
  err "model returned an empty reply; see ${RESP_FILE}"
  rm -f "${OUT}"
  exit 8
fi

echo "external assessment written: ${OUT}"
if [[ "${PROVIDER}" == "gemini" ]]; then
  echo "  tokens: prompt=${PT} final_content=${CT} total=${TT}   model=${RETURNED_MODEL}   (see ${TOKENS_FILE})"
else
  echo "  tokens: prompt=${PT} completion=${CT} reasoning=${RT} total=${TT}   model=${RETURNED_MODEL}"
fi
echo "  NOTE: advisory only — import with 'codeos-review.sh review ... --assessment ${OUT} --packet ${PACKET}'."
echo "        An external assessment never satisfies a required review round."
