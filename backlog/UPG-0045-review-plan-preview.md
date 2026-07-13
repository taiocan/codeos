---
feature_id: UPG-0045
slug: review-plan-preview
title: Review Plan Preview — codeos-reviewer plan
status: COMPLETE
priority: P2
depends_on: []
related_features: [UPG-0046, UPG-0049, UPG-0042, UPG-0027]
supersedes: []
superseded_by: []
---

# Upgrade: review-plan-preview — Review Plan Preview (`codeos-reviewer plan`)

**Priority**: P2
**Status**: PROPOSED
**Type**: script-tooling

## Problem

Before every `codeos-reviewer review <feature> <stage> <artifacts…>` call, the human/Claude has
to manually reconstruct: which artifacts matter for this stage, whether context files should be
`--sha-only`, whether this round should be `full` or `--mode delta --base <sha>`, and roughly how
big the resulting packet will be. This reconstruction is undocumented tribal knowledge, re-derived
every round from `docs/reviewer-pipeline.md` §14 and the change record's own scope section.

Concretely observed failure modes this causes (see `docs/reviewer-pipeline.md` §4b, §14, and
`backlog/UPG-0031-review-delta-mode-fix.md`): forgetting to pass a required artifact,
accidentally including an unrelated large file that blows the packet budget
(`backlog/UPG-0042-reviewer-packet-efficiency.md`), and choosing `full` when `delta` was clearly
correct for an R2+ round (or vice versa). None of these are enforced or even previewed today —
the first the human/Claude finds out is when the packet manifest or a budget warning prints
*after* the (already-executed) Codex call.

## Upgrade

Not decided by this brief — questions for implementer to resolve:

### 1. What "plan" computes

A read-only, no-Codex-call preview of what a subsequent `review` invocation would send:
resolved artifact list, evidence-mode recommendation per artifact (full / delta / sha-only),
estimated `review_content_bytes` against `CODEOS_PACKET_BUDGET_BYTES`, and (if resolvable) the
stage's expected-output/checklist text from the reviewer task packet. Whether "resolved artifact
list" requires a new artifact-resolution source (see UPG-0049's policy registry) or can start as
"echo back exactly what the human/Claude passes, annotated with size/mode guidance" is an open
implementation question — the latter is far cheaper and may deliver most of the value alone.

### 2. Command shape

E.g. `codeos-reviewer plan <feature> <stage> <artifacts…> [--mode …] [--base …]` mirroring
`review`'s argument shape, or a distinct subcommand with its own flags. Must not require Codex
network access or read/write any review-run state — pure local computation from the same inputs
`review` would use.

### 3. Relationship to `--print-packet`

`scripts/codeos-review.sh` / the Rust engine already supports a packet-inspection path (see
`docs/reviewer-pipeline.md` §11, "Local prechecks"). Determine whether `plan` is a new, higher-level
view on top of the existing packet-building code path, or whether `--print-packet` already
covers this need well enough that `plan` should only add the size/mode *recommendation* layer
(not re-build packet-preview machinery that already exists).

## Scope

New `codeos-reviewer plan` subcommand (or equivalent flag) in `tools/reviewer/src/`, reusing the
existing packet-building code path from the `review` subcommand rather than duplicating it.
Advisory output only — never writes files, never calls Codex, never changes review-run state.

Out of scope for this brief:
- Auto-selecting or auto-downgrading evidence mode (§14 of `docs/reviewer-pipeline.md` is explicit
  that mode selection stays human/practitioner-controlled — this upgrade previews, never decides).
- Any structured `ReviewRun` record (`UPG-0046`) — `plan` output may or may not need to persist
  anywhere; if it does, that's `UPG-0046`'s concern, not this one's.
- Any change to `dba-system.md` or downstream stage doctrine.

## Value

Medium-high. Directly reduces the failure modes that motivated `UPG-0042` (packet bloat) and
the `EMPTY_PACKET` fail-closed guard added by `UPG-0031` — both are downstream symptoms of not
knowing, in advance, what a review call will actually send. A cheap preview step catches most of
these before a Codex round is spent on a malformed or oversized packet.

Trade-offs: another subcommand to maintain in lockstep with `review`'s own artifact/mode
handling; risk of the preview drifting from what `review` actually sends if the two code paths
aren't shared carefully (see "Relationship to `--print-packet`" above — this is the main design
risk to resolve before implementing).

## Risk

Deciding hastily risks:
- Building a second, subtly different packet-construction code path that drifts from the real one
  `review` uses, making the preview actively misleading.
- Making `plan` feel like a required step (contradicts the "advisory, never a gate" philosophy) —
  it must stay optional, exactly like the existing `--print-packet` inspection path.

## Guardrail

The plan preview must:
- Never call Codex, never write to `reviews/`, never mutate any tracked file.
- Share the actual packet-construction logic `review` uses (not reimplement it), so the preview
  cannot drift from reality.
- Remain optional — `review` must keep working exactly as today without ever requiring a prior
  `plan` call.
- Never auto-select or auto-downgrade an evidence mode; only recommend, per `docs/reviewer-pipeline.md`
  §14's existing guardrail language.

## Related

- **UPG-0042**: Reviewer packet efficiency — the packet-bloat problem this upgrade helps prevent
  earlier in the workflow.
- **UPG-0031**: Review script delta-mode fix + fail-closed `EMPTY_PACKET` guard — the failure mode
  this upgrade helps catch *before* a Codex call is spent, not just fail closed after.
- **UPG-0044**: Reviewer Pipeline Architecture Refresh — `docs/reviewer-pipeline.md` §14 (evidence
  modes) and §5 (coverage states) are the doctrine this preview must stay consistent with.
- **UPG-0046**: ReviewRun Structured Records — a natural (but not required) place to persist a
  plan if `plan`'s output needs to be referenced by a later `review` call.
- **UPG-0049**: External Review Policy Registry — if built, would let `plan` resolve required
  artifacts from policy instead of only echoing back what was typed.
- Proposed by the human during a 2026-07-12 review-architecture discussion (see
  `reviews/review-log.md` and `changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md`
  for the discussion this backlog batch originated from).

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260712-002 | changes/UPG-0045__CHG-20260712-002__review-plan-preview.md | Add `codeos-reviewer plan` reusing packet::build() | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0045__CHG-20260712-002__S1 | CHG-20260712-002 | 1-Intent | R1 | NO OBJECTION |
| RVS__UPG-0045__CHG-20260712-002__S2 | CHG-20260712-002 | 2-Acceptance | R1→R2 | DO NOT ADVANCE → NO OBJECTION |
| RVS__UPG-0045__CHG-20260712-002__S3 | CHG-20260712-002 | 3-Implement | R1→R2 | DO NOT ADVANCE → NO OBJECTION |
| RVS__UPG-0045__CHG-20260712-002__S4 | CHG-20260712-002 | 4-Reconcile | R1 | NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
