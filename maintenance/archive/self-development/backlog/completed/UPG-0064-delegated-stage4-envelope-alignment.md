---
feature_id: UPG-0064
slug: delegated-stage4-envelope-alignment
title: Delegated Stage-4 Envelope Alignment and Governed Pilot
status: CLOSED   # CHG-A shipped; CHG-B ran 2026-08-21 and was negative
priority: P2
depends_on: [UPG-0051, UPG-0052, UPG-0063]
related_features: [UPG-0060, UPG-0062, UPG-0056]
supersedes: []
superseded_by: []
---

# Upgrade: delegated Stage-4 envelope alignment and governed pilot

**Priority**: P2
**Status**: IN_PROGRESS — **CHG-A COMPLETE** (accepted 2026-08-05); CHG-B not started

> **CHG-A shipped.** Six caller-declared role flags with zero authority inference; positionals
> preserved but visibly degraded as ROLE UNSPECIFIED; conflicting roles fail closed pre-network;
> labels byte-identical into the request; artifact content unmodified. The prompt states each role's
> authority and carries UPG-0063's deferral rule with an explicit anti-fabrication guard. 47 tests.
> The mechanism remains `status: disabled` and nothing has been run against a real model.
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
`dba/03-prompts/delegation/codeos-implementer-task.md`, the only prompt the delegate receives:

| Term | Occurrences |
|---|---|
| Architecture Baseline | **0** |
| Implementation Profile | **0** |
| Cohort | **0** |
| Deferral / deferred | **0** |

Its output contract asks for three loose sections — `contract_satisfaction`, `event_emission`,
`notes` — and had no notion of a deferral trace. `dba/04-tools/implementer/codeos-implement.sh` compounded this by
labelling every input identically as `--- APPROVED ARTIFACT: <path> ---`, so a behavioral contract and
an Architecture Baseline were indistinguishable to the model; only layout exemplars carried a distinct
label. *(Stated in past tense: CHG-A corrected both, 2026-08-04. The measurements above are the
pre-change state that motivated the feature.)*

UPG-0062 planned the prompt rewrite that would have fixed part of this and **closed on cost before
Step 3**, so it never happened. The envelope was completed; the delegated execution path was never
updated to carry it.

**Why this must be fixed before any pilot.** The Stage-4 reviewer checklist now asks for the deferral
trace (UPG-0063). Running a pilot today would hand the delegate a feature containing an explicit
deferral, never tell it that deferrals exist or that resolving one incurs an obligation, and then
measure whether it recorded the trace. It would not. That is a harness defect reported as a model
defect — the precise error UPG-0060 made, documented and corrected in
`maintenance/archive/self-development/changes/UPG-0062__CHG-20260803-002__premise-test-evidence.md`. Repeating it on the strength of our own
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

**CHG-B's first action, before any model is called (AJ-023):** a dry-run invocation in the pilot's
exact shape — every governed artifact declared by role flag, no positionals — against the stub
endpoint. CHG-A's Step 4 found that 45 passing tests coexisted with the tool being undrivable in
precisely that shape, because no test used it. The pilot must not discover this again with a real
model and real tokens.

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

## CHG-B case selection — frozen 2026-08-21, before any billed call

Source repository: EvidenceAtlas at `46e0a8bde9673cc8883c5d8c888fd856ca5f1379`. Delegate model
`deepseek-v4-flash` — recorded separately from UPG-0060's `deepseek-chat` evidence, never pooled
with it. All 14 EA contracts are `status: APPROVED`; only EA-0001 is implemented, so every case
below is genuinely unimplemented work.

| Case | Feature | Why this one satisfies the case rule |
|---|---|---|
| 1 — almost no architectural freedom | **EA-0010 policy_registry** | Its Dependency Role states `Artifact/data dependency: none`: no upstream feature's output feeds it, so structure is fixed by the contract rather than chosen by the implementer. |
| 2 — a real but bounded implementation choice | **EA-0003 corpus_construction** | The same feature UPG-0060's gate used, so quality is comparable against both historical arms under a changed model and a corrected harness. Its contract leaves bounded structural choice — historically 8 contract/schema violations. |
| 3 — an explicit Stage 1-3 deferral | **BLOCKED** | See below. |

**Case 3 — BLOCKED. Missing prerequisite: no approved downstream Specification Package carries a
material Stage 1-3 deferral that is still open at Stage 4 entry.** Every deferral located across
EvidenceAtlas and PlotSpot is one of two things that Stage 4 does not resolve:

- resolved at Stage 2 by the feature's own contract — EA-0002, EA-0005, EA-0008, EA-0009, EA-0010,
  EA-0012 all carry explicit Stage 2 resolution notes;
