---
feature_id: UPG-0020
slug: stack-drift-detector
title: Stack / Config Drift Detector
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stack-drift-detector — Stack / Config Drift Detector

**Priority**: P3
**Status**: IN_PROGRESS
**Type**: toolkit-upgrade
**Related**: stack-manifest, readiness-checklist, ci-profile

## Problem

Dependencies or config can change without manifest updates.

## Upgrade

Add a simple detector.

## Scope

Dependency/config drift detection at readiness/release.

## Proposed artifact(s)

A detector rule integrated with the readiness checklist.

## Design notes

Rule:

```text
If dependency/config files changed and readiness checklist does not include stack
reconciliation, block release.
```

## Value

Medium. Useful once stack manifest exists.

## Risk

False positives blocking release on benign config changes.

## Guardrail

Depends on `stack-manifest` existing; keep the watched-file set explicit.

## DBA-philosophy note

No behavioral rule touched. Operational release gate; depends on `stack-manifest`. Trigger-based,
not memory-based.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260703-002 | `changes/UPG-0020__CHG-20260703-002__stack-drift-detector.md` | New Rust subcommand: `check-drift` on the reviewer binary | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0020__CHG-20260703-002__S3__R1 | CHG-20260703-002 | 3-Implement | R1 | CHANGES ADVISED |
| REV__UPG-0020__CHG-20260703-002__S3__R2 | CHG-20260703-002 | 3-Implement | R2 | CHANGES ADVISED |
| REV__UPG-0020__CHG-20260703-002__S3__R3 | CHG-20260703-002 | 3-Implement | R3 | CHANGES ADVISED (AC-10 REJECTED) |
| REV__UPG-0020__CHG-20260703-002__S4__R1 | CHG-20260703-002 | 4-Reconcile | R1 | CHANGES ADVISED (AC-10 REJECTED) |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
