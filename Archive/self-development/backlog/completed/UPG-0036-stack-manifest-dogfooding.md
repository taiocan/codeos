---
feature_id: UPG-0036
slug: stack-manifest-dogfooding
title: Stack Manifest & Drift Reconciliation Dogfooding for Codeos Self-Development
status: COMPLETE
priority: P2
depends_on: [UPG-0017, UPG-0020]
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: stack-manifest-dogfooding — Stack Manifest & Drift Reconciliation Dogfooding

**Priority**: P2
**Status**: COMPLETE
**Type**: self-dev-governance
**Related**: stack-manifest (UPG-0017), stack-drift-detector (UPG-0020)

## Problem

`templates/stack-manifest.md` and `templates/stack-reconciliation-report.md` (UPG-0017) and
the `check-drift` subcommand (UPG-0020) are complete, tested, and fully built — but only as
downstream-facing machinery. Codeos has never instantiated a stack manifest for its own
`tools/reviewer` Rust crate, and `tools/reviewer/Cargo.toml` has changed exactly once before
(UPG-0032, establishing the original 9 dependencies) with no reconciliation record at all. If
Codeos requires this discipline of downstream projects but never applies it to itself, the
toolkit is performative rather than dogfooded.

## Upgrade

Instantiate a live, self-dev-scoped stack manifest (`status/stack-manifest.md`) and a narrow
trigger rule in `CLAUDE.md`: any self-dev change whose Step 1 "What changes" table touches a
watched stack/dependency file must include a stack-reconciliation-report instance in the same
change, verified at Step 4 — and `check-drift` is actually run against the Codeos repo when
that happens. Backfill the one real historical gap (UPG-0032's original dependency set) as a
clearly-labeled retroactive record, not a claim that the process existed at the time.

## Scope

Self-dev evidence hygiene only. Explicitly **not** a new governance layer, not doctrine, not
an independent approval authority, and not a downstream-doctrine change.

## Proposed artifact(s)

- `status/stack-manifest.md` — new, live status file (Codeos's own actual stack + dependency
  approval rule; explicitly labeled as evidence/status, not authority).
- A directory of per-change reconciliation-report instances (file names ending in
  `stack-reconciliation-report.md`, so `check-drift`'s existing suffix match recognizes them),
  one retroactive entry for UPG-0032, going forward one per change that touches a watched file.
- A new, narrowly-scoped rule in `CLAUDE.md` stating the trigger, expressed in 4-step-loop
  terms (Step 1 declares, Step 2 adds verification criteria if watched files are touched,
  Step 4 verifies + runs `check-drift`) — no downstream Stage 9/10/readiness-checklist
  language imported.

## Design notes

Trigger rule (4-step-loop terms, not downstream stage terms):

```text
Dependency/stack-file changes must be declared in Step 1 "What changes" or explicitly
re-triaged before implementation if discovered later. If Step 1 declares a watched-file
change, Step 2 must include a verification criterion for it. Step 4 verifies the
stack-reconciliation-report instance exists and runs `check-drift` against the change.
Human approval at each gate remains the authority; the manifest and reconciliation reports
are evidence, not authority.
```

`status/stack-manifest.md` is explicitly labeled: "records the current observed stack and
dependency-policy status for Codeos self-development; not an independent approval authority.
If it conflicts with the self-dev workflow, CLAUDE.md and the approved change record govern."

Watched files for Codeos's own repo (subset of `check-drift`'s existing hardcoded list that
actually applies here): `Cargo.toml`, `Cargo.lock`. The tool's other watched patterns
(`pyproject.toml`, `package.json`, `Dockerfile`, `config/*.yaml`, etc.) exist for downstream
repos and currently match nothing in this repo — noted, not removed from the tool.

## Value

Medium-high. Closes a real dogfooding gap; keeps the toolkit's own dependency changes honest
without inventing new bureaucracy.

## Risk

Scope creep into a second governance layer if the trigger rule is written loosely, or if
downstream stage language leaks into the self-dev version.

## Guardrail

Evidence hygiene tied to watched files, not a new doctrine. `status/stack-manifest.md` is
status/evidence, never authority. `dba-system.md` untouched. No autonomous enforcement — all
gates remain human-approved.

## DBA-philosophy note

Not applicable in the downstream sense — this is self-dev governance, not DBA doctrine. The
underlying principle carried over: trigger-based record-keeping (diff-driven), not
memory-driven, and human approval remains the sole authority at every gate.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260705-001 | `changes/UPG-0036__CHG-20260705-001__stack-manifest-dogfooding.md` | `status/stack-manifest.md` + CLAUDE.md trigger rule + UPG-0032 backfill | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| REV__UPG-0036__CHG-20260705-001__S1__R1 | CHG-20260705-001 | 1-Intent | R1 | DO NOT ADVANCE (bookkeeping bundled into unrelated prior commit, unevidenced) |
| REV__UPG-0036__CHG-20260705-001__S1__R2 | CHG-20260705-001 | 1-Intent | R2 | DO NOT ADVANCE (claim still unevidenced in packet) |
| REV__UPG-0036__CHG-20260705-001__S1__R3 | CHG-20260705-001 | 1-Intent | R3 | NO OBJECTION |
| REV__UPG-0036__CHG-20260705-001__S2__R1 | CHG-20260705-001 | 2-Acceptance | R1 | DO NOT ADVANCE (frontmatter/trace-header mismatch; unverified backfill premise) |
| REV__UPG-0036__CHG-20260705-001__S2__R2 | CHG-20260705-001 | 2-Acceptance | R2 | NO OBJECTION |
| REV__UPG-0036__CHG-20260705-001__S3__R1 | CHG-20260705-001 | 3-Implement | R1 | DO NOT ADVANCE (AC-2/AC-10 exact-grep specs broken by line-wrapping) |
| REV__UPG-0036__CHG-20260705-001__S3__R2 | CHG-20260705-001 | 3-Implement | R2 | DO NOT ADVANCE (AC-7 forward-claim: "recorded in Step 4 below") |
| REV__UPG-0036__CHG-20260705-001__S3__R3 | CHG-20260705-001 | 3-Implement | R3 | NO OBJECTION |
| REV__UPG-0036__CHG-20260705-001__S4__R1 | CHG-20260705-001 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
