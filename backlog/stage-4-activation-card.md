# Upgrade: stage-4-activation-card — Stage 4 Activation Card

**Priority**: P1
**Status**: BACKLOG
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
