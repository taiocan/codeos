---
feature_id: UPG-0005
slug: current-verified-state
title: Current Verified State Block
status: PROPOSED
priority: P0
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: current-verified-state — Current Verified State Block

**Priority**: P0
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: feature-registry, repair-before-next-feature

## Problem

Long-running Codeos work can continue from stale chat memory, wrong branch, old artifacts,
dirty working tree, or unresolved gaps.

## Upgrade

Add a mandatory current-state snapshot at session start and before sensitive stage
transitions.

## Scope

Session start (Stage 0) and before sensitive stage transitions.

## Proposed artifact(s)

Best implementation:
1. add the rule to `CLAUDE.md`;
2. implement the actual block in `prompts/00-session-start.md`;
3. optionally update `templates/project-CLAUDE.md` so new projects inherit it.

Important design decision: the current verified state should be generated automatically each
session. It should not be a hand-maintained document that becomes stale.

## Design notes

Proposed block:

```markdown
# Current Verified State

Repository:
Branch:
Commit SHA:
Working tree:
- clean / dirty
- untracked files:

Active feature:
Current approved stage:
Approved artifacts present:
- Intent:
- Contract:
- Event schema:

Implementation exists:
Tests exist:
Runtime events available:
Last reconciliation status:
Last replay status:
Open GAP/MISMATCH/MISSING items:
Open Stage 9 refinements:
Open Stage 10 architectural issues:
Session scope:
Forbidden actions this session:
```

Automatic data sources:

```bash
git branch --show-current
git rev-parse --short HEAD
git status --short
ls intents/
ls contracts/
ls events/
ls tests/
ls reviews/
```

If Codeos has a feature registry, Claude should read it. If the registry and filesystem
disagree, Claude must report the disagreement and stop for human clarification.

When updated:
- at Stage 0 session start;
- before Stage 4 implementation;
- before Stage 7 reconciliation;
- before Stage 9 refinement;
- before pre-release/readiness review.

## Value

Very high. Prevents stale-state mistakes and makes reviewer-agent work more reliable.

## Risk

It becomes stale if stored as manually edited text.

## Guardrail

Generate it from repo state. Do not manually maintain it except for explicit human notes such
as session scope or forbidden actions.

## DBA-philosophy note

No rule changed, but directly protects rule integrity: prevents gate decisions being made
against stale state. The block is **generated, never authoritative** — artifacts + git remain
truth.

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
