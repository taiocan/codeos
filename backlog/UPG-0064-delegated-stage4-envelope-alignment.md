---
feature_id: UPG-0064
slug: delegated-stage4-envelope-alignment
title: Delegated Stage-4 Envelope Alignment and Governed Pilot
status: IN_PROGRESS
priority: P2
depends_on: [UPG-0051, UPG-0052, UPG-0063]
related_features: [UPG-0060, UPG-0062, UPG-0056]
supersedes: []
superseded_by: []
---

# Upgrade: delegated Stage-4 envelope alignment and governed pilot

**Priority**: P2
**Status**: IN_PROGRESS — CHG-A at Step 2 (Acceptance Criteria)
**Type**: prompt + script-tooling (CHG-A), then a measured pilot (CHG-B)

## Problem — an integration defect, not an architecture-design problem

Three features built a governed envelope around Stage 4:

- **UPG-0051** — Architecture Synthesis Gate: a feature cannot enter Stage 4 without its cohort and a
  current approved Architecture Baseline. Protects cross-feature and structural architecture.
- **UPG-0052** — Implementation Profile: binding implementation constraints (language and scope), so
  an implementer cannot freely choose a different technical foundation.
- **UPG-0063** — Deferral → Resolution trace: when Stages 1-3 explicitly defer a material decision,
  Stage 4 must expose how it was resolved, whether it is interim, and what should supersede it.

**None of that reaches the delegated implementer.** Measured against
`prompts/codeos-implementer-task.md`, the only prompt the delegate receives:

| Term | Occurrences |
|---|---|
| Architecture Baseline | **0** |
| Implementation Profile | **0** |
| Cohort | **0** |
| Deferral / deferred | **0** |

Its output contract asks for three loose sections — `contract_satisfaction`, `event_emission`,
`notes` — and has no notion of a deferral trace. `scripts/codeos-implement.sh` compounds this by
labelling every input identically as `--- APPROVED ARTIFACT: <path> ---`, so a behavioral contract and
an Architecture Baseline are indistinguishable to the model. Only layout exemplars carry a distinct
label today.

UPG-0062 planned the prompt rewrite that would have fixed part of this and **closed on cost before
Step 3**, so it never happened. The envelope was completed; the delegated execution path was never
updated to carry it.

**Why this must be fixed before any pilot.** The Stage-4 reviewer checklist now asks for the deferral
trace (UPG-0063). Running a pilot today would hand the delegate a feature containing an explicit
deferral, never tell it that deferrals exist or that resolving one incurs an obligation, and then
measure whether it recorded the trace. It would not. That is a harness defect reported as a model
defect — the precise error UPG-0060 made, documented and corrected in
`changes/UPG-0060__CHG-20260803-002__premise-test-evidence.md`. Repeating it on the strength of our own
write-up would be worse than making it the first time.

## Why the question is worth reopening at all

UPG-0060 concluded that the delegate **cannot derive an architecture from a behavioral contract**.
That conclusion stands and is not being revisited.

This asks a different question: **can it operate correctly inside an architecture that is supplied,
labelled, and binding?** UPG-0060's own final run is the reason to think it might — given an explicit
specification of the required mechanisms, the delegate fixed 6 of 6 contract violations in one
iteration. The envelope UPG-0051/0052/0063 now provides is a *standing, governed* version of that
specification rather than a hand-written one.

## Authority boundary — the delegate produces a candidate, never the report

**DeepSeek returns candidate evidence:** code changes; contract-satisfaction evidence;
event-emission evidence; the architectural decisions it made; a Deferral → Resolution trace candidate
where applicable; and notes/uncertainties.

**Codeos/Claude assembles the authoritative Stage-4 Review Package** from that candidate plus
repository state.

This is deliberate. Making the delegate emit the canonical Stage-4 artifact would expand the
experiment from *"can a delegated implementer satisfy an approved design envelope?"* into *"can it
also correctly operate Codeos's governance and reporting protocol?"* Those are different questions
with different failure modes, and mixing them makes any failure harder to attribute.

