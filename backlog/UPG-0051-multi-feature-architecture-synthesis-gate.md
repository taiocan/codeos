---
feature_id: UPG-0051
slug: multi-feature-architecture-synthesis-gate
title: Multi-Feature Architecture Synthesis Gate
status: COMPLETE
priority: P1
depends_on: []
related_features: [UPG-0052]
supersedes: []
superseded_by: []
---

# Upgrade: multi-feature-architecture-synthesis-gate — Multi-Feature Architecture Synthesis Gate

**Priority**: P1
**Status**: COMPLETE
**Type**: downstream-doctrine

## Problem

Downstream multi-feature projects (the originating case: EvidenceAtlas, a ~14-feature project
sharing canonical research artifacts, identity/version semantics, governance decisions, and
cross-feature event flows) have no doctrine mechanism to synthesize a shared workspace/crate/
event-transport model from cross-feature evidence. `dba-system.md` permits horizontal Stage 1–3
scheduling per feature, but nothing stops one feature's Stage 4 implementation from silently
becoming the architectural precedent every later feature in the same cohort inherits — workspace,
crate boundaries, and shared infrastructure end up extrapolated from one early implementation
(and one partially-implemented feature) rather than from the complete approved behavioral and
event topology. Confirmed by direct read of `dba-system.md`: there is no cohort gate, no
Architecture Baseline concept, and no cross-feature review mechanism beyond the existing (optional)
Architecture Journal and the Feature Registry.

## Upgrade

