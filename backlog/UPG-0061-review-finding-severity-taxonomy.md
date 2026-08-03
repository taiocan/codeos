---
feature_id: UPG-0061
slug: review-finding-severity-taxonomy
title: Separate Behavioral Blockers from Record-Hygiene Findings in Review
status: PROPOSED
priority: P2
depends_on: []
related_features: [UPG-0047, UPG-0060]
supersedes: []
superseded_by: []
---

# Upgrade: review-finding-severity-taxonomy — Separate Behavioral Blockers from Record-Hygiene Findings

**Priority**: P2
**Status**: PROPOSED
**Type**: prompt (+ possible script-tooling)

## Problem

The reviewer's `IN-SCOPE BLOCKER` category currently gives equal weight to things that are not equally
important:

- an actual behavioral defect in a script or prompt;
- an incorrect claim about evidence;
- a stale comment or count;
- a file the packet omitted;
- wording that overclaims.

Documentary truth genuinely matters — Codeos is a governance toolkit and false claims in its records
are real defects. But treating all five alike makes review cost track *artifact churn* rather than
*risk*.

**Concrete origin (UPG-0060 `CHG-20260803-001`, 2026-08-03).** Step 3 took six review rounds and
raised eight `IN-SCOPE BLOCKER` findings. Every one was an artifact-governance defect — an overclaim,
a stale header, an unevidenced count, a packet omission. **None was a defect in the tool.** The
implementation satisfied all 20 acceptance criteria from its first run. Read from the dashboard,
"8 in-scope blockers over 6 rounds" reads like a troubled implementation; the truth was a correct
implementation with sloppy paperwork around it. That is a misleading signal to leave in the
institutional record, and an expensive one to produce.

## Upgrade

Split the blocker category so a finding's classification says *what kind* of thing is wrong:

- **BEHAVIORAL BLOCKER** — the artifact, script, or prompt would behave wrongly, unsafely, or contrary
  to a stated guarantee.
- **RECORD BLOCKER** — the artifact's claims about itself are false or unsupported: stale counts,
  overclaims, unevidenced assertions, packet omissions.

Both still block advancement — a false record is not acceptable — but they are counted, logged, and
reported separately, so the dashboard and the review log distinguish "the tool was broken" from "the
paperwork was wrong."

Open design questions for Step 1, deliberately not settled here:

- Whether a round consisting solely of RECORD findings should be resolvable by a lighter path than a
  full re-review (the expensive part is the round trip, not the fix).
- Whether repeated RECORD findings of the same kind should escalate, since a pattern of stale counts
  suggests a missing automated check rather than carelessness.
- Whether some RECORD findings are better prevented by tests than by review — as `CHG-20260803-001`
  did when it turned a documented-process-list overclaim into an automated allowlist scan.

## Scope

**In scope:** `prompts/codeos-reviewer-task.md`'s TRIAGE RULE and finding format; the review-log entry
shape; possibly the parser in `tools/reviewer/` if the categories are machine-read.

**Out of scope:** weakening the requirement that records be true; any change that lets a false claim
advance a gate; the advisory, non-gatekeeping character of review.

## Value

Makes review cost track risk. Keeps the institutional record honest about what actually went wrong,
so a future reader can tell a troubled implementation from a well-implemented change with untidy
bookkeeping.

## Risk

Splitting the category could be read as licence to treat record defects as minor. The framing must be
that both block; only the *reporting* separates. Mitigated by keeping both in the blocking set.

## Related

- **UPG-0047** (Structured Finding Lifecycle) — the finding-id and classification machinery this
  would extend.
- **UPG-0060** — the change that made the cost concrete.
- **AJ-021** — round-by-round review is structurally blind to cumulative disproportion; this is the
  same failure mode seen in review *cost* rather than design scope.
