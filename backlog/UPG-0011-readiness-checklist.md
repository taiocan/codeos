---
feature_id: UPG-0011
slug: readiness-checklist
title: Lightweight PR / Pre-Release Readiness Checklist
status: PROPOSED
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: readiness-checklist — Lightweight PR / Pre-Release Readiness Checklist

**Priority**: P1
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: release-evidence-package, stack-drift-detector, verify-only-mode

## Problem

Codeos reconciliation verifies behavioral alignment. It does not fully answer whether a change
is ready to merge or release.

## Upgrade

Add a lightweight readiness checklist after Stage 8/9.

## Scope

After Stage 8 replay success or Stage 9 refinement success; before merge / release / final
acceptance.

## Proposed artifact(s)

`templates/readiness-checklist.md`

## Design notes

```markdown
# Readiness Checklist

Feature:
Branch:
Commit:
PR:

Behavioral readiness:
- Stage 7 reconciliation complete:
- No unresolved GAP/MISMATCH/MISSING:
- Stage 8 replay complete:
- Stage 9 refinement complete or not needed:

Operational readiness:
- Tests run:
- Tests skipped explained:
- CI status:
- No unrelated files:
- Runtime evidence sanitized:
- No raw secrets/PII in logs:
- Docs updated if needed:
- Stack/config manifest checked if dependency/config changed:
- Release notes do not overclaim:
- Rollback/revert path known:

Decision:
- READY
- NOT READY
- READY WITH KNOWN LIMITATIONS

Remaining risks:
```

Difference from Stage 10: this is not Stage 10. Stage 10 is architectural refinement for
non-behavioral structural work. Readiness checklist is an operational merge/release gate.

## Value

Medium-high. Useful when moving from development evidence to merge/release decision.

## Risk

Overlaps with reconciliation.

## Guardrail

Keep it short and operational. Do not repeat Stage 7.

## DBA-philosophy note

Adds an **operational gate** distinct from the behavioral stages — it must not duplicate or
override Stage 7 reconciliation. Decision authority stays with the human.

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
