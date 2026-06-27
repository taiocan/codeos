# Upgrade: branch-helper — Optional Branch Creation Helper

**Priority**: P2
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: workflow-profiles, stage-4-activation-card, feature-registry

## Problem

Branch naming and timing can be inconsistent.

## Upgrade

Add a small helper convention or script.

## Scope

Tooling / convention.

## Proposed artifact(s)

Proposed command behavior:

```text
codeos branch <feature_id>
```

Creates `feature/<feature_id>` or, for split mode:

```text
feature/<feature_id>-artifacts
feature/<feature_id>-implementation
feature/<feature_id>-runtime-replay
feature/<feature_id>-refinement
```

## Design notes

Alternative: no script. Just document branch convention.

## Value

Medium. Useful if you move toward PR workflows.

## Risk

Unnecessary tooling.

## Guardrail

Start with documentation only.

## DBA-philosophy note

No rule touched. Pure convenience tooling; doc-first to avoid premature automation.
