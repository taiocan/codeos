# Upgrade: reviewer-decision-integrity — Bind stage approval to the full reviewed provenance

**Priority**: P1
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: reviewer-decision-brief, reviewer-pipeline (docs/reviewer-pipeline.md), reviewer-artifact-schemas (docs/reviewer-artifact-schemas.md)

## Problem

The v0 reviewer pipeline enforces a stale-approval guard, but it only re-hashes the **named
reviewed artifacts** at decision time. A cross-model review of the pipeline (2026-06-27)
identified that this leaves approval able to apply to a stale or non-durable reviewed state:

1. **Partial provenance recheck.** `decision APPROVE_STAGE` refuses only when a *named*
   artifact is `CHANGED`. It does not re-check the recorded `review_commit`, `workspace_dirty`
   state, or `diff_hash`. Files that were present only in the reviewed diff (not named
   artifacts), or edits made after review, can change without tripping the guard as long as the
   named artifacts still hash-match — breaking the claim that the assessment is pinned to the
   reviewed state.
2. **Approval against an uncommitted workspace.** The pipeline supports reviews against
   `<review_sha> (+ uncommitted workspace changes)` with `workspace_dirty: true`, but the HUMAN
   DECISION record binds only to `Commit reviewed: <sha>`. The approved state may therefore not
   correspond to any durable git object, so the "last sound OK point" is not reproducible from
   git alone.

> **Status note:** the provenance/coverage **matrix** has shipped (see
> `docs/reviewer-artifact-schemas.md`): `APPROVE_STAGE` eligibility is governed by
> `provenance_integrity` (COMMIT_BOUND / WORKSPACE_BOUND / REDACTED_BOUND / PARTIAL_BOUND /
> UNBOUND), and WORKSPACE_BOUND approvals re-verify artifact SHA + diff hash + packet SHA +
> workspace_dirty at decision time. This item now covers only the **deeper** work below:
> durable workspace snapshots (so a workspace-bound approval is reproducible from an object,
> not just re-hashing the live tree), structured per-feature decision ledgers, machine-readable
> JSON Schema validation, parser hardening, and CI enforcement.

## Upgrade

Bind `APPROVE_STAGE` to reproducing the **stored review provenance** with durable guarantees
beyond the shipped minimal guard (exact `diff_hash` recomputation independent of commit
identity, durable workspace snapshots, structured decision ledgers).

## Scope

`scripts/codeos-review.sh` (`decision`) + `docs/reviewer-pipeline.md` +
`docs/reviewer-artifact-schemas.md`. Behavioral change to the decision guard only; no hooks.

## Design notes

- At `APPROVE_STAGE`, read the latest assessment's metadata and require: current `HEAD` ==
  recorded `review_commit` (base SHA portion), current `diff_hash` of the reviewed pathspec ==
  recorded `diff_hash`, and current `workspace_dirty` == recorded value. Any mismatch → refuse
  unless `--force "<reason>"` records an explicit `[STALE OVERRIDE]`.
- Either **forbid** `APPROVE_STAGE` when `workspace_dirty: true`, or require the decision to
  bind to a durable snapshot identifier (e.g. a stash/commit/tree hash) that is proven to exist
  before approval is logged.
- Optional (same review): store `excluded_paths` as a **list** rather than a space-separated
  string (paths with spaces are not round-trippable); extend the fail-closed validator to also
  require `Log summary` and the decision-time verification block in the log record.

## Value

High. Closes the gap between "advisory review" and "approval bound to a reproducible state" —
the core DBA traceability claim.

## Risk

Over-tightening could refuse legitimate workspace-dirty pilots; keep the `--force` override and
clear messaging.

## Guardrail

`--force "<reason>"` always available; refusals must name exactly which provenance field
diverged. Human authority preserved.

## DBA-philosophy note

Strengthens **artifact authority / runtime evidence** binding at the human gate without moving
the gate. The reviewer stays advisory; this only makes a recorded *approval* prove it matches
the reviewed state. Deferred from the v0 schema PR (which intentionally stayed narrow).
