---
feature_id: UPG-0071
slug: deepseek-v4-pro-requalification
title: DeepSeek V4-Pro Requalification Across Three Codeos Roles
status: DECIDED — NO / NO / NO for V4-Pro-at-max (2026-08-21)
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
