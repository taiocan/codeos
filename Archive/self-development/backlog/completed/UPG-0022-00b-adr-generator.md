---
feature_id: UPG-0022
slug: 00b-adr-generator
title: 00b to ADR Candidate Generator
status: COMPLETE
priority: P3
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: 00b-adr-generator — 00b to ADR Candidate Generator

**Priority**: P3
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: solution-discovery-00b, config-discovery

## Problem

Expanded 00b may identify architecture risks but they can be lost.

## Upgrade

Generate ADR candidates from 00b.

## Scope

Pre-Stage-1 → ADR candidate handoff.

## Proposed artifact(s)

ADR candidate output:

```markdown
# ADR Candidates

Decision needed:
Why now:
Features affected:
Options:
Risk if deferred:
Does this affect behavior:
Recommended route:
- Stage 1–3
- Stage 10
- no action yet
```

## Design notes

Keeps 00b architecture risks from being lost by turning them into explicit, routable ADR
candidates — without making 00b authoritative.

## Value

Medium. Keeps 00b useful without making it authoritative.

## Risk

ADR candidates treated as approved decisions.

## Guardrail

Candidates only; non-authoritative until routed through Stage 1–3 or Stage 10.

## DBA-philosophy note

Touches **artifact authority**: ADR candidates are non-authoritative until routed through an
approved stage. Keeps 00b discovery from silently becoming architecture.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260703-004 | `changes/UPG-0022__CHG-20260703-004__00b-adr-generator.md` | New Rust subcommand: `generate-adr-candidates` | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0022__CHG-20260703-004__S1__R1 | CHG-20260703-004 | 1-Intent | R1 | DO NOT ADVANCE (packet scope-polluted by uncommitted UPG-0021 work; roadmap contradiction) |
| REV__UPG-0022__CHG-20260703-004__S1__R2 | CHG-20260703-004 | 1-Intent | R2 | NO OBJECTION |
| REV__UPG-0022__CHG-20260703-004__S2__R1 | CHG-20260703-004 | 2-Acceptance | R1 | NO OBJECTION |
| REV__UPG-0022__CHG-20260703-004__S3__R1 | CHG-20260703-004 | 3-Implement | R1 | NO OBJECTION |
| REV__UPG-0022__CHG-20260703-004__S4__R1 | CHG-20260703-004 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
