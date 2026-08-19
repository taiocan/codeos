---
module: implementer-shim
generated_by: deepseek
verified: none
generated_against_commit: b5114dd
---

<!--
Descriptive documentation of how `dba/04-tools/implementer/codeos-implement.sh` currently works.
Explanatory only, and never an authority for behavior or structure: the code is the truth about the
actual implementation. If this note disagrees with the code, the note is stale and gets corrected.

Drafted by DeepSeek from the source and NOT verified claim-by-claim. It describes what the source
does and how; it does not explain why the tool was designed this way. Treat any explanatory-sounding
sentence as unverified unless it is attributed to a comment in the source.
-->

# implementer-shim (`codeos-implement.sh`)

## Overview

`dba/04-tools/implementer/codeos-implement.sh` is a Bash script that acts as an out-of-band, opt-in DeepSeek implementer for DBA Stage 4 (Implementation) and Stage 5 (Tests). It produces a draft candidate only, staged under `.codeos-state/deepseek-candidates/`; it never writes into `modules/` or `tests/`, and it never commits. The script’s header states that a human controls candidate promotion, and that package authority, advisory review, and delivery-cycle verification apply unchanged.

The script is described in its header as a companion to `dba/04-tools/reviewer/codeos-review.sh`, mirroring that shim’s entry-point discipline: git-repo precondition, self-dev-vs-downstream context resolution, fail-closed preconditions, and an explicit exit-code table. It is off by default, controlled by an activation status file.

## Invocation and CLI

Usage as defined by the script:

```text
codeos-implement.sh [role flags] [--exemplar PATH] [--repair-candidate PATH]
                    [--repair-output PATH] <feature_id> <stage:4|5> [artifact-path...]
```

Options must precede positional arguments. The `--` separator stops option parsing; remaining arguments are positional. An unknown option exits with code 3.

### Artifact role flags

The caller declares each artifact’s authority explicitly. The script performs no inference of role from path, filename, content, headings, or directory.

| Option | Role label used in the request |
|---|---|
| `--contract PATH` | `BEHAVIORAL CONTRACT` |
| `--event-schema PATH` | `EVENT SCHEMA` |
| `--architecture PATH` | `PROJECT ARCHITECTURE` |
| `--profile PATH` | `IMPLEMENTATION PROFILE` |

These flags may be repeated. Each stores paths in a dedicated array.

### Other options

| Option | Meaning |
|---|---|
| `--exemplar PATH` | Shows a real repository file as a `LAYOUT EXEMPLAR`; context for naming/placement conventions, not a specification to implement. |
| `--repair-candidate PATH` | Shows a previous candidate as a `PRIOR ATTEMPT`. |
| `--repair-output PATH` | Shows build/test output from that prior attempt as `FEEDBACK`. |

### Positional arguments

- `feature_id` — used only to name the staging directory; slashes are replaced with underscores.
- `stage` — exactly `4` or `5`; any other value is a usage error.
- `artifact-path...` — optional positional artifacts, labelled `APPROVED ARTIFACT (ROLE UNSPECIFIED)`. They are supported for backward compatibility and never silently satisfy a declared role.

At least one artifact must be supplied, either via a role flag or as a positional artifact. A call with zero artifacts exits with code 3.

## Entry-point checks and activation

The script enforces preconditions in a fixed sequence.

1. **Git repository check**  
   `git rev-parse --show-toplevel` is run first. Failure exits with code 1.

2. **Usage and argument validation**  
   Options are parsed, then `feature_id` and `stage` are required. Stage must be `4` or `5`. At least one artifact must be present. Missing/incorrect usage exits with code 3.

3. **Dependency check**  
   `curl` and `jq` must be on `PATH`. Missing dependency exits with code 2.

4. **Context resolution**  
   The script resolves its own repository root and the caller’s repository root:

   - `SCRIPT_DIR` is the physical directory of `BASH_SOURCE[0]`.
   - `CODEOS_ROOT` is `SCRIPT_DIR/../../..`, resolved physically.
   - `CALLER_ROOT` is the caller’s git top level, resolved physically.

   If `CALLER_ROOT == CODEOS_ROOT`, the context is self-dev; otherwise it is downstream.

5. **Activation status**  
   The status file location depends on context:

   - self-dev: `${CODEOS_ROOT}/maintenance/config/delegated-implementation.yaml`
   - downstream: `${CALLER_ROOT}/.codeos/02-architecture/delegated-implementation.yaml`

   The file is parsed as follows:
   - File absent → `disabled`.
   - Exactly one non-blank line equal to `status: disabled` → `disabled`.
   - Exactly one non-blank line equal to `status: enabled` → `enabled`.
   - Anything else → configuration error, exit code 5.

   If the status is not `enabled`, the script exits with code 4 and refuses to run.

