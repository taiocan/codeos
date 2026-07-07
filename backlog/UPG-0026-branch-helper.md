---
feature_id: UPG-0026
slug: branch-helper
title: Optional Branch Creation Helper
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: branch-helper — Optional Branch Creation Helper

**Priority**: P2
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: workflow-profiles, stage-4-activation-card, feature-registry

## Problem

Branch naming and timing can be inconsistent.

## Upgrade

Add a small helper convention or script.

## Scope

Tooling / convention.

## Proposed artifact(s)

Proposed command behavior:

```text
codeos branch <feature_id>
```

Creates `feature/<feature_id>` or, for split mode:

```text
feature/<feature_id>-artifacts
feature/<feature_id>-implementation
feature/<feature_id>-runtime-replay
feature/<feature_id>-refinement
```

## Design notes

Alternative: no script. Just document branch convention.

## Value

Medium. Useful if you move toward PR workflows.

## Risk

Unnecessary tooling.

## Guardrail

Start with documentation only.

## DBA-philosophy note

No rule touched. Pure convenience tooling; doc-first to avoid premature automation.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260707-001 | `changes/UPG-0026__CHG-20260707-001__branch-helper.md` | Document Profile C's split-mode branch-naming convention in `docs/workflow-profiles.md` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0026__CHG-20260707-001__S1__R1 | CHG-20260707-001 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0026__CHG-20260707-001__S2__R1 | CHG-20260707-001 | 2-Acceptance | R1 | DO NOT ADVANCE (AC-5 self-contradiction) |
| REV__UPG-0026__CHG-20260707-001__S2__R2 | CHG-20260707-001 | 2-Acceptance | R2 | NO OBJECTION |
| REV__UPG-0026__CHG-20260707-001__S3__R1 | CHG-20260707-001 | 3-Implement | R1 | NO OBJECTION |
| REV__UPG-0026__CHG-20260707-001__S4__R1 | CHG-20260707-001 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
