---
feature_id: UPG-0071
slug: deepseek-v4-pro-requalification
title: DeepSeek V4-Pro Requalification Across Three Codeos Roles
status: DECIDED — NO / NO / NO for V4-Pro-at-max (2026-08-21); Stage-5 diagnostic appended, verdict unchanged
priority: P2
depends_on: []
related_features: [UPG-0060, UPG-0064, UPG-0066, UPG-0069]
supersedes: []
superseded_by: []
---

# Upgrade: DeepSeek V4-Pro requalification

## Question

> Is `deepseek-v4-pro` at its recommended maximum reasoning setting good enough to use for these
> Codeos roles?

Not *"which of model tier or reasoning effort caused any change?"* — Flash ran at
`reasoning_effort: high` and Pro runs at `max`, so both move together and every result is attributed
to the **tested configuration**, never to model tier alone. No Flash-at-max arm is run: it would cost
real attempts to answer a question no decision depends on.

A negative result is strong evidence to retire DeepSeek for that role. A positive result establishes
V4-Pro-at-max as viable and claims nothing about the cause.

Three roles are decided **independently**. `NO / YES / YES` is a valid and useful outcome — DeepSeek
does not have to win every role to be worth using in one.

## Frozen controls — fixed 2026-08-21 before the first billed call

| | Value |
|---|---|
| Model | `deepseek-v4-pro`, official API, `thinking: enabled`, `reasoning_effort: max` |
| Output bounds | default 32768, then **one** explicit 65536 retry after `length`; every attempt counts |
| Repair iterations | none — Flash's Stage-4 cases were single-shot, so Pro is too |
| Stage-4 cases | EA-0010 policy_registry, EA-0003 corpus_construction (EvidenceAtlas `46e0a8bde96`) |
| Stage-4 case 3 | **BLOCKED**, unchanged — no approved package carries a Stage 1-3 deferral still open at Stage 4 entry, and no easier case substitutes |
| Stage-5 features | F-0001 source_inventory, F-0004 source_fitness, F-0006 regional_availability (PlotSpot `c3b8215317b`) |
| Reviewer packet | `canonical-packet.txt`, `sha256 2a5ed7d4…6894f`, byte-for-byte |
| Harness | unchanged since `f206cb9`, which predates every Flash run |
| Codex | **not re-run** for the reviewer comparison; its recorded arm is reused |

Flash evidence is added to, never overwritten. `deepseek-chat`, V4-Flash, and V4-Pro figures are
never pooled.

### The one harness change

`reasoning_effort` was a literal `"high"` in both adapters and is now read from
`CODEOS_DEEPSEEK_REASONING_EFFORT`, defaulting to `high`. Nothing else changed.
`codeos-implement-tests.sh` asserts `.reasoning_effort == "high"` and runs without the variable set,
so it stays green as the guard that the default did not move — 51 implementer tests and the reviewer
adapter suite pass unchanged.

## Frozen Stage-5 mutations — fixed before any Pro candidate was read

UPG-0066 fixed "falsifier or mutation evidence" as a measure, but no concrete mutations were ever
authored because Flash produced nothing to test. These are derived from the approved contracts alone.
Authoring them now does not disadvantage Flash: Flash failed earlier, at completion, so its result
stays "completion failure — no candidate available for quality evaluation" and no retroactive test
can change it.

A candidate test suite must **pass** against the real implementation and **fail** against every
mutation below. Each mutation is first validated against the committed direct-path tests: a mutation
those tests do not catch is an inert mutation and is reported as such rather than counted against the
candidate.

