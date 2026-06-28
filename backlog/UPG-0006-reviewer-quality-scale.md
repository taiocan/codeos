---
feature_id: UPG-0006
slug: reviewer-quality-scale
title: Reviewer Summary Quality Scale
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-quality-scale — Reviewer Summary Quality Scale

**Priority**: P2
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-full-diff

## Problem

Reviewer outputs can be fluent but not useful.

## Upgrade

Require reviewer to label evidence quality.

## Scope

Reviewer agent output.

## Proposed artifact(s)

Evidence-quality scale embedded in reviewer output.

## Design notes

Proposed scale:

```text
A — Direct evidence from artifact/diff/test/runtime log
B — Strong inference from code and tests
C — Plausible but not directly proven
D — Speculative
E — Unknown / not reviewed
```

Every recommendation should include:

```markdown
Recommendation:
Evidence quality:
Most important uncertainty:
What human should inspect if time is limited:
```

> In the automated pipeline this is the optional `EVIDENCE: <A–E>` line. It is only "implemented"
> when the reviewer actually emits it; otherwise the log records `Evidence: not reported`.

## Value

High. Prevents reviewer from sounding more certain than the evidence supports.

## Risk

Reviewer ignores the scale or grades inconsistently.

## Guardrail

Concern level (what the reviewer thinks) and evidence quality (how well supported) are separate
axes — keep both.

## DBA-philosophy note

No rule touched. Improves reviewer honesty (separates conviction from evidence). Advisory only.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

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
