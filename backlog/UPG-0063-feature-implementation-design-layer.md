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

> **Origin.** UPG-0062's Q2 reported that this gap exists. **That finding was subsequently found to
> rest on a flawed method and has been downgraded** — see the correction in
> `changes/UPG-0062__CHG-20260803-002__premise-test-evidence.md` §5. The gap is now *unproven*, and
> smaller than reported if it exists at all. **Step 1's first job is therefore to establish whether
> there is a problem here at all**, not to design a remedy for one.

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

### State of the evidence — weaker than first reported

UPG-0062 originally reported the gap as established. A retrofit check found the method unsound: it
grepped approved contracts for the *implementation's* vocabulary (`is_locked`, `newtype`) and read the
absence of those words as absence of the rule. Corrected findings:

- **EA-0001: retracted.** Its contract explicitly states the Locked rule — "transitions to Locked only
  when both … are present and each is bound to the exact Research Brief version being locked"
  (line 134), with a version-binding falsification row at line 157. `is_locked` transcribes an
  approved rule. At most **one** candidate `NEW DESIGN` survives (the validator seam), not four.
- **EA-0004: unreliable.** The "10 of 10 `NEW DESIGN`" figure should not be relied on. Its contract
  specifies more than was credited — the distinct-underlying-source counting rule, the totality
  requirement over scope, and the representation-substitution invariant are all stated.
- **What may still stand:** a resolver seam where the contract explicitly leaves ownership open;
  choosing a narrowing type so an illegal state is unrepresentable rather than validating at runtime;
  enforcement-by-absence. These are plausibly decisions, not transcriptions — but that is now a
  hypothesis, not a finding.

**The honest position: the gap is unproven.** It may be real and small, or it may substantially
dissolve under a correct reading. Step 1 must settle that before anything is designed.

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

## Step 1 — establish whether there is a problem, before designing anything

**Q0, and it gates everything else: does a material `NEW DESIGN` decision actually exist in shipped
DBA code?**

Method, corrected from UPG-0062's:

1. For a sampled implemented feature, list the load-bearing mechanisms in the code.
2. For each, read the approved contract, event schema, Baseline and Cohort Logical Design **for the
   rule, in the artifact's own vocabulary** — never by searching for the implementation's names.
3. Apply the materiality test below.
4. Classify only what survives both.

Sample: EA-0001 (already partly done — one candidate survives), plus at least one feature from an
independent project. **PlotSpot** is available and unexamined: three implemented modules
(`source_inventory`, `dataset_profile`, `source_snapshot`, ~2,000 lines) against approved contracts
`F-0001`…`F-0006`. An independent codebase is worth more here than a second EvidenceAtlas feature,
since EvidenceAtlas's artifacts were largely authored in one style.

**If Q0 finds nothing material, UPG-0063 closes** — no gap, no remedy, and the recording-cost question
is moot. That is an acceptable and cheap outcome.

**If Q0 finds material decisions,** then and only then measure the recording cost and answer:

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
