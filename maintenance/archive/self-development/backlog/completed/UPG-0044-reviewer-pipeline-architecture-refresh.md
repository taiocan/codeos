---
feature_id: UPG-0044
slug: reviewer-pipeline-architecture-refresh
title: Refresh Reviewer Pipeline Architecture Documentation
status: COMPLETE
priority: P3
depends_on: []
related_features: [UPG-0042, UPG-0037, UPG-0027]
supersedes: []
superseded_by: []
---

# Upgrade: reviewer-pipeline-architecture-refresh — Refresh Reviewer Pipeline Architecture Documentation

**Priority**: P3
**Status**: PROPOSED
**Type**: documentation

## Problem

`docs/reviewer-pipeline.md` (520 lines, still headed `status: PILOT`) documents the reviewer
system as it stood around UPG-0003/UPG-0006. Since then the system has grown a layered
architecture that the doc does not narrate as a whole:

- **Two review domains** with different cadences: self-dev `PROFILE-0..5` (`prompts/codeos-self-dev.md`
  Step 0a) vs. downstream DBA's flat R1/R2/R3 cadence pinned by UPG-0037.
- **Evidence/packet modes** — full, delta, sha-only — added incrementally by UPG-0027's
  sub-changes and refined by UPG-0031 (delta-mode base-vs-working-tree fix) and UPG-0042
  (oversized-packet warnings, evidence-mode guidance).
- **Fail-closed packet coverage states** — `EMPTY_PACKET`, `SECRET_REDACTION`, `FULL`/`PARTIAL` —
  scattered across §4b/§4c/§14 rather than presented as one coverage model.
- **Record ownership split** across `changes/`, `backlog/`, `status/self-development.md`,
  `reviews/review-log.md`, and `reviews/codex/*` (the Self-Reference Boundary from UPG-0001),
  which is easy to re-derive wrong without a single picture.

A reader (including future Claude sessions) has to reconstruct the whole-system model by
reading `docs/reviewer-pipeline.md` plus `prompts/codeos-self-dev.md` plus several
`changes/UPG-0027__*` and `UPG-0042`/`UPG-0037` records. There is no single place that shows
how governance, packet-building, evidence modes, and durable records fit together.

## Upgrade

Refresh `docs/reviewer-pipeline.md` in place (no new standalone doc — avoids a second file
describing the same system) to add an architecture-level framing on top of the existing
how-to content:

- A four/five-layer model: human gate → workflow doctrine (self-dev loop vs. downstream
  DBA stages) → review engine (packet builder, evidence modes, Codex call) → durable
  records (change record, backlog feature thread, status dashboard, review log, raw
  Codex output).
- Human authority vs. advisory reviewer stated as the core rule up front (Codex produces
  advisory evidence; the human gate decides — already true today, currently implicit
  across several sections rather than stated once, clearly, near the top).
- Self-dev `PROFILE-0..5` vs. downstream flat R1/R2/R3 cadence, named as two distinct,
  intentionally non-unified cadences (cross-reference `prompts/codeos-self-dev.md` Step 0a
  and the UPG-0037 change record — do not re-litigate either).
