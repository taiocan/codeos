---
module: deepseek-transport
generated_by: deepseek
verified: none
generated_against_commit: b5114dd
---

<!--
Descriptive documentation of how `dba/04-tools/reviewer/codeos-review-deepseek.sh` currently works.
Explanatory only, and never an authority for behavior or structure: the code is the truth about the
actual implementation. If this note disagrees with the code, the note is stale and gets corrected.

Drafted by DeepSeek from the source and NOT verified claim-by-claim. It describes what the source
does and how; it does not explain why the module was designed this way. Treat any
explanatory-sounding sentence as unverified unless it is attributed to a comment in the source.
-->

# deepseek-transport

`codeos-review-deepseek.sh` is a Bash script that describes itself in its header as “transport only”: it sends a canonical reviewer packet to the DeepSeek chat-completions API and writes the raw reply back out. The header also states that the script “parses nothing, decides nothing, and writes no reviewer record”; its observable behavior does parse response-envelope metadata for token accounting and finish-reason checking, but it does not interpret the model’s message content beyond copying it to the assessment file.

The script is one half of the external-assessment path shown in the header comment:

```sh
codeos-review.sh plan <feature> <stage> --emit-packet PACKET <artifacts...>
codeos-review-deepseek.sh PACKET ASSESSMENT
codeos-review.sh review <feature> <stage> --assessment ASSESSMENT --packet PACKET \
    --reviewer-label deepseek-v4-flash <artifacts...>
```

## Usage and interface

```
codeos-review-deepseek.sh <packet-file> <assessment-file>
```

Arguments:

- `<packet-file>`: the reviewer packet to send.
- `<assessment-file>`: path where the model reply is written on success.

Environment:

| Variable | Default | Behavior |
|---|---|---|
| `DEEPSEEK_API_KEY` | *(required)* | Read only into the HTTP `Authorization` header via a curl config on stdin. The script refuses to run if unset or empty, before any network call. |
| `CODEOS_DEEPSEEK_MODEL` | `deepseek-v4-flash` | Value of `model` in the request body. |
| `CODEOS_DEEPSEEK_URL` | `https://api.deepseek.com/chat/completions` | HTTP endpoint for the POST request. |
| `CODEOS_DEEPSEEK_MAX_TOKENS` | `32768` | Must be `32768` or `65536`; validated before any network call. If another value is supplied, the script exits with usage error. |

Exit codes:

| Code | Meaning |
|---|---|
| `0` | Success; assessment file written. |
| `2` | `curl` or `jq` not found on `PATH`. |
| `3` | Usage error, or unsupported `CODEOS_DEEPSEEK_MAX_TOKENS` value. |
| `6` | `DEEPSEEK_API_KEY` unset or empty; no network call made. |
| `7` | Packet file missing or empty. |
| `8` | DeepSeek API/transport error, non-2xx HTTP response, non-`stop` finish reason, or empty model reply. |

## Validation phase

The script checks, in order:

1. That exactly two arguments were supplied.
2. That `curl` and `jq` are available via `command -v`.
3. That the packet file exists and is non-empty, using `[[ -s "${PACKET}" ]]`.
4. That `CODEOS_DEEPSEEK_MAX_TOKENS` is one of the two allowed values.
5. That `DEEPSEEK_API_KEY` is non-empty.

Failures at these points exit before any output directory is created or any request is sent.

## Request construction

After validation, the script:

- Creates the output directory with `mkdir -p "$(dirname "${OUT}")"`.
- Sets three sidecar paths:
  - `${OUT}.request.json`
  - `${OUT}.response.json`
  - `${OUT}.tokens.txt`
- Builds the request body with `jq`:

```json
{
  "model": "<CODEOS_DEEPSEEK_MODEL>",
  "messages": [
    {
      "role": "user",
      "content": "<contents of packet file>"
    }
  ],
  "thinking": { "type": "enabled" },
  "reasoning_effort": "high",
  "max_tokens": <CODEOS_DEEPSEEK_MAX_TOKENS>,
  "stream": false
}
```

The packet contents are read via `jq --rawfile`. A source comment states that this avoids a single argv element, which is capped at 128 KiB on Linux.

The request body is written to `${OUT}.request.json` before the HTTP call.

## HTTP call

The API key is placed into a curl config line:

```sh
header = "Authorization: Bearer ${DEEPSEEK_API_KEY}"
```

That line is piped to `curl -K -`, so the key is read from stdin rather than appearing in argv, in the request body, or in any written file.

The curl invocation:

```sh
curl -sS -K - \
    -H 'Content-Type: application/json' \
    -X POST "${DS_URL}" \
    --data-binary @"${REQ_BODY}" \
    -o "${RESP_FILE}" \
    -w '%{http_code}'
```

The response body is written to `${OUT}.response.json`; the HTTP status code is captured in `HTTP_CODE`. The pipeline is guarded with `|| true`, so a curl failure does not abort the script before the status code is examined.

If `HTTP_CODE` is not a 2xx code, the script prints an error and exits `8`. The response file remains on disk.

## Token accounting

After a 2xx response, the script extracts these fields from `${OUT}.response.json` with `jq`, using `// "?"` for any missing value:

- `usage.prompt_tokens`
- `usage.completion_tokens`
- `usage.total_tokens`
- `usage.prompt_cache_hit_tokens`
- `usage.prompt_cache_miss_tokens`
- `usage.completion_tokens_details.reasoning_tokens`
- `model`
- `choices[0].finish_reason`

It writes one line to `${OUT}.tokens.txt` with this format:

```text
prompt_tokens=... completion_tokens=... total_tokens=... prompt_cache_hit_tokens=... prompt_cache_miss_tokens=... reasoning_tokens=... requested_model=... returned_model=... finish_reason=... max_tokens=...
```

This accounting line is written before the finish reason is checked. A source comment states that this keeps a truncated or otherwise non-normal response attributable and costed.

## Finish-reason check

If `choices[0].finish_reason` is not `stop`, the script:

- Prints an error with the actual `finish_reason`.
- If the reason is `length` and `MAX_TOKENS` was `32768`, prints a hint to retry once with `CODEOS_DEEPSEEK_MAX_TOKENS=65536`.
- Exits `8` without writing the assessment file.

Only a natural `stop` is treated as a completed assessment.

## Assessment output

On `finish_reason=stop`, the script writes the model output:

```sh
jq -j '.choices[0].message.content // empty' "${RESP_FILE}" > "${OUT}"
```

`-j` writes the content byte for byte without adding a trailing newline. The script then guarantees a single trailing newline if the file is non-empty and does not already end in one.

If the resulting file is empty, the script removes it, prints an error, and exits `8`.

On success, the script prints:

```text
external assessment written: <OUT>
  tokens: prompt=<PT> completion=<CT> reasoning=<RT> total=<TT>   model=<RETURNED_MODEL>
  NOTE: advisory only — import with 'codeos-review.sh review ... --assessment <OUT> --packet <PACKET>'.
        An external assessment never satisfies a required review round.
```

## Files produced

| Path | Contents |
|---|---|
| `<OUT>` | Model message content written only on success with `finish_reason=stop` and non-empty content. |
| `<OUT>.request.json` | The exact request body sent to DeepSeek. |
| `<OUT>.response.json` | The full API response envelope, retained even on non-2xx or non-stop outcomes. |
| `<OUT>.tokens.txt` | One-line token and model accounting record, written after a 2xx response and before the finish-reason check. |

The packet file is read but not modified. The header comment notes that the packet must keep its `.meta.json` sidecar beside it; the script itself does not inspect that sidecar.
