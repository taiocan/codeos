---
feature_id: UPG-0073
slug: stage5-economic-qualification
title: Stage-5 Economic Qualification — Gemini-Assisted vs Direct Claude
status: PROPOSED — controls frozen 2026-08-23, neither arm run
priority: P2
depends_on: [UPG-0072]
related_features: [UPG-0060, UPG-0064, UPG-0066, UPG-0069, UPG-0071, UPG-0072]
supersedes: []
superseded_by: []
---

# Upgrade: Stage-5 economic qualification

## Question

UPG-0072 established that `gemini-3.7-flash` at `reasoning_effort: high` **can** produce usable
Stage-5 test suites: 3 of 3 usable, 8 of 8 applicable mutations killed. That is a capability result.
It says nothing about value.

> Does Gemini preserve enough Stage-5 quality while **materially reducing Claude supervision**
> compared with Claude doing the same work directly?

If the answer is no, there is little reason to keep investing in alternative-model delegation.

**Scope is Stage 5 only.** The reviewer alternative closed NO (UPG-0069, UPG-0071, UPG-0072) and
Stage-4 delegation closed NO (UPG-0064, UPG-0071, UPG-0072). Neither is reopened here. No new models,
providers, prompts, harness abstractions, or comparison roles. Delegated implementation stays
`status: disabled` by default, and no policy or default changes inside this experiment.

## Frozen controls — fixed 2026-08-23 before either arm ran

### Feature selection — by rule, not by expectation

Selected by an objective rule stated before any content was read: **every remaining PlotSpot feature
with an approved Intent, Contract, and Event Schema and a committed Stage-4 implementation,
excluding UPG-0072's three.** That rule yields exactly three, with no discretion exercised:

| Feature | Module | Intent |
|---|---|---|
| F-0002 | `source_snapshot` | Reproducible Source Snapshot and Provenance |
| F-0003 | `dataset_profile` | Measured Dataset Profile |
| F-0005 | `source_freshness` | Source Freshness Status |

F-0007 and F-0008 have specification artifacts but no Stage-4 implementation, so they are excluded by
the same rule rather than by choice. UPG-0072's F-0001, F-0004, and F-0006 are excluded because both
arms must work from specifications neither has seen exercised.

| | Value |
|---|---|
| Repository | PlotSpot at `c3b8215` |
| Frozen package, per feature | approved Contract + Event Schema, with the Stage-4 implementation as supporting context — identical bytes to both arms |
| Gemini configuration | `gemini-3.7-flash`, `reasoning_effort: high`, unchanged from UPG-0072 |
| Output bounds | default 32768, then **one** explicit 65536 retry after `length`; every attempt counts |
| Model repair iterations | none. Only the UPG-0072 mechanical repair boundary applies |
| Ladder | none. No second Gemini model, no other provider |

### Arm order — the only ordering that keeps the direct arm clean

**The direct-Claude arm runs first.** Claude is the direct arm, so once Claude has read a Gemini
candidate that arm can no longer be produced uncontaminated. Running Gemini first would make the
direct arm unmeasurable, whatever the bookkeeping said. The Gemini candidate is not read, listed, or
described until all three direct suites are written and committed.

### Repair boundary — carried over unchanged from UPG-0072

A mechanical repair may adapt generated test code to an **already-existing** public interface. It may
not create new observability, alter production behavior, or change the frozen mutation set. Adding a
missing required field is mechanical only if its value is irrelevant to the behavior under test; if
choosing it requires interpreting an approved artifact, that is semantic repair and the feature fails.

### Specification-drift rule — carried over unchanged

> A candidate assertion supported by the approved package is **not** wrong merely because the current
> implementation is nonconformant.

```text
candidate disagrees with implementation
  -> is the candidate supported by approved artifacts?
       NO  -> candidate defect
       YES -> implementation defect (not a candidate failure); record the drift
```

