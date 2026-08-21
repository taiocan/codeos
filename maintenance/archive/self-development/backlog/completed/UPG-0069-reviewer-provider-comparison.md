---
feature_id: UPG-0069
slug: reviewer-provider-comparison
title: Reviewer Provider Comparison on One Canonical Packet
status: DECIDED — DeepSeek is not a substitute reviewer (2026-08-21)
priority: P2
depends_on: []
related_features: [UPG-0060, UPG-0064, UPG-0066]
supersedes: []
superseded_by: []
---

# Upgrade: reviewer provider comparison on one canonical packet

## Decision

Answer one question with evidence rather than impression:

> Can DeepSeek serve as a fallback reviewer when Codex is unavailable?

Both arms have now run against the **same packet bytes**, so the comparison measures the model
rather than the infrastructure — the failure mode UPG-0060 made once and UPG-0064 documented.
**Answer: no.** See the decision below.

Frozen evidence and the reproduction procedure:
`maintenance/reviews/experiments/UPG-0069-deepseek-comparison/`.

## Temporary policy while this is open

**Codex is the required reviewer. DeepSeek is experimental and advisory only. An external assessment
does not satisfy a required review round.** This is stated normatively in
`dba/02-policies/review/v2.md`; it is repeated here as context, not as a second authority.

## Comparison measures

Fixed before the Codex arm runs. **Do not change these after seeing the Codex result** — a rubric
adjusted to fit a result measures the adjuster, not the reviewers.

| Measure | Question |
|---|---|
| True findings | Did it find a confirmed defect? |
| False positives | Did it report something that is not a defect? |
| Missed important defects | Did the other reviewer find something material it missed? |
| Protocol compliance | Was the result parseable under the reviewer contract? |
| Completion | Did it finish without truncation or incomplete output? |
| Cost | What input, output, and reasoning resources were consumed? |
| Decision usefulness | Could the result safely support the review workflow? |

## DeepSeek arm result

Scored against the measures above, from the frozen evidence:

- **True findings** — 1 of 1 confirmed. The documented import command omitted the required
  `--packet` flag, so the documented flow would have failed. Fixed.
- **False positives** — none in this run. (A prior run over a different packet produced one, caused
  by untracked files being invisible to the diff; that gap has since been repaired.)
- **Missed important defects** — not yet determinable; requires the Codex arm.
- **Protocol compliance** — failed. The finding used a decorated header
  (`### Finding 1 / Severity: …`) that the finding parser does not accept, giving
  `parse_status: FAILED` and `assessment_status: INCOMPLETE`. The finding was counted as unrecorded
  rather than lost.
- **Completion** — failed at the default bound. `finish_reason: length` at `max_tokens: 32768` with
  the entire completion budget spent on reasoning and no visible output. Succeeded only at 65536.
- **Cost** — 71,050 tokens on the refused attempt plus 78,925 on the accepted one; roughly 12
  minutes of wall time across both.
- **Decision usefulness** — the findings were worth having; the record was not self-sufficient. A
  reader had to read the raw body, because the structured finding list was empty.

## Codex arm result

Ran 2026-08-21 on `canonical-packet.txt`, `sha256` re-verified before and after. Model `gpt-5.6-sol`
via `codex exec`, `model_reasoning_effort: high`, one attempt.

**Packet-only isolation, proven rather than assumed.** `codex exec --sandbox read-only` restricts
writes, not reads: a probe under exactly those flags read `/home/rimo/projects/Codeos/CLAUDE.md`
successfully, and `codex sandbox -c sandbox_mode=read-only` confirmed it deterministically. The arm
therefore ran inside a `bwrap` mount namespace binding only the CLI's own package directory, `/usr`,
`/etc`, and an empty working directory — with `~/projects` unbound. The same probe under that
configuration returned NOT-READABLE. The packet arrived on stdin, so the tree Codex could read was
the packet and nothing else, matching the DeepSeek arm's evidence boundary.

