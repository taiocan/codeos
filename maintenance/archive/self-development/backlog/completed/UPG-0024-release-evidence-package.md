---
feature_id: UPG-0024
slug: release-evidence-package
title: Pre-Release Evidence Package
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: release-evidence-package — Pre-Release Evidence Package

**Priority**: P3
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: readiness-checklist, reviewer-decision-brief, approval-dashboard

## Problem

Before release, artifacts may be scattered across stage files, reports, runtime logs, replay
results, and reviewer summaries.

## Upgrade

Create a release evidence package.

## Scope

Pre-release aggregation.

## Proposed artifact(s)

`reviews/release-evidence-[feature_id].md`

## Design notes

```markdown
Feature:
Branch/PR:
Approved artifacts:
Stage reports:
Reviewer briefs:
Reconciliation result:
Replay result:
Verification-only report:
Readiness checklist:
Known limitations:
Release decision:
```

## Value

Medium-high for serious releases.

## Risk

Duplicative with existing reviews.

## Guardrail

Generate from existing artifacts, do not manually rewrite.

## DBA-philosophy note

No rule touched. **Generated** from existing artifacts (not a new authored surface) — avoids
creating a second source of truth.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260706-002 | `changes/UPG-0024__CHG-20260706-002__release-evidence-package.md` | New Rust subcommand: `generate-release-evidence` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0024__CHG-20260706-002__S1__R1 | CHG-20260706-002 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0024__CHG-20260706-002__S2__R1 | CHG-20260706-002 | 2-Acceptance | R1 | DO NOT ADVANCE (AC-3/AC-6 contradiction; AC-12 too broad) |
| REV__UPG-0024__CHG-20260706-002__S2__R2 | CHG-20260706-002 | 2-Acceptance | R2 | NO OBJECTION |
| REV__UPG-0024__CHG-20260706-002__S3__R1 | CHG-20260706-002 | 3-Implement | R1 | NO OBJECTION |
| REV__UPG-0024__CHG-20260706-002__S4__R1 | CHG-20260706-002 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
