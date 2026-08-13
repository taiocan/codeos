---
feature_id: UPG-0048
slug: review-ledger-event-sourcing
title: Review Ledger Event Sourcing (Speculative — Long-Term)
status: PROPOSED
priority: P3
depends_on: [UPG-0046, UPG-0047]
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: review-ledger-event-sourcing — Review Ledger Event Sourcing (Speculative — Long-Term)

**Priority**: P3
**Status**: PROPOSED
**Type**: script-tooling

## Problem

Today `reviews/review-log.md` is simultaneously an append-only durable record *and* the
human-readable presentation of that record. `maintenance/archive/self-development/status/self-development.md` is a similarly
hand-maintained projection of the same underlying facts (which change is at which step, which
review series accepted what). Keeping these in sync is currently a manual discipline enforced by
`maintenance/archive/self-development/retired-process/codeos-self-dev.md`'s step instructions ("Activate the row," "mark COMPLETE") — reliable
so far, but it is manual work that scales linearly with review volume, and any file could in
principle drift from the others since nothing mechanically ties them together.

## Upgrade

**This is explicitly the most speculative and highest-risk item in this batch** — flagged as
such by design, not a near-term commitment. Not decided by this brief — questions for
implementer to resolve, more open-ended than the other items in this batch:

### 1. Whether to do this at all

The human's own proposal (see Related) frames this as "Option 3 — Full Review OS... probably too
much for now," preferring the lighter `UPG-0046`/`UPG-0047` combination ("Option 2 — Review
Control Plane Lite"). This brief exists so the idea is captured and not lost, not because it is
recommended for near-term implementation. A future re-triage of this item, informed by how much
friction remains *after* `UPG-0046`/`UPG-0047` ship, should re-decide whether this is still worth
doing — it may turn out the lighter upgrades already remove most of the pain this one targets.

### 2. Event log shape and source-of-truth boundary

An illustrative sketch (**not an approved schema**) — an append-only `reviews/review-events.jsonl`
recording atomic events (`review_planned`, `packet_built`, `review_completed`,
`finding_recorded`, `human_decision`), with `reviews/review-log.md` and
`maintenance/archive/self-development/status/self-development.md` becoming *generated projections* rather than hand-maintained files.

```json
{"type":"review_completed","review_id":"REV__...","concern":"NO_OBJECTION","evidence":"B"}
{"type":"human_decision","review_id":"REV__...","decision":"APPROVED","rationale":"..."}
```

If pursued, this needs an explicit answer to: what happens to the *existing* several thousand
lines of `reviews/review-log.md` history (backfill into events retroactively? leave pre-migration
history as-is and start the event log fresh from a cutover point? — the latter is far cheaper and
matches how `UPG-0029`'s durability policy already handles "pre-policy entries were not committed,
classified retroactively" as a precedent for non-destructive cutover).

### 3. Generation vs. append-only tension

`maintenance/archive/self-development/status/self-development.md`'s own header already states "Mutable — maintained by the 4-step
self-development loop" — i.e., it is *expected* to be hand-edited today. Moving to
generated-projection would change that expectation. Whether a generated dashboard can still
support the kind of manual annotation the current file sometimes carries (see rows with
historical/legacy notes) needs a concrete answer before this is more than a sketch.

## Scope

Deliberately unscoped beyond "explore whether `reviews/review-log.md` and
`maintenance/archive/self-development/status/self-development.md` should become generated projections of an append-only event log."
Any real implementation of this item should itself go through Step 1 Intent with a much narrower,
concretely-bounded scope than this brief states — this brief is the parking lot for the idea, not
an implementation plan.

Out of scope for this brief (and likely for whatever narrower change eventually implements part
of it):
- Migrating historical `reviews/review-log.md` content into the new event format as a hard
  requirement — see "cutover, not backfill" note above.
- Any change to the human-gated approval model — events record decisions, they do not make them.
- Doing this before `UPG-0046` and `UPG-0047` ship and prove out (or fail to prove out) the
  smaller structured-record approach first.

## Value

Very high, long-term, *if* review volume grows enough that manual dashboard maintenance becomes
the bottleneck. At current volume (per `maintenance/archive/self-development/status/self-development.md`'s row count), the manual
discipline has held up reliably — this is insurance against future scale, not a fix for a
present, felt problem.

Trade-offs: this is the highest-effort, highest-risk item in the batch. Real risk of turning
Codeos self-development tooling into "a workflow database" (the human's own words) — a
meta-system distraction from the toolkit's actual purpose. Should not be started opportunistically;
should be re-justified against real friction, not built because the architecture diagram looks
cleaner.

## Risk

Deciding hastily risks:
- Building a general-purpose event-sourcing system as a solo side-project inside a toolkit repo,
  disproportionate to the actual review volume it serves.
- Breaking the append-only-and-human-readable guarantee that makes `reviews/review-log.md`
  trustworthy today, in exchange for a generation pipeline that itself becomes a new source of
  bugs (a generator that mis-renders a projection is a worse failure mode than a human typo in a
  hand-written log, because it looks authoritative).
- Scope creep into "let's also generate `backlog/features.md`" and similar — each additional
  generated surface is a new place a generation bug can misrepresent reality to a human at a gate.

## Guardrail

If ever implemented, this upgrade must:
- Preserve every append-only and durability guarantee in `dba/06-reference/reviewer-pipeline.md` §4/§4a — the
  event log itself must be at least as durable and human-auditable as today's
  `reviews/review-log.md`.
- Never make the event log (or its generated projections) authoritative over the human's decision
  — same rule as `UPG-0046`/`UPG-0047`, restated because this item raises the stakes the most.
- Not be started before `UPG-0046` and `UPG-0047` (its likely prerequisites) have shipped and been
  used for long enough to show whether the lighter approach already solves the problem.
- Be re-scoped into a concrete Step 1 Intent, much narrower than this brief, before any
  implementation begins — this brief is intentionally a parking-lot sketch, not a plan.

## Related

- **UPG-0046**: ReviewRun Structured Records — likely prerequisite; an event log without
  structured per-round records first has nothing well-formed to emit events about.
- **UPG-0047**: Structured Finding Lifecycle — likely prerequisite for the same reason.
- Proposed by the human during a 2026-07-12 review-architecture discussion as "Option 3 — Full
  Review OS," explicitly flagged by the human themselves as "probably too much for now" in favor
  of the lighter "Option 2 — Review Control Plane Lite" (`UPG-0046`/`UPG-0047`). See
  `reviews/review-log.md` and
  `maintenance/archive/self-development/changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md`.

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