Scored against the seven measures:

- **True findings** — 3 of 3 confirmed against the packet bytes themselves:
  1. *Packet metadata is not cryptographically bound to the exported packet bytes.*
     `load_exported_packet` checks feature, stage, and artifact set against the sidecar and never
     hashes the content, so altered or emptied packet bytes can be imported under a sidecar claiming
     valid coverage. The `EMPTY_PACKET` guard reads `coverage_state` from that same sidecar, so it
     does not catch it either.
  2. *Untracked-file discovery fails open.* `git_untracked_files` ends in
     `.output().map(…).unwrap_or_default()` — it ignores both a spawn failure and a non-zero Git
     exit, returning an empty list, so the packet can report full coverage while omitting untracked
     files. This is the same class of gap the reviewed change existed to close.
  3. *The documented external-assessment command cannot run as shown* — the `--packet` omission,
     the one finding the DeepSeek arm also found.
- **False positives** — none.
- **Missed important defects** — none. It found the DeepSeek arm's only finding and two more.
- **Protocol compliance** — passed. `parse_status: OK`, `assessment_status: COMPLETE`, all three
  findings recorded, `unparsed_findings_count: 0`.
- **Completion** — completed on the first attempt, `turn.completed`, no truncation.
- **Cost** — 57,958 tokens (50,912 input, 11,008 of them cached; 7,046 output of which 6,214
  reasoning); 2m56s wall.
- **Decision usefulness** — the record is self-sufficient: three classified findings with file-level
  evidence, readable without opening the raw reply.

One observation outside the measures: Codex reported scope drift for the two untracked workspace
notes files in the packet; DeepSeek reported none.

## Comparison

| Measure | DeepSeek `deepseek-v4-flash` | Codex `gpt-5.6-sol` |
|---|---|---|
| True findings | 1 confirmed | 3 confirmed |
| False positives | 0 | 0 |
| Missed important defects | both integrity defects | none |
| Protocol compliance | FAILED (`parse_status: FAILED`, 1 finding unrecorded) | PASSED |
| Completion | failed at 32768; succeeded only at 65536 | first attempt |
| Cost | 149,975 tokens over two attempts, ~12 min | 57,958 tokens, ~3 min |
| Decision usefulness | findings worth having; record not self-sufficient | record self-sufficient |

## Decision — DeepSeek is not a fallback reviewer

On identical evidence Codex found three real defects to DeepSeek's one, and the two DeepSeek missed
are the material ones: both weaken the packet-integrity guarantees the reviewed change existed to
establish, and **both are still live in the current tree**. DeepSeek also failed the reviewer
protocol, needed two attempts, and cost 2.6x the tokens and 4x the wall time.

No policy changes. `dba/02-policies/review/v2.md` already says Codex is the required reviewer and
that an external assessment supplies findings without satisfying a review round; this evidence
confirms that text rather than amending it. DeepSeek keeps exactly the standing it already has —
advisory findings under a Review Waiver when the required reviewer cannot run.

**Consequence requiring a separate human decision:** this arm found a new integrity defect, which is
the one condition the freeze below reserved for further work. Findings 1 and 2 are unfixed at HEAD
and are reviewer-integrity defects, not DeepSeek engineering. They need their own backlog item and
are not repaired under this feature.

## Out of scope while frozen

No further DeepSeek-specific engineering unless a new integrity defect is found. Specifically not:
making the finding parser more permissive, optimising the 65536-token path, adding parser exceptions
for one model's formatting, adding another reviewer provider, abstracting a provider layer,
resolving the two historical archive anomalies noted in the corpus regression test, or changing the
required-review policy.

## Completion

Complete. The Codex arm ran against `canonical-packet.txt`, both arms are scored against the seven
measures, and the fallback question is decided: no. That closes the fallback question and does not
reopen the reviewer integrity guarantees, which stand on their own — the two defects the Codex arm
found are new work, tracked separately.
