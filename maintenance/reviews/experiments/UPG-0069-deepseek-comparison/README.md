# Frozen reviewer comparison experiment — DeepSeek arm

Evidence for the open question in `maintenance/backlog/UPG-0069-reviewer-provider-comparison.md`:
is DeepSeek usable as a fallback reviewer? The DeepSeek arm ran on 2026-08-18. The Codex arm has not
run; Codex was unavailable.

## The rule this directory exists to enforce

**The Codex arm MUST read `canonical-packet.txt`, byte for byte. Do not regenerate the packet.**

A regenerated packet would differ — the working tree has moved on, and the manifest carries a
generation timestamp — and the two arms would then be reviewing different evidence. Any difference
in findings could be attributed to the packet rather than to the model, which is the failure mode
this comparison exists to avoid.

```text
sha256(canonical-packet.txt) = 2a5ed7d4d74dddd95222f2327e5245e54b7d1247eacba6f7882924e7ae26894f
```

Verify before use: `sha256sum -c canonical-packet.sha256`

## Contents

| File | What it is |
|---|---|
| `canonical-packet.txt` | The exact bytes DeepSeek read. 151,534 bytes. |
| `canonical-packet.txt.meta.json` | Its sidecar: feature, stage, artifacts, hashes, coverage. Required by `review --packet`. |
| `deepseek-raw-answer.txt` | DeepSeek's reply, verbatim. |
| `deepseek-response-envelope.json` | The API response with `reasoning_content` (168,997 chars) removed — it is not an input to any comparison measure. Everything else verbatim. |
| `deepseek-attempt-2-tokens.txt` | Accounting for the successful attempt. |
| `deepseek-parsed-assessment.md` | The record Codeos produced from that reply. |

## What was reviewed

Feature `UPG-EXTPILOT2`, stage `selfdev-step-1`, at review commit `360b838`, over the working tree
holding the external-assessment change. Named artifacts: `dba/04-tools/reviewer/contract/v4.md`,
`dba/02-policies/review/v2.md`, `dba/04-tools/reviewer/codeos-review-deepseek.sh`.

**Coverage: `SECRET_REDACTION`** (1 redaction), with 5 untracked files included in full. Before the
untracked-file repair the same evidence reported `FULL_COVERAGE` while the new implementation
modules were invisible to the reviewer.

## Result

- `finish_reason: stop` on the second attempt.
- Reported concern `DO NOT ADVANCE`, evidence grade `B`.
- `parse_status: FAILED`, `assessment_status: INCOMPLETE` — 1 declared finding used a decorated
  header (`### Finding 1 / Severity: …`) that the finding parser does not accept, so it was counted
  as unrecorded rather than lost. Effective concern therefore `DO NOT ADVANCE`.
- The single finding was real and was fixed: the adapter documented the import command without the
  required `--packet` flag.

## Cost and completion

| Attempt | max_tokens | prompt | completion | reasoning | total | finish_reason | wall |
|---|---|---|---|---|---|---|---|
| 1 | 32768 | 38,282 | 32,768 | 32,768 | 71,050 | `length` | 4m49s |
| 2 | 65536 | 38,282 | 40,643 | 39,695 | 78,925 | `stop` | ~7m |

Attempt 1 spent its entire completion budget on reasoning and produced no visible output. It was
refused and nothing was staged; its accounting file was later overwritten by attempt 2, so those
figures are recorded here rather than kept as a file. Both attempts count toward cost.

A prior run on 2026-08-18 over a smaller 57,331-byte packet completed at `max_tokens: 32768`
(prompt 18,153 / reasoning 30,818 / total 50,052, 4m39s) and found the packet-identity defect that
prompted the integrity repair. That packet is not the comparison packet.

## Reproducing the DeepSeek arm

```bash
export DEEPSEEK_API_KEY=...
CODEOS_DEEPSEEK_MAX_TOKENS=65536 \
  dba/04-tools/reviewer/codeos-review-deepseek.sh canonical-packet.txt /tmp/answer.txt
```

Model `deepseek-v4-flash`, `thinking: enabled`, `reasoning_effort: high`, `stream: false`, no system
message — the packet carries the reviewer task prompt itself. Sampling is not pinned, so a rerun
will not reproduce the reply verbatim; the frozen answer above is the record of what was assessed.

## Running the Codex arm

Codex must receive these exact bytes on stdin, not a rebuilt packet. Record its reply the same way
the DeepSeek reply was recorded, then score both against the seven measures fixed in the backlog
brief. Those measures were fixed before Codex ran and must not change after its result is seen.
