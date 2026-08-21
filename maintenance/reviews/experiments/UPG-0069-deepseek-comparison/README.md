# Frozen reviewer comparison experiment — DeepSeek arm

Evidence for `maintenance/archive/self-development/backlog/completed/UPG-0069-reviewer-provider-comparison.md`: is DeepSeek usable as a
fallback reviewer? The DeepSeek arm ran on 2026-08-18, the Codex arm on 2026-08-21, both on the same
packet bytes. Answer: no — the scoring and decision live in the brief.

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
| `codex-raw-answer.txt` | Codex's reply, verbatim. |
| `codex-response-envelope.json` | Its JSONL event stream, with the final message text replaced by a length note — that text is the raw answer file. |
| `codex-tokens.txt` | Accounting for the single Codex attempt. |
| `codex-parsed-assessment.md` | The record Codeos produced from that reply, through the same import path. |
| `deepseek-v4-pro-raw-answer.txt` | The V4-Pro reply, verbatim. |
| `deepseek-v4-pro-response-envelope.json` | Its API response with `reasoning_content` removed; everything else verbatim. |
| `deepseek-v4-pro-tokens.txt` | Accounting for the single V4-Pro attempt. |
| `deepseek-v4-pro-parsed-assessment.md` | The record Codeos produced from that reply, same import path. |

## What was reviewed

Feature `UPG-EXTPILOT2`, stage `selfdev-step-1`, at review commit `360b838`, over the working tree
holding the external-assessment change. Named artifacts: `dba/04-tools/reviewer/contract/v4.md`,
`dba/02-policies/review/v2.md`, `dba/04-tools/reviewer/codeos-review-deepseek.sh`.

**Coverage: `SECRET_REDACTION`** (1 redaction), with 5 untracked files included in full. Before the
untracked-file repair the same evidence reported `FULL_COVERAGE` while the new implementation
modules were invisible to the reviewer.

## DeepSeek V4-Flash arm result

- `finish_reason: stop` on the second attempt.
- Reported concern `DO NOT ADVANCE`, evidence grade `B`.
- `parse_status: FAILED`, `assessment_status: INCOMPLETE` — 1 declared finding used a decorated
  header (`### Finding 1 / Severity: …`) that the finding parser does not accept, so it was counted
  as unrecorded rather than lost. Effective concern therefore `DO NOT ADVANCE`.
- The single finding was real and was fixed: the adapter documented the import command without the
  required `--packet` flag.

## DeepSeek V4-Flash cost and completion

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

## Codex arm result

`finish_reason` equivalent `turn.completed` on the first attempt. `DO NOT ADVANCE`, evidence `B`,
`parse_status: OK`, `assessment_status: COMPLETE`, 3 findings recorded and 0 unrecorded. 57,958
tokens (50,912 in / 7,046 out, 6,214 of it reasoning), 2m56s. All three findings verified against the
packet bytes; two of them are integrity defects still live at HEAD.

## DeepSeek V4-Pro arm result (UPG-0071)

Added 2026-08-21, same packet bytes, `deepseek-v4-pro` at `reasoning_effort: max` — a different
configuration from the Flash arm's `high`, so results belong to the configuration, not to model tier.
Flash evidence above is unchanged.

`stop` on the **first** attempt at 32768 (Flash needed the 65536 retry): 38,295 prompt / 30,747
completion / 29,363 reasoning / 1,384 final content / 69,042 total, 7m54s. Reported `CHANGES
ADVISED`, evidence `B`, effective `DO NOT ADVANCE`.

`parse_status: FAILED`, `assessment_status: INCOMPLETE` — both findings used a multi-line
`Finding:` / `Severity:` / `Classification:` block the parser does not accept, so `findings: []`. Two
confirmed defects: the `--packet` omission, and a contract false claim neither other arm reported
(the import path "applies the same evidence selection, packet construction … as `review`", packet
line 193). It missed both packet-integrity defects the Codex arm found and listed those two
properties among the *supported* claims.

Scoring and the per-role decision live in
`maintenance/archive/self-development/backlog/completed/UPG-0071-deepseek-v4-pro-requalification.md`.

## Reproducing the DeepSeek arms

```bash
export DEEPSEEK_API_KEY=...
CODEOS_DEEPSEEK_MAX_TOKENS=65536 \
  dba/04-tools/reviewer/codeos-review-deepseek.sh canonical-packet.txt /tmp/answer.txt
```

Model `deepseek-v4-flash`, `thinking: enabled`, `reasoning_effort: high`, `stream: false`, no system
message — the packet carries the reviewer task prompt itself. The V4-Pro arm is the same command with
`CODEOS_DEEPSEEK_MODEL=deepseek-v4-pro CODEOS_DEEPSEEK_REASONING_EFFORT=max` and no
`CODEOS_DEEPSEEK_MAX_TOKENS` override, since it completed at the default bound. Sampling is not pinned, so a rerun
will not reproduce the reply verbatim; the frozen answer above is the record of what was assessed.

## How the Codex arm ran

`codex exec --sandbox read-only` does **not** prevent reads outside `--cd`: a probe under those exact
flags read `/home/rimo/projects/Codeos/CLAUDE.md`, and `codex sandbox -c sandbox_mode=read-only`
confirmed it without a model in the loop. The arm therefore ran inside a `bwrap` mount namespace with
`~/projects` unbound, where the same probe returned NOT-READABLE:

```bash
bwrap --ro-bind /usr /usr --ro-bind /bin /bin --ro-bind /lib /lib --ro-bind /lib64 /lib64 \
  --ro-bind /etc /etc --proc /proc --dev /dev --tmpfs /tmp \
  --bind ~/.codex ~/.codex --bind "$OUT" "$OUT" --chdir "$OUT/empty" \
  --setenv HOME ~ --setenv PATH /usr/bin:/bin \
  ~/.codex/packages/standalone/current/bin/codex exec --json --sandbox read-only \
    --skip-git-repo-check --cd "$OUT/empty" -c model_reasoning_effort=high \
    -o "$OUT/codex-final.txt" - < canonical-packet.txt > "$OUT/codex-events.jsonl"
```

Two details matter for reproduction: invoke the binary by its real path (invoking the
`~/.local/bin/codex` symlink inside the namespace breaks Codex's sibling-helper lookup), and do not
use a login shell (`/etc/profile` puts an older npm-installed `codex` first on PATH).

Both replies were then imported through the same `review --assessment --packet` path, which is what
makes protocol compliance comparable rather than asserted. Both records carry
`reviewed_packet_sha256: 2a5ed7d4…6894f`.
