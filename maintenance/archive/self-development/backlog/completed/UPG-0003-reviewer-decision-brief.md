---
feature_id: UPG-0003
slug: reviewer-decision-brief
title: Reviewer Agent for Stage Gate Decision Briefs
status: COMPLETE
priority: P0
depends_on: []
related_features: []
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-decision-brief — Reviewer Agent for Stage Gate Decision Briefs

**Priority**: P0
**Status**: COMPLETE
**Type**: toolkit-upgrade
**Related**: reviewer-full-diff, reviewer-quality-scale, reviewer-verification-packet

## Outcome (closed 2026-08-19)

Delivered, and surpassed by the implementation. The per-stage checks in the Design notes below ship
verbatim in the reviewer engine's stage checklist table (`dba/04-tools/reviewer/engine/src/packet.rs`),
which covers stages 1-9 and adds five stages this brief never anticipated (framing, decomposition,
intake, charter, architecture). The decision brief itself is the append-only review-log entry:
recommendation, effective concern, evidence grade A-E, coverage state, one-line summary, and a human
decision slot.

Two elements were intentionally not adopted. The `APPROVE / REQUEST CHANGES / BLOCK` vocabulary was
replaced by `NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE`, reserving APPROVE for the human — this
is what prevents the "reviewer becomes a second gate" risk named under Risk below. Stage 10 does not
exist; the process has nine stages.

## Problem

The current human bottleneck is reviewing every stage artifact. The human must inspect
intent, contract, event schema, implementation claims, tests, reconciliation, replay, and
refinement material. This is valuable but expensive.

## Upgrade

Add a separate reviewer agent that reviews each stage artifact and produces a short decision
brief for the human. The reviewer does not approve stages. The reviewer compresses evidence
so the human can decide faster.

## Scope

Applies to all stages.

## Proposed artifact(s)

`dba/03-prompts/review/reviewer-decision-brief.md`

## Design notes

Reviewer output:

```markdown
# Reviewer Decision Brief

Feature:
Stage reviewed:
Artifact reviewed:
Recommendation:
- APPROVE
- REQUEST CHANGES
- BLOCK

Confidence:
Risk level:
Main evidence:
Missing evidence:
Potential DBA violations:
Potential drift:
Unapproved behavior risk:
Implementation detail introduced too early:
Questions for human:
Suggested decision:
```

Reviewer checks by stage:

- **Stage 1 Intent**: actor/outcome clarity; no implementation detail; scope boundary
  explicit; stable guarantees clear; ambiguity flagged.
- **Stage 2 Contract**: every intent outcome has observable contract coverage; failure paths
  named; invariants testable; no white-box/internal implementation claims.
- **Stage 3 Event Schema**: every relevant contract scenario has event coverage; event names
  are stable; required fields are clear; no speculative telemetry.
- **Stage 4 Implementation**: code traces to approved contract/schema only; no unapproved
  events; no hidden behavior; no unrelated files; implementation report is complete.
- **Stage 5 Tests**: behavior tested, not private internals; failure paths tested;
  event/telemetry tests present; replay tests prepared where applicable.
- **Stage 6 Runtime**: runtime evidence captured; event log is bounded/sanitized; correlation
  chains visible; unexpected/missing events reported.
- **Stage 7 Reconciliation**: ALIGNED/GAP/MISMATCH/MISSING judgments supported; no weak
  evidence hidden behind fluent summary; gaps routed to the right next action.
- **Stage 8 Replay**: replay evidence actually checks event sequence and schema conformance;
  nondeterminism explained; missing fixtures reported.
- **Stage 9 Refinement**: trigger is valid; proposed fix is minimal; no redesign disguised as
  refinement; affected artifacts identified.
- **Stage 10 Architectural Refinement**: behavior remains unchanged; structural need is real;
  ADR/ARD need identified; no contract/schema drift hidden inside refactor.

> Note: the actual automated implementation (see `dba/04-tools/reviewer/contract/v4.md`) keeps the
> reviewer **advisory and non-gatekeeping** — it uses concern-level vocabulary
> (NO OBJECTION / CHANGES ADVISED / DO NOT ADVANCE), reserving APPROVE for the human, to
> stay aligned with the existing `dba/03-prompts/review/pipeline-reviewer.md` stance.

## Value

Very high. Saves human time while preserving human approval. Improves quality because a
second model checks artifacts adversarially.

## Risk

Reviewer becomes a second gate or rubber-stamp.

## Guardrail

Reviewer may recommend. Human decides. Reviewer must not rewrite artifacts unless explicitly
asked.

## DBA-philosophy note

Touches the **review flow** and risks **gatekeeping**. Safe only while the reviewer stays
advisory and read-only; the non-negotiable human-approval gate (rule #1) must remain
untouched. The current external guarantees are in `dba/04-tools/reviewer/contract/v4.md`.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
