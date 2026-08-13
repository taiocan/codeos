---
feature_id: UPG-0013
slug: stage-4-activation-card
title: Stage 4 Activation Card
status: COMPLETE
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stage-4-activation-card — Stage 4 Activation Card

**Priority**: P1
**Status**: PROPOSED
**Type**: toolkit-upgrade
**Related**: branch-helper, workflow-profiles, current-verified-state

## Problem

Stage 4 already has the three important inputs: intent, contract, and event schema. A large
OAP-style execution packet would duplicate those inputs. The missing piece is not behavior
specification. The missing piece is activation metadata: branch, repo state, scope,
out-of-scope files, and reporting obligations.

## Upgrade

Add a small Stage 4 Activation Card.

## Scope

Stage 4 primarily. Optionally reused by Stage 5 and 6.

## Proposed artifact(s)

`templates/stage-4-activation-card.md`

## Design notes

```markdown
# Stage 4 Activation Card

Feature ID:

Approved input artifacts:
- Intent:
- Contract:
- Event schema:

Current repo state:
- Branch:
- Commit SHA:
- Working tree:
- Active feature:
- Current approved stage:

Branch policy:
- Existing branch:
- New branch required:
- Proposed branch name:

Implementation scope:
- Files likely in scope:
- Files explicitly out of scope:

Execution constraints:
- Do not change approved artifacts.
- Do not add events not in schema.
- Do not add behavior not traceable to contract.
- Stop if implementation requires new behavior.
- Report if contract/schema appears insufficient.

Required output:
- Files changed:
- Contract clauses satisfied:
- Events emitted:
- Tests not yet written:
- Runtime evidence not yet captured:
- Risks/blockers:
```

Who defines the branch? Default policy:
- if already on a feature branch, record it and continue;
- if on `main` or wrong branch, Claude proposes a branch name;
- human approves branch creation unless project policy allows automatic branch creation;
- deterministic branch name should be derived from feature ID.

Suggested convention: `feature/<feature_id>`

For split PRs:

```text
feature/<feature_id>-artifacts
feature/<feature_id>-implementation
feature/<feature_id>-runtime-replay
feature/<feature_id>-refinement
```

## Value

Medium-high. Useful because it bounds scope and records current state. It should stay small.

## Risk

It duplicates intent/contract/schema.

## Guardrail

Do not restate behavior from approved artifacts. Reference them.

## DBA-philosophy note

No rule changed. Must **reference, not restate** approved artifacts — restating behavior would
create a second behavioral surface and invite drift.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260701-005 | `changes/UPG-0013__CHG-20260701-005__stage-4-activation-card.md` | New stage-4-activation-card template — metadata-only card referencing approved artifacts without restating behavior | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
