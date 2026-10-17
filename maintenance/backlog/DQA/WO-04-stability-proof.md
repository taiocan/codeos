# WO-04 — Complete the Delivery-Stability Proof

NORMAL · after WO-03 · Charter criteria 4 · continues STAB WO-03

## Goal

Prove that ordinary feature delivery no longer requires CodeOS changes.

## Work

- Freeze CodeOS capability work.
- Deliver normal JamFree features under the existing STAB definition.
- Record CodeOS changes, Candidate Bundles, assessments, repeated human decisions, invalidations,
  provider calls, and status time from existing evidence.
- Record the first real case of an upstream feature going STALE under an inherited claim; F-0006
  did not exercise it.
- Classify any interruption before acting: product defect, project gap, expected governance, or
  CodeOS correctness defect.

## Acceptance criteria

1. STAB reaches 3 qualifying features, at least 2 after the DBA-28 baseline.
2. No qualifying feature requires a CodeOS capability or conformance change.
3. Each repeated human decision corresponds to a changed judged surface.
4. Provider-call totals reconcile and incidental invalidations are zero.
5. Status cost remains bounded as accepted features grow.

A CodeOS correctness repair is allowed, but disqualifies that feature and must gain a regression
test before the proof resumes.
