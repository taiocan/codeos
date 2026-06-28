---
feature_id: UPG-0028
slug: reviewer-self-reference-recursion
title: reviewer self-reference recursion (scoping improvement)
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Backlog: reviewer self-reference recursion (scoping improvement)

**Status:** open (filed by change `0004-review-fixes`, 2026-06-27)
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
