---
feature_id: UPG-0014
slug: reviewer-full-diff
title: Reviewer Agent with Full Diff Access
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-full-diff — Reviewer Agent with Full Diff Access

**Priority**: P2
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-quality-scale, workflow-profiles

## Problem

Reviewer summaries are much better if the reviewer sees the actual diff, not only stage
artifacts.

## Upgrade

Allow reviewer agent to inspect: approved artifacts; generated stage artifact; full git diff;
Stage 4–6 reports; tests; runtime evidence; CI output.

## Scope

Reviewer agent capability.

## Proposed artifact(s)

Reviewer modes: `Artifact Review`, `Diff Review`, `Test Review`, `Runtime Evidence Review`,
`Pre-Release Review`.

## Design notes

Diff review checklist:

```markdown
# Diff Review

Changed files:
Unrelated files:
Approved artifacts modified:
Implementation-only files:
Test-only files:
Runtime fixture files:
Docs/config files:

DBA traceability:
- every implementation change traces to contract/schema:
- every event traces to schema:
- every test traces to contract/failure path:
- no hidden behavior:

Risk:
- security:
- privacy:
- architecture:
- dependency:
- config:
- release:

Recommendation:
```

> The automated pipeline (`docs/reviewer-pipeline.md`) feeds the diff into the reviewer's
> evidence packet **after secret/size filtering** (see `verify-only-mode` and the pipeline's
> B4 safety layer).

## Value

Very high. This should be one of the first upgrades.

## Risk

Reviewer becomes too verbose.

## Guardrail

Reviewer must output short decision brief first, details second.

## DBA-philosophy note

Extends the **advisory** reviewer (read-only). Diff access strengthens traceability checks but
must respect secret filtering and stay non-gatekeeping.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260702-004 | `changes/UPG-0014__CHG-20260702-004__reviewer-full-diff.md` | Auto-include Full Context Diff in packet when `--mode delta --base` active (no new flag; Rust-only; bash shim boundary documented) | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
