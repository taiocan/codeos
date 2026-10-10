# WO-04 — Process History

ECP-4 · NORMAL · after WO-03 (needs only WO-01) · Charter: O-3, C-6, C-7

## Goal

`status --history [--since <date>]` reports factual friction counts and conservative elapsed
durations from existing records.

## In scope

- Counts: accepted features; Candidate Bundles; BLOCKED and `REVIEW_EXECUTION_INVALID`
  assessments; failed verifications; repeated human decisions (more than one receipt per subject
  and checkpoint); observations opened and closed; implementation escapes (observations routed
  `implementation_defect` to a historically accepted feature).
- NOT_READY is not persisted → UNOBSERVED.
- Elapsed durations only, each naming the two records it spans: observation open → closure;
  Specification Package approval → F9 acceptance.
- Attributed waiting (human, tooling) only when a record establishes the cause; none does today →
  UNOBSERVED.

## Out of scope

Scores, rankings, thresholds; new persisted records (Rule 4); charts.

## Sources

`ReceiptStore`, `CandidateStore`, `VerificationStore`, review records and review-log parsing in
`evidence.rs`, `ObservationStore`.

## Acceptance criteria

1. Each count names its source and is proven by a fixture.
2. `--since` filters by record timestamp; the default window is stated in the output.
3. No field presents elapsed time as waiting time.
4. NOT_READY and attributed waiting show UNOBSERVED, not 0.
5. Default `status` output is unchanged without `--history`.

## Verification

README baseline plus new `engine/tests/process_history.rs`.