UPG-0072 showed this cuts both ways: a suite can pass baseline by *not testing* a drifted field.
Both arms are checked for that, symmetrically.

### Frozen mutations — authored from the approved Contracts alone, before either arm ran

Each is tied to an explicit approved rule. A mutation with no module-level injection point is recorded
**inapplicable** rather than scored, as UPG-0072's M3 was. Mutation validity comes from the approved
Contract, never from whether the committed direct-path suite dies; that suite's result is reported as
comparison only.

| # | Feature | Mutation | Approved rule it must violate |
|---|---|---|---|
| M1 | F-0002 | Accept a snapshot with no integrity fingerprint | Failure Path 3 `IntegrityFingerprintMissing` |
| M2 | F-0002 | Accept a snapshot with no retained evidence | Failure Path 4 `RetainedEvidenceMissing` |
| M3 | F-0002 | Collapse repeated retrievals into one preserved retrieval | Invariant: each preserved retrieval remains separate evidence; Falsification Scenario "Repeated Retrieval Collapsed" |
| M4 | F-0003 | Accept a measured finding with no source snapshot | Failure Path 1 `SourceSnapshotMissing` |
| M5 | F-0003 | Report a sampled or partial measurement as full-population | Invariant: known measurement outcomes remain distinguishable |
| M6 | F-0003 | Collapse measurement runs into one | Invariant: measurement runs remain separate evidence; Falsification Scenario "Measurement Runs Collapsed" |
| M7 | F-0005 | Accept a supported freshness status with no policy basis | Failure Path 5 `FreshnessPolicyBasisMissing` |
| M8 | F-0005 | Report conflicting freshness evidence as supported | Failure Path 8 `FreshnessEvidenceConflicting` |
| M9 | F-0005 | Normalize away meaning and scope differences in freshness comparability | Invariant: freshness comparability preserves differences; Boundary "Investor Sees Not Comparable Across Different Meanings" |

## Quality is judged before cost

A cost result for an unusable artifact is meaningless, so each arm is judged on quality first. Per
feature, each arm must:

1. represent the approved specification;
2. compile after only permitted mechanical repair;
3. identify implementation/specification drift rather than copying implementation behavior;
4. pass against conformant behavior;
5. kill the frozen applicable mutations;
6. require no semantic rewrite to become usable.

## Measurement — recorded per feature, never combined into a score

- Claude input and output tokens;
- Gemini tokens (delegated arm only);
- mechanical repair required, described by the **semantic work** it took, not by raw compiler-error
  count — UPG-0072's 112 errors collapsed to six repair classes, and the count was the less
  informative number;
- semantic defects or omissions;
- applicable mutations killed;
- usable without semantic rewrite: YES/NO;
- wall time, as secondary context only.

Claude usage is measured from actual Anthropic session usage over matched boundaries:

```text
Gemini arm      = first inspection of the Gemini candidate -> final usable/unusable judgement
Direct arm      = start of generation -> equivalent verification judgement

Excluded from both: experiment setup, API waiting, bookkeeping, cross-arm synthesis,
                    unrelated repository work.
```

**Primary decision metric: Claude tokens avoided per usable Stage-5 result.** Combined
Gemini + Claude token count is reported for cost visibility only and is never the objective.

## Decision rule — fixed before either arm ran

```text
Gemini quality materially below direct Claude
  -> reject Stage-5 adoption

Gemini quality acceptable, Claude usage not materially lower
  -> no practical value; keep disabled

Gemini quality acceptable AND Claude usage materially lower
  -> Stage-5 delegation has demonstrated value
     -> propose adoption separately, as its own smallest change
```

No policy, default, or activation status changes inside this experiment.

## After the experiment

If Gemini fails the value test, the alternative-model delegation line closes. If it passes, the
smallest Stage-5-only adoption change is proposed separately. A Stage-5 result is **not** generalized
to the reviewer or Stage-4 roles, both of which are closed NO.

## Results

Neither arm run.