The trace candidate must still carry the same semantic information UPG-0063 requires — it is candidate
evidence consumed at Stage 4, not the authoritative record.

## CHG-A — minimal harness alignment

**Artifacts carry their authority role**, replacing the flat `APPROVED ARTIFACT` label. Not decorative
metadata: the label tells the model *how* each input binds.

```
--- BEHAVIORAL CONTRACT: …        behavior that must be satisfied
--- EVENT SCHEMA: …               observable events that must be emitted correctly
--- ARCHITECTURE BASELINE: …      binding architectural constraint, not behavior to invent
--- COHORT LOGICAL DESIGN: …      binding shared design constraint
--- IMPLEMENTATION PROFILE: …     binding implementation constraint
--- LAYOUT EXEMPLAR: …            structural/context example, not authority
```

**The prompt imports UPG-0063's rule semantically:** if implementation resolves a material decision
that an approved upstream artifact **explicitly deferred**, report that resolution — using UPG-0063's
five fields. Do **not** report ordinary implementation technique choices, or matters that were merely
unspecified rather than explicitly deferred. Both exclusions carry over, and no phrase list is
normative.

Nothing else is added to CHG-A unless implementation reveals a genuine prerequisite.

## CHG-B — three-case Stage-4 pilot, contingent on CHG-A

Three deliberately different features, chosen so the cases fail differently:

| Case | Feature shape | What it tests |
|---|---|---|
| 1 | Almost no architectural freedom | Baseline competence inside the envelope |
| 2 | A real but bounded implementation choice | Whether the choice is made *within* the constraints |
| 3 | **Contains an explicit Stage 1-3 deferral** | Whether UPG-0063 works when the implementer is a different model |

Case 3 is the most valuable: it validates UPG-0063 prospectively against an independent implementer,
which is materially stronger evidence than the retrospective Q0 fixtures.

**Binding precondition (set by CHG-A, human 2026-08-04):** CHG-B must declare **every** governed
artifact with an explicit role flag. Positionals are compatibility-only and may not be used in the
pilot — otherwise the experiment would run through the degraded path CHG-A exists to remove.

**Measured — not "did it compile":** contract compliance; architecture violations; invented
requirements; missed explicit deferrals; unnecessary design decisions; Codex findings by severity;
reconciliation work required; human corrections required.

**The decisive metric is reconciliation burden.** UPG-0060 established that delegation can look cheap
while merely moving work from implementation into review and reconciliation.

**No autonomous Stage 4 → 5 → 6 loop.** The delegate produces artifacts; every existing gate applies
unchanged.

## Out of scope

**Stage 5 delegation.** It has a different failure mode — a model can implement a contract reasonably
while writing tests that confirm its own interpretation rather than falsify the contract. It opens as
a separate experiment only if Stage-4 evidence is positive, and would compare generated tests against
the approved contract's falsification scenarios.

Also out: any new architecture artifact or layer; autonomous multi-stage execution; changing any human
gate; making the delegate emit the canonical Stage-4 report; enabling the mechanism by default;
revisiting UPG-0060's "cannot derive architecture" conclusion.

## Success and abandonment

**Adoption evidence requires** the delegated candidate to satisfy the contract and respect the
architecture envelope **and** reconciliation burden to be materially lower than implementing directly.
Case 3 must specifically show the deferral was noticed and traced.

**Abandon if** the delegate violates the supplied architecture, misses the explicit deferral despite
being told the rule, or reconciliation approaches the cost of direct implementation. A negative result
closes the delegation question for good rather than prompting another harness round — CHG-A is the
last harness correction this line of work gets.

## Related

- **UPG-0060** — closed; established that architecture cannot be *derived* from a contract. Not revisited.
- **UPG-0062** — closed; its planned prompt rewrite never shipped, which is part of why this gap exists.
- **UPG-0051 / UPG-0052 / UPG-0063** — the envelope this makes visible to the delegate.
