# WO-01 — Root-Cause Table of P3 Invalidations

NORMAL · analysis only · Charter: Problem, Measure

## Goal

Classify every invalidation from the evidence already collected, so that WO-02 designs only what
the data justifies. No TomTom calls and no CodeOS changes.

## Cases

1. **F-0005 page change** (`e9c9518`): F-0001 to F-0004 stale; four F4 receipts stale.
2. **rustls patch** (`874728d`): four accepted features stale.
3. **F-0002 test-only repair** (`b713734`): F-0002 production evidence regenerated.
4. **Inherited production evidence:** F-0003 and F-0004 rerun the F-0001 and F-0002 journeys.
5. **Evidence that cannot be refreshed:** F-0003's production evidence was current but insufficient,
   and it does not bind its verifier, so the improved verifier (`3818ff6`) could not replace it
   (STAB WO-04).

## Method

Write one row per invalidated feature and evidence item, with these columns:

- the identity that caused it (an input file, `implementation_state` or `product_state`);
- whether the judged surface or dependency actually changed;
- **necessary** or **incidental**;
- the narrowest binding that would have been correct;
- the cost incurred (TomTom calls, assessments, human decisions).

Sources: JamFree `verifications.jsonl`, `decisions.jsonl`, the candidate bundles, Git, and the STAB
WO-04 findings.

## Acceptance criteria

The human accepts the table. Each incidental row names the goal (1 to 4) that would remove it.

## Results