- Packet/evidence modes (full / delta / sha-only) consolidated with a use-when / main-risk
  table, matching what UPG-0042 already added in §14 (extend, don't duplicate).
- `EMPTY_PACKET` fail-closed behavior and the delta-mode base-vs-working-tree fix (UPG-0031)
  stated as one coverage-state model.
- Diagrams (Mermaid, since this is a `.md` file rendered on GitHub/most viewers) for at
  least: the high-level flow, the packet-building/coverage-state flow, and the
  record-ownership split.
- Any "ReviewRun" / control-plane-lite / event-ledger framing is documented explicitly as
  **future direction, not implemented behavior** — clearly separated from the current-state
  sections so it cannot be read as describing shipped functionality.
- Drop or rewrite the `status: PILOT` framing where it is now stale, without re-litigating
  whether the Bash reviewer wrapper is still a "pilot" (that classification is out of scope
  here — flag it as a finding for a separate change if it looks wrong, don't resolve it in
  this change).

## Scope

`docs/reviewer-pipeline.md` — restructured/extended in place. Documentation only.

Out of scope:
- No new code (no changes to `scripts/codeos-review.sh`, `tools/reviewer/*`, or any binary).
- No new `ReviewRun` records, no event ledger, no generated review-log changes.
- No changes to `CLAUDE.md` or `dba-system.md` unless Step 1 explicitly reclassifies scope
  (not expected — this doc is self-dev only, see Related).
- No re-litigation of self-dev `PROFILE-0..5` or downstream R1/R2/R3 cadence design — cite,
  don't redesign.
- No new standalone `docs/reviewer-architecture.md` file — content lands in the existing doc.

## Value

Low-to-medium, cumulative. Benefits:
- Reduces re-derivation cost: a future session (self-dev or downstream) can read one section
  instead of reconstructing the layered model from `prompts/codeos-self-dev.md` + several
  change records.
- Makes the human-authority/advisory-reviewer rule and the two-cadence split explicit rather
  than implicit, reducing risk of someone conflating self-dev profiles with downstream
  cadence (or vice versa).
- Gives the packet/evidence-mode and coverage-state material a single narrative home instead
  of three partially-overlapping sections.

Trade-offs:
- Doc-only change; no functional benefit, no test coverage change.
- Risk of the doc drifting stale again if a future packet/cadence change forgets to update it
  (same risk that motivated this refresh in the first place — not newly introduced).

## Risk

Deciding hastily risks:
- Documenting the "ReviewRun"/control-plane-lite direction ambiguously, so a future reader
  mistakes aspirational architecture for current behavior.
- Duplicating rather than replacing existing §4b/§4c/§14 content, leaving the doc longer
  without being clearer.
- Silently expanding scope into `prompts/codeos-self-dev.md` or `CLAUDE.md` under the guise
  of "just adding a cross-reference."

## Guardrail

The refresh must:
- Keep `docs/reviewer-pipeline.md` as the single home for this content (no parallel doc).
- Clearly separate current-state sections from future-direction sections (label future
  direction explicitly, e.g. "Future direction — not implemented").
- Preserve every existing normative statement (I/O behavior, exit codes, fail-closed
  guarantees) — this is a reframing/consolidation, not a rewrite of behavior claims.
- Leave `CLAUDE.md` and `dba-system.md` untouched unless Step 1 explicitly says otherwise.

## Related

- **UPG-0042**: Reviewer packet efficiency — added the §14 evidence-mode material this
  change consolidates and extends.
- **UPG-0037**: Downstream default stage review — source of the downstream flat R1/R2/R3
  cadence this doc must describe accurately without re-litigating.
- **UPG-0027**: Replacing review scripts — source of the packet manifest / delta-mode /
  `EMPTY_PACKET` behavior this doc narrates.
- **UPG-0001**: Feature Thread Traceability — source of the Self-Reference Boundary /
  record-ownership split this doc's diagram must match.

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260712-001 | changes/UPG-0044__CHG-20260712-001__reviewer-pipeline-architecture-refresh.md | Refresh docs/reviewer-pipeline.md architecture framing | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0044__CHG-20260712-001__S1 | CHG-20260712-001 | 1-Intent | R1→R2 | CHANGES ADVISED → NO OBJECTION |
| RVS__UPG-0044__CHG-20260712-001__S2 | CHG-20260712-001 | 2-Acceptance | R1 | NO OBJECTION |
| RVS__UPG-0044__CHG-20260712-001__S3 | CHG-20260712-001 | 3-Implement | R1→R2 | CHANGES ADVISED → CHANGES ADVISED (budget exhausted, fixed inline) |
| RVS__UPG-0044__CHG-20260712-001__S4 | CHG-20260712-001 | 4-Reconcile | R1→R2 | CHANGES ADVISED → NO OBJECTION |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
