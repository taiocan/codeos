---
feature_id: UPG-0004
slug: stage-4-6-reports
title: Structured Stage 4–6 Reports
status: SUPERSEDED
priority: P0
depends_on: []
related_features: []
supersedes: []
superseded_by: ["inline Review Package (dba/05-guidance/templates/review-package.md)"]
---

# Upgrade: stage-4-6-reports — Structured Stage 4–6 Reports

**Priority**: P0
**Status**: SUPERSEDED
**Type**: toolkit-upgrade
**Related**: stage-report-generator, current-verified-state

## Outcome (closed 2026-08-19)

Superseded by the inline Review Package. `CHG-20260630-001` built the 127-line template, and commit
`960ba9d` ("Simplify downstream DBA layout and lifecycle ownership", 2026-08-14) deleted it in full.
Nothing references it.

The requirement itself survives, owned inline by the workflow prompts rather than by a persisted
report. `04-implement.md` requires changed paths, Contract satisfaction mapping, event mapping,
failure mapping, and any deferral trace; `05-tests.md` requires changed paths, results, coverage
mapping, and explicitly uncovered runtime behavior; `06-observe.md` requires scenarios executed,
evidence collected, failures, skipped/blocked paths, and environment limits. Both stage prompts
state the decision outright — "This stage creates no separate workflow artifact" and "creates no
separate durable report".

The replacement is `dba/05-guidance/templates/review-package.md`: 19 lines, inline, never a file,
regenerated from current artifacts. That is a deliberate directional choice against this brief's
three-report, ~39-field persisted format, not an oversight.

## Problem

Stage 4, 5, and 6 are where most hidden work happens. Short end-of-stage summaries do not
show enough detail about what was changed, tested, skipped, assumed, or observed.

## Upgrade

Require structured reports for implementation, test creation, and runtime execution.

## Scope

Stages 4–6 only.

## Proposed artifact(s)

`dba/05-guidance/templates/stage-4-6-report.md`

## Design notes

Stage 4 Implementation Report:

```markdown
# Stage 4 Implementation Report

Feature:
Approved artifacts used:
- Intent:
- Contract:
- Event schema:

Files changed:
Files inspected but not changed:
Contract clauses implemented:
Schema events emitted:
Correlation ID propagation:
Runtime artifacts touched:
Unimplemented clauses:
Assumptions:
Blocked items:
Requires earlier-stage change:
Unexpected complexity:
```

Stage 5 Test Report:

```markdown
# Stage 5 Test Report

Feature:
Approved artifacts used:
Behavioral tests added:
Failure-mode tests added:
Invariant tests added:
Telemetry/event tests added:
Replay tests added:
Tests run:
Tests passed:
Tests failed:
Tests skipped:
Tests not run:
Known test gaps:
Why gaps are acceptable or not acceptable:
```

Stage 6 Runtime Evidence Report:

```markdown
# Stage 6 Runtime Evidence Report

Feature:
How the system was run:
Input fixture/scenario:
Runtime command:
Runtime log path:
Events captured:
Unexpected events:
Missing expected events:
Correlation chains observed:
Sanitization status:
Raw logs committed:
- yes/no
- if yes, why safe:
Derived replay fixtures produced:
Ready for reconciliation:
Known runtime gaps:
```

## Value

Very high. This directly addresses the transparency gap. It lets the human and reviewer see
what happened behind Stage 4–6.

## Risk

Reports become verbose boilerplate.

## Guardrail

Each field must be concise. Empty sections must say `none`, `not run`, or `not applicable`.

## DBA-philosophy note

No non-negotiable rule touched. Pure transparency aid for Stages 4–6; strengthens the
evidence chain that reconciliation (Stage 7) consumes.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260630-001 | maintenance/archive/self-development/changes/UPG-0004__CHG-20260630-001__stage-4-6-report-template.md | Create dba/05-guidance/templates/stage-4-6-report.md | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
