---
feature_id: UPG-0053
slug: implementation-profile-scaffolding-dba-init
title: Implementation-Profile Scaffolding in dba-init.sh
status: PROPOSED
priority: P3
depends_on: [UPG-0052]
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: implementation-profile-scaffolding-dba-init — Implementation-Profile Scaffolding in dba-init.sh

**Priority**: P3
**Status**: PROPOSED
**Type**: script-tooling

## Problem

`scripts/dba-init.sh` has no knowledge of the Implementation Profile artifact proposed in
`UPG-0052`. A naive "always scaffold a proposal with no activation path" behavior does not by
itself ensure a scaffolded profile ever gets surfaced or resolved before Stage 4, especially for
single-feature projects that never run an Architecture Synthesis Gate (`UPG-0051`). There is also a
risk of `dba-init.sh` instead over-reaching — auto-generating a full Cargo workspace before any
Architecture Synthesis Gate has approved a crate topology, locking in structure before evidence
exists.

## Upgrade

- Every new project receives a **scaffolded, non-binding Rust-first proposal file**
  (`status: proposed`, per `UPG-0052`'s single non-binding pre-approval state — no separate
  "provisional" state), never an approved implementation decision — matching `UPG-0052`'s policy
  choice exactly.
- `dba-init.sh` instantiates from **`UPG-0052`'s canonical profile template/schema** — no duplicate
  copy of the YAML structure, and no divergent lifecycle terminology, embedded in shell code.
- Never scaffold Cargo/workspace structure at init time — that remains gated behind an approved
  Architecture Baseline (`UPG-0051`) or, for single-feature projects, a simpler direct human
  decision.
- Human can edit/replace the profile before Stage 4; re-running init must not silently overwrite a
  human-edited profile (idempotency, verified by smoke test).
- **Explicit sequencing note:** this UPG only creates the file, staying `proposed` and non-binding —
  surfacing/resolving it before Stage 4 is `UPG-0052`'s scope (its session-start and onboarding
  awareness pieces). This UPG should not land ahead of `UPG-0052`'s session-start/onboarding piece,
  or the scaffolded file it creates would have no activation path yet.

Not committed for v1, flagged as an open, non-essential design question: explicit init CLI modes
(e.g. `--implementation-profile none|rust-first`).

## Scope

`scripts/dba-init.sh` and its smoke-test coverage only. Tests: generated file conforms to
`UPG-0052`'s canonical schema; existing file never overwritten; missing parent directory created;
idempotent reruns; file stays `proposed`/non-binding, never silently becomes `approved`; no Cargo
files created.

Out of scope: any `dba-system.md` text; any Cargo/workspace generation logic.

## Value

Keeps new-project scaffolding consistent with the profile mechanism without prematurely fixing
crate structure; small, mechanical, easy to verify with a smoke run.

Trade-offs: none significant — this is a narrow, low-risk follow-up once `UPG-0052`'s schema is
stable.

## Risk

Low — mostly an idempotency/smoke-test verification concern (does re-running init clobber a
human-edited profile file), and a sequencing risk if this ships before `UPG-0052`'s activation path
(session-start/onboarding awareness) exists.

## Guardrail

- Never auto-generate Cargo/workspace files at init.
- Profile file written is always `proposed`/non-binding, never pre-approved.
- Re-running init must not silently overwrite a human-edited profile.
- Do not land ahead of `UPG-0052`'s session-start/onboarding scaffolding — this brief's file-creation
  slice needs that activation path to exist for the scaffolded file to mean anything.

## Related

- **UPG-0052**: Implementation Profile Framework — defines the schema and activation path this
  scaffolds; hard dependency.
- **UPG-0051**: Multi-Feature Architecture Synthesis Gate — the gate that ultimately approves crate
  structure this script must not preempt; not a dependency.
- Proposed by the human during a 2026-07-19 discussion of EvidenceAtlas's architecture sequencing
  (see `reviews/review-log.md`).

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
