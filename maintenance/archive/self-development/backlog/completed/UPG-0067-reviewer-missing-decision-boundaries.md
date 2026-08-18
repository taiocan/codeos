---
feature_id: UPG-0067
slug: reviewer-missing-decision-boundaries
title: Reviewer Has No Stage for the Charter and Architecture Decision Boundaries
status: COMPLETE
priority: P1
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-missing-decision-boundaries — Charter and Architecture Review Stages

**Priority**: P1
**Status**: COMPLETE — shipped 2026-08-18 in 916fc23
**Type**: tool (+ prompt)

## Problem

The review policy states that advisory review runs by default immediately before each decision
boundary required by the selected doctrine, and before a conditional architecture decision required
by the selected architecture policy. The reviewer cannot honour that for two of those boundaries,
because it has no stage identifier for them.

Reviewer contract v3 admits only `framing`, `decomposition`, and `intake` as support-workflow
reviews, plus numeric Stage IDs `1`–`9`. Neither `"charter"` nor `"architecture"` appears anywhere in
`dba/04-tools/reviewer/engine/src/`.

The two unreachable boundaries are:

| Boundary | Doctrine adapter | Owning prompt |
|---|---|---|
| Solution Charter approval | `purpose-approval` | `dba/03-prompts/workflow/support-solution-charter.md` |
| Architecture Synthesis entry | `architecture-entry` | `dba/03-prompts/workflow/support-architecture-synthesis.md` |

Both are genuine approval boundaries under DBA-3 doctrine. The Charter boundary is now reachable in
practice: doctrine v3 requires an approved Charter before the first Specification Package approval,
so every downstream project adopting DBA-3 hits it.

The only currently available response is the policy's Review Waiver — the human records an explicit
waiver and continues. That is the correct escape hatch for missing tooling, but using it routinely at
a doctrine-required boundary makes the default review timing unenforceable at exactly the two points
with the widest blast radius.

`support-session-handoff` and `support-session-orientation` are correctly absent; they are not
decision boundaries.

## Evidence

Encountered in PlotSpot on 2026-08-16 while drafting its first Solution Charter. The reviewer could
not be invoked for the boundary, and the Charter reached the human without advisory review.

## Upgrade

Add the two missing stage identifiers to the reviewer's stage vocabulary, with the packet's
expected-output and checks text for each, and admit them in contract v3 alongside the existing
support-workflow reviews.

Open for Step 1, not settled here:

- Whether the architecture stage is one identifier or distinguishes scope creation from revision.
- What the Charter packet includes beyond the Charter itself — the framing that fed it and the
  approved Intents whose `serves_outcomes` an outcome change would affect are both candidates.

## Scope

**In scope:** stage vocabulary and packet text in `dba/04-tools/reviewer/engine/src/packet.rs`;
supported-stage documentation in `dba/04-tools/reviewer/contract/v3.md`; reviewer tests.

**Out of scope:** the advisory, non-gatekeeping character of review; the Review Waiver mechanism,
which remains the correct response to genuinely unavailable tooling; any change to what the Charter
or Architecture Synthesis prompts require.

## Value

Makes the review policy's default timing achievable at every boundary the doctrine defines, rather
than at all boundaries except the two most consequential ones.

## Risk

Low. Additive stage vocabulary; existing stages and records are unaffected. The main risk is scope
growth into a general "review any artifact" mechanism, which the two named boundaries deliberately
bound.
