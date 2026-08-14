---
feature_id: UPG-0049
slug: external-review-policy-registry
title: External Review Policy Registry
status: PROPOSED
priority: P3
depends_on: []
related_features: [UPG-0045, UPG-0037, UPG-0030]
supersedes: []
superseded_by: []
---

# Upgrade: external-review-policy-registry — External Review Policy Registry

**Priority**: P3
**Status**: PROPOSED
**Type**: script-tooling

## Problem

Stage-specific review policy — which stage requires review, what its expected output looks like,
what its checklist is, what cadence applies (self-dev `PROFILE-0..5` per
`maintenance/archive/self-development/retired-process/codeos-self-dev.md` Step 0a, vs. downstream's flat R1/R2/R3 per `dba-system.md`'s
"Default Advisory Review") — currently lives partly in Rust (`dba/04-tools/reviewer/engine/src/` stage/packet
logic), partly in `maintenance/archive/self-development/retired-process/codeos-self-dev.md` and `dba-system.md` prose, and partly in
`dba/03-prompts/review/codeos-reviewer-task.md`'s packet-injected checklist text. A doctrine change to *what a
stage expects* (e.g. `UPG-0037`'s introduction of default downstream review, or any future
addition of a new self-dev step or downstream stage) currently requires touching Rust source,
not just policy documents — raising the cost and risk of what should be a documentation-level
change.

## Upgrade

Not decided by this brief — questions for implementer to resolve:

### 1. What actually needs externalizing

Before designing a YAML policy format, the implementer should inventory exactly which pieces of
stage-specific logic currently live in Rust vs. prose, and how much of it is genuinely
data-shaped (a checklist string, a cadence number, an expected-output description — good
externalization candidates) vs. actual control flow (packet construction order, coverage-state
floor logic — not good candidates, and should stay in Rust regardless of this upgrade). It is
plausible that only a small fraction of current stage logic is worth moving; this brief should
not be read as committing to a large YAML-ification of the reviewer engine.

### 2. Illustrative policy shape

**Not an approved schema** — sketched only to make the "what would move" question concrete:

```yaml
# ILLUSTRATIVE ONLY — not an approved schema
stage_id: selfdev-step-3
name: Implement
domain: self-dev
review:
  required: true
  default_mode: full
  r2_plus_mode: delta
  max_rounds: 2   # PROFILE-2; profile-derived, not hardcoded per-stage
```

The active downstream round budget already has one owner (`dba/02-policies/review/v2.md`'s
Review Round Limit). Any policy-registry design must not duplicate that limit across per-stage
files.

### 3. Downstream compatibility

`dba-system.md`'s stage table and "Default Advisory Review" section are downstream doctrine
(`downstream-doctrine` triage class, per `CLAUDE.md`) — any change that moves downstream stage
policy into an external registry is `downstream-doctrine` or `both` scope, requiring the
downstream-compatibility acceptance criteria `CLAUDE.md` mandates for that class (generated
project still loads `.codeos/toolkit/dba-system.md`; stage tables and prompt filenames move together).
This is a materially higher-rigor path than a self-dev-only change and should be scoped
accordingly — likely worth splitting into a self-dev-only first slice (externalize self-dev step
policy only) before touching downstream stage policy at all.

## Scope

Likely touches `dba/04-tools/reviewer/engine/src/` (wherever stage-specific text/cadence is currently read) and
potentially a new `review-policies/` directory (exact location TBD by the implementer, consistent
with the rest of the toolkit's file layout in `CLAUDE.md`'s Self-Development File Layout table).

Out of scope for this brief:
- Moving packet-construction control flow (evidence-mode selection, coverage-state floor
  computation) into policy files — that is genuine code, not data, and should stay in Rust.
- Any downstream doctrine change beyond what a `downstream-doctrine`-scoped Step 1 explicitly
  declares — this brief does not pre-authorize touching `dba-system.md`.
- Building this before `UPG-0045` (Review Plan Preview), if `UPG-0045` turns out not to need a
  policy registry for its own artifact-resolution logic — the two are related but neither is a
  hard prerequisite for the other; sequencing should follow which one is actually needed first.

## Value

Medium, mostly about lowering the cost of *future* stage/cadence doctrine changes rather than
fixing a current pain point — `UPG-0037` (downstream default review) shipped successfully as a
Rust + prose change; this upgrade would make the *next* one like it cheaper, not fix something
broken today.

Trade-offs: introduces a second source of truth (policy file vs. doctrine prose in
`dba-system.md`/`maintenance/archive/self-development/retired-process/codeos-self-dev.md`) that must be kept in sync, trading "change requires
touching Rust" for "change requires touching Rust AND a policy file AND doctrine prose" unless
the doctrine prose itself is generated from the policy file (a further, likely out-of-scope,
escalation).

## Risk

Deciding hastily risks:
- Externalizing control-flow logic that should stay in Rust, turning YAML files into a shadow
  programming language (a well-known anti-pattern in policy-registry designs).
- Fragmenting the active policy-owned round budget into per-stage duplicate files that drift from
  each other.
- Under-scoping the downstream-doctrine rigor this requires if it ever touches `dba-system.md`'s
  stage table (see "Downstream compatibility" above).

## Guardrail

The policy registry, if built, must:
- Only move genuinely data-shaped content (checklist text, cadence numbers, expected-output
  strings) — packet-construction control flow stays in Rust.
- Preserve `dba/02-policies/review/v2.md` as the single owner of the active downstream round budget.
- Treat any downstream-doctrine-touching slice as `downstream-doctrine`/`both` scope under
  `CLAUDE.md`, with the full downstream-compatibility acceptance criteria that class requires —
  never as an incidental side effect of a self-dev-only change.

## Related

- **UPG-0037**: Default Advisory Review Across the Full Downstream DBA Workflow — the kind of
  doctrine change this upgrade aims to make cheaper next time.
- **UPG-0030**: Lean Self-Development Review Profiles — source of the existing profile-keyed
  round-budget table this upgrade must not fragment.
- **UPG-0045**: Review Plan Preview — a plausible consumer of policy data (resolving required
  artifacts per stage), though not a hard dependency in either direction.
- Proposed by the human during a 2026-07-12 review-architecture discussion (see
  `.codeos/05-review/reviews/review-log.md` and `maintenance/archive/self-development/changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md`).

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| (none yet) | — | — | PROPOSED |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