- explicitly routed somewhere other than implementation — EA-0011's Design Tensions #3-4 are left to
  the Contract Cohort Check or Architecture Synthesis Gate with the contract narrowed to a closed
  three-source set; EA-0013's open items are returned to Stage 1/2; EA-0014's AJ-008 item is a scope
  exclusion the feature does not implement. PlotSpot has no Stage-4 deferral and no Implementation
  Profile artifact at all.

Case 3 is therefore recorded as unrunnable with its reason, not replaced by a weaker case. It becomes
runnable when a downstream feature reaches Stage 4 with a material decision its own approved artifacts
explicitly defer to implementation.

## CHG-B result — ran 2026-08-21

Delegate `deepseek-v4-flash`, EvidenceAtlas at `46e0a8bde96`, every governed artifact declared by
role flag and no positionals. **AJ-023 precondition satisfied first**: the identical invocation shape
was dry-run against `stub-deepseek-server.py` and staged a candidate, and the preserved packet shows
all five labels present — `BEHAVIORAL CONTRACT`, `EVENT SCHEMA`, `PROJECT ARCHITECTURE`,
`IMPLEMENTATION PROFILE`, `LAYOUT EXEMPLAR`.

### What the corrected harness and stronger model did fix

Both candidates **compiled clean on first delivery** — correct module layout, a correct workspace
manifest, and, for EA-0010, all six schema events with the required base fields and a `uuid`
dependency the schema's `uuid-v4` `event_id` actually requires. Under `deepseek-chat` the same tool
produced a candidate that did not compile as delivered. The harness handicaps AJ-022 identified are
gone.

### What did not change

| Case | Feature | Codex review (stage 4) | Findings |
|---|---|---|---|
| 1 | EA-0010 policy_registry | `DO NOT ADVANCE`, evidence **A** | 3 IN-SCOPE BLOCKERS (2 High, 1 Medium) |
| 2 | EA-0003 corpus_construction | `DO NOT ADVANCE`, evidence **A** | 2 IN-SCOPE BLOCKERS (both High) + 1 pilot artifact |
| 3 | — | not run | BLOCKED, see selection above |

EA-0010's blockers are core behavior, not polish: lookup matches on target identity alone and
expressly ignores `scope` and `applicability`, so it cannot tell an applicable rule from an
inapplicable one and reports every same-target active version as a conflict; both governance-safety
guarantees ("never configures unexposed behavior", "never neuters a checkpoint") rest entirely on
caller-supplied assertions the module never establishes; and the contract's "prior versions always
remain available" has no persistence or reconstruction path. `LookupOutcome` also returns a private
type through a public API.

EA-0003's blockers are the same shape: `stopping_basis` can be emitted as `stopping_criteria_met`
while a criterion is unresolved, making the Execution Report internally contradictory; and the
observable start-before-completion invariant is not enforced by the recording API, since a caller
can discard the start event and append only completion. Its third finding — unrelated EA-0010 code
in the change set — is an artifact of both candidates sharing one scratch worktree, not a defect in
the candidate.

### Cost

| Case | Attempt 1 (32768) | Attempt 2 (65536) | Total | Wall |
|---|---|---|---|---|
| EA-0010 | 53,318, `length`, **0 output chars** | 76,477, `stop`, staged | 129,795 | 12m47s |
| EA-0003 | 55,486, `length`, **0 output chars** | 79,111, `stop`, staged | 134,597 | 12m36s |

264,392 DeepSeek tokens for two candidates, both `DO NOT ADVANCE`. Every attempt at the default bound
spent its entire completion budget on reasoning and returned no visible output at all.

### Measured against the corrected objective

- **A — Claude tokens spent:** measured for the pilot block as a whole, see UPG-0066's identical
  section; the two pilots ran interleaved and share one measured block.
- **B — quality:** neither candidate is usable without correction. Both fail on the invariant-dense
  core the approved contract exists to pin down.
- **C — Claude-token savings: UNKNOWN.** No comparable direct-Claude cost exists for EA-0010 or
  EA-0003 under this harness, and no counterfactual is inferred. Note separately that no Claude
  *output* tokens were displaced either, because no candidate could be adopted as delivered — that
  is an observation about B, not a measured saving.

### Verdict

**CHG-B is complete and negative.** The feature's own abandonment rule applies as written: "abandon
if the delegate violates the supplied architecture … or reconciliation approaches the cost of direct
implementation. A negative result closes the delegation question for good rather than prompting
another harness round — CHG-A is the last harness correction this line of work gets." Condition 0
(harness) and condition 1 (a materially stronger delegate model) have both now been satisfied, and
the result did not change: the delegate executes a specification well and does not satisfy one.
Case 3 remains unrun and is recorded as BLOCKED rather than assumed either way.
