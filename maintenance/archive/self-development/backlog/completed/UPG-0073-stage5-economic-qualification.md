---
feature_id: UPG-0073
slug: stage5-economic-qualification
title: Stage-5 Economic Qualification — Gemini-Assisted vs Direct Claude
status: DECIDED — Stage-5 delegation rejected (2026-08-23): quality 1 of 3 vs 3 of 3, Claude usage not materially lower; alternative-model delegation line closed
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

## Results — ran 2026-08-23

The direct arm ran first and to completion before any Gemini candidate was read, listed, or
described, so it is uncontaminated by construction rather than by assertion. All three Gemini
candidates completed on the first attempt at the default bound: 36,354 + 43,252 + 44,907 =
**124,513 Gemini tokens**.

### Quality, judged first

| | Direct Claude | Gemini-assisted |
|---|---|---|
| F-0002 | 10 tests, baseline pass, **3/3 mutations killed** | 15 tests, compiled as delivered, baseline pass, **3/3 mutations killed** |
| F-0003 | 10 tests, **3/3 killed**, and found a live implementation defect | compiled after mechanical repair, then **17 of 18 tests failed baseline** |
| F-0005 | 13 tests, baseline pass, **3/3 killed** | compiled after mechanical repair, then **20 of 22 tests failed baseline** |
| **Usable without semantic rewrite** | **3 of 3** | **1 of 3** |
| **Applicable mutations killed** | **9 of 9** | **3 of 3 on its one usable feature** |

### Why two Gemini features failed

Both failed for one reason, and it is the same reason V4-Pro failed two features in UPG-0071: the
approved Event Schema states `"event_id": "uuid-v4"`, and the F-0003 and F-0005 suites supply
readable ids such as `"evt-event-001"` and `"corr-event-001"`. The implementation enforces the rule
and returns `InvalidEventMeta("event_id must be uuid-v4")`. Supplying conforming values requires
interpreting the approved schema, which is **semantic repair** under the frozen boundary, so both
features fail rather than being repaired past it.

**This directly qualifies UPG-0072's uuid-v4 finding.** There, Gemini produced conforming uuid-v4
ids in both suites, and that was recorded as a real model-level capability difference over V4-Pro.
Here, on the same configuration and the same repair boundary, it produced conforming ids for F-0002
and non-conforming ids for F-0003 and F-0005 — **inconsistently, within a single run**. The UPG-0072
result was not wrong, but it was not reliable either, and reliability is what a delegated role needs.

The mechanical repair burden was again a small number of repeated interface misunderstandings, not
diverse semantic failures: the same six classes UPG-0072 recorded — `EventMeta.timestamp` →
`timestamp_ms`, private field reads → existing accessors, a pure forwarding `Store` impl for
`&mut T`, and borrow-structure adjustments. F-0002 needed none of them and compiled as delivered.

### What the direct arm found that the delegated arm did not reach

The direct F-0003 suite surfaced a **live implementation defect**: `event_log::derive_sibling_event_meta`
rejects any primary `event_id` whose final hex digit is odd, with "primary event_id must end with an
even hex digit when a sibling event is derived". The approved Event Schema constrains `event_id` only
to `uuid-v4`, whose final hex digit is random — so roughly **half of all schema-conformant event ids
are rejected** whenever a sibling event is derived, and nothing in the approved package states this
constraint. Under the frozen drift rule the assertion is supported by the approved package and the
implementation is independently nonconformant, so this is an implementation defect, not a suite
defect. It is isolated in its own test so it cannot mask the surrounding scenarios; that one test is
the single baseline failure in the direct arm's F-0003 column above.

This is PlotSpot's finding to act on, not Codeos's.

### Cost, judged second

Claude usage over the matched boundaries, from actual Anthropic session usage:

| | Direct Claude | Gemini-assisted |
|---|---|---|
| Turns | 61 | 25 |
| Output tokens | 72,786 | 21,350 |
| Fresh input | 122 | 50 |
| Cache creation | 133,389 | 25,675 |
| Cache read | 21,342,608 | 9,668,289 |
| **Usable results produced** | **3** | **1** |

The primary metric is Claude tokens avoided **per usable Stage-5 result**, which is the only
comparison that survives the arms producing different numbers of usable artifacts:

```text
Claude output per usable result     direct 24,262   gemini 21,350   ->  12% lower
Claude cache read per usable result direct  7.1M    gemini  9.7M    ->  36% HIGHER
Claude cache creation per usable    direct 44,463   gemini 25,675   ->  42% lower
```

Gemini tokens, reported for cost visibility only and never pooled with Claude figures: 124,513.

**Claude usage is not materially lower.** On the headline output measure the gap is 12% — inside the
noise of a three-feature sample — and on cached input the delegated arm is materially *worse* per
usable result, because two of its three candidates consumed judgement effort and produced nothing.

## Decision

```text
Gemini quality materially below direct Claude  ->  reject Stage-5 adoption
```

The first branch of the frozen decision rule fires on quality alone: 1 of 3 usable against 3 of 3,
with the direct arm additionally surfacing a live implementation defect the delegated arm never
reached. The cost result fails independently — Claude usage is not materially lower on any measure
that accounts for usable output, and is worse on one.

**Stage-5 delegation has not demonstrated value.** Delegated implementation stays `status: disabled`,
no policy or default changes, and no adoption is proposed.

### The honest reading of the two experiments together

UPG-0072 asked whether Gemini *can* do Stage 5 and got YES on 3 of 3 with 8 of 8 mutations killed.
UPG-0073 asked whether that is *worth anything* and got NO on both halves — quality did not hold on
unseen features, and supervision cost did not fall. The capability result was real; it simply did not
survive contact with features chosen by rule rather than already exercised.

The single most useful transferable finding is not about Gemini at all: a Stage-5 suite generated
from the current approved package finds specification drift that stale committed suites structurally
cannot see. That held in UPG-0071 (V4-Pro on F-0001), and it held again here — for the **direct**
arm, on F-0003. The value is in regenerating Stage-5 tests from the approved package, not in who or
what generates them.

## Closing the line

Per this experiment's rule and UPG-0072's frozen disposition, the alternative-model delegation line
is **closed**. The `CODEOS_LLM_PROVIDER` Gemini branch, its environment variables, and its tests are
removed; the evidence stays in this brief, in UPG-0072, and in Git history.

Reopening requires a new, specific hypothesis with its own measurement — not another harness round,
and not another model.