6. **Artifact existence**  
   Positional artifacts and role-flag artifacts must exist; a missing artifact exits with code 7.

7. **Role conflict check**  
   A path may not be declared under two different authority roles. Roles tracked are `BEHAVIORAL CONTRACT`, `EVENT SCHEMA`, `PROJECT ARCHITECTURE`, `IMPLEMENTATION PROFILE`, and `LAYOUT EXEMPLAR`. The check runs before any network call. A conflict exits with code 12. Repeated declaration under the same role is not treated as a conflict.

8. **Exemplar and repair-input existence**  
   - `--exemplar` paths must exist; a missing exemplar exits with code 9.
   - `--repair-candidate` and `--repair-output` paths must exist; a missing repair-input exits with code 10.

9. **API key check**  
   `DEEPSEEK_API_KEY` must be set and non-empty. If missing while enabled, the script exits with code 6 before any network call.

10. **Configuration validation**  
   The task prompt file must exist at `${CODEOS_ROOT}/dba/03-prompts/delegation/codeos-implementer-task.md`; if not, exit code 8.

## Environment variables

| Variable | Behavior |
|---|---|
| `DEEPSEEK_API_KEY` | Required when enabled. Read only into the HTTP `Authorization` header via a curl config on stdin. Never placed in argv, the request body, the preserved packet, the response dump, or any candidate file. |
| `CODEOS_DEEPSEEK_MODEL` | Optional. Default `deepseek-v4-flash`. |
| `CODEOS_DEEPSEEK_URL` | Optional. Default `https://api.deepseek.com/chat/completions`. |
| `CODEOS_DEEPSEEK_MAX_TOKENS` | Optional. Default `32768`. The only other supported value is `65536`. Any other value is a usage error, exit code 3. |

The source comment ties `65536` to one explicit pilot retry after `finish_reason=length`; a retry is a new invocation, so both attempts remain visible.

## Staging area

The script creates a timestamped, non-overwriting staging directory under the caller’s repository:

```text
${CALLER_ROOT}/.codeos-state/deepseek-candidates/${FEATURE_SAFE}-stage-${STAGE}/${TS}.XXXXXX
```

- `TS` is `date -u +%Y%m%dT%H%M%SZ`.
- `FEATURE_SAFE` is the feature id with `/` replaced by `_`.
- `mktemp -d` guarantees a unique leaf even for same-second re-runs.
- A `candidate/` subdirectory is created inside the stage directory.
- The run log is `${STAGE_ROOT}/implement-log.md`.

## Request construction

### System and user contents

The system message is the full contents of the task prompt file.

The user content is written to `${STAGE_DIR}/user_content.txt` and contains:

- A header: `DELEGATED IMPLEMENTATION REQUEST`, `feature_id`, `stage`, and `output_nonce`.
- A statement telling the model to produce the candidate following the strict output contract and to use the nonce verbatim in every marker.
- Declared role artifacts, emitted in a fixed order:
  - `BEHAVIORAL CONTRACT`
  - `EVENT SCHEMA`
  - `PROJECT ARCHITECTURE`
  - `IMPLEMENTATION PROFILE`
- Positional artifacts labelled `APPROVED ARTIFACT (ROLE UNSPECIFIED)`, with a note that they are supporting context only.
- `LAYOUT EXEMPLAR` files, labelled as context only and explicitly not to be implemented, modified, or copied for domain behavior.
- If any repair inputs are present, a `REPAIR REQUEST` block containing `PRIOR ATTEMPT` files and `FEEDBACK` files, with instructions to re-emit the complete candidate rather than a patch.

Artifact content is emitted byte-for-byte via `cat`.

### Audit packet

`${STAGE_DIR}/packet.txt` contains the full system and user contents. It contains no API key.

### JSON request body

The request body is written to `${STAGE_DIR}/request.json` using `jq`:

```jq
{
  model: $model,
  messages: [
    {role: "system", content: $sys},
    {role: "user", content: $usr}
  ],
  thinking: {type: "enabled"},
  reasoning_effort: "high",
  max_tokens: $max_tokens,
  stream: false
}
```

The source comment notes that `--rawfile` is used for `sys` and `usr` rather than `--arg`, because passing the packet as a single argv element can exceed Linux’s 128 KiB `MAX_ARG_STRLEN` limit. There is no `response_format`; the candidate is returned as plain text under the delimited output protocol.

## API call and response handling

The API call is made with `curl`:

```bash
printf 'header = "Authorization: Bearer %s"\n' "$DEEPSEEK_API_KEY" \
  | curl -sS -K - \
      -H 'Content-Type: application/json' \
      -X POST "$DS_URL" \
      --data-binary @"$REQ_BODY" \
      -o "$RESP_FILE" \
      -w '%{http_code}'
```

