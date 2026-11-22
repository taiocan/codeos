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

## Baseline (2026-10-18)

Source: JamFree `172282b`, `.codeos/06-workflow` (497 verification records, 124 receipts, 51
production evidence files). Recompute with `python3 -I wo04-matrix.py <clone>/.codeos/06-workflow`.
Two runs count as one execution only when their inputs are identical: the same command (minus its
per-run report path) on the same implementation state. Equal output is never used as proof.

### Repetition matrix

| Action | Records → distinct | Class of each repeat | Duplication |
|---|---|---|---|
| smoke, behavior, repeatability, Playwright | 281 → 189 | 88 identical execution for another feature; 4 broad-binding invalidation (a Specification Package edit reran commands that do not read it) | execution, attribution |
| pre-candidate / F9 qualification | 38 → 17 | 13 identical execution for another feature; 7 identical execution, same feature, nothing changed (operator rerun of a current record); 1 legitimate (qualification script changed) | execution |
| F3 product journey | 79 → 61 | 17 shared observation with distinct judgments: same command and state, same Observed Surface, but each feature's journey definition carries its own transition; 1 broad-binding invalidation | observation, attribution |
| F5 production-source evidence | 51 runs → 45 legs | 6 shared observation with distinct judgments: the verifier makes one forecast request per leg (outbound or return) and date and checks each claim against it, yet ran once per feature | observation |
| F4 / F8 | 15 receipts → 7 Observed Surfaces | shared observation with distinct judgments: F-0002..F-0005 on `bd685607`; F-0001 and F-0006 on `a9a2852f`, `91451275`, `89c5edb7`. F4 and F8 on one surface are distinct questions | decision |
| F7 runtime events | 29 → 28 | legitimate distinct validation | — |

Accepted-frontier feedback forced no repeat at one implementation state; it shows only as the
differing `accepted_frontier` binding of F-0003..F-0005 (the stale-upstream frontier case).

### Counterfactual (one execution per identical input set, one session per Observed Surface)

- Deterministic executions 319 → 206; journey runs 79 → 61 observations.
- Production: 1,314 → 1,152 receipted successful calls (6 legs × 27 avoidable). These are historical
  receipts, not the incident accounting, which stays 162 measured calls plus an unknown amount.
- Human: 15 judgments stay 15; observation sessions 15 → 7. Feature outcomes are unchanged.
- At state `7bcc9146` alone: 24 runs of four project-wide commands → 4; 9 journey runs → 2
  observations; F4 on `bbb0e3e8`: 1 recorded and 3 pending (F-0003..F-0005, parked) → 1 session.

### Where reuse is solved and where it is still forced

- Solved by FAI: inherited claims (F-0006 `arrive-by-departures` from F-0001 made no production
  run); delivered-state binding (test-only changes keep production evidence current, `71adabb`);
  revalidation reuses current records.
- Still forced: records are keyed by (feature, checkpoint, verification), and a Candidate Bundle
  binds that feature's own record identities, so a project-wide command must rerun per feature.
  The production verifier takes one subject's claim set. Receipts are per feature, with no
  observation-session identity shared by several judgments.

### Finding: the fixture provider is outside the execution identity

`30a6b63` changed the deterministic fixture emulator (`provider-boundary/server.mjs`) but not
implementation state `7bcc9146`. F-0002's journey before it and F-0003..F-0005's after it bind the
same state and Observed Surface `bbb0e3e8`. A record therefore stays current across a change in a
deterministic input it consumed, and "identical inputs" cannot rest on implementation state alone.
Recorded for WO-05, not repaired here.

### Status cost (regression workload)

`status --json`, debug build, on disposable clones (wall time, single runs; 172282b three runs):

| Snapshot | Accepted / stale | Receipts | Stage 8 review records | Bundles | Wall |
|---|---|---:|---:|---:|---:|
| `426b228` | 5 / 1 | 106 | 61 | 26 | 4.9 s |
| `2601a52` | 6 / 4 | 122 | 69 | 34 | 13.1 s |
| `172282b` | 6 / 3 | 124 | 70 | 35 | 12.4–13.0 s |

At `172282b` one command makes 136 acceptance assessments, 110 `current_acceptance_valid` calls,
22 accepted-frontier derivations, 156 revalidation-bundle lookups (each walks every review record)
and 4,545 file hashes, for 6 features. Git is 94 calls and 0.36 s. Each feature's acceptance
validity is a current projection computable once per command, but it is recomputed for every
consumer's frontier, and review records are rescanned per lookup. Criterion 5 is therefore not met
today. The frozen workload is `172282b` with these counts and times.

### Frozen fixture cases (made executable in WO-07, when there is new behavior to assert)

| Case | JamFree commit |
|---|---|
| State `7bcc9146`: six features, identical executions and observations | `2601a52` (records `64ce312`–`2601a52`, product `56d28a2`) |
| F-0001/F-0006 identical-surface F4 pairs | `a9a2852f` at `4b0eaed`, `91451275` at `dba7bee`, `89c5edb7` at `e5f4447` |
| Direct and inherited production claims | `172282b` (F-0001 direct, F-0006 inherited) |
| Stale-upstream frontier | `172282b` (existing replay case) |
