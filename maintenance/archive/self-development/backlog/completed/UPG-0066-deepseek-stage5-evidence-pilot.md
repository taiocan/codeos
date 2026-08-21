---
feature_id: UPG-0066
slug: deepseek-stage5-evidence-pilot
title: DeepSeek Stage-5 Evidence Pilot
status: CLOSED   # pilot ran 2026-08-21: 0 of 3 candidates produced
priority: P2
depends_on: [UPG-0060]
related_features: [UPG-0064]
supersedes: []
superseded_by: []
---

# Upgrade: DeepSeek Stage-5 evidence pilot

## Decision

Test one narrow hypothesis before adding any new delegation role or workflow integration:

> DeepSeek produces net-useful Stage 5 candidates when failure is cheap and independent verification
> costs less than primary-model test generation.

The durable selection rule is task-shaped, not model- or stage-shaped. This feature does not establish
permanent DBA-to-model roles.

## Current Change

Make the existing disabled experiment compatible with the current DeepSeek V4 API, account for
cache, reasoning, and termination usage, and make Stage 5 instructions applicability-driven. Preserve
the existing role-labelled packet, staging boundary, manual promotion, and single-shot invocation.

No downstream DBA doctrine, model router, benchmark framework, experiment registry, automatic cost
calculator, automatic retry, Stage 7 integration, or automatic promotion is added.

## Stage 5 Pilot

Use three realistic downstream features with approved Specification Packages and independently
authored Stage 4 implementations. For each feature, produce two isolated results from the same
approved package, implementation, and repository state:

1. Direct Stage 5, without access to the DeepSeek candidate or its corrections.
2. A DeepSeek candidate followed by independent verification/correction, without access to the
   direct-path tests or findings.

The Contract and Event Schema own expected behavior. Implementation is supporting context only for
public interfaces and mechanical integration. Generate tests for all applicable approved behavior;
do not manufacture event or replay requirements for external-observation features.

Record only:

- correctness — missed requirements, incorrect tests, private coupling, unapproved assumptions, and
  applicable falsifier or mutation evidence;
- usefulness — usable candidate content, local correction, or rewrite;
- supervision — retries, corrections, primary-model effort, and human effort;
- cost — actual DeepSeek and supervising-model usage and cost.

The pilot is positive only when there is no material correctness loss, at least two candidates are
usable without rewrite, and aggregate delivery cost or effort is materially lower than the direct
path. A 30% supervising-token reduction is a target, not a gate. Mixed or ambiguous evidence stops
progression.

## Termination and Follow-up

`finish_reason: stop` is required for a valid candidate. After `length`, the operator may make one
explicit 65536-token retry; after `insufficient_system_resource`, one identical retry. Every attempt
counts toward cost. Repeated truncation, repeated resource failure, or `content_filter` makes that
case unusable.

Stage 7 remains a one-shot, read-only experiment contingent on an explicit positive Stage 5 decision.
UPG-0064's bounded Stage 4 pilot remains deferred pending later evidence and a separate human
decision.

The three-feature pilot requires eligible downstream repositories and therefore remains pending; no
such Specification Packages are present under the current local projects workspace.

## Compatibility Evidence

On 2026-08-14, a real `deepseek-v4-flash` Stage 5 smoke request against a synthetic
external-observation feature completed with `finish_reason: stop`, staged one candidate test, and
that test passed. Usage accounting reported 2,805 prompt, 7,246 completion, 6,715 reasoning, 0
cache-hit, and 2,805 cache-miss tokens. This establishes API and protocol compatibility only; the
synthetic run is not Stage 5 quality or net-benefit evidence.

## Pilot feature selection — frozen 2026-08-21, before any billed call

Source repository: PlotSpot at `c3b8215317bd7b9b3b711f2a925ee9c71d5cb708`. Delegate model
`deepseek-v4-flash`. The prerequisite the earlier note reported as absent is now satisfied: PlotSpot
F-0001…F-0006 each have an approved contract and event schema, an independently authored Stage 4
implementation under `modules/`, and committed behavioral and replay tests.

| Feature | Module | Shape |
|---|---|---|
| F-0001 | `source_inventory` | Smallest package (10.8 KB contract, 68-line schema) — plain inventory behavior |
| F-0004 | `source_fitness` | Mid-sized (12.2 KB, 105 lines) — decision logic over measured inputs |
| F-0006 | `regional_availability` | Largest (17.8 KB, 156 lines) — the most event- and state-heavy of the six |

Selected for spread across package size and behavioral shape, so the three cases can fail
differently. Each has exactly five contract scenarios, so scenario count is not what varies.

**Arm 1 is the committed tests.** They were authored directly, with no access to any DeepSeek
candidate — the isolation this design requires. A fresh direct arm is impossible to isolate today
because those tests are already in the tree, so re-running it would buy contaminated evidence at real
Claude cost. Consequence, recorded before the run: the direct arm carries no credible Claude-cost
evidence, so `Claude-token savings: UNKNOWN` for this pilot. The third clause of the positive bar —
materially lower delivery cost — is therefore **inconclusive**, and this pilot returns a quality
verdict plus the delegated arm's measured Claude supervision cost instead of a savings judgement.

## Pilot result — ran 2026-08-21

Delegate `deepseek-v4-flash`, PlotSpot at `c3b8215317b`, contract and event schema declared by role
flag with the Stage 4 implementation supplied as supporting context.

**No candidate was produced for any of the three features.** Every attempt terminated
`finish_reason: length` with the entire completion budget spent on reasoning and **zero output
characters** — at the default bound and again at the single permitted 65536 retry.

| Feature | Attempt 1 (32768) | Attempt 2 (65536) | Total | Outcome |
|---|---|---|---|---|
| F-0001 source_inventory | 42,792, `length`, 0 chars | 75,559, `length`, 0 chars | 118,351 | unusable |
| F-0004 source_fitness | 48,007, `length`, 0 chars | 80,773, `length`, 0 chars | 128,780 | unusable |
| F-0006 regional_availability | 52,200, `length`, 0 chars | 84,968, `length`, 0 chars | 137,168 | unusable |

384,299 DeepSeek tokens, six attempts, ~36 minutes of wall time, nothing staged. The brief's own
termination rule decides all three: "Repeated truncation … makes that case unusable." No further
retry is permitted and none was taken.

### Measured against the corrected objective

- **A — Claude tokens spent** running and supervising the whole pilot block (this pilot plus
  UPG-0064's, which ran interleaved): 36,355 output, 70,302 fresh input, 132 uncached input, and
  14,427,340 cached-read input, measured from actual Anthropic session usage rather than estimated.
- **B — quality and usefulness:** zero usable candidates of three. Correctness, falsifier coverage,
  and mutation evidence could not be scored, because nothing was produced to score.
- **C — Claude-token savings: UNKNOWN**, as frozen before the run. The direct arm has no credible
  Claude-cost evidence, and nothing was delivered that could have displaced Claude work.

### Verdict

**Negative, and decisive on its own terms.** The pilot's positive bar required at least two
candidates usable without rewrite; it got none. The stated hypothesis — that DeepSeek produces
net-useful Stage 5 candidates when failure is cheap — is not supported: on these three real features
the model did not terminate normally even once. This is a harder failure than a weak candidate, and
it is not a harness defect: the same tool, the same day, produced compiling Stage 4 candidates from
the same repository shape.

Stage 7 remains a one-shot experiment contingent on a positive Stage 5 decision, which this is not.
