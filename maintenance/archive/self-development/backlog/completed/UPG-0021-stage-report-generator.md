---
feature_id: UPG-0021
slug: stage-report-generator
title: Stage Report Generator
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stage-report-generator — Stage Report Generator

**Priority**: P3
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: stage-4-6-reports

## Problem

Stage reports can become manual burden.

## Upgrade

Generate reports from git diff, test output, and runtime files where possible.

## Scope

Stage 4–6 report generation.

## Proposed artifact(s)

Generated Stage 4–6 report skeleton, filled automatically where possible.

## Design notes

Inputs:

```text
git diff
git status
test output
runtime_events.jsonl
approved artifact paths
```

Output: Stage 4–6 report skeleton, filled automatically where possible.

## Value

Medium. Useful after report templates prove valuable.

## Risk

Automation hides errors.

## Guardrail

Generated report must say what was inferred vs human/model-written.

## DBA-philosophy note

No rule touched, but automation must **label inferred vs authored** content so generated
evidence is never mistaken for verified human/model judgment.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260703-003 | `changes/UPG-0021__CHG-20260703-003__stage-report-generator.md` | New Rust subcommand: `generate-report` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0021__CHG-20260703-003__S2__R1 | CHG-20260703-003 | 2-Acceptance | R1 | CHANGES ADVISED |
| REV__UPG-0021__CHG-20260703-003__S2__R2 | CHG-20260703-003 | 2-Acceptance | R2 | CHANGES ADVISED |
| REV__UPG-0021__CHG-20260703-003__S2__R3 | CHG-20260703-003 | 2-Acceptance | R3 | CHANGES ADVISED |
| REV__UPG-0021__CHG-20260703-003__S2__R4 | CHG-20260703-003 | 2-Acceptance | R4 | CHANGES ADVISED |
| REV__UPG-0021__CHG-20260703-003__S2__R5 | CHG-20260703-003 | 2-Acceptance | R5 | NO OBJECTION |
| REV__UPG-0021__CHG-20260703-003__S3__R1 | CHG-20260703-003 | 3-Implement | R1 | DO NOT ADVANCE |
| REV__UPG-0021__CHG-20260703-003__S3__R2 | CHG-20260703-003 | 3-Implement | R2 | NO OBJECTION |
| REV__UPG-0021__CHG-20260703-003__S4__R1 | CHG-20260703-003 | 4-Reconcile | R1 | DO NOT ADVANCE |
| REV__UPG-0021__CHG-20260703-003__S4__R2 | CHG-20260703-003 | 4-Reconcile | R2 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
