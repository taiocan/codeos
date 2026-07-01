---
feature_id: UPG-0006
slug: reviewer-quality-scale
title: Reviewer Summary Quality Scale
status: COMPLETE
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-quality-scale — Reviewer Summary Quality Scale

**Priority**: P2
**Status**: COMPLETE
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

Approved scale (evidence basis, not confidence):

```text
A — Directly verified in the artifact, diff, or output shown in the packet
B — Verified with multiple direct pieces of evidence, but coverage is not complete
C — Partially verified, partially inferred from structure or context
D — Mostly inferred from structure or indirect evidence
E — Hypothesis or very limited basis — little to no direct evidence
```

Mandatory output fields (last three lines of reviewer output):

```
LOG SUMMARY: <verdict> — <single most important point>
EVIDENCE: <A|B|C|D|E>
HIGHEST-IMPACT UNCERTAINTY: <one sentence — what single thing, if wrong, most affects this assessment>
```

> `EVIDENCE:` is **mandatory** in CHG-20260701-008. `HIGHEST-IMPACT UNCERTAINTY:` is emitted
> but not machine-parsed in this change (parser scope is deferred).

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
| CHG-20260701-008 | `changes/UPG-0006__CHG-20260701-008__reviewer-quality-scale.md` | Make EVIDENCE mandatory, redefine A–E as evidence-basis semantics, add HIGHEST-IMPACT UNCERTAINTY field | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
