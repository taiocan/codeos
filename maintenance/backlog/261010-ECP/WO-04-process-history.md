# WO-04 — Process History

ECP-4 · NORMAL · after WO-03 (needs only WO-01) · Charter: O-3, C-6, C-7

## Goal

`status --history [--since <YYYY-MM-DD|all>]` reports factual counts and elapsed durations from
existing records.

**Invariant:** history counts or times only what existing records directly establish. A time
window uses only a timestamp CodeOS records as the event time. Chronological correlation is
never promoted into causal process evidence.

## In scope

- `--since` is inclusive from 00:00:00 UTC; the default is the last 30 days and is stated in the
  output. The terminal shows the HISTORY view only; `--json` adds `history` to the projection.
- Metrics `{metric, state: OBSERVED|UNOBSERVED, value, source, window: in_window|all_time}`:
  `accepted_features` (distinct subjects), `additional_decision_receipts`,
  `failed_verifications`, `non_progressing_assessments` and `execution_invalid` (review-log
  entries timed by their headings), `candidate_bundles` (`all_time`: no event time),
  `observations_opened`/`_closed`, `implementation_defect_routes`.
- UNOBSERVED: `implementation_escapes` (no record establishes that a defect applied to the
  accepted state), `not_ready` and `evidence_invalidations` (not persisted), and
  `attributed_waiting` (no record establishes a cause).
- Elapsed durations, each naming its two records: observation → closure; accepted package
  approval → F9 acceptance, only when the current artifacts still hash to the acceptance
  bindings, otherwise UNOBSERVED.

## Out of scope

Scores, rankings, thresholds; new persisted records (Rule 4); charts.

## Sources

`ReceiptStore`, `VerificationStore`, `CandidateStore`, `ObservationStore`, the review log with
`evidence::review_permits_progression`, `hashing::file_sha256`.

## Acceptance criteria

1. Each count names its source and is proven by a fixture.
2. `--since` filters by event time, inclusively from 00:00 UTC; the default window is stated.
3. No field presents elapsed time as waiting time.
4. Undefinable metrics show `UNOBSERVED` with `value: null`, never 0 and never omitted.
5. Plain `status` output is unchanged without `--history`.

## Verification

README baseline plus `engine/tests/process_history.rs`.
