---
feature_id: UPG-0047
slug: structured-finding-lifecycle
title: Structured Finding Lifecycle
status: PROPOSED
priority: P2
depends_on: [UPG-0046]
related_features: [UPG-0045, UPG-0048]
supersedes: []
superseded_by: []
---

# Upgrade: structured-finding-lifecycle — Structured Finding Lifecycle

**Priority**: P2
**Status**: PROPOSED
**Type**: script-tooling

## Problem

Reviewer findings are five-category triaged prose (`IN-SCOPE BLOCKER` / `IN-SCOPE NON-BLOCKER` /
`OUT-OF-SCOPE BACKLOG` / `REJECTED` / `SELF-REFERENCE`/`REVIEW-BOOKKEEPING` — see
`prompts/codeos-reviewer-task.md`'s TRIAGE RULE and `docs/reviewer-pipeline.md` §4c/§7). This
triage is well-designed and consistently applied, but a finding's full lifecycle — raised in R1,
fixed, re-verified as resolved in R2 — exists only as free text scattered across the raw
assessment (`reviews/codex/*.md`), the change record's own "R1 fixes" prose, and a manual
`reviews/review-log.md` rollup. There is no way to ask "is finding X resolved?" other than
reading prose and trusting the transcription.

This session's `UPG-0044` change hit exactly this: a Step 3 R2 finding was itself about a
*previous fix's own evidence claim* being incomplete (the R1 fix for "unsupported absence claim"
cited a grep command that itself omitted one of the three terms it claimed to cover). Tracking
that kind of fix-introduces-a-new-gap chain by hand, across free-text rounds, is exactly where
transcription risk compounds.

## Upgrade

Not decided by this brief — questions for implementer to resolve:

### 1. Finding record shape

An illustrative sketch (**not an approved schema**) — a structured record per finding, with a
status field distinct from its triage classification:

```yaml
# ILLUSTRATIVE ONLY — not an approved schema
finding_id: FND__REV__UPG-0043__S3__R1__001
review_id: REV__UPG-0043__CHG-20260711-002__S3__R1
classification: IN_SCOPE_BLOCKER   # the five-category TRIAGE RULE value, unchanged
severity: High
acceptance_criterion: AC-1          # optional link, when the finding maps to a stated AC
summary: "Test names not preserved"
status: open                        # open | resolved | waived | backlog | rejected
resolved_by: null                   # review_id of the round that resolved it, once resolved
```

The **triage classification** (five-category) stays exactly as-is — this upgrade does not touch
that vocabulary. What's new is a **status** field (open/resolved/waived) and a `resolved_by` link,
so "R1 blockers fixed in R2" becomes a checkable fact instead of a prose claim.

### 2. Who writes it

Whether findings are structured by parsing the reviewer's existing free-text output (regex/LLM
extraction of the `Finding: / Severity: / Classification:` block already emitted per
`prompts/codeos-reviewer-task.md`'s required output shape — likely mechanical, since that shape is
already fairly regular), or whether the reviewer prompt itself should be changed to *also* emit a
structured block. Changing the reviewer prompt's required output shape is a bigger, riskier move
(prompt changes ripple to every review call) — the implementer should prefer parsing the existing
shape unless it proves unreliable.

### 3. Resolution tracking granularity

Whether "resolved" is asserted by the human/Claude at fix time (self-reported, cheap, matches
current practice) or requires the *next* review round to explicitly confirm resolution before the
status flips (stronger guarantee, costs a review round per finding, may be excessive for
low-severity findings). This is a real tradeoff between cost and rigor that should be a Step 1/2
design decision, not assumed here.

## Scope

Depends on `UPG-0046` (ReviewRun Structured Records) existing first — a finding needs a
`review_id` to attach to. Likely touches `tools/reviewer/src/assessment.rs` (where the existing
`parse_review_output` function already parses the reviewer's free-text findings block — see
`tools/reviewer/src/assessment.rs:16`) to extend parsing into structured findings, plus wherever
`UPG-0046` decides to store `ReviewRun` records.

Out of scope for this brief:
- Changing the five-category TRIAGE RULE vocabulary itself, or `prompts/codeos-reviewer-task.md`'s
  required output shape, unless parsing the existing shape proves unworkable.
- Auto-resolving findings without a human/Claude asserting the fix — status transitions remain
  something the acting agent records, not something inferred automatically from a diff.
- Any dashboard/UI beyond what already exists (`status/self-development.md`,
  `backlog/UPG-####-*.md`'s Feature Thread rollup) — those may eventually *consume* structured
  findings but building new surfaces for them is separate scope.

## Value

Medium-high, mainly as a force-multiplier for `UPG-0046`. Makes "which findings are still open,"
"which round resolved finding X," and "did a fix introduce a new gap" mechanically answerable
instead of requiring a full prose re-read — directly addresses the fix-introduces-a-new-gap
pattern observed in `UPG-0044`'s own Step 3 R1→R2.

Trade-offs: extraction/parsing of free-text reviewer output is inherently a little brittle;
a malformed or unusually-shaped reviewer response could produce a finding record that doesn't
parse cleanly, and the fallback behavior (drop it? flag it low-confidence? fail closed like
`UNCLASSIFIED` does today?) needs a decision.

## Risk

Deciding hastily risks:
- Building parsing that's brittle against reviewer output variance, silently losing findings.
- Letting "status: resolved" become an implicit approval mechanism that erodes the
  human-decides guarantee — the status field must describe *what the acting agent did*, never
  substitute for the human's decision at the gate.

## Guardrail

The structured finding lifecycle must:
- Never let a finding's `status` field stand in for, or bypass, the human's approval decision at
  the gate — it is bookkeeping about findings, not a second gate.
- Preserve the exact five-category TRIAGE RULE vocabulary (`docs/reviewer-pipeline.md` §7,
  `prompts/codeos-reviewer-task.md`) — `status` is additive, not a replacement classification.
- Fail closed (flag as unparseable, never silently drop) when a reviewer's free-text output
  doesn't match the expected findings shape — same principle as the existing `UNCLASSIFIED`
  handling for malformed `LOG SUMMARY` lines (`docs/reviewer-pipeline.md` §6).

## Related

- **UPG-0046**: ReviewRun Structured Records — hard dependency; findings need a `review_id` to
  attach to.
- **UPG-0045**: Review Plan Preview — could eventually surface "open findings from the last
  round" as part of its preview, once findings are structured.
- **UPG-0048**: Review Ledger Event Sourcing — would consume structured findings as one of its
  event types, if ever pursued; not a prerequisite either direction.
- Proposed by the human during a 2026-07-12 review-architecture discussion (see
  `reviews/review-log.md` and `changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md`
  — that change's own Step 3 R1→R2 finding chain is the concrete motivating example cited above).

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| (none yet) | — | — | PROPOSED |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
