---
feature_id: UPG-0063
slug: feature-implementation-design-layer
title: Governed Home for Feature-Level Implementation-Design Decisions
status: PROPOSED
priority: P1
depends_on: [UPG-0051, UPG-0058]
related_features: [UPG-0062, UPG-0052]
supersedes: []
superseded_by: []
---

# Upgrade: feature-implementation-design-layer

**Priority**: P1
**Status**: PROPOSED
**Type**: downstream-doctrine (likely) — scope to be settled at Step 1

> **Origin.** Discovered by UPG-0062's Q2 premise test (2026-08-04), which established the gap and was
> explicitly forbidden from solving it. This feature exists to solve it. UPG-0062's delegation
> hypothesis failed on cost and is closed; this finding is independent of that failure and outlives it.

## Problem

Codeos's approved architecture artifacts determine **what** must be true and **who** is responsible.
They do not determine **by what mechanism** a feature-local invariant is enforced. That decision is
real, load-bearing, and currently recorded nowhere.

**Evidence the gap exists** (UPG-0062, `changes/UPG-0062__CHG-20260803-002__premise-test-evidence.md`):

- Writing EA-0004's full implementation design from approved artifacts produced **ten mechanism
  allocations, all ten classified `NEW DESIGN`** — not determined by any approved artifact. The
  artifacts fixed language, ownership, transactionality, correlation, event shapes, field lists and
  layout, and fixed *none* of the ten mechanisms by which the invariants would hold.
- EvidenceAtlas's Cohort Logical Design §8 assigns ownership of EA-0003's quality/duplicate/
  stopping-basis validation and **explicitly declines** to specify the mechanism. The artifacts are
  working as designed; they are simply the wrong altitude.
- A shipped, human-approved feature already carries such decisions in code alone. EA-0001's
  `modules/research_brief/src/lib.rs` contains a `ResearchContractValidator` seam, an `is_locked`
  predicate computing lockedness by conjunction of two injected decision references, and an
  `evaluate_change` gate. Its approved contract says "Locked" fifteen times as a *state* and never
  says how lockedness is computed. None of these appear in any approved artifact.

So Stage 4 makes architectural decisions that no gate ever sees as decisions. They are approved only
implicitly, by approving the code that embodies them. A reviewer at Stage 4 sees the mechanism as an
implementation detail; a reader six months later cannot tell a deliberate design choice from an
incidental one; and a reconciliation cannot check a decision that was never stated.

**Scope of the evidence, stated honestly:** existence, not prevalence. One shipped feature plus one
designed-but-unimplemented feature establish that the gap is real. They do not establish frequency.
Step 1 should examine 2-3 representative implemented features before deciding how heavy the remedy
should be.

## Upgrade

Define the governed home for feature-level implementation-design decisions:

- **Where it sits** relative to Stage 3 (Event Schema) and Stage 4 (Implementation) — a Stage 3b, a
  Stage 4 precondition, or a section of an existing artifact.
- **What it contains** — at minimum, per contract invariant and falsification scenario, the enforcing
  mechanism, and a `SOURCE-DERIVED` / `NEW DESIGN` classification so a reader can tell a restatement
  from a decision.
- **Its authority** — explicitly subordinate to the approved Baseline, Cohort Logical Design, Contract
  and Event Schema. It must never become a second architecture authority; where it conflicts with an
  approved artifact, the approved artifact wins.
- **Its lifecycle** — when it is written, when it is revised, what happens when an approved artifact
  changes underneath it, and whether it is regenerable or hand-maintained.
- **Its approval semantics** — whether it needs a human gate of its own, or rides the Stage 4 gate
  with the mechanism decisions made explicit rather than implicit.

## Open questions for Step 1

- **Is a new artifact needed at all,** or does an existing one (Cohort Logical Design, or a Stage 4
  section) extend to cover it? Prefer extension over addition.
- **How heavy?** The gap is real but the remedy could be as light as a required "mechanism decisions"
  section in the Stage 4 output, recording what was decided and why. UPG-0056's lesson (AJ-021)
  applies: a rigorous framework here would be disproportionate to a problem that may be solved by a
  paragraph.
- **Does it apply to every feature, or only where invariant density warrants it?** Prevalence work
  feeds this.
- **What does it cost?** UPG-0062 measured a full design at 62% of implementation cost. A mandatory
  artifact that expensive would be a serious tax on every feature. A lighter record of only the
  `NEW DESIGN` decisions would cost far less — quantifying that is part of Step 1.

## Value

Makes load-bearing design decisions visible to the gate that is supposed to govern them, and legible
to whoever reads the code later. Independent of delegation: it improves reconciliation, review, and
institutional memory whether or not any external model is ever used.

## Risk

Over-engineering. The finding justifies *recording* mechanism decisions; it does not by itself justify
a new stage, a new approval gate, or a new template. AJ-021 is the cautionary precedent — a request
for a simple on/off switch grew into a versioned governance framework across seven review rounds
before a human reset it.

A second risk: cost. If the remedy approaches UPG-0062's measured 62%, it would be rejected on those
grounds alone. The remedy must be much lighter than a full design document.

## Blocking relationship

Any future delegated-implementation tooling (UPG-0062 CHG-C or a successor) is **blocked** on this
feature. A delegation engine consuming an experimental, non-authoritative design artifact would create
exactly the ungoverned second architecture authority this feature exists to prevent. UPG-0062 is
independently closed on cost, so this blocks nothing currently in flight.

## Related

- **UPG-0062** — discovered the gap; forbidden from solving it. Its evidence file is the primary source.
- **UPG-0051 / UPG-0058** — the approved architecture artifacts this sits below; assessed as necessary
  but not sufficient as implementation specifications.
- **AJ-021** — round-by-round review is blind to cumulative disproportion; relevant to keeping the
  remedy proportionate.
