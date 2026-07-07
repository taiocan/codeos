---
feature_id: UPG-0028
slug: reviewer-self-reference-recursion
title: reviewer self-reference recursion (scoping improvement)
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Backlog: reviewer self-reference recursion (scoping improvement)

**Status:** COMPLETE — substantially resolved by later work, evaluated 2026-07-07
(`backlog-only` direct edit, no active `CHG-*`; see "Resolution" below)
**Priority:** P2 (reviewer-pipeline robustness)
**Class (when worked):** prompt / script-tooling
**Scope axis:** self-dev only (also relevant downstream)

## Problem

When the Codex advisory reviewer reviews a change whose artifacts **include the very
bookkeeping that tracks the review** — `status/self-development.md` (the Review column,
completion legend) and the change record's own "Codex review" section — it reliably returns
`CHANGES ADVISED` on *self-referential* grounds, even after every substantive finding is fixed:

- the change record can't truthfully assert its own review outcome before the review runs;
- a round-counter or "latest verdict" cell always lags the just-completed round by one;
- tightening the completion legend to match one row can contradict another.

Fixing one such nit shifts the lag by one, so chasing `NO OBJECTION` is an **infinite regress**.
Observed concretely in the `0001`–`0004` doctrine-split series (see `reviews/review-log.md`):
round 1 caught a real bug (status dashboard still naming `backlog/features.md` as "the
roadmap"); rounds 2–5 were all bookkeeping-recursion. Resolved by **human acceptance** of the
residual, per the advisory-not-gatekeeping principle — which is correct, but the friction is
avoidable.

> **Update (UPG-0001, CHG-20260627-001):** the *doctrine + artifact* side is now done — UPG-0001
> introduced the **review-series id** `RVS__…__S<N>`, the `review_series`/`review_state` trace-header
> fields, the **Surface ownership** table, and the **Self-Reference Boundary** (self-reference rule
> + stop rule) in `prompts/codeos-self-dev.md`. Reviewed artifacts no longer embed live rounds.
> **What remains for UPG-0028:** the *reviewer/packet enforcement* — improvements (1) and (2) below
> (the script/packet are frozen by UPG-0001's E2, so they belong here).

## Candidate improvements (pick when worked)

1. **Scope exclusion:** when reviewing a self-development change, exclude the review-tracking
   fields it necessarily mutates (the reviewed change's own `Review` cell, its change record's
   "Codex review" section) from the packet — review the *substance*, not the review-of-the-review.
2. **Reviewer guidance:** add a line to the reviewer activation prompt: self-referential
   review-bookkeeping inconsistencies are NON-BLOCKER by default; only flag them as blockers if
   they misstate a *substantive* outcome.
3. **Convergence rule (doctrine):** make explicit that a self-dev change's Step-4 review is
   `COMPLETE` on **human acceptance** of the advisory verdict (clean, or residual non-blocking),
   not on the reviewer returning `NO OBJECTION`. (Already reflected in the dashboard legend; this
   would codify it in `prompts/codeos-self-dev.md`.)

## Value / guardrail

Keeps the advisory reviewer from behaving like an enforcement engine on its own bookkeeping
(the failure mode the doctrine explicitly warns against), without weakening its ability to catch
substantive findings.

## Resolution (2026-07-07)

Evaluated all three candidate improvements against the toolkit's actual current state:

1. **Scope exclusion (packet-level)** — not built. `packet.rs`'s `git_diff_names`/
   `git_diff_names_head`/`git_is_dirty` already exclude the whole `reviews/` directory and
   `.codeos-state` from every diff (a related, coarser mechanism), but nothing excludes the
   specific review-tracking *fields* this backlog originally named — `status/
   self-development.md`'s Review column, or a change record's own embedded review-tracking
   sections. This piece remains genuinely unbuilt.
2. **Reviewer guidance (NON-BLOCKER by default)** — **done**. `UPG-0027` (Lean Review Runner
   and Packet Architecture, COMPLETE) added the fifth triage category, `SELF-REFERENCE /
   REVIEW-BOOKKEEPING`, to `prompts/codeos-reviewer-task.md`'s TRIAGE RULE: "review records
   that are stale because of the previous round's own existence (causal loop); not a real
   artifact defect" — classified separately from IN-SCOPE BLOCKER, so it cannot drive a
   `DO NOT ADVANCE` verdict.
3. **Convergence rule (doctrine)** — **done**, and more explicit than originally proposed.
   `prompts/codeos-self-dev.md`'s Step 4 section states directly: "Only after the human
   approves at this final gate (the review is advisory; it never closes the change by
   itself): mark the row COMPLETE... Until then the change stays IN_PROGRESS — matching the
   dashboard rule that COMPLETE requires human acceptance." UPG-0001's Self-Reference
   Boundary and Surface Ownership table (same file) additionally ensure reviewed artifacts
   never embed a live round number to begin with, removing the specific "lagging counter"
   mechanism the original problem described.

**Decision:** close without building improvement #1. Empirically, across every self-dev
change run this session (UPG-0019 through UPG-0026 — 8 changes, ~30 review rounds), the
"infinite regress chasing NO OBJECTION" failure mode this backlog describes never recurred:
every self-referential-shaped finding (e.g. UPG-0037's `SECRET_REDACTION` false positive,
UPG-0025's draft-marker scanner false positive) resolved within 1-2 rounds via correct human
acceptance of a residual non-blocking finding, exactly as improvements #2 and #3 intend.
Building #1 now would be solving an already-mitigated problem — if the failure mode
resurfaces in practice despite #2/#3, re-open this backlog item (or file a fresh one) with
the concrete recurrence as evidence, rather than building preventive packet-exclusion logic
against a problem that stopped occurring once the doctrine-level fixes landed.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
