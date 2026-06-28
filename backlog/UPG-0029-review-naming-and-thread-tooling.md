---
feature_id: UPG-0029
slug: review-naming-and-thread-tooling
title: Review-file REV__ naming + codeos-review.sh output support + feature-thread checks
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: review-naming-and-thread-tooling — REV__ review naming + thread tooling

**Priority**: P2
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: UPG-0001 (feature-thread-traceability), reviewer pipeline

> Filed by `CHG-20260627-001` (UPG-0001) as the explicit OUT-OF-SCOPE-BACKLOG follow-up for the
> review-file naming + tooling that UPG-0001 deliberately deferred. UPG-0001 establishes the
> `REV__UPG-####__CHG-…__S<N>__R<N>` id as a **documented manual convention** only; it does not
> rename review files and does not change `scripts/codeos-review.sh` behavior.

## Problem

After UPG-0001, the Feature Thread model and IDs are a documented convention, but:

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

When the manual convention has proven useful, optionally:

1. teach `scripts/codeos-review.sh` to derive and emit the `REV__UPG-####__CHG-…__S<N>__R<N>`
   review id (filename + in-packet/in-log id), keeping it advisory and read-only;
2. provide a migration to rename historical `reviews/codex/*` files where practical (truthful,
   non-destructive);
3. add `scripts/check_feature_threads.sh` — a read-only checker for the UPG-0001 acceptance
   invariants (unique ids, every active brief has `feature_id` + `## Feature Thread`,
   `features.md` one-to-one map, no plain `000N` used as both feature and change id);
4. update the two comment-only references to `backlog/reviewer-decision-integrity.md` inside
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

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