| Feature | Mutation | Approved rule it must violate |
|---|---|---|
| F-0001 | Accept a candidate whose `responsible_organization_evidence_refs` is empty | Failure Path 2 `OfficialResponsibilityUnsupported` |
| F-0001 | Drop the `candidate_dataset_evidence_refs` completeness guard | Failure Path 4 `CandidateIdentityIncomplete` |
| F-0001 | Return candidates from every country in a country-scoped view | Invariant: every inventory view is scoped to a selected country |
| F-0004 | Report `LIMITED` decisions as `ACCEPT` | Invariant: ACCEPT/LIMITED/DEFER/REJECT stay distinct and are not collapsed |
| F-0004 | Emit a supported decision without its sparse-data threshold | Failure Path 5 + invariant: every supported decision shows its threshold |
| F-0004 | Overwrite prior decision fields on revision instead of preserving them | Boundary: reassessment preserves the prior decision |
| F-0006 | Allow `available` without an ACCEPT or LIMITED fitness decision for the same scope | Failure Path 11 + invariant 3 |
| F-0006 | Skip the upstream approval check so unapproved artifacts project | Failure Path 10 + invariant 2 |
| F-0006 | Normalize away freshness / measurement-basis differences in region comparison | Boundary: region comparison preserves evidence differences |

## Measurement

Per attempt: returned model identity, prompt / completion / reasoning tokens, **final-content
tokens** (completion minus reasoning — zero for every Flash Stage-5 attempt), finish reason, wall
time.

Claude cost is measured from actual Anthropic session usage with a narrow boundary:

```text
Claude supervision cost =
  Claude usage from the first inspection of the DeepSeek candidate
  through the final usable/unusable judgement.

Excluded: experiment setup, API execution and waiting, bookkeeping,
          cross-experiment synthesis, unrelated repository work.
```

Engineering quality, usability without rewrite, Claude supervision cost, DeepSeek token cost, and
wall time are reported separately and never collapsed into one score. Where no credible
direct-Claude baseline exists, `Claude-token savings: UNKNOWN`; nothing is inferred from a
counterfactual.

## Results — ran 2026-08-21

Every one of the eleven attempts below returned `model: deepseek-v4-pro` with `reasoning_effort: max`;
no attempt silently fell back to another model.

### Arm 1 — reviewer (UPG-0069 DeepSeek arm re-run; Codex not re-run)

Same `canonical-packet.txt`, hash verified before and after. **`stop` on the first attempt at the
default 32768 bound**, where Flash needed the 65536 retry: 38,295 prompt, 30,747 completion, 29,363
reasoning, **1,384 final-content tokens**, 69,042 total, 7m54s.

| Measure | Flash | **Pro-at-max** | Codex |
|---|---|---|---|
| True findings | 1 | **2** | 3 |
| False positives | 0 | 0 | 0 |
| Missed important defects | both integrity defects | **both integrity defects** | none |
| Protocol compliance | FAILED | **FAILED** (2 findings unrecorded) | PASSED |
| Completion | 2 attempts | **1 attempt** | 1 attempt |
| Cost | 149,975 tokens, ~12 min | **69,042 tokens, ~8 min** | 57,958 tokens, ~3 min |
| Decision usefulness | record not self-sufficient | **record not self-sufficient** | self-sufficient |

Pro found the `--packet` defect Flash and Codex both found, and **one defect neither reported**:
`contract/v4.md` claimed the import path "applies the same evidence selection, packet construction …
as `review`" while the implementation requires `--packet` and adopts the exported bytes without
reconstructing anything — verified at packet line 193, and since repaired at HEAD. It also reported
that the tracked diff was not self-contained (`run.rs` untracked while `main.rs` adds `mod run;`).
That is factually true of the reviewed dirty-tree state and qualifies under the triage rule's
"prevents the work from running", but it reflects the review snapshot rather than the shipped
artifact, and is recorded here as such rather than counted as a third defect.

Two failures decide this arm. First, **protocol**: Pro used a multi-line
`Finding: … / Severity: … / Classification: …` block, the parser rejected both findings, and the
record is `parse_status: FAILED`, `assessment_status: INCOMPLETE`, `findings: []` — a different
formatting mistake from Flash's decorated header, with the same consequence. Second, and worse,
Pro **affirmatively certified the two properties that were actually broken**, listing "recorded
packet is the exported packet byte-for-byte" and "untracked files are shown or downgrade coverage"
among the supported claims. Those are exactly the two live integrity defects the Codex arm found
(now UPG-0070). A reviewer that misses a defect is weak; one that certifies the broken property is
worse.

