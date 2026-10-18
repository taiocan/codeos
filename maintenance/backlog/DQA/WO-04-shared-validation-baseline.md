# WO-04 — Shared-Validation Root-Cause Baseline

NORMAL · analysis only · after WO-03 · Charter criteria 4–7

## Goal

Establish exactly which JamFree validation work is one shared execution or observation and which
work remains legitimately feature-specific. Make no CodeOS or JamFree behavior change.

## Work

- Inventory each repeated smoke, journey, behavior, repeatability, Playwright, production-source,
  F4, and F8 action by product state, command, environment, evidence identity, feature, governed
  question, Observed Surface, and input scope.
- Classify each repetition as identical execution, shared observation with distinct judgments,
  legitimate distinct validation, broad-binding invalidation, or accepted-frontier feedback, and
  separate execution, observation, attribution, and decision duplication.
- Measure status time as T(N accepted, S stale) on the six-feature JamFree snapshot (about 6 s at
  the FAI baseline, about 16 s on 2026-10-17) and freeze it as a regression workload.
- Calculate the counterfactual unique executions, human observation sessions, provider calls, and
  feature outcomes without treating equal output alone as proof of semantic equivalence.
- Freeze replay fixtures for JamFree state `7bcc9146`, the three F-0001/F-0006 identical-surface
  F4 pairs, direct and inherited production claims, and the stale-upstream frontier case.

## Acceptance criteria

1. Every repeated action is classified from durable JamFree evidence.
2. Every proposed reuse names the execution inputs or observation that are genuinely identical.
3. Every human reuse separates the one observation session from each feature/claim judgment.
4. The baseline identifies where FAI already solved reuse and where Candidate Bundle, journey, or
   receipt semantics still force repetition.
5. Status evaluation does no work proportional to historical receipts or evidence when the current
   projection can be computed once per command.
6. The human accepts the matrix as the sole empirical input to WO-05.
