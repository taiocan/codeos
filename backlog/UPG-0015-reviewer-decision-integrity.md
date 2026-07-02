---
feature_id: UPG-0015
slug: reviewer-decision-integrity
title: Bind stage approval to the reviewed provenance
status: COMPLETE
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-decision-integrity — Bind stage approval to the reviewed provenance

**Priority**: P1
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-engine-v1, reviewer-pipeline (docs/reviewer-pipeline.md), reviewer-artifact-schemas (docs/reviewer-artifact-schemas.md)

## Problem

The v0 reviewer pipeline is a **manual advisory logging pilot**. The `decision` command *records*
a human decision and, as a best-effort audit aid, re-hashes the named reviewed artifacts and flags
MATCH/CHANGED — but it does **not** enforce anything. It does not bind an approval to a reproducible
reviewed state, does not re-verify the full reviewed provenance (commit, diff hash, packet bytes,
workspace state), and makes no rollback guarantee. So an `APPROVE_STAGE` entry in the log is a
record of human intent, not proof that approval traces to an unchanged, reviewer-seen repository
state.

A formal approval-integrity subsystem was prototyped and then **deliberately removed** from the v0
PR (it had grown beyond the advisory-MVP scope and kept accreting unresolved internal
contradictions). This item captures that subsystem as **future work** so it can be designed
deliberately rather than drifted into.

## Upgrade

Bind `APPROVE_STAGE` to **reproducing the stored review provenance**, with durable guarantees:

1. **Full provenance recheck at decision time.** Re-verify not just the named artifacts but the
   recorded `review_commit`, `diff_hash` of the reviewed pathspec, `reviewed_packet_sha256`, and
   `workspace_dirty`. Any mismatch refuses approval unless an explicit, named override is recorded.
   *(CHG-20260702-002 increment: provenance fields are re-verified and reported in the Provenance
   block; only `coverage_state: CRITICAL_OMISSION / EMPTY_PACKET` triggers a gate. Packet-hash
   mismatch and commit-drift are advisory warnings in this increment. The "any mismatch refuses"
   guarantee is the full-vision target for a later increment.)*
2. **Binding modes** (the removed "provenance_integrity" axis): `COMMIT_BOUND` (HEAD == review
   commit, clean tree, artifacts hash-match) vs `WORKSPACE_BOUND` (uncommitted reviewed content,
   re-verified via artifact SHA + diff hash + packet SHA + workspace_dirty). `CRITICAL_OMISSION` /
   `EMPTY_PACKET` / unverifiable states are **software-enforced stops** (automated progression
   blocked); human-overridable with mandatory rationale. See AJ-011 and Guardrail below.
3. **Durable workspace snapshots** so a workspace-bound approval is reproducible from a git object
   (stash/tree/commit hash) rather than by re-hashing the live tree.
4. **Rollback semantics**: a recorded approval names the exact "last sound OK point" to return to.
5. **Dirty-workspace decision policy**: either forbid `APPROVE_STAGE` while dirty, or require it to
   bind to a proven durable snapshot.
6. **Re-hash the saved packet + assessment on every approval path**, not only workspace-bound ones,
   so approval cannot point at mutated evidence.
   *(CHG-20260702-002 increment: packet re-hash performed on APPROVE_STAGE path; REQUEST_CHANGES /
   STOP write a Provenance block informational only. Full-path enforcement is the future-vision target.)*
7. **Override/waiver vocabulary**: explicit `[STALE OVERRIDE]` / `[WORKSPACE OVERRIDE]` (provenance
   mismatch) and `[SECURITY WAIVER]` / `[COVERAGE WAIVER]` (coverage degradation), each requiring a
   human reason. Reserve and explicitly **forbid** any "unreviewed override" path.
8. **Machine-readable artifact validation**: JSON Schema for the assessment header / log records,
   parser hardening (e.g. `excluded_paths` as a structured list, not a space-joined string), and a
   decision-time verification block in the log record.
9. **CI enforcement** of the above schemas/validation.
10. **Per-feature structured decision ledger** (vs the single global append-only markdown log).
11. **Stronger stale-approval prevention** across stages.

## Scope

Full vision: `scripts/codeos-review.sh` (`decision`, and the provenance fields in `review`) +
`docs/reviewer-pipeline.md` + `docs/reviewer-artifact-schemas.md`. Behavioral change to the decision
guard; **no hooks**.

*(CHG-20260702-002 increment: implemented in the typed Rust engine —
`tools/reviewer/src/cmd/decision.rs`, `log.rs`, `main.rs`, `tests/smoke.rs`. The Bash script path
remains valid for any future Bash-only deployments; this increment targets the Rust engine.)*

## Value

High. Closes the gap between "advisory review" and "approval bound to a reproducible state" — the
core DBA traceability claim. But it is **not** required for the advisory-logging MVP, and adding it
prematurely (in Bash) is what caused the original scope drift.

## Risk

Over-tightening could refuse legitimate workspace-dirty pilots; keep clear overrides with named
reasons and precise "which provenance field diverged" messaging. Building it in Bash risks the same
internal-contradiction churn — prefer the typed engine.

## Guardrail

`CRITICAL_OMISSION` and `EMPTY_PACKET` are **mandatory stop conditions for automated
progression**. Advancement requires an explicit human override with recorded rationale.
The override does not invalidate the finding; it records that the human intentionally
accepted the associated risk. Refusals must name exactly which provenance field or
coverage state triggered the stop.

**Distinction:** "automated hard stop" ≠ "human-non-overridable." The CLI enforces the
stop; the human remains the final authority (consistent with Rule 1: explicit human
correction overrides everything). No state is absolutely non-overridable when a human
is present — but no state can be silently bypassed by automation alone. See AJ-011.

## DBA-philosophy note

Strengthens **artifact authority / runtime evidence** binding at the human gate without moving the
gate. The reviewer stays advisory; this only makes a recorded *approval* prove it matches the
reviewed state. Deferred from the v0 advisory pilot, which intentionally stays narrow.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260702-002 | `changes/UPG-0015__CHG-20260702-002__decision-provenance-binding.md` | Provenance binding at decision time: packet integrity recheck, commit-drift warning, coverage gate (software stop / human-overridable) | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
