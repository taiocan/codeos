# Upgrade: stage-4-6-reports — Structured Stage 4–6 Reports

**Priority**: P0
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: stage-report-generator, current-verified-state

## Problem

Stage 4, 5, and 6 are where most hidden work happens. Short end-of-stage summaries do not
show enough detail about what was changed, tested, skipped, assumed, or observed.

## Upgrade

Require structured reports for implementation, test creation, and runtime execution.

## Scope

Stages 4–6 only.

## Proposed artifact(s)

`templates/stage-4-6-report.md`

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