This brief proposes direction to carry into Step 1 of the 4-step self-development loop; several
mechanics are deliberately left open for Step 1/2 rather than pre-decided here (see "Not decided by
this brief" below), following the same pattern `UPG-0049` uses.

### Proposed direction

- **A new conditional structural approval gate**, not mere coordination. An approved Architecture
  Baseline becomes required (alongside Intent/Contract/Event Schema) for Stage 4 eligibility within
  a declared core cohort. The reviewer stays advisory; **human approval is what gives the baseline
  authority.**
- **Narrow subordination clause**, added to `dba-system.md`'s existing Truth Authority and Conflict
  Resolution list (`dba-system.md:19-28`) — one addition, not a competing hierarchy: "The approved
  Architecture Baseline is authoritative only for project-level structural decisions not fixed by
  approved behavioral artifacts. It never overrides Intent, Contract, Event Schema, explicit human
  correction, or safety/authorization invariants. Conflicts with runtime evidence are handled
  through the existing reconciliation rules; runtime behavior does not silently amend the
  baseline." This does not decide Intent/Contract/Event Schema's precedence relative to each
  other — that stays out of scope. Conflicts between the baseline and behavioral authority resolve
  through the *existing* Truth Authority rules, not a new blanket "behavioral artifacts always win"
  restatement — rule 2's runtime-can-override-intent-text nuance must not be flattened.
- **Core cohort test**: a core cohort contains two or more features whose independent
  implementation choices could *materially constrain* each other's canonical ownership, dependency
  direction, persistence boundary, integration contract, shared infrastructure, or deployment
  topology. Merely sharing infrastructure (runtime, persistence) is evidence to inspect, not
  automatic inclusion — a broader "shares runtime/persistence/artifacts/events" test would pull
  nearly every feature into one cohort and make the gate universal rather than conditional. Cohort
  declaration must specify who declares it, where it's recorded, whether features can be
  added/removed, and whether a project may have multiple cohorts.
- **Leaner cohort review model**: Intent Cohort Check and Contract Cohort Check are *recommended*
  after their respective waves (duplicate outcomes, missing/circular ownership, inconsistent
  actors; canonical ownership, lifecycle/failure consistency, circular preconditions). Event
  Cohort Check is *required* as part of the final synthesis input (event ownership, envelope
  uniformity, correlation, observational-vs-integration classification). **Architecture Baseline
  approval is the single new mandatory project-level gate** — not four separate mandatory gates.
- Baseline distinguishes **authoritative decisions** (manually approved) from **derived
  views/matrices** (ownership matrix, dependency graph, producer/consumer matrix — regenerable,
  carrying provenance to source artifacts) — never a second canonical model that can silently drift
  from the approved artifacts it was built from.
- Baseline has **identity and version, and so does cohort membership**: a baseline approves a
  *specific versioned cohort membership set*. Adding or removing a feature does not silently
  rewrite that cohort. A material membership change creates a new cohort/baseline version and
  triggers impact assessment; it does **not** retroactively invalidate work already performed under
  the previously approved version unless the assessment finds an actual conflict — this is what
  keeps the gate from becoming an open-ended freeze where discovering one new core feature voids
  every in-flight Stage 4. Approved baseline versions aren't silently rewritten; a replacement
  supersedes and is recorded; implementations record which baseline version governed them; Stage 10
  may amend/supersede structural decisions only when behavior is unchanged; a behavioral change
  always returns to the affected feature's earlier stage.
- Naming stays **"Architecture Synthesis Gate" / "Core Architecture Baseline"** — deliberately not
  "Architecture Discovery," kept distinguishable from the existing optional, non-authoritative
  pre-Stage-1 Discovery session type (`00a-solution-discovery.md`), whose output is never approved
  architecture.

### Not decided by this brief — questions for the implementer to resolve

- Where cohort declaration + gate status actually live — candidates: extend
  `features/registry.yaml` (already the recommended multi-feature index), a new
  `architecture/cohorts.yaml`, or the baseline artifact plus a pre-baseline declaration record.
  Whatever is chosen, Stage 4 entry and session-start must be able to check cohort membership,
  baseline existence/approval, and applicable version.
- How the reviewer covers a project-level gate with no Stage ID or feature ID of its own — options:
  a project-level reviewer command, a recognized stage identifier (e.g. `architecture`), a
  synthetic cohort identifier, or an explicit tooling deferral + waiver until support lands.
- Exact baseline artifact schema fields beyond the versioning requirements above.
- Exact content of the new Architecture Synthesis prompt — it must load cohort Intent/Contract/
  Event Schema + Architecture Journal entries, separate derived observations from decisions,
  produce the baseline, return behavioral gaps to earlier stages, and stop for human approval —
  mechanically modeled on Stage 10's own step-gated loop.

## Scope

`dba-system.md` (new section + Stage 3→4 cross-reference + Stage 10 clarification), a new
Architecture Synthesis prompt (e.g. `prompts/03b-architecture-synthesis.md`), a baseline template,
cohort declaration/state mechanism (location per open question above), a Stage 4 applicability
check, a session-start awareness update, reviewer/review integration or an explicit documented
deferral, and regression tests confirming the gate is actually wired — not just documented (this
proposal is itself a reaction to `patterns/rust-project-structure.md` being orphaned from the
doctrine/prompt chain; the same mistake should not repeat here).

Out of scope: any Rust-specific content (→ `UPG-0052`); no new numbered stage inserted into the
per-feature 9-step loop — this is a cohort-level gate sitting between cohort Stage 3 and cohort
Stage 4, not Stage 3.5 for every feature.

## Value

Converts workspace/crate/event-transport architecture decisions for a coupled multi-feature project
from an extrapolation based on one early implementation into a synthesis based on the complete
approved behavioral and event topology — directly addressing the risk that observed in EvidenceAtlas
(one feature implemented, thirteen still at brief stage, a shared-crate proposal inferred mainly
from the one implementation and an assumption about a second feature's needs).

Trade-offs: adds a mandatory project-level pause for declared core cohorts — slower time-to-first-
Stage-4 for cohort features, and real review overhead even in the leaner one-mandatory-gate model.

## Risk

Deciding hastily risks:
- Turning DBA into a horizontal waterfall if the cohort test is under-enforced — reviewing dozens
  of artifacts before any runtime feedback, early schema mistakes propagating across the whole
  cohort, cross-feature assumptions becoming mutually reinforcing.
- An inert gate: doctrine text with no working cohort-state mechanism, reviewer integration, or
  wired prompt is exactly the failure mode that orphaned `rust-project-structure.md`.
- An open-ended freeze if cohort membership isn't versioned — one newly discovered core feature
  could otherwise be read as invalidating every already-approved baseline and in-flight Stage 4.

## Guardrail

- Conditional only, under the stronger cohort test — never a universal gate for single-feature or
  loosely-coupled/plugin-style projects.
- The baseline may constrain implementation structure but never invent or alter behavior; any
  discovered behavioral gap returns to the owning feature's earlier stage, never patched in the
  baseline directly.
- Baseline authority is strictly subordinate to `dba-system.md`'s existing Truth Authority and
  Conflict Resolution rules — this upgrade adds one clause to that list, it does not redefine it.
- Cohort membership is versioned; a later membership change does not retroactively invalidate prior
  approved work absent an actual conflict.

## Related

- **UPG-0052**: Implementation Profile Framework — an independent integrating peer, not a
  dependency in either direction. Architecture Synthesis *may optionally consume* an approved
  Implementation Profile when one exists.
- **UPG-0054**: Contract-to-Implementation Failure Boundary — a related but independent Stage 4/5
  guidance item discovered during the same review.
- Proposed by the human during a 2026-07-19 discussion of EvidenceAtlas's architecture sequencing
  (see `reviews/review-log.md`).

## Feature Thread

> Canonical thread rollup for this feature. Compact links/IDs only; full detail lives in the
> change records and review files. May be maintained manually.

### Changes

| Change ID | File | Purpose | State |
|---|---|---|---|
| CHG-20260719-001 | `changes/UPG-0051__CHG-20260719-001__architecture-synthesis-gate.md` | Add the Architecture Synthesis Gate doctrine, prompt, template, and registry schema | COMPLETE |

### Reviews

| Review ID | Change ID | Step | Round | Verdict |
|---|---|---|---|---|
| RVS__UPG-0051__CHG-20260719-001__S1 | CHG-20260719-001 | 1-Intent | R1→R2 | R1 DO NOT ADVANCE → R2 NO OBJECTION |
| RVS__UPG-0051__CHG-20260719-001__S2 | CHG-20260719-001 | 2-Acceptance | R1→R2 | R1 NO OBJECTION → R2 NO OBJECTION (post CHANGES-ADVISED revision) |
| RVS__UPG-0051__CHG-20260719-001__S3 | CHG-20260719-001 | 3-Implement | R1→R2 | R1 DO NOT ADVANCE → R2 NO OBJECTION |
| RVS__UPG-0051__CHG-20260719-001__S4 | CHG-20260719-001 | 4-Reconcile | R1 | NO OBJECTION — ACCEPTED |

### Findings Tracked Inside This Feature

| Finding ID | Review ID | Classification | Resolution |
|---|---|---|---|
| (Step 1) reviewer-deferral claim unverifiable in packet | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — reframed as author finding, re-checkable at Step 4 |
| (Step 1) "regression tests" scope conflict | RVS__…__S1 | IN-SCOPE BLOCKER | Fixed — clarified as Step 4 grep-based wiring verification |
| (Step 3) version-history check underspecified | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — exact naming convention defined (current file vs. `history/core-baseline-v<N>.md`) |
| (Step 3) undeclared registry `notes` field | RVS__…__S3 | IN-SCOPE BLOCKER | Fixed — field removed |
| (Step 4) live eligibility could accept a historical version | pre-Step-4 human note | IN-SCOPE BLOCKER | Fixed — live check pinned to current version only; historical files reframed as provenance-only |
| Native `codeos-reviewer` support for `architecture-synthesis` stage id | RVS__…__S1, reconfirmed S4 | OUT-OF-SCOPE BACKLOG | Not filed as a new UPG; noted for visibility, deferred via Review Waiver |

### Follow-up Features

| Feature ID | Reason | Source finding |
|---|---|---|
| — | Native reviewer stage-id support noted but not yet spun into its own UPG | Step 1 / Step 4 findings above |
