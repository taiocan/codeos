---
feature_id: UPG-0069
slug: reviewer-provider-comparison
title: Reviewer Provider Comparison on One Canonical Packet
status: FROZEN (awaiting Codex arm)
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

The DeepSeek arm has run. The Codex arm has not; Codex was unavailable. Both arms review the **same
packet bytes**, so the comparison measures the model rather than the infrastructure — the failure
mode UPG-0060 made once and UPG-0064 documented.

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

## Out of scope while frozen

No further DeepSeek-specific engineering unless a new integrity defect is found. Specifically not:
making the finding parser more permissive, optimising the 65536-token path, adding parser exceptions
for one model's formatting, adding another reviewer provider, abstracting a provider layer,
resolving the two historical archive anomalies noted in the corpus regression test, or changing the
required-review policy.

## Completion

This feature completes when the Codex arm has run against
`canonical-packet.txt`, both arms are scored against the seven measures, and the fallback policy
question is decided either way. A negative result closes the fallback question; it does not reopen
the reviewer integrity guarantees, which stand on their own.
