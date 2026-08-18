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
#   DEEPSEEK_API_KEY  required. Read only into the HTTP Authorization header via a curl config on
#                     stdin — never in argv, the request body, or any written file.
#   CODEOS_DEEPSEEK_MODEL       optional, default "deepseek-v4-flash".
#   CODEOS_DEEPSEEK_URL         optional, default "https://api.deepseek.com/chat/completions".
#   CODEOS_DEEPSEEK_MAX_TOKENS  optional, default 32768. The only other supported value is 65536,
#                               for one explicit retry after finish_reason=length. A retry is a new
#                               invocation so both attempts remain visible and count toward cost.
#
# Exit codes:
#   0  success — assessment written
#   2  missing dependency (curl or jq not found on PATH)
#   3  usage error, or an unsupported max-tokens bound (checked before any network call)
#   6  DEEPSEEK_API_KEY unset or empty — refuse before any network call
#   7  the packet file does not exist or is empty
#   8  DeepSeek API / transport error, or a response that did not terminate normally
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

MODEL="${CODEOS_DEEPSEEK_MODEL:-deepseek-v4-flash}"
DS_URL="${CODEOS_DEEPSEEK_URL:-https://api.deepseek.com/chat/completions}"
MAX_TOKENS="${CODEOS_DEEPSEEK_MAX_TOKENS:-32768}"
if [[ "${MAX_TOKENS}" != "32768" && "${MAX_TOKENS}" != "65536" ]]; then
  err "CODEOS_DEEPSEEK_MAX_TOKENS must be 32768 or 65536; got '${MAX_TOKENS}'"
  exit 3
fi

if [[ -z "${DEEPSEEK_API_KEY:-}" ]]; then
  err "DEEPSEEK_API_KEY is unset or empty; refusing to run (no network call made)."
  exit 6
fi

OUT_DIR="$(dirname "${OUT}")"
mkdir -p "${OUT_DIR}"
RESP_FILE="${OUT}.response.json"
TOKENS_FILE="${OUT}.tokens.txt"

# --rawfile, not --arg: a single argv element is capped at 128 KiB on Linux (MAX_ARG_STRLEN) and a
# realistic reviewer packet exceeds that. Reading from the file avoids argv entirely.
REQ_BODY="${OUT}.request.json"
jq -n --arg model "${MODEL}" \
      --argjson max_tokens "${MAX_TOKENS}" \
      --rawfile packet "${PACKET}" \
  '{model:$model,
    messages:[{role:"user",content:$packet}],
    thinking:{type:"enabled"},
    reasoning_effort:"high",
    max_tokens:$max_tokens,
    stream:false}' > "${REQ_BODY}"

HTTP_CODE="$(
  printf 'header = "Authorization: Bearer %s"\n' "${DEEPSEEK_API_KEY}" \
    | curl -sS -K - \
        -H 'Content-Type: application/json' \
        -X POST "${DS_URL}" \
        --data-binary @"${REQ_BODY}" \
        -o "${RESP_FILE}" \
        -w '%{http_code}' \
    || true
)"
if [[ ! "${HTTP_CODE}" =~ ^2[0-9][0-9]$ ]]; then
  err "DeepSeek API call failed (http_code=${HTTP_CODE:-none}); response saved to ${RESP_FILE}"
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
printf 'prompt_tokens=%s completion_tokens=%s total_tokens=%s prompt_cache_hit_tokens=%s prompt_cache_miss_tokens=%s reasoning_tokens=%s requested_model=%s returned_model=%s finish_reason=%s max_tokens=%s\n' \
  "${PT}" "${CT}" "${TT}" "${PCH}" "${PCM}" "${RT}" "${MODEL}" "${RETURNED_MODEL}" \
  "${FINISH_REASON}" "${MAX_TOKENS}" > "${TOKENS_FILE}"

# Only a natural stop is a complete assessment. A truncated reply is preserved and accounted, but is
# never written to the assessment path where it would be imported as if it were whole.
if [[ "${FINISH_REASON}" != "stop" ]]; then
  err "response did not terminate normally (finish_reason=${FINISH_REASON}); see ${RESP_FILE}"
  if [[ "${FINISH_REASON}" == "length" && "${MAX_TOKENS}" == "32768" ]]; then
    err "       retry once with CODEOS_DEEPSEEK_MAX_TOKENS=65536"
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
echo "  tokens: prompt=${PT} completion=${CT} reasoning=${RT} total=${TT}   model=${RETURNED_MODEL}"
echo "  NOTE: advisory only — import with 'codeos-review.sh review ... --assessment ${OUT} --packet ${PACKET}'."
echo "        An external assessment never satisfies a required review round."
