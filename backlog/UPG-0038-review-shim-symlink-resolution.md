---
feature_id: UPG-0038
slug: review-shim-symlink-resolution
title: Fix codeos-review.sh Binary Resolution for Symlinked Downstream Projects
status: COMPLETE
priority: P1
depends_on: [UPG-0032]
related_features: [UPG-0037]
supersedes: []
superseded_by: []
---

# Upgrade: review-shim-symlink-resolution — Fix codeos-review.sh Binary Resolution for Symlinked Downstream Projects

**Priority**: P1
**Status**: COMPLETE
**Type**: script-tooling

## Problem

`scripts/codeos-review.sh` resolves `REPO_ROOT` via `git rev-parse --show-toplevel`, then
looks for the reviewer binary at `${REPO_ROOT}/tools/reviewer/target/release/codeos-reviewer`.
When invoked from within a downstream project (e.g. `.codeos/scripts/codeos-review.sh` run
from `/home/rimo/projects/FundFlow`), `git rev-parse --show-toplevel` resolves to the
*downstream project's own* repo root, not through the `.codeos` symlink to Codeos — so the
shim looks for a binary that only exists in Codeos, not in the downstream project, and fails
with "binary not found."

Discovered during UPG-0037's Step 3 verification (2026-07-05): direct invocation of the
compiled binary works correctly against real FundFlow artifacts; the shim wrapper does not.
This is a pre-existing bug tracing to the shim's original design (UPG-0027/UPG-0032), not
introduced by UPG-0037 — filed as an out-of-scope-for-UPG-0037 follow-up.

## Upgrade

Fix `scripts/codeos-review.sh` to resolve the binary relative to the shim script's own
location (which, via the `.codeos` symlink, correctly points at Codeos's `tools/reviewer/`)
rather than the calling repo's git root — e.g. resolve `SCRIPT_DIR` via
`$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)` (following symlinks) and derive the binary
path from that, falling back to the current `git rev-parse` behavior only for Codeos's own
self-development use (where the shim's location and the repo root coincide).

## Scope

`scripts/codeos-review.sh` only. No change to the Rust binary itself.

## Value

High for downstream usability — without this fix, UPG-0037's entire default-review practice
is unusable via the documented shim invocation from any real downstream project; only direct
binary invocation works today.

## Risk

Must not break Codeos's own self-development usage of the same shim (where repo-root and
shim-location currently coincide) — needs a compatibility check against both usage modes.

## Guardrail

No change to binary discovery precedence beyond fixing path resolution — `--provider` CLI
flag and `CODEOS_REVIEWER_PROVIDER` env var precedence remain unchanged.

## DBA-philosophy note

Not applicable — pure tooling fix, no doctrine change.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260707-002 | `changes/UPG-0038__CHG-20260707-002__review-shim-symlink-resolution.md` | Fix `scripts/codeos-review.sh` binary resolution for symlinked downstream projects | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0038__CHG-20260707-002__S1__R1 | CHG-20260707-002 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0038__CHG-20260707-002__S2__R1 | CHG-20260707-002 | 2-Acceptance | R1 | NO OBJECTION |
| REV__UPG-0038__CHG-20260707-002__S3__R1 | CHG-20260707-002 | 3-Implement | R1 | DO NOT ADVANCE (dropped git-repo precondition) |
| REV__UPG-0038__CHG-20260707-002__S3__R2 | CHG-20260707-002 | 3-Implement | R2 | NO OBJECTION |
| REV__UPG-0038__CHG-20260707-002__S4__R1 | CHG-20260707-002 | 4-Reconcile | R1 | DO NOT ADVANCE (stale §10 architecture claims) |
| REV__UPG-0038__CHG-20260707-002__S4__R2 | CHG-20260707-002 | 4-Reconcile | R2 | DO NOT ADVANCE (stale scope-check sentence) |
| REV__UPG-0038__CHG-20260707-002__S4__R3 | CHG-20260707-002 | 4-Reconcile | R3 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