**Reviewer fallback: NO.** Better than Flash on completion and on findings, still unusable as a
review record.

### Arm 2 — Stage 4 (UPG-0064 frozen cases)

Both cases: `length` at 32768, then `stop` on the single permitted 65536 retry, then a clean build
first try in an isolated worktree — one worktree per candidate, so the cross-module scope finding
Flash's EA-0003 review raised cannot recur.

| Case | Attempt 1 | Attempt 2 | Total | Wall | Build | Codex review |
|---|---|---|---|---|---|---|
| EA-0010 | 53,335, `length`, 2,458 content | 47,727, `stop`, 5,851 content | 101,062 | 14m30s | clean | `DO NOT ADVANCE`, evidence A, 2 High blockers |
| EA-0003 | 55,503, `length`, 0 content | 80,795, `stop`, 10,320 content | 136,298 | 22m26s | clean | `DO NOT ADVANCE`, evidence A, 3 blockers (1 High, 2 Medium) |

**The headline Flash defects reproduce.** EA-0010: "lookup accepts only `target_decision_point`, and
`active_for_target` filters only exact target equality … a non-applicable active rule can be returned
as the applicable rule" — the same applicability-blind lookup, with `scope` and `applicability`
stored but never consulted. EA-0003: "the caller can discard the started event and append only the
completed event, contradicting the claimed invariant" — the same non-structural start-before-completion
ordering. Pro's second EA-0010 blocker is the same governance-guarantee family as Flash's; its two
EA-0003 Medium blockers (source mappings emitted without checking they name Plan requirements;
derivative mappings contradicting aggregate coverage) are new.

Neither candidate is usable without rewrite; both fail on the invariant-dense core rather than on
mechanics. **Stage-4 implementation delegation: NO.**

### Arm 3 — Stage 5 (UPG-0066 frozen features)

**The completion failure is gone.** All three features produced real final content on the permitted
retry, against Flash's six-attempt zero-output baseline:

| Feature | Attempt 1 | Attempt 2 | Total | Wall | Candidate |
|---|---|---|---|---|---|
| F-0001 | 42,804, `length`, 0 content | 44,329, `stop`, 9,543 content | 87,133 | 14m12s | 1,105 lines, 12 tests |
| F-0004 | 48,020, `length`, 6,785 content | 40,830, `stop`, 7,251 content | 88,850 | 11m24s | 742 lines, 15 tests |
| F-0006 | 52,214, `length`, 6,408 content | 67,240, `stop`, 7,948 content | 119,454 | 16m01s | 904 lines, 17 tests |

The test names map cleanly onto each contract's happy path, named failure paths, and falsification
scenarios — this is on-target content, not filler.

**None of the three compiles against the real implementation**, so all three fail frozen criterion 2
and no repair iteration is permitted:

- **F-0001** — four instances of one borrow-lifetime mistake (`E0716`) in its own helper usage; a
  mechanical Rust error, not a misreading of the contract.
- **F-0004** — constructs `EventMeta { … timestamp: … }`; the real field is `timestamp_ms`.
- **F-0006** — omits the required `event_id` and `timestamp_ms` from `EventMeta`.

Recorded plainly because it bounds the claim: `EventMeta` is defined in `modules/event_log`, which the
frozen packet does not include — for F-0004 and F-0006 the delegate was guessing at a type it could
not see, while F-0001, where the type *is* in the supplied file, failed on its own lifetime error
instead. Flash received the identical inputs, so the comparison is fair, but these two failures are
not evidence about behavioral comprehension.

Criteria 3-5 are ordered after compilation, so behavior representation and the frozen mutation set
could not be evaluated; **mutation evidence is unobtainable for this run** and the mutation table
above stands unused.

**Stage-5 test generation: NO** under the frozen protocol.

### Cost and Claude supervision

DeepSeek: 69,042 tokens (Arm 1), 237,360 (Arm 2), 295,437 (Arm 3). Flash comparators, never pooled:
149,975 / 264,392 / 384,299.

