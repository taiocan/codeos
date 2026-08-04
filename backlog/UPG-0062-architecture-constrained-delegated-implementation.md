---
feature_id: UPG-0062
slug: architecture-constrained-delegated-implementation
title: Architecture-Constrained Delegated Implementation (Stage 4)
status: CLOSED   # premise test negative (2026-08-04); no delegated tooling built
priority: P2
depends_on: [UPG-0051, UPG-0058, UPG-0052]
related_features: [UPG-0060, UPG-0032, UPG-0056]
supersedes: []
superseded_by: []
---

# Upgrade: architecture-constrained-delegated-implementation

**Priority**: P2
**Status**: CLOSED — premise test negative (2026-08-04). No delegated tooling was built.

> **Outcome.** Q1 failed against a threshold precommitted before any measurement: producing EA-0004's
> Feature Implementation Design from approved artifacts cost **0.619** of implementing directly by
> output (≤0.40 required) and **0.802** weighted (≤0.50). The saving this feature depended on is not
> there. UPG-0060's apparent 10:1 leverage did not survive: that feedback was written with a failed
> candidate in hand and only had to name six deltas, whereas specifying a design from artifacts alone
> has no such shortcut — the bias AC-1 was written to catch, confirmed.
>
> **Q2 outlives the feature.** Ten of ten mechanism allocations were `NEW DESIGN`, and a shipped
> feature (EA-0001) already carries four such decisions in code alone. Codeos has a real gap between
> logical architecture and implementation. Filed as **UPG-0063**; deliberately not solved here.
**Type**: script-tooling + Rust engine (pilot first; no doctrine change proposed)

> **Relationship to UPG-0060.** UPG-0060 is closed and is not reopened by this feature. Its evidence
> is *prior evidence* for a different hypothesis. UPG-0060 tested whether a cheap model can derive an
> implementation from a behavioral contract. This feature tests whether it can execute an
> architecture that has already been derived and approved. Those are different questions, and
> UPG-0060's own final run is the reason to think the second may have a different answer.

## Problem

Stage 4 is the most generation-heavy step in the downstream loop, and Codeos has no way to move that
generation off Claude's budget. UPG-0060 tried and returned a negative result — but its final
measurement located the failure precisely enough to be actionable rather than merely discouraging.

## Evidence from UPG-0060

Three runs against EvidenceAtlas EA-0003 (approved Stage 1-3, 22 KB contract, 9 invariants,
2 falsification scenarios):

| Run | Setup | Result |
|---|---|---|
| 1 | Original harness | Did not compile; no manifest; 8 contract violations |
| 2 | Corrected harness (manifest permitted, layout exemplar, plain-text output, abstraction wording fixed) | **Compiled clean, zero edits**; correct module naming; real timestamps; **6 contract violations remained** |
| 3 | Run 2 + a Claude-authored feedback document naming each violation and the mechanism required | **6 of 6 violations fixed** — a genuine architectural response |

Run 3 produced a narrower `CoverageTargetType` making an illegal mapping *unrepresentable*;
`resolve_underlying` following `duplicate_of` chains transitively with cycle protection; a completion
function that takes the Plan and the started event and returns `Option`, refusing to emit for an
attempt that never started; and an opaque `QualityConcept` never branched on in domain logic.

**The generalizable finding:** the delegate executes a precise architectural specification well and
cannot derive one from a contract. What separated run 2 from run 3 was not model capability, prompt
tone, or harness mechanics — it was the presence of an explicit invariant→mechanism allocation.

Two honest caveats carried forward, not buried:

- Run 3 still did not compile unaided (a missing `Hash` derive — the same class of error as run 1).
  Mechanical defects persist and must be measured separately from architectural fidelity.
- The feedback that made run 3 work was ~2.5 KB and required reading EA-0003's contract closely
  enough to identify all six violations. **Whether producing that allocation costs less than writing
  the code is the open question this feature exists to answer** — see "Architectural question" below.

## New hypothesis

When a feature has approved Stage 1-3 artifacts, an approved Architecture Baseline, an applicable
Implementation Profile, **and an explicit allocation of its invariants to implementation
mechanisms**, an external model can produce the implementation with materially less Claude generation
work while preserving architectural intent.

The external model is a **constrained executor**, never an architect.

## Delegation boundary

| Claude / Codeos owns — never delegated | The external implementer may own |
|---|---|
| Interpreting behavioral contracts | Writing code that realizes the approved design |
| Architectural synthesis | Local implementation detail that changes no boundary |
| Component selection and boundaries | Mechanical completion of explicitly bounded source changes |
| **Allocating invariants to mechanisms** | — |
| Interfaces and ownership boundaries | — |
| Reconciliation and review | — |

The implementer must not invent, replace, simplify, or reinterpret approved architecture. Where the
approved artifacts are insufficient or contradictory it returns an explicit inability-to-implement
result rather than designing.

## Relationship to Architectural Synthesis (UPG-0051 / UPG-0058) — critical assessment

**The Architecture Baseline is *not* sufficient as an implementation specification.** This was checked
against the templates and against a real approved instance, not assumed.

`templates/architecture-baseline.md` (UPG-0051) carries crate/workspace topology, dependency
direction, shared infrastructure, integration style, persistence boundaries, and an Implementation
Profile reference. `templates/cohort-logical-design.md` (UPG-0058) adds the logical ERD, entity
ownership, identity strategy, module interface map, command/query responsibilities, transaction
boundaries, **validation ownership**, event-emission rules, and read-model design.

Together they answer **who owns what** and **where the boundaries are**. Neither answers **by what
mechanism a feature-local invariant is enforced** — and every one of EA-0003's six failures was a
mechanism question.

The proof is in the approved instance. EvidenceAtlas's Cohort Logical Design §8 says of exactly the
invariants that failed:

> "Where a classification concept's *ownership* is still open (EA-0003's quality/duplicate/
> stopping-basis…), consistent-application validation stays with the feature that applies the
> classification (EA-0003)…"

That correctly assigns *ownership* to EA-0003 and explicitly declines to specify the *mechanism*.
The cohort artifacts are working as designed; they are simply the wrong altitude. Searching that
36 KB approved document finds one mention of "duplicate", one of "quality", and zero of
`scope_fully_examined`.

**Smallest missing information:** a per-feature **Feature Implementation Design** — for each contract
invariant and falsification scenario, the mechanism that enforces it and where it lives (a narrowing
type, a derivation, a validation step, a construction-time guarantee).

**This is not a derived view — corrected at the Step 1 gate (human, 2026-08-04).** An earlier draft of
this brief called it one. That was wrong and the error mattered. When the approved architecture says
*where* an invariant belongs but not *how* it is enforced, choosing "a transitive resolver with cycle
protection" is a **new design decision**, not a restatement of anything approved. Presenting it as
derived would have created a second architecture layer that looked like it needed no governance
precisely because of what it was called.

The artifact must therefore keep two kinds of content visibly separate, per row:

| Classification | Meaning | Requirement |
|---|---|---|
| `SOURCE-DERIVED` | Traces to an approved artifact | Must cite artifact + section |
| `NEW DESIGN` | Not determined by any approved artifact | Must be marked as a design decision, never attributed to the Baseline |

Whether the `NEW DESIGN` content needs a governed home — an approval gate, a lifecycle, a place in the
artifact hierarchy — is an open question this feature must answer, not assume. See "Architectural
question" below.

**The larger possibility this exposes.** If Stage 4 routinely requires mechanism decisions that no
approved artifact determines, then Codeos has a real gap between logical architecture and
implementation that Claude has been **silently bridging inside the code** on every feature to date.
Those decisions are governed today only indirectly — the human approves the resulting code, never the
decision itself, and it is never recorded as a decision. That would make this feature valuable
independently of whether delegation is ever adopted.

## What the Stage 4 prompt gives Claude that no architecture artifact contains

`prompts/04-implement.md` supplies method that is nowhere in the architecture artifacts: the
structural-orientation blast-radius step, the Representation Ban's three-step procedure (identify
sites → choose one resolution strategy → apply uniformly), the Contract-to-Implementation Failure
Boundary with its two-approval rule and Failure Mapping Table, and correlation-ID-first ordering.
A delegated implementer receives none of this today. Any packet must carry the operative parts.

**Discovered issue, recorded not fixed:** `prompts/04-implement.md:102` states *"No additional
abstractions. If the contract doesn't require it, don't build it."* This is the same wording UPG-0060
identified as pushing an implementer away from invariant-carrying structure. Claude resolves the
tension against "every contract clause must be satisfied"; a weaker reader does not. Changing that
prompt is **downstream-doctrine and out of scope here** — recorded so it is not silently inherited.
The delegated prompt must not copy it.

## Tool architecture — Rust engine, Bash shim

The UPG-0060 prototype has the ownership backwards: `scripts/codeos-implement.sh` is 393 lines of
Bash doing packet construction, HTTP, parsing, path enforcement, and staging, while the established
pattern is `scripts/codeos-review.sh` at 106 lines shimming a 3,027-line Rust engine.

This feature moves the logic to Rust — configuration and validation, packet construction, artifact
resolution, allowed-path enforcement, request construction, HTTP, response parsing, candidate
staging, exit-code semantics, temp-file lifecycle, audit outputs — and reduces the shim to locating
the binary and forwarding arguments. The Bash prototype's *behavioral contract* (exit codes,
fail-closed ordering, staging layout, the delimited output protocol) is reusable evidence and should
be carried over deliberately; its implementation complexity should not be preserved to minimize a
diff.

**Guardrail:** no changes to `tools/reviewer`, no shared orchestration framework, no generalized HTTP
infrastructure, no CLI consolidation. If code later wants sharing with the reviewer, that is a
separate architectural opportunity to record, not to build here.

## Proposed pilot

**Feature: EvidenceAtlas `EA-0004` evidence_extraction.** Stage 3 approved, 15.8 KB contract,
11 KB schema, 16 invariants, 3 falsification scenarios, cohort member under the same approved
Baseline and Cohort Logical Design, Rust via the approved Implementation Profile.

**EA-0003 is deliberately not reused: it is contaminated.** The delegate was handed an explicit
specification of its six correct mechanisms during UPG-0060. Re-running it would measure recall of a
supplied answer, not derivation under constraint. EA-0004 is its direct downstream consumer —
comparable in kind, genuinely unseen.

**Arm A** — architecture-constrained delegation: packet carries approved intent, contract, schema,
Baseline, Cohort Logical Design, Implementation Profile, layout exemplar, invariant allocation,
allowed paths, forbidden changes, build command. Claude then reconciles normally.
**Arm B** — Claude implements the same feature by the normal Stage 4 process.

Both arms run to a reconciled, review-ready candidate. Neither is promoted without the human gate.

## Measurements

Reported separately; no single number stands in for the verdict.

- **Architectural fidelity** — compile success as delivered; architectural deviations; invariant
  violations; contract violations; invented behavior; omitted required behavior; unnecessary
  architecture changes. Mechanical quality is scored *separately* from architectural correctness,
  because UPG-0060 showed they move independently.
- **Reconciliation cost** — Claude input and output tokens; count and severity of corrections; and
  the decisive one: **whether corrections are local or architectural rewrites.**
- **External-model cost** — tokens and iteration count.
- **Total workflow cost** — delegation + reconciliation + review against normal implementation +
  review. Reduced Claude output tokens alone is explicitly not a success claim.

## Success criterion

Adoption requires **both**: architectural fidelity (the candidate follows the approved design without
Claude redesigning or substantially restructuring it) **and** material net benefit. A positive result
inside measurement noise is not sufficient — UPG-0060's repair arm already produced a marginal
positive that did not justify adoption.

## Abandonment criterion

Reject if Claude must reconstruct the architecture; if explicit invariants remain misplaced despite
being stated in the allocation; if reconciliation approaches the effort of direct implementation; if
total cost is not materially better; or if the governance and safety surface grows out of proportion
to the saving. **A negative result is an acceptable outcome and closes the feature.**

## Non-goals

Delegating architectural synthesis, Stage 1-3, or any human decision. Replacing the Codex reviewer.
Autonomous multi-stage execution. Stage 5 delegation — considered separately, and only after Stage 4
evidence exists. Changing DBA governance or weakening any human gate. Productionising delegation
before the pilot succeeds. Expanding the Rust work beyond this tool.

## Governance constraints

External output is a candidate only; the reviewer stays advisory; human approval stays authoritative;
no auto-advance; approved architecture outranks generated code; implementation may not silently change
approved architecture, and any architecture change discovered during implementation returns through
the existing governance path. **No second architecture authority is created in the prompt or the
tool.**

## UPG-0060 lessons carried forward

Manifests and supporting files permitted when the implementation requires them; layout exemplars
included; no JSON-escaping of source; file-based input to avoid the 128 KiB single-argument limit;
sufficient output-token capacity configured (UPG-0060's repair run truncated at 8,192 and staged
nothing); mechanical compile quality distinguished from architectural correctness; model capability
never inferred from harness defects; economic value never claimed without measuring reconciliation.

## The two questions Step 2 must answer

Revised at the Step 1 gate (human, 2026-08-04). Step 2 is a **premise test executed before any Rust
is written**, and it tests two things, not one.

**Q1 — Is producing the Feature Implementation Design materially cheaper than direct implementation?**

If deriving it requires the same close contract reading as implementing the feature, this hypothesis
collapses into UPG-0060's finding and the feature stops at Step 2 — cheaply, before any engine exists.

Evidence for: UPG-0060's run-3 feedback was ~2.5 KB and produced 610 lines (~25 KB) of correct Rust,
roughly 10:1. Evidence against, and this is the risk: that feedback was written **with a failed
candidate in hand**, which is strictly easier than writing a design from approved artifacts alone.
The premise test must therefore be run the hard way — EA-0004's design produced from approved
artifacts only, with no candidate available — or it measures the wrong thing.

**Q2 — Does Codeos need a governed home for mechanism decisions?**

Independent of delegation. If the design's `NEW DESIGN` rows turn out to be substantial, then Stage 4
has been making unrecorded architectural decisions on every feature, approved only implicitly via the
code they produced. That is worth knowing whether or not an external model is ever used, and it may be
this feature's most durable output.

Q2 must be answered before any Rust is built, not after. Building a delegation engine that consumes an
artifact whose authority and lifecycle are undecided would create exactly the ungoverned second
architecture layer this feature is supposed to avoid.

**Sequencing:** Q1 fails → stop UPG-0062. Q1 passes → answer Q2 and settle the artifact's authority
and lifecycle → only then build the engine.

## Related

- **UPG-0060** — prior evidence; closed, not reopened.
- **UPG-0051 / UPG-0058** — the approved architecture artifacts this consumes; assessed above as
  necessary but not sufficient.
- **UPG-0052** — Implementation Profile, supplying language and scope.
- **UPG-0032** — the reviewer's Rust engine, the reference pattern for tool architecture; explicitly
  not modified.
