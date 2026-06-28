---
feature_id: UPG-0017
slug: stack-manifest
title: Stack Manifest with Automatic Reconciliation
status: PROPOSED
priority: P2
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stack-manifest — Stack Manifest with Automatic Reconciliation

**Priority**: P2
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: stack-drift-detector, config-discovery, readiness-checklist

## Problem

A stack manifest is useful, but hand-maintained stack documentation becomes stale.

## Upgrade

Create a two-layer stack record: (1) stable stack decisions; (2) automatically checked
dependency/config inventory.

## Scope

Stack/config knowledge; checked on dependency/config diffs, not every stage.

## Proposed artifact(s)

`docs/stack-manifest.md`

Optional: `templates/stack-reconciliation-report.md`

## Design notes

Stack manifest structure:

```markdown
# Stack Manifest

## Stable stack decisions
Language/runtime:
Package manager:
Test framework:
Event log format:
Replay test location:
Database/persistence:
External services:
Configuration system:
Deployment target:
Allowed dependency categories:
Forbidden dependency categories:

## Dependency policy
When a new dependency may be added:
Who approves:
Required justification:
Where it must be documented:
Required tests:

## Configuration policy
Where config lives:
How config schema is validated:
Secret vs non-secret config:
Environment-specific config:
Defaulting policy:
```

Automatic reconciliation — do not rely on humans to remember updates:

```text
If any dependency/config file changed, stack manifest reconciliation is required.
```

Files to watch: `pyproject.toml`, `poetry.lock`, `requirements.txt`, `Cargo.toml`,
`Cargo.lock`, `package.json`, `package-lock.json`, `pnpm-lock.yaml`, `Dockerfile`,
`docker-compose.yml`, `.env.example`, `config/*.toml`, `config/*.yaml`, `settings.*`.

Stack reconciliation report:

```markdown
# Stack Reconciliation Report
Dependency/config files changed:
New dependency:
Removed dependency:
Version change:
Runtime impact:
Test impact:
Configuration impact:
Stack manifest update needed:
- yes/no
If no, why:
```

When updated: Stage 10 if dependency/config change is structural; Stage 9 if required for
targeted refinement; readiness checklist before merge/release.

## Value

Medium. Useful for knowing the stack, without stale manual maintenance.

## Risk

Manifest goes stale.

## Guardrail

Manifest update is triggered by dependency/config diffs, not by memory.

## DBA-philosophy note

No behavioral rule touched. Trigger-based (diff-driven), not memory-driven — keeps the record
honest without adding a manual maintenance burden.

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