Claude supervision, measured from actual Anthropic session usage, bounded to first inspection of the
candidate through the final judgement, excluding setup, API waiting, bookkeeping, and synthesis:

| Arm | Claude output | Fresh input | Uncached input | Cached read |
|---|---|---|---|---|
| 1 — reviewer scoring | 11,953 | 22,303 | 36 | 5,849,845 |
| 2 — Stage-4 judging | 7,777 | 14,739 | 26 | 4,364,805 |
| 3 — Stage-5 judging | 6,055 | 8,087 | 16 | 2,773,000 |

`Claude-token savings: UNKNOWN` for all three arms — no credible direct-Claude baseline exists for
any of this work, and nothing is inferred from a counterfactual. Note separately that no arm produced
an adoptable artifact, so no Claude output was displaced; that is an observation about quality, not a
measured saving.

## Decision

```text
                         V4-Pro-at-max
Reviewer fallback        NO
Stage-4 implementation   NO
Stage-5 test generation  NO
```

All three attributed to the tested configuration — `deepseek-v4-pro` at `reasoning_effort: max` —
never to model tier alone, and never pooled with `deepseek-chat` or V4-Flash figures.

Pro is clearly stronger than Flash: it completes, it reasons less wastefully, it costs less per
useful attempt, it found a real defect two other reviewers missed, and it produced three
contract-shaped test suites where Flash produced nothing. None of that reaches any role's bar.

**Nothing changes as a consequence.** The required-reviewer policy stands, the delegated implementer
stays `status: disabled`, and no default configuration moves. The one-line `reasoning_effort`
parameterization keeps its `high` default.

**The single open question this run did not settle**, recorded rather than pursued: whether the
Stage-5 candidates would pass after the trivial mechanical repairs, and whether the `EventMeta`
mismatches would disappear if `modules/event_log` were in the packet. Answering it needs a repair
iteration and a packet change — both forbidden by the frozen protocol and by the stop rule. It is a
new experiment with a new hypothesis, and needs its own human decision, not an extension of this one.

## Stage-5 diagnostic — mechanical repair only, 2026-08-21

The Stage-5 NO above rested on a compile error, which does not distinguish "almost usable but
mechanically wrong" from "semantically wrong". This settles that with **no new DeepSeek call**:
repair the existing V4-Pro suites mechanically, then run them against the real implementations and
the frozen mutations.

**The repair boundary.** Mechanical repair may make a candidate compile against interfaces already
present in source, but must not change expected values, assertions, control flow, scenario coverage,
or any behavior a helper represents. Adding a missing required field is mechanical only if its value
is irrelevant to the behavior under test; if choosing the value requires interpreting an approved
artifact, that is semantic repair and the feature fails. The committed direct-path suites were not
opened during repair — repair used the module source and the compiler's own errors only.

### Refined acceptance rule (applies symmetrically to every feature)

> A candidate test that fails against the real implementation is **not** a candidate failure when the
> failed assertion is directly required by an approved artifact and the implementation is
> independently confirmed nonconformant.

The original criterion assumed the implementation was conformant. A Stage-5 candidate is supposed to
test the approved specification, not to reproduce a faulty implementation:

```text
candidate disagrees with implementation
  -> is the candidate supported by approved artifacts?
       NO  -> candidate defect
       YES -> implementation defect
```

### Per-feature result

| Feature | Repair needed | Class | Result |
|---|---|---|---|
| F-0001 | 4 × bind a temporary to a `let` (12 lines) | mechanical | **PASS** |
| F-0004 | `timestamp` → `timestamp_ms`, then uuid-v4 event/correlation ids | **semantic** | **FAIL** |
| F-0006 | `meta()` omits `event_id` and `timestamp_ms` entirely; both need uuid-v4 values | **semantic** | **FAIL** |

