---
feature_id: UPG-0012
slug: repair-before-next-feature
title: Repair-Before-Next-Feature Workflow Gate
status: PROPOSED
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: repair-before-next-feature — Repair-Before-Next-Feature Workflow Gate

**Priority**: P1
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: current-verified-state, feature-registry, readiness-checklist

## Problem

It is easy to start a new feature while the current one has unresolved gaps, failed replay, CI
failure, structural risk, or release blocker.

## Upgrade

Add a workflow rule: unresolved work blocks new behavioral features.

## Scope

Workflow rule at session start / before starting a new behavioral feature.

## Proposed artifact(s)

`CLAUDE.md` or `prompts/00-session-start.md`

## Design notes

Rule:

```text
Do not start a new behavioral feature while the current feature has unresolved:
- Stage 7 GAP/MISMATCH/MISSING;
- Stage 8 replay failure;
- required Stage 9 refinement;
- Stage 10 structural blocker;
- failing CI;
- unresolved reviewer BLOCK;
- unresolved pre-release blocker.
```

Routing:
- Behavioral issue → Stage 9 targeted refinement or rerun affected earlier stage.
- Structural issue → Stage 10 architectural refinement.
- Release/package issue → Readiness checklist / release blocker.

## Value

Medium-high. Protects quality and prevents unfinished evidence chains.

## Risk

Can block exploratory work.

## Guardrail

Allow explicit human override:

```text
Human may explicitly suspend a feature and start another, but the suspended feature must remain
marked as blocked/incomplete.
```

## DBA-philosophy note

Adds a **workflow gate**. Preserves human authority via the explicit override; the suspended
feature must stay marked blocked so the evidence chain is never silently abandoned.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260701-004 | `changes/UPG-0012__CHG-20260701-004__repair-before-next-feature.md` | Add repair-before-next-feature rule to 00-session-start.md with 7 blocking conditions and human-override clause | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
