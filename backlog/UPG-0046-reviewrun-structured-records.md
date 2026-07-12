---
feature_id: UPG-0046
slug: reviewrun-structured-records
title: ReviewRun Structured Records
status: PROPOSED
priority: P2
depends_on: []
related_features: [UPG-0045, UPG-0047, UPG-0048, UPG-0001, UPG-0029]
supersedes: []
superseded_by: []
---

# Upgrade: reviewrun-structured-records — ReviewRun Structured Records

**Priority**: P2
**Status**: PROPOSED
**Type**: script-tooling

## Problem

A single review round today is reconstructable only by reading four things together: the
`## <timestamp> REVIEW —` block in `reviews/review-log.md`, the raw assessment under
`reviews/codex/<ts>-...md`, the packet under `reviews/codex/packets/<ts>-...packet.txt`, and the
change record's own prose summary of "what R1 found, what R2 found." Nothing ties these four
together as one addressable unit — the mapping is by matching timestamps and filenames, done by
eye. `docs/reviewer-pipeline.md` §4e (added by `UPG-0044`) documents this five-way ownership
split explicitly, which makes clear how much reconstruction a reader has to do.

This is workable at the current review volume but was already a source of friction in this
session's own `UPG-0044` change: three separate change-record sections (Implementation Notes,
Reconciliation table, and the final `review-log.md` rollup) each had to independently restate
"what R1 found, what got fixed, what R2 said" — the same facts, transcribed three times by hand,
with real risk of the transcriptions drifting from each other (and, per that same change's own
review findings, they did drift at least once before being caught).

## Upgrade

Not decided by this brief — questions for implementer to resolve:

### 1. What a `ReviewRun` actually is

A structured record (see illustrative sketch below — **not a final schema**) uniquely identified
by the existing `REV__…__R<N>` naming convention (`backlog/UPG-0001-feature-thread-traceability.md`),
one per round, holding: the review's inputs (feature/change/stage/round/artifacts/mode/base),
the packet's coverage state and hashes (already recorded in `reviews/review-log.md` today), the
reviewer's raw verdict, and — critically — a place for the round's findings to live in a form
another process (or the next round's `plan`, if `UPG-0045` is built) can read back mechanically
rather than by re-parsing prose.

```yaml
# ILLUSTRATIVE ONLY — not an approved schema
review_id: REV__UPG-0043__CHG-20260711-002__S3__R1
feature_id: UPG-0043
change_id: CHG-20260711-002
stage: selfdev-step-3
round: 1
artifacts: [...]
evidence: {mode: full, coverage_state: FULL_COVERAGE, review_content_bytes: 42871}
verdict: {codex_concern: NO_OBJECTION, effective_concern: NO_OBJECTION, evidence_grade: B}
human_decision: {decision: APPROVED, rationale: null, decided_at: null}
```

### 2. Where it lives and what's authoritative

Whether this is a new file per round under `reviews/runs/<review_id>/`, or an additional
machine-readable sidecar next to the existing `reviews/codex/<ts>-...md` assessment (same
directory, `.yaml` alongside the `.md`), or a restructuring of `reviews/review-log.md` itself
into a more structured (but still append-only, still human-readable) format. The existing
append-only guarantee on `reviews/review-log.md` (`docs/reviewer-pipeline.md` §4) and the
Self-Reference Boundary (`UPG-0001`) must both be preserved regardless of the chosen shape.

### 3. Relationship to existing artifacts

This must **not** duplicate what `reviews/review-log.md` and `reviews/codex/*.md` already record
faithfully today (feature/stage/branch/hashes/coverage/verdict — see
`docs/reviewer-artifact-schemas.md`). The value-add is specifically: (a) one addressable id per
round instead of matching timestamps by eye, and (b) a place for findings to be structured
(deferred to `UPG-0047`, not this brief) rather than prose-only. If the existing artifacts
already satisfy (a) well enough once given a name, this upgrade may reduce to "assign and cross-
reference `REV__…__R<N>` ids consistently" rather than a new file format — that's a legitimate,
smaller outcome the implementer should consider before building new storage.

## Scope

Likely touches: `tools/reviewer/src/log.rs` (or a new module) to emit/read the structured record;
possibly a new `reviews/runs/` directory (or sidecar files next to existing `reviews/codex/`
artifacts) — implementer's choice per the open questions above. Must preserve every existing
guarantee in `docs/reviewer-pipeline.md` §4/§4a (append-only, committed/durable vs. scratch
classification) and §4e (record-ownership split) for whichever artifacts already exist.

Out of scope for this brief:
- Structured findings themselves (`findings.yaml` or equivalent) — that's `UPG-0047`, which
  depends on this one existing first.
- Any event-sourcing / `reviews/review-events.jsonl` source-of-truth migration — that is the far
  larger, higher-risk `UPG-0048`, deliberately kept separate and not a prerequisite for this
  smaller upgrade.
- Any change to the advisory/human-gated review philosophy itself.

## Value

High, if `UPG-0047` (structured findings) is also built — the two together are what the human's
proposal called "Review Control Plane Lite." Alone, this upgrade mainly buys addressability and
removes cross-file transcription risk; most of the compounding value comes once findings are
structured on top of it.

Trade-offs: another artifact format to keep in sync with `docs/reviewer-artifact-schemas.md`'s
existing normative schemas; risk of two competing "sources of truth" (the new structured record
vs. the existing prose log) if the migration is partial.

## Risk

Deciding hastily risks:
- Building parallel bookkeeping that duplicates rather than replaces the current
  `reviews/review-log.md` + `reviews/codex/*.md` pair, doubling maintenance without buying
  addressability.
- Silently changing what counts as the durable/authoritative record without updating
  `docs/reviewer-pipeline.md` §4a's committed/durable vs. scratch policy to match.

## Guardrail

The structured record must:
- Never become authoritative *instead of* human approval — it stores evidence of a decision, it
  does not make one (same rule as everything else in this pipeline).
- Preserve the append-only guarantee wherever it touches or replaces `reviews/review-log.md`.
- Preserve the Self-Reference Boundary (`UPG-0001`): a change record still references a stable
  `review_series` + `review_state`, never a live round, regardless of how the round itself is
  stored.
- Be introduced incrementally — do not require rewriting all historical `reviews/codex/*.md`
  entries into the new shape as a precondition; new format applies going forward.

## Related

- **UPG-0001**: Feature Thread Traceability — the `REV__…__R<N>` naming convention and
  Self-Reference Boundary this upgrade must preserve.
- **UPG-0029**: Review artifact durability + packet naming policy — the existing
  committed/durable vs. scratch classification (`docs/reviewer-pipeline.md` §4a) this upgrade's
  new artifact type must be classified under.
- **UPG-0045**: Review Plan Preview — a natural producer/consumer relationship if `plan` output
  is persisted anywhere.
- **UPG-0047**: Structured Findings — depends on this upgrade existing first (findings need a
  `ReviewRun` to attach to).
- **UPG-0048**: Review Ledger Event Sourcing — a much larger, separate upgrade that this one is
  *not* a required step toward, though it would build on it if ever pursued.
- Proposed by the human during a 2026-07-12 review-architecture discussion (see
  `reviews/review-log.md` and `changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md`).

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