F-0004 and F-0006 fail on the same thing: their approved Event Schemas state `"event_id": "uuid-v4"`
and `"correlation_id": "uuid-v4"` in the required base fields, and the suites use readable ids
(`"event-corr-happy"`, `"correlation-happy"`). The schema was in both packets. Supplying conforming
values is Claude supplying contract conformance the delegate got wrong, so both features fail the
boundary rather than being repaired past it. F-0001, by contrast, used real uuid-v4 event ids.

### F-0001 — the candidate detected specification drift, not implementation defects

After repair: 9 of 12 tests pass. The 3 failures are the candidate enforcing the **currently
approved** Event Schema against an implementation that predates it. Corrected after checking the
history — these are **not** coding defects:

| | Implementation (2026-07-26) | Approved package today |
|---|---|---|
| `OfficialCandidateRecorded.publisher_claims` | `array<string>` | `array<object>` with `claim` + `discovery_evidence_refs` |
| `PublisherClaimEvidenceMissingRejected` | no `publisher_claim` field | `publisher_claim` required |
| Contract happy path | — | "each publisher claim is shown with the discovery evidence that supports that claim" |

PlotSpot `9b5b424` (2026-08-14, "resolve package review findings") made all three changes and
`b60b5f0` approved them. F-0001's Stage 4 implementation and Stage 5 tests both date from
2026-07-26 and **conformed to the schema approved at the time**. F-0004, F-0005, and F-0006 were
implemented on 2026-08-17, after the revision; only F-0001 was never re-entered against its revised
package.

So the finding is **live specification drift**, and the actionable item — PlotSpot's to take, not
Codeos's — is a post-acceptance re-entry of F-0001 against its current approved package. One item,
not two defects.

**The committed direct-path suite passes unmutated (17 tests) and cannot see any of this**: it was
written on 2026-07-26 against the superseded schema. That is the durable point for delegation — a
Stage-5 suite regenerated from the current approved package surfaces drift that stale tests
structurally hide.

### F-0001 mutation results

| Mutation | Approved rule | Candidate | Committed direct suite |
|---|---|---|---|
| M1 — accept empty `responsible_organization_evidence_refs` | Failure Path 2 `OfficialResponsibilityUnsupported` | **KILLED** (4th failure appears) | KILLED (1 failure) |
| M2 — drop the `candidate_dataset_evidence_refs` guard | Failure Path 4 `CandidateIdentityIncomplete` | **KILLED** — `test_missing_candidate_identity_rejects_and_does_not_record` | **SURVIVES** (17 pass) |
| M3 — country-scoped view returns every country | Invariant: every inventory view is scoped to a selected country | **not applicable** | not applicable |

M3 is a defect in the mutation, not in either suite: the country filter lives in
`list_official_candidates`, a trait method implemented by the caller-supplied store, so there is no
module-level injection point and each suite brings its own store. It is recorded as inapplicable
rather than scored.

M2 is the case worth naming: a contract-valid mutation that the committed direct suite survives and
the delegated suite kills. Mutation validity came from the approved Contract, never from whether the
direct suite died — the direct suite's record is reported here as a comparison, not as a filter.

### Conclusion — Stage 5 stops

```text
F-0001  PASS   and exposed live specification drift the direct suite cannot see
F-0004  FAIL   semantic: schema-mandated uuid-v4 ids
F-0006  FAIL   semantic: same, plus both base fields absent

1 of 3 passes -> below UPG-0066's >=2 of 3 bar -> STOP
F-0004 itself failed, so the context-complete rerun is not earned and was not run.
```

The conclusion is more informative than the original NO without changing it: **V4-Pro demonstrated
real semantic value on F-0001 — a suite generated from the current approved schema, which is exactly
why it caught drift the 2026-07-26 direct-path tests cannot — but reliability across features is
still insufficient.** Two of three suites violate base-field rules stated in the very schema they were
given.

No DeepSeek call was made for this diagnostic. Claude supervision for the whole repair-and-diagnosis
block, measured from actual Anthropic session usage: 27,270 output, 43,526 fresh input, 70 uncached
input, 14,078,869 cached read. `Claude-token savings: UNKNOWN` — unchanged, no baseline exists.
