---
feature_id: UPG-0059
slug: wave-gated-batch-review
title: Wave-Gated Batch Review for Multi-Feature Stage 1-3 Cohorts
status: COMPLETE
priority: P2
depends_on: [UPG-0051, UPG-0058]
related_features: [UPG-0057]
supersedes: []
superseded_by: []
---

# Upgrade: wave-gated-batch-review — Wave-Gated Batch Review for Multi-Feature Stage 1-3 Cohorts

**Priority**: P2
**Status**: COMPLETE
**Type**: downstream-doctrine

## Problem

Multi-feature DBA projects pay a real efficiency cost from the current model's per-feature,
per-stage individual review sessions: at cohort scale, Stage 1/2/3 approval fragments into many
separate round trips, each requiring a fresh context switch. A batched-drafting-and-review
proposal was considered to fix this, but its original form removed the per-feature human approval
gate entirely (draft all of Stage 1-3 for every feature, review only at the very end) — which
maximizes blast radius: a bad Stage-1 assumption for one feature could compound through two more
stages, and spread to sibling features drafted in the same batch, before any human ever saw it.

## Upgrade

A **Wave-Gated Batch Review** model, layered on the existing Multi-Feature Architecture Synthesis
Gate's Cohort Check mechanism (`dba-system.md`): drafting and human attention batch per stage
across a cohort (one wave = one stage for every member feature, one human session per wave), but
the approval decision stays individual, per feature, per artifact version — Non-Negotiable Rule #1
is unchanged. Adds: independent-justification requirement for feature-local drafts (sibling drafts
may inform, never become unstated authority); progressive cross-stage checks (Stage 2:
Contract-vs-Intent upstream alignment; Stage 3: the complete Column Check against both upstream
artifacts); advisory-but-not-ignorable check wording (a material unresolved contradiction makes
approval unsupported without an explicit human override); partial-wave outcomes with targeted
reassessment via a lightweight impact assessment (reusing the existing baseline/logical-design
versioning pattern, applied one level down); and versioned cohort membership so a
deferred/removed/split feature requires an explicit membership revision rather than silently
blocking Architecture Synthesis for the rest of the cohort. Also extends `01-intent.md`'s and
`02-contract.md`'s existing "Ambiguity Detection" sections with three allowed response forms: a
proposed answer with rationale, bounded alternatives with a recommendation, or an explicit "no
defensible proposal, here is the missing decision/evidence" — never a forced, manufactured answer.
`03-event-schema.md` has no equivalent existing section and is explicitly out of scope for this
requirement (discovered during Step 3 implementation; a candidate for a future small follow-up, not
done here).

Full design history — four rounds of critical review converging on this shape — lives in
`/home/rimo/.claude/plans/calude-consider-this-inputs-steady-pnueli.md`.

**Definition of success**: a cohort's Stage 1-3 work can be drafted and reviewed in three batched
sessions (one per stage) instead of 3×N individual sessions, with every per-feature approval
decision still made and recorded individually, no all-or-nothing wave outcomes, and Architecture
Synthesis never indefinitely blocked by a feature that has been explicitly deferred or removed from
the cohort.

## Scope

**In scope**: `dba-system.md`'s Multi-Feature Architecture Synthesis Gate section (Wave Gate
definition; independent-justification + traceable-harmonization rules; progressive Row/Column Check
definitions and sequencing; advisory-but-not-ignorable wording; targeted reassessment via impact
assessment; versioned cohort membership for deferred/removed features); `01-intent.md`'s and
`02-contract.md`'s existing "Ambiguity Detection" sections (three response forms; `02-contract.md`'s
existing STOP-and-return-to-Stage-1 behavior stays unchanged). `03-event-schema.md` has no existing
equivalent section and is explicitly out of scope for the three-response-forms requirement.

**Out of scope**: any change to Controlled Plain English (`UPG-0057`) — CPE precision level is
explicitly not used as a draft/final-rigor proxy; any change to `tools/reviewer/src` — review
execution stays per-feature-per-stage, a current reviewer-architecture constraint documented as
operational guidance, not doctrine; any change to Architecture Synthesis's own 4-step pipeline
(`UPG-0051`/`UPG-0058`) beyond stating its existing Stage-3-across-the-cohort precondition against
versioned membership; any new Stage ID or new mandatory gate beyond the existing per-feature
Stage 1/2/3 gates.

## Value

Cuts review-session fragmentation for multi-feature cohorts from 3×N individual sessions to 3
batched sessions, without weakening the incremental-approval discipline that limits blast radius
when a Stage 1-3 assumption turns out to be wrong — and without the packet-budget or CPE-conflation
problems the original proposal would have introduced.

## Risk

Low-medium. No code — pure doctrine text. Main risk is under-specifying the reused patterns
(impact assessment, versioned membership) in a way that drifts from how they already work for
baseline/logical-design changes; mitigated by stating both as explicit extensions of the existing
rules, not new mechanisms.

## Guardrail

- No feature's Stage 1/2/3 approval is ever decided as part of a group; a Wave Gate batches the
  human's session, never the decision.
- A Column Check never runs before the stages it compares exist — no full cross-stage check before
  Stage 3.
- CPE precision level is never used as a proxy for draft-vs-final rigor.
- Packet-budget/reviewer-execution details stay out of `dba-system.md`'s normative text.

## Related

- Extends `UPG-0051`'s Multi-Feature Architecture Synthesis Gate (Cohort Check, versioned cohort
  membership) and `UPG-0058`'s Cohort Logical Design (versioned-impact-assessment pattern reused
  here one level down).
- Explicitly does not touch `UPG-0057` (Controlled Plain English) beyond citing why its precision
  level is the wrong knob for this problem.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the change
> records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260728-001 | `changes/UPG-0059__CHG-20260728-001__wave-gated-batch-review.md` | Wave-Gated Batch Review doctrine | COMPLETE |

### Reviews

| Review Series | Change ID | Step | Rounds | Accepted Verdict |
|---|---|---|---|---|
| RVS__UPG-0059__CHG-20260728-001__S1 | CHG-20260728-001 | 1-Intent | 2 | NO OBJECTION (R1 DO NOT ADVANCE — unrelated `UPG-0057` bookkeeping in the diff — split into standalone commit `28d934f`) |
| RVS__UPG-0059__CHG-20260728-001__S2 | CHG-20260728-001 | 2-Acceptance | 1 | NO OBJECTION |
| RVS__UPG-0059__CHG-20260728-001__S3 | CHG-20260728-001 | 3-Implement | 2 | NO OBJECTION (R1 DO NOT ADVANCE — Change Intent/AC11 still claimed `03-event-schema.md` was extended, contradicting Implementation Notes' own discovery — fixed) |
| RVS__UPG-0059__CHG-20260728-001__S4 | CHG-20260728-001 | 4-Reconcile | 2 | NO OBJECTION (R1 CHANGES ADVISED — backlog brief still had the stale `03-event-schema.md` claim; Findings table missing the Step 3 entry — both fixed) |

### Findings Tracked Inside This Feature

| Review Series | Classification(s) | Resolution |
|---|---|---|
| RVS__UPG-0059__CHG-20260728-001__S1 | 1 IN-SCOPE BLOCKER | See change record Reconciliation |
| RVS__UPG-0059__CHG-20260728-001__S3 | 1 IN-SCOPE BLOCKER | See change record Reconciliation |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | — | — |
