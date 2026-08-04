---
feature_id: UPG-0063
slug: deferral-resolution-trace
title: Deferral → Resolution Trace for Stage 4
status: COMPLETE
priority: P1
depends_on: []
related_features: [UPG-0062, UPG-0051, UPG-0058]
supersedes: []
superseded_by: []
---

# Upgrade: deferral → resolution trace for Stage 4

**Priority**: P1
**Status**: COMPLETE — accepted 2026-08-04 (`CHG-20260804-001`)

> **Shipped.** `prompts/04-implement.md` Output Format item 5 (conditional; omitted entirely when
> nothing was deferred), plus one advisory Stage-4 reviewer question and its tests. All 17 acceptance
> criteria passed. Enforcement is deliberately the lean model: the Stage 4 author carries the
> obligation, the reviewer asks actively, and a missing record is a traceability defect rather than an
> implementation failure. No automatic deferral discovery, no new stage, gate, or standalone artifact.
>
> **Not yet proven in use.** No downstream feature has passed through Stage 4 under this obligation.
> Whether authors notice deferrals, and whether the reviewer's question surfaces omissions, is the
> evidence that would justify anything stronger — and nothing stronger was added pre-emptively.

**Type**: downstream-doctrine + script-tooling — the working hypothesis held: no new stage, no
standalone artifact, no new gate.

## Problem

Approved artifacts sometimes **explicitly defer** a design or behavioral question — they name it and
state that this artifact does not settle it. Stage 4 must resolve some of those deferrals for an
implementation to exist at all. Codeos has no governed way to record **how** a deferral was resolved,
**whether the resolution is temporary**, or **whether an upstream artifact must later be updated**.

The resolution happens regardless. It is simply invisible: approved implicitly, by approving the code
that embodies it, and indistinguishable afterwards from a decision nobody made deliberately.

## Evidence — Q0, and the correction that produced it

Full record: `changes/UPG-0063__Q0-classification-evidence.md`. Method and decision rule were
committed before any classification (`1b0dbd1`).

**This feature's original motivating claim was wrong and was retracted.** UPG-0062 reported that
approved artifacts "do not determine by what mechanism a feature-local invariant is enforced." That
rested on a method error — grepping contracts for the *implementation's* vocabulary (`is_locked`,
`newtype`), finding nothing, and reading absence of the code's names as absence of the rule. The
artifacts in fact determine a great deal: EA-0001's contract states the Locked rule outright at line
134. See `changes/UPG-0062__CHG-20260803-002__premise-test-evidence.md` §5.

Q0 re-ran the analysis properly across **two independent DBA projects** and found a narrower, better
grounded gap. The strongest cases are not ones where anyone inferred silence — they are ones where the
approved artifact **says it is leaving the question open**:

| Explicit deferral (quoted from the approved artifact) | Resolved in code as | |
|---|---|---|
| *"Validation ordering is not prescribed."* — PlotSpot `F-0001/2/3` schemas | First-failure-wins in a chosen order; fixes which failure signal a caller observes for multiply-invalid input | material |
| *"canonical ownership is unresolved until Architecture Synthesis."* — PlotSpot `F-0001/2/3` contracts | A hardcoded literal map inside the feature module; **nothing records that it is interim or must move** | material |
| *"MANUAL-PENDING: validator semantics — orchestration only"* — EA-0001 contract | A `ResearchContractValidator` trait seam | material |
| *"No evaluation order between `NoApprovedPlan` and `StalePlanVersion` is prescribed."* — EA-0003 schema | Not yet implemented — same pattern, pending | pending |

Three confirmed instances across four features and two projects, so this is a recurring pattern rather
than one project's habit. Correctly **excluded** by the same test: `is_locked` (the rule is stated
upstream), newtypes, helper seams, trait injection as technique, and the transaction boundary (stated
in the baseline).

## What a deferral is — semantic, never phrase-based

> An **explicit deferral** is a statement in an approved artifact that a specific design or behavioral
> question is deliberately left unresolved *by that artifact* — whatever wording is used. It names a
> question, asserts that the artifact does not settle it, and often indicates where or when it would
> be settled.

**Phrase search is discovery assistance, never the definition.** Scanning for *"not prescribed"*,
*"unresolved"*, *"left open"*, or *"MANUAL-PENDING"* is a useful way to find candidates, and that is
all it may ever be. If the definition were the phrase list, an author could write an equivalent
deferral in different words and bypass the mechanism entirely — governance that a synonym defeats is
not governance.

Two distinctions the definition must carry:

- **A deferral is not silence.** An artifact that simply never mentions a question has not deferred
  it. A deferral is an *affirmative* statement of non-resolution. This is what keeps the mechanism
  bounded: it does not require surveying everything an artifact failed to say.
- **A deferral is not implementation freedom.** An artifact that settles the behavior while leaving
  the technique open has deferred nothing. Choosing a `BTreeSet` is not resolving a deferral.

## Remedy — the Deferral → Resolution trace (shipped)

For each **material** explicit deferral that Stage 4 resolves, record only:

| Field | |
|---|---|
| Source artifact + deferral | which artifact, which question |
| Chosen resolution | what was decided |
| Where implemented | the function, type, or module |
| Final or interim | is this settled, or standing in for a future decision |
| If interim: expected superseder | which upstream decision or artifact should replace it |

Nothing else. **No survey of all invariants. No `SOURCE-DERIVED` inventory. No new architecture
document** — Step 1 confirmed none was necessary. Shipped as Output Format item 5 in
`prompts/04-implement.md`.

The last two fields carry most of the value. PlotSpot's hardcoded vocabulary map is an interim
resolution of a deferral whose own artifact says it is *"unresolved until Architecture Synthesis"* —
and nothing anywhere records that it must move when Synthesis lands. That is the failure mode this
feature exists to close.

**Materiality still gates entry.** Only deferrals whose resolution determines invariant placement,
component responsibility, state or data integrity, or future architectural freedom. If changing the
resolution would preserve public behavior and matter to nobody, it is not recorded.

**Empty is the expected common case** — most features defer nothing — and must never be rendered as an
empty table or a "none" ceremony.

## Authority and conflict

The trace is **subordinate to approved upstream artifacts** and never becomes a second architecture
authority. It records what was decided under an authority the artifact itself granted.

**On conflict:** if a recorded resolution conflicts with an approved artifact, that conflict must be
**reconciled** — it does not resolve itself by the approved artifact silently winning, and the
resolution may never override or reinterpret the artifact. A conflict may mean the implementation
cannot legitimately continue until the upstream artifact is amended through its own governance path.

## How the Step 1 open questions resolved

- **Does the Stage 4 output already have a home?** Yes — the existing Output Format. Placed as its own
  item because the Review Package's "Key architectural decisions" field covers precisely the *ordinary
  implementation choice* category; the prompt now routes technique choices back there so the two do
  not compete.
- **Who identifies the deferral, and can that be checked?** The Stage 4 author identifies it; the
  reviewer asks actively. **A missing record is not mechanically detectable** — detecting one needs
  deferrals enumerated, which without phrase-matching cannot be automated. Accepted at the gate as the
  right trade for a traceability obligation.
- **Does the existing Stage 4 gate suffice?** Yes; nothing stronger was added.
- **What is the marginal recording cost?** Not measured, and deliberately so — the question was moot
  once the shape settled at five conditional fields. It would matter again only if the obligation grew.
- **Do interim resolutions need a follow-up mechanism?** Not yet. Recording the expected superseder is
  the whole mechanism; whether anything must chase it is a question for evidence from use.

## Open — what use will decide

Nothing here is settled by the fact that it shipped. Whether authors notice deferrals, whether the
reviewer's question surfaces omissions, and whether interim resolutions actually get retired are all
answerable only after downstream features pass through Stage 4 under this obligation.

## Value

Makes deliberate resolutions visible as deliberate, and interim ones visible as temporary. Improves
review (a reviewer can check a resolution against the deferral that authorised it), reconciliation,
and institutional memory. Independent of delegation.

## Risk

**Over-engineering** — AJ-021's precedent: a simple on/off switch grew into a versioned governance
framework across seven review rounds before a human reset it. The evidence justifies *recording* a
handful of resolutions. It does not justify a new stage, gate, template, or mandatory artifact.

**Phrase-dependence** — the failure mode named above, and the reason the shipped prompt contains **no
phrase list at all**, not even as illustration. If the mechanism degenerates into grepping for "not
prescribed", it will be bypassed by paraphrase and will also fire on prose that merely contains the
words.

**Boilerplate** — a trace that must be filled in for every feature becomes ritual and drifts. The
materiality gate and empty-is-normal default exist to prevent that.

## Non-goals

A new DBA stage. A new approval gate. A standalone design artifact. A survey of all implementation
decisions. Anything resembling UPG-0062's Feature Implementation Design. Fixing the PlotSpot defect
found during Q0 — that is filed separately as
`PlotSpot/refinements/F-0001-known-access-form-canonicalization.md` and is PlotSpot's to triage.

## Related

- **UPG-0062** — closed on cost; its Q2 finding was retracted and re-derived here. Its evidence file
  carries the correction.
- **UPG-0051 / UPG-0058** — the approved architecture artifacts; assessed as carrying more
  architectural substance than the earlier analysis credited.
- **AJ-021** — the precedent for keeping the remedy proportionate.
