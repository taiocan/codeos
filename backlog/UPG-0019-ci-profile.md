---
feature_id: UPG-0019
slug: ci-profile
title: CI Integration Profile
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: ci-profile — CI Integration Profile

**Priority**: P3
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: workflow-profiles, stack-drift-detector

## Problem

Local evidence may not match CI.

## Upgrade

Define how Codeos evidence maps to CI checks.

## Scope

CI integration (optional).

## Proposed artifact(s)

Proposed CI checks:

```text
behavioral tests
replay tests
schema conformance tests
lint/typecheck
no unapproved event names
no raw runtime log leakage
stack manifest reconciliation if dependency/config changed
```

## Design notes

Maps existing Codeos evidence types (behavioral tests, replay, schema conformance) onto CI
gates so local and CI evidence converge.

## Value

Medium-high if you use CI heavily.

## Risk

CI maintenance overhead.

## Guardrail

Start with minimal checks.

## DBA-philosophy note

No rule touched. CI enforces existing evidence types (schema conformance, no unapproved
events) — reinforces, rather than alters, the behavioral chain.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260706-001 | `changes/UPG-0019__CHG-20260706-001__ci-profile.md` | New doc: `docs/ci-integration-profile.md` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0019__CHG-20260706-001__S1__R1 | CHG-20260706-001 | 1-Intent | R1 | NO OBJECTION |
| REV__UPG-0019__CHG-20260706-001__S2__R1 | CHG-20260706-001 | 2-Acceptance | R1 | NO OBJECTION |
| REV__UPG-0019__CHG-20260706-001__S3__R1 | CHG-20260706-001 | 3-Implement | R1 | NO OBJECTION (evidence B — check_drift.rs not shown) |
| REV__UPG-0019__CHG-20260706-001__S3__R2 | CHG-20260706-001 | 3-Implement | R2 | DO NOT ADVANCE (missing EXIT_CONFIG exit path) |
| REV__UPG-0019__CHG-20260706-001__S3__R3 | CHG-20260706-001 | 3-Implement | R3 | NO OBJECTION |
| REV__UPG-0019__CHG-20260706-001__S4__R1 | CHG-20260706-001 | 4-Reconcile | R1 | DO NOT ADVANCE (numeric exit codes not shown) |
| REV__UPG-0019__CHG-20260706-001__S4__R2 | CHG-20260706-001 | 4-Reconcile | R2 | DO NOT ADVANCE (evidence attributed to wrong file) |
| REV__UPG-0019__CHG-20260706-001__S4__R3 | CHG-20260706-001 | 4-Reconcile | R3 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
