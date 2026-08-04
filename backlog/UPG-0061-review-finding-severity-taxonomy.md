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

## Problem — reframed 2026-08-04, after UPG-0060/0062/0063

> **A known procedural rule can exist in the journal and still fail repeatedly, because nothing
> reliably surfaces or enforces that rule at the exact workflow transition where it must be applied.**

That is the primary problem. The taxonomy below is a secondary, still-useful improvement, but it is
**not** the main gap and should not be mistaken for the fix.

**Why the reframing.** This feature was originally filed because review effort tracked artifact churn
rather than risk. Three subsequent features supplied better evidence — and it pointed somewhere else:

- **AJ-020 already contains the right knowledge.** It says, specifically, that the dashboard row must
  be updated *before* the step review runs. It was written after the same failure occurred twice.
- **It then failed three more times in UPG-0063 alone**, plus repeatedly in UPG-0062. The rule was
  correct, specific, and about exactly the transition being got wrong — and it still did not reach the
  point of action.
- **AJ-016 behaved identically**: evidence must be embedded, not summarised. Journaled, then violated
  again in UPG-0063 Step 3.

So the failure is not that the knowledge is missing, badly categorised, or wrongly prioritised. It is
that a retrospective journal is the wrong delivery mechanism for a rule whose whole content is *"do X
at moment Y."* Writing it down more clearly, or classifying its violations more precisely, does not
address that.

**One distinction the evidence forced, and it matters for scope.** Not every recurrence is the same
kind of failure. UPG-0063 Step 4 produced one that looks superficially like the others but is not:
the change record asserted `state: COMPLETE` / `review_state: ACCEPTED` before the review had run and
before the human had accepted. That is not stale bookkeeping — it is **a false governance-state
assertion**, an artifact claiming the human gate had occurred when it had not. Catching it before
acceptance preserved the gate's meaning. Any taxonomy this feature produces must keep that separable
from record drift, because the two warrant completely different responses.

## What this points toward

**Point-of-action safeguards, not more retrospective documentation.** Something that surfaces the
relevant rule at the transition where it applies — a step-entry checklist, a precondition the tooling
already knows how to check, a prompt-level reminder at the moment the row is written.

**And a hard proportionality constraint.** Codeos must not become a rigid workflow engine. AJ-021 is
the standing precedent: a request for an on/off switch grew into a versioned governance framework
across seven review rounds before a human reset it. The remedy here should be small enough that its
absence would be the only thing anyone noticed.

Two candidate shapes, neither settled:

- The self-dev step prompt names the two or three rules that actually recur, at the point of each step
  transition — cheapest possible, no machinery.
- The reviewer's existing precheck asserts what it can already see (dashboard loop step matches the
  change record's `current_step`) — mechanical, but only for rules that are mechanically checkable.

**Deliberately not proposed:** enforcing AJ-016/AJ-020 as gates, a rules engine, or a general
"journal compliance" mechanism.

## Secondary problem — the original filing



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
