# WO-08 — Complete the Delivery-Stability Proof

NORMAL · after WO-07 · Charter criterion 8 · continues STAB WO-03

## Goal

Prove that ordinary feature delivery under the accepted shared-validation baseline no longer
requires CodeOS changes or duplicate proof of one product state.

## Work

- Freeze CodeOS capability work.
- Resume STAB WO-03 P3 from 1/3 and deliver normal JamFree features, only as the product needs
  them, under the existing STAB definition.
- F-0003..F-0005 stay parked unless a new qualifying feature, release qualification, or an
  explicit DQA acceptance case needs them.
- Record unique validation executions separately from feature claim attributions, and human
  observation sessions separately from governed outcomes.
- Continue recording CodeOS changes, Candidate Bundles, assessments, invalidations, provider calls,
  and status time from existing evidence.
- Classify any interruption before acting: product defect, project gap, expected governance, or
  CodeOS correctness defect.

## Acceptance criteria

1. STAB reaches 3 qualifying features, at least 2 after the accepted shared-validation baseline.
2. No qualifying feature requires a CodeOS capability or conformance change.
3. Compatible validation identities execute once; every additional consumer is attribution only.
4. Each human observation session occurs once and every recorded outcome was explicitly selected.
5. Provider-call totals reconcile and incidental invalidations are zero.
6. Status cost remains bounded as accepted features and claim attributions grow.

A CodeOS correctness repair is allowed, but disqualifies that feature and must gain a regression
test before the proof resumes.

## Baseline (2026-10-19)

- CodeOS frozen from `main` `106386a` (DBA-29 issued and merged, not pushed). Any later CodeOS
  change is classified before acting; a correctness repair disqualifies its feature.
- JamFree `446dc43` on DBA-29: Bootstrap B1–B5 PASS (B3 `af0d44e`, B4 re-confirmed by the human).
- F-0001..F-0006 STALE after the compatibility repair (`9ad902e`). They are revalidated only when
  product work needs them, then in one shared v13 run; F-0003..F-0005 stay parked.
- P3: 1/3 qualifying (F-0003, historical); 2 more needed, both after this baseline.
- `status --json` on a JamFree clone: 3.4–3.5 s (three runs, debug build).
- Provider accounting from here is the adapter ledger; earlier totals stay as recorded.
