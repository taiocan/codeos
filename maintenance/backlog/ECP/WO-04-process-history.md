# WO-04 — Process History

- **Charter refs:** ECP-4, O-3, C-6, C-7
- **Classification:** NORMAL
- **Depends on:** WO-03 (technically only WO-01)

## Goal

`status --history [--since <date>]` reports factual counts of process friction and conservative
elapsed durations, derived from existing records.

## In scope

- **Counts:**
  - accepted features: acceptance receipts;
  - Candidate Bundles: `CandidateStore`;
  - BLOCKED assessments and `REVIEW_EXECUTION_INVALID`: review records and the review log;
  - failed verifications: verification records;
  - repeated human decisions: more than one receipt for the same subject and checkpoint;
  - observations opened and closed;
  - implementation escapes: observations routed `implementation_defect` with a target feature
    that is historically accepted.
- **NOT_READY:** not persisted today, so it reports UNOBSERVED.
- **Durations:** elapsed only, derived from timestamps.
  - Observation open → closure.
  - Specification Package approval → F9 acceptance.
  - Each duration states the two records it spans.
- **Attributed waiting time** (waiting for a human, or blocked on tooling) is reported only when
  a record establishes the cause. No record does today, so the output shows it as UNOBSERVED.

## Out of scope

- Scores, rankings, and thresholds.
- Persisting new records (that would be PROTECTED).
- Charts.

## Authoritative sources to reuse

`ReceiptStore::all`, `CandidateStore::all`, `VerificationStore`, `evidence::latest_review_record`
plus the review-log parsing in `evidence.rs`, `ObservationStore`.

## Acceptance criteria

1. Each count names its source, and a fixture proves each count.
2. `--since` filters by record timestamp, and the default window is stated in the output.
3. No output field presents elapsed time as waiting time.
4. NOT_READY and attributed waiting show UNOBSERVED, not 0.
5. Default `status` output is unchanged when `--history` is absent.

## Verification

The README baseline, plus a new `engine/tests/process_history.rs`.
