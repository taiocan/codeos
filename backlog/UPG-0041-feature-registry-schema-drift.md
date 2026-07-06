---
feature_id: UPG-0041
slug: feature-registry-schema-drift
title: Reconcile feature-registry.yaml Schema vs Real-World Drift (FundFlow)
status: PROPOSED
priority: P2
depends_on: [UPG-0009, UPG-0023]
related_features: [UPG-0037]
supersedes: []
superseded_by: []
---

# Upgrade: feature-registry-schema-drift — Reconcile feature-registry.yaml Schema vs Real-World Drift (FundFlow)

**Priority**: P2
**Status**: PROPOSED
**Type**: downstream-doctrine

## Problem

`generate-approval-dashboard` (UPG-0023) works correctly against `templates/feature-
registry.yaml`'s canonical schema, but fails to parse `/home/rimo/projects/FundFlow/features/
registry.yaml` — a real, live downstream project's actual registry. Discovered 2026-07-06
while demonstrating the dashboard tool.

The drift, field by field:

| Field | Canonical template (UPG-0009) | FundFlow's actual registry |
|---|---|---|
| `slug` | required | absent entirely |
| `status` | `active` / `suspended` / `complete` / `blocked` | `stage0` … `stage9` / `complete` / `stage0-hypothesized` (stage baked into the status string) |
| `current_stage` | separate field, `0`-`10` or `null` | absent — no separate stage field |
| `blockers` | list field | absent — uses free-form `notes` instead |

Notably, FundFlow's `stage0-hypothesized` status value maps more precisely onto
`dba-system.md`'s own Onboarding (Session Type D) vocabulary than the canonical template
does — this may indicate the *template* is what's behind, not FundFlow's registry.

## Upgrade

Not decided by this brief — six questions for whoever picks this up to resolve deliberately,
not patch quickly:

1. Should canonical UPG-0009 evolve to include stage-aware status values like
   `stage0-hypothesized`, matching `dba-system.md`'s actual vocabulary more closely?
2. Should `current_stage` remain a separate field, or be derived from a `stageN`-shaped
   status string?
3. Is `slug` truly required, or should it fall back to `feature_id` when absent?
4. Must `blockers` stay a structured list, or can free-form `notes` remain valid (with the
   dashboard tool reading whichever is present)?
5. Does FundFlow's `features/registry.yaml` need an actual migration to the reconciled
   schema, once one is settled?
6. Should dashboard/reviewer tooling that encounters an unrecognized registry shape fail
   with a clear schema-drift diagnostic (naming the specific missing/mismatched fields)
   rather than a generic parse error — even before this reconciliation is designed?

## Scope

`templates/feature-registry.yaml`'s schema, `tools/reviewer/src/cmd/
generate_approval_dashboard.rs`'s parsing tolerance (if the reconciliation decides to widen
it), and possibly `/home/rimo/projects/FundFlow/features/registry.yaml` (a FundFlow-side
migration, not a Codeos repo file — out of this repo's direct control, coordinate with
whoever maintains that project).

## Value

Medium-high. Without this, `generate-approval-dashboard` — built specifically so humans can
navigate in-flight feature state — cannot actually be used on the one real downstream project
that exists today.

## Risk

Deciding hastily (e.g., "just make slug optional") risks papering over a deeper doctrine
question: whether the canonical registry schema itself needs updating to reflect how
Onboarding-originated projects actually track stage-hypothesized features. Do not patch
without first answering the six questions above.

## Guardrail

No blind FundFlow rewrite and no silent dashboard-tool tolerance patch until the underlying
schema question is deliberately decided — this is itself the first acceptance criterion for
whatever change picks this up.

## DBA-philosophy note

Touches the canonical `templates/feature-registry.yaml` schema (a `.codeos/templates/` file
downstream projects use) — classified `downstream-doctrine`, same rigor tier as UPG-0009,
UPG-0037.

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
