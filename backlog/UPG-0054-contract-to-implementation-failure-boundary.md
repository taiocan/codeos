---
feature_id: UPG-0054
slug: contract-to-implementation-failure-boundary
title: Contract-to-Implementation Failure Boundary
status: PROPOSED
priority: P2
depends_on: []
related_features: [UPG-0052]
supersedes: []
superseded_by: []
---

# Upgrade: contract-to-implementation-failure-boundary — Contract-to-Implementation Failure Boundary

**Priority**: P2 (not required to start Architecture Synthesis, but should land before substantial
Stage 4 Rust implementation work begins)
**Status**: PROPOSED
**Type**: downstream-doctrine

## Problem

Approved Stage 2 failure classifications define observable behavioral outcomes, but there's no
disciplined convention for distinguishing rich internal/technical errors from contract-visible
failure classifications during implementation. Without it, Stage 4 work risks silently converting
unexpected failures into misleading business outcomes, leaking internal error detail as a
contract-visible failure, or panicking because no approved classification exists for a routine
technical error (storage/serialization/I/O). This item was split out of `UPG-0052` (Implementation
Profile Framework), where it was originally proposed but identified as scope drift — it is a
cross-language Stage 4/5 concern, not an implementation-profile concern.

## Upgrade

Add generic Stage 4/5 guidance to `dba-system.md`, with a Rust-specific realization in
`patterns/rust-project-structure.md`. Two boundaries, kept distinct: a **behavioral boundary**
(observable business/governance outcomes) and a **technical API boundary** (a function may
legitimately propagate storage, serialization, I/O, or other internal error types).

The rule: only failure classifications approved by the Stage 2 Contract may be exposed as
classified behavioral outcomes. A failure event may be emitted only when that event is also present
in the approved Stage 3 Event Schema — a Contract-approved classification alone does not authorize
emitting it as an event. Internal and technical failures may propagate through richer implementation
error types, but they must remain distinguishable from contractual outcomes and must never be
silently mapped to one. Every internal-to-contractual classification mapping is explicit and
reviewable.

Stage 5 verifies all four:
1. approved contractual failures produce the correct observable classification;
2. emitted failure events conform to the approved Stage 3 schema;
3. technical failures never masquerade as approved behavioral failures;
4. no unapproved failure event is emitted.

**No universal error library or single canonical enum is prescribed** — this is deliberately
compatible with Rust's `Result` model and any other language's native error handling.

## Scope

`dba-system.md` (Stage 4/5 guidance addition), `patterns/rust-project-structure.md` (Rust
realization).

Out of scope: any specific Rust crate/library choice; the Implementation Profile mechanism itself
(`UPG-0052`).

## Value

Closes a real gap surfaced during the Rust-first / Architecture Synthesis discussion without
hitchhiking on the Implementation Profile brief and diluting its scope. Keeps
`dba_governance_record`-style shared envelopes and per-feature error types from being designed
inconsistently once multiple features start Stage 4 implementation.

Trade-offs: none significant — mostly a documentation/guidance addition.

## Risk

Main risk is scope creep toward prescribing a specific error-handling library or a one-enum rule —
explicitly blocked by this brief's guardrail.

## Guardrail

Never mandate a specific error-handling crate or a single canonical enum; only mandate that the
internal-to-contractual mapping be explicit, reviewable, and Stage-5-tested in both directions.

## Related

- **UPG-0052**: Implementation Profile Framework — shares the same Rust pattern file; sequence
  alongside or after it, no hard dependency. This item was split out of `UPG-0052`'s original scope.
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
