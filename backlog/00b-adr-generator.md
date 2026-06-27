# Upgrade: 00b-adr-generator — 00b to ADR Candidate Generator

**Priority**: P3
**Status**: BACKLOG
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
