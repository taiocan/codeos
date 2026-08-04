---
feature_id: UPG-0063
slug: feature-implementation-design-layer
title: Expose Load-Bearing Implementation Decisions Inside Stage 4
status: PROPOSED
priority: P1
depends_on: []
related_features: [UPG-0062, UPG-0051, UPG-0058]
supersedes: []
superseded_by: []
---

# Upgrade: expose load-bearing implementation decisions inside Stage 4

**Priority**: P1
**Status**: PROPOSED
**Type**: to be settled at Step 1 — **the working hypothesis is that this needs no new stage, no new
artifact, and no new gate**

> **Origin.** UPG-0062's Q2 established the gap and was explicitly forbidden from solving it. This
> feature solves it — as leanly as possible.

## Hypothesis

> **Can Codeos expose only load-bearing `NEW DESIGN` decisions inside the existing Stage 4 workflow,
> at very low marginal cost, without creating a new stage, artifact, or gate?**

Only if that fails should a dedicated Feature Implementation Design layer be considered. The starting
position is deliberately the leanest thing that could work: a short **Implementation Decisions**
section in the Stage 4 output, reviewed by the **existing Stage 4 human gate**.

## Problem

Codeos's approved architecture artifacts determine **what** must be true and **who** is responsible.
They do not determine **by what mechanism** a feature-local invariant is enforced. Those decisions are
real, load-bearing, and currently recorded nowhere — approved only implicitly, by approving the code
that embodies them.

**Evidence** (UPG-0062, `changes/UPG-0062__CHG-20260803-002__premise-test-evidence.md`):

- Writing EA-0004's full implementation design from approved artifacts produced **ten mechanism
  allocations, all ten `NEW DESIGN`**. The artifacts fixed language, ownership, transactionality,
  correlation, event shapes, field lists and layout — and none of the ten mechanisms.
- EvidenceAtlas's Cohort Logical Design §8 assigns ownership of EA-0003's classification validation
  and **explicitly declines** to specify the mechanism. The artifacts work as designed; they are the
  wrong altitude for this.
- EA-0001, shipped and human-approved, carries a `ResearchContractValidator` seam, an `is_locked`
  predicate computing lockedness by conjunction of two injected decision references, and an
  `evaluate_change` gate. Its approved contract says "Locked" fifteen times as a *state* and never
  says how it is computed. None of these appear in any approved artifact.

**Existence, not prevalence.** One shipped feature plus one designed feature establish the gap is
real. They establish nothing about frequency. Step 1 should sample 2–3 implemented features before
deciding how heavy the remedy needs to be.

## The cost question — correcting an anchoring error

UPG-0062 measured a Feature Implementation Design at **62% of implementation cost**. That figure is
**not this feature's cost**, and carrying it over would misprice the whole thing.

UPG-0062 measured *deriving and writing a complete design, from approved artifacts, before
implementation*. Everything had to be specified, including mechanisms a competent implementer would
have chosen correctly unaided.

**Stage 4 makes these decisions anyway.** They are not optional work this feature adds; they are work
already happening, invisibly. The incremental cost here is therefore:

> **the cost of making an already-necessary decision explicit** — not the cost of designing it.

When Claude decides during implementation that lockedness is `brief-approval-reference ∧
research-began-reference`, recording it is one line:

> `Lockedness → conjunction of the injected brief-approval and research-began references; implemented in is_locked().`

That is a fundamentally different quantity from 62%, and it may be small enough to be free in
practice. **Step 1 must measure the recording cost specifically** — retrofit the record for one or two
already-implemented features and measure only the marginal effort — rather than reasoning from
UPG-0062's number.

## What gets recorded

**Only material `NEW DESIGN` decisions.** Not a row per invariant.

If an invariant's mechanism is already determined upstream, restating it adds nothing and creates
boilerplate that will drift. `SOURCE-DERIVED` remains useful as an *analysis* step — it is how you
decide whether something needs recording — but it should not become a permanent column that every
feature must fill in.

### The materiality test

A decision is recorded only if:

> **Would changing this mechanism, while preserving the same public behavior, materially affect an
> invariant, a boundary, the state model, data integrity, or future implementation freedom?**

If no, it is ordinary implementation detail and is **not** governed. This deliberately excludes loop
choice, helper structure, naming, collection types, error-message wording, and the rest of the long
tail that would otherwise flood the record and make it worthless.

### Shape (illustrative, not settled)

| Decision | Mechanism | Why material |
|---|---|---|
| Lockedness | conjunction of injected brief-approval and research-began references | Determines whether mutation is permitted |
| Duplicate coverage | resolve to distinct underlying source before counting | Prevents false coverage inflation |

Populated only when a genuinely load-bearing decision exists. **Empty is the expected common case**
and must never be rendered as an empty table or a "none" ceremony.

## Authority and conflict

The record is **subordinate to approved upstream artifacts**. It never becomes a second architecture
authority.

**On conflict:** if a recorded implementation decision conflicts with an approved artifact, that
conflict **must be reconciled** — it does not resolve itself by the approved artifact silently
winning, and the implementation decision may never override or reinterpret the approved artifact. A
conflict may well mean the implementation cannot legitimately continue until the upstream artifact is
amended through its own governance path. This follows Codeos's existing rule that unresolved conflicts
are surfaced to the human rather than silently resolved.

## Open questions for Step 1

- **Does the Stage 4 output already have a home for this?** Prefer extending an existing section over
  adding one. Check `prompts/04-implement.md`'s existing output format and the Stage 4 report template
  before proposing anything new.
- **What is the true marginal recording cost?** Retrofit EA-0001 and one other implemented feature;
  measure only the incremental effort of writing the record, not of making the decision.
- **How often is the record non-empty?** Feeds the prevalence question and determines whether this is
  a routine section or a rare one.
- **Does the existing Stage 4 gate suffice?** The hypothesis says yes. What would have to be true for
  it not to?
- **Does the reviewer need to know?** A recorded decision is checkable against the contract in a way an
  unrecorded one is not — possibly a free improvement to advisory review, possibly scope creep.

## Value

Makes load-bearing design decisions visible to the gate that already governs them, and legible to
whoever reads the code later. Improves reconciliation, review, and institutional memory. Independent
of delegation — worth doing whether or not any external model is ever used.

## Risk

**Over-engineering is the main risk, and the precedent is specific.** AJ-021: a request for a simple
on/off switch grew across seven review rounds into a versioned governance framework before a human
reset it. The finding here justifies *recording* material decisions. It does not by itself justify a
new stage, a new gate, a new template, or a mandatory artifact — and Step 1 should have to argue hard
for any of those against the lean default.

**Second risk: boilerplate.** A record that must be filled in for every feature becomes ritual, drifts
from the code, and ends up worse than nothing. The materiality test and the empty-is-normal default
exist to prevent that.

## Non-goals

A new DBA stage. A new approval gate. A mandatory per-invariant design document. Anything resembling
UPG-0062's full FID. Delegated implementation — that is closed and this feature is not a route back
to it.

## Related

- **UPG-0062** — discovered the gap; closed on cost. Its evidence file is the primary source, and its
  62% figure is explicitly *not* this feature's cost basis (see above).
- **UPG-0051 / UPG-0058** — the approved architecture artifacts this sits below.
- **AJ-021** — the cautionary precedent for keeping the remedy proportionate.
