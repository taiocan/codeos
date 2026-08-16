---
feature_id: UPG-0007
slug: solution-discovery-00b
title: Expanded 00b Solution Discovery / Feature Topology Stage
status: SUPERSEDED
priority: P0
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: solution-discovery-00b — Expanded 00b Solution Discovery / Feature Topology Stage

**Priority**: P0
**Status**: SUPERSEDED
**Type**: toolkit-upgrade
**Related**: config-discovery, 00b-adr-generator

## Problem

This proposal was superseded on 2026-08-16 by the approved Solution Framing simplification. The
active workflow keeps only problem, vision, candidate outcomes, scope, candidate constraints, and
open architecture concerns; Feature Decomposition and Architecture Synthesis retain their separate
responsibilities.

Codeos is strong once a feature enters Stage 1. But before Stage 1, there is a need to
brainstorm possible features, common vocabulary, event families, configuration needs,
architecture pressure, and shared concepts without prematurely approving architecture or
feature lists.

## Upgrade

Expand Stage 00b into a non-authoritative solution discovery stage.

## Scope

Pre-Stage-1 discovery.

## Proposed artifact(s)

`dba/03-prompts/workflow/00a-solution-discovery.md` (renamed from `00b-solution-discovery.md` by `UPG-0039`,
2026-07-07, to resolve a prefix collision with `00b-feature-brief.md` — Discovery precedes
Feature Brief in the actual workflow)

Optional output: `docs/solution-discovery.md`

Core principle: 00b output is not approved DBA truth. It is planning hypothesis only. Only
Stage 1 intent, Stage 2 contract, and Stage 3 event schema become approved behavioral truth.

## Design notes

00b output structure:

```markdown
# Solution Discovery / Feature Topology

Product/domain problem:
User/workflow context:
Candidate actors:
Candidate feature list:
Candidate outcomes:
Shared vocabulary/concepts:
Possible event families:
Potential event spine pressure:
Potential shared infrastructure:
Potential configuration requirements:
Potential configuration schema:
External integrations:
Data persistence needs:
Operational constraints:
Security/privacy concerns:
Likely failure modes:
Potential Stage 10 / ADR needs:
Architecture risks:
Vertical drift risks:
Recommended first feature:
Features to defer:
Explicit non-decisions:
What must NOT be treated as approved:
```

Configuration discovery — 00b should identify configuration needs early:

```markdown
# Configuration Hypotheses

Config item:
Purpose:
Feature(s) likely affected:
Default:
Required/optional:
Secret/non-secret:
Environment-specific:
Runtime-changeable:
Needs schema validation:
Potential event impact:
```

## Value

Very high. This helps identify common denominators across multiple future features without
losing DBA advantages.

## Risk

The model may treat 00b hypotheses as approved architecture.

## Guardrail

Every 00b artifact must contain:

```text
This document is non-authoritative planning material.
It does not approve features, architecture, contracts, schemas, events, or implementation.
If this document conflicts with later approved DBA artifacts, the approved DBA artifacts win.
```

## DBA-philosophy note

Touches **artifact authority**: 00b output must never be treated as approved DBA truth. Only
Stages 1–3 produce approved behavioral truth. The non-authoritative banner is the guardrail
that keeps this from eroding intent primacy.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260630-005 | `maintenance/archive/self-development/changes/UPG-0007__CHG-20260630-005__solution-discovery-00b.md` | New 00b solution-discovery prompt + Session Type E in session-start | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
