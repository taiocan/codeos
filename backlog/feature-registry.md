# Upgrade: feature-registry — Feature Registry / Branch Binding

**Priority**: P2
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: current-verified-state, workflow-profiles, branch-helper, approval-dashboard

## Problem

Branch, feature, stage, artifacts, PR, and review state can drift apart.

## Upgrade

Extend or introduce a feature registry that binds feature ID to branch, stage, PR, and status.

## Scope

Cross-feature state index.

## Proposed artifact(s)

`feature-registry.yaml` (note: CLAUDE.md already references `features/registry.yaml` as the
authoritative human-maintained index — reconcile naming during implementation).

## Design notes

```yaml
features:
  listing-ingestion:
    status: active
    branch: feature/listing-ingestion
    current_stage: 6
    intent: intents/listing-ingestion.md
    contract: contracts/listing-ingestion_contract.md
    event_schema: events/listing-ingestion_schema.md
    pr: null
    last_commit: abc123
    reconciliation_status: pending
    replay_status: pending
    blockers: []
```

When updated (automatically): Stage 0 session start; after stage approval; after branch
creation; after PR creation; after reconciliation/replay; after suspension/completion.

## Value

Medium-high. Makes automation and reviewer-agent context much easier.

## Risk

Registry becomes stale.

## Guardrail

Generate warnings if registry disagrees with filesystem/git state.

## DBA-philosophy note

Touches **artifact authority** boundary: the registry is an index, not truth. It must warn —
and defer — on any disagreement with the filesystem/git/artifacts, never override them.
