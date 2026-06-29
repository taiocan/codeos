---
feature_id: UPG-0029
slug: review-naming-and-thread-tooling
title: Review artifact durability + packet naming policy + feature-thread tooling
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: review-naming-and-thread-tooling — Review artifact durability + packet naming policy

> **Recommended next pickup** (a narrow, doc-first policy pass — may need no script change). Issue
> #1 below is the primary scope; naming/tooling (#2–#5) stay optional until the manual convention
> proves insufficient.

**Priority**: P2
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: UPG-0001 (feature-thread-traceability), reviewer pipeline

> Filed by `CHG-20260627-001` (UPG-0001) as the explicit OUT-OF-SCOPE-BACKLOG follow-up for the
> review-file naming + tooling that UPG-0001 deliberately deferred. UPG-0001 establishes the
> `REV__UPG-####__CHG-…__S<N>__R<N>` id as a **documented manual convention** only; it does not
> rename review files and does not change `scripts/codeos-review.sh` behavior.

## Problem

After UPG-0001, the Feature Thread model and IDs are a documented convention, but gaps remain:

* **Audit-trail durability (primary).** `reviews/review-log.md` references full Codex assessments
  under `reviews/codex/*` by **path + sha**, but most of those files are **untracked** — 27 of 28
  referenced assessments are not committed (one pre-UPG-0029 assessment is already committed; all
  packet files are untracked). A path+sha pointer to an uncommitted file is not durable for another
  checkout or reviewer. There is no policy for which review artifacts are committed vs scratch vs
  summarized.
* the advisory reviewer (`scripts/codeos-review.sh`) still emits assessment/packet filenames in
  the legacy `${ts}-${feature}-stage-${stage}-${sha}` shape, not the
  `REV__UPG-####__CHG-…__S<N>__R<N>` review-id shape;
* existing review files under `reviews/codex/` are not renamed to the new convention;
* there is no automated check that every active backlog brief carries a `feature_id` + a
  `## Feature Thread` section, that `UPG-####` / `CHG-*` ids are unique, and that
  `backlog/features.md` maps each `UPG-####` to exactly one file.

These were intentionally left manual until the convention proves itself (per UPG-0001's
non-goals: "do not add mandatory tooling unless the manual convention proves insufficient").

## Upgrade

**1. Review-artifact durability policy — the primary, first concrete issue (doc/policy only).**
Decide which review artifacts are **committed**, **summarized**, or **kept local-only**, and align
`reviews/review-log.md` references to that decision. Rule:

> Commit durable review evidence only when it is referenced as part of the official audit trail.
> Scratch reviews remain untracked. If a `review-log.md` entry references a full assessment by
> path+sha, that assessment must **either** be committed **or** the log entry must clearly mark it
> as local-only / non-durable.

This avoids both extremes — a bloated repo and a fake audit trail — and closes the concrete gap
inherited from UPG-0001 (path+sha references to untracked `reviews/codex/*` files). This issue may
need **no script change** at all; scope it narrowly.

Then, only **if/when** the manual convention proves insufficient (optional, later):

2. teach `scripts/codeos-review.sh` to derive and emit the `REV__UPG-####__CHG-…__S<N>__R<N>`
   review id (filename + in-packet/in-log id), keeping it advisory and read-only;
3. provide a migration to rename historical `reviews/codex/*` files where practical (truthful,
   non-destructive);
4. add `scripts/check_feature_threads.sh` — a read-only checker for the UPG-0001 acceptance
   invariants (unique ids, every active brief has `feature_id` + `## Feature Thread`,
   `features.md` one-to-one map, no plain `000N` used as both feature and change id);
5. update the two comment-only references to `backlog/reviewer-decision-integrity.md` inside
   `scripts/codeos-review.sh` to the renamed path (UPG-0001 left these untouched to keep the
   script byte-identical / behavior-frozen).

## Scope

`self-dev only`. Must not change the reviewer's advisory/read-only/non-gatekeeping guarantees,
and must not become mandatory tooling before the manual convention is proven.

## Value

Removes manual bookkeeping for the thread model once it is established; keeps review filenames
self-describing.

## Guardrail

Advisory-only; no enforcement. Tooling stays optional until the manual convention is shown
insufficient.

---

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260629-001 | changes/UPG-0029__CHG-20260629-001__review-durability.md | Review artifact durability policy (doc-only; issue #1) | IN_PROGRESS |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