The key is supplied only through the curl config on stdin. If the HTTP response code is not in the 2xx range, the script exits with code 8 and reports the saved response file.

For a successful transport response, the script records accounting values from the JSON response into `${STAGE_DIR}/tokens.txt`:

- prompt tokens
- completion tokens
- total tokens
- prompt cache hit tokens
- prompt cache miss tokens
- reasoning tokens
- requested model
- returned model
- finish reason
- max tokens

The model’s reply text is extracted with `jq -r '.choices[0].message.content // empty'` into `${STAGE_DIR}/model_content.txt`.

A candidate is eligible for parsing only when `finish_reason` is `stop`. Otherwise:

- The script exits with code 8.
- If `finish_reason` is `length` and `MAX_TOKENS` is `32768`, the error message states that one explicit retry with `CODEOS_DEEPSEEK_MAX_TOKENS=65536` is permitted.
- An empty content reply also exits with code 8.

## Output protocol parsing and staging

A per-run nonce is generated from 8 bytes of `/dev/urandom`, hex-encoded. The parser accepts markers only with that exact nonce.

### Marker syntax

The model’s reply is expected to use delimited markers:

```text
<<<CODEOS:<nonce>:FILE:<path>>>     ... <<<CODEOS:<nonce>:ENDFILE>>>
<<<CODEOS:<nonce>:SECTION:<name>>>  ... <<<CODEOS:<nonce>:ENDSECTION>>>
```

Allowed section names are:

- `contract_satisfaction`
- `event_emission`
- `notes`
- `deferral_resolution`

### Pass 1: frame validation

An `awk` program validates the entire response and writes a manifest to `${STAGE_DIR}/.frame-manifest`. It rejects:

- FILE markers inside an open block
- empty candidate paths
- duplicate candidate paths
- duplicate section names
- unknown section names
- unterminated blocks
- responses with no candidate file blocks
- any marker mismatch

Errors are written to `${STAGE_DIR}/.frame-error`. If that file is non-empty, the script exits with code 11, states that nothing was staged, and preserves the raw reply.

### Path validation

Before writing any candidate files, the script validates every manifest path. It rejects absolute paths and any path containing `..`; those exit with code 8. It also enforces the stage-area allowlist:

- Stage 4 → files must start with `modules/`
- Stage 5 → files must start with `tests/`

The single exception is `CANDIDATE_BLOCKED.md`, which the model can use to report insufficient artifacts. The script does not constrain what kind of file is staged within the allowed area.

### Pass 2: writing

A second pass reads the manifest and writes each block:

- File blocks are written to `${STAGE_DIR}/candidate/<path>`.
- Section blocks are written to `${STAGE_DIR}/<section>.txt`.
- If a block has an empty range, an empty file is created.
- Otherwise, `sed -n "${s},${e}p"` extracts the exact line range from `model_content.txt`.

Content is emitted byte-for-byte between markers, never re-encoded.

If the model omitted `contract_satisfaction`, `event_emission`, or `notes`, empty sidecar files are created so the audit set is uniform. `deferral_resolution` is not auto-created; the source comment says its presence is itself the signal that the model reported a resolved deferral.

## Logging and terminal output

The script appends one invocation record to `${STAGE_ROOT}/implement-log.md` containing:

- timestamp, feature id, stage
- requested and returned model
- finish reason and max tokens
- token counts
- number of candidate files
- staging directory

On success, it prints:

```text
candidate staged: <dir>/candidate  (N file(s))
tokens: ...
audit: packet=... response=... log=...
NOTE: candidate only — promote manually; the Stage N human gate + advisory review still apply.
```

## Exit codes

| Code | Condition |
|---|---|
| 0 | Success — candidate staged |
| 1 | Not inside a git repository |
| 2 | Missing dependency (`curl` or `jq` not found) |
| 3 | Usage error |
| 4 | Mechanism disabled or status file absent |
| 5 | Activation status file malformed |
| 6 | `DEEPSEEK_API_KEY` unset or empty while enabled |
| 7 | Artifact path does not exist |
| 8 | DeepSeek API/transport error, unsafe candidate path, or missing task prompt |
| 9 | `--exemplar` path does not exist |
| 10 | `--repair-candidate` / `--repair-output` path does not exist |
| 11 | Model response violates the delimited output protocol; nothing staged |
| 12 | Same artifact path declared under two different authority roles |

## External processes

The script never runs a build, test, compile, or package-manager command, never `eval`s, and never shells out to a project-supplied command. Build output is an input supplied by the caller.

The external commands the script invokes are exactly:

```text
git curl jq awk sed cat tr od head date mkdir mktemp rmdir dirname
```

The source comment states that `dba/04-tools/implementer/tests/codeos-implement-tests.sh` scans the script against this list and fails if any external tool outside it appears, so the comment cannot silently drift from the code.
