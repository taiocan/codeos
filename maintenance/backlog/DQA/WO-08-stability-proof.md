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

## Interruptions

- 2026-10-19, F-0007 (via F-0002's revalidation): **CodeOS correctness defect**, two regressions
  from DQA. An `events`-mode feature could not prepare a candidate.
  - WO-02 gave revalidation the generic currency rule, which lacks the validator binding a
    runtime-event record carries, so every events-mode revalidation read as stale.
  - WO-06 made `verify-events` record F5g Playwright bindings, which under v13 include
    Playwright's execution identity, so under DBA-29 even an ordinary candidate refused them.

  Repaired with one owner, `event_evidence::runtime_event_bindings`, used by the writer and both
  readers, plus a regression test `tests/runtime_events.rs` (DBA-28 and DBA-29) that fails on the
  previous code. The reviewer's readiness had the same gap (WO-02): every reader now asks
  `checker::current_record_bindings`, and `lifecycle_contract.rs` covers it. Per this WO, F-0007
  does not qualify; P3 stays 1/3.
- 2026-10-19, same step: the qualified release refused to build while unrelated untracked files
  existed anywhere in the repository. Conformance repair (`8307735`): untracked files are refused
  only inside the CodeOS source boundary (`dba/`, `dba-system.md`).

## F-0007 — Return departure window (accepted 2026-10-19, non-qualifying)

JamFree `3d36e5c` (spec) → `1b6d12a` (accepted, Candidate Bundle `875779ed`). Non-qualifying:
the CodeOS correctness repair above was needed. P3 stays 1/3.

- **Executions vs attributions:** 18 Validation Evidence Sets (executions) behind 40 verification
  records written since the baseline, for F-0007 and F-0002's two revalidations. After the first
  run, F-0002 reused F-0007's smoke, journey, behavior and repeatability runs; Playwright and
  qualification ran once for both; SR2 reattributed every record without re-execution.
- **Human sessions:** 3, each with one explicitly stated outcome (F4, F4 again after the SR2
  Contract amendment, F8); no outcome propagated.
- **Provider calls:** 29 (one F-0002 return leg, ledger reconciled with receipts: 27 forecast +
  2 place search). F-0007 made none (inherited claim). F-0002's second revalidation reused its
  production evidence.
- **Invalidations:** F-0002 was staled twice by F-0007's product changes inside its return
  surface (legitimate), and once by a verifier edit made for F-0005 (reverted, `3f42864`).
- **Stage 8:** three rounds. Round 1: two valid blockers (sub-minute difference shown as zero,
  incomplete browser assertions), repaired. Round 2: exactness read as second precision; the
  human amended the Contract (SR2, nearest minute) instead.
- **Status:** 3.8–4.0 s on a clone with seven features (3.4–3.5 s with six at the baseline).
- **Follow-up for F-0005:** its production verifier's window text must adopt "under 1 min
  longer" when F-0005 is next revalidated (`3f42864`).

## F-0008 — Return arrive by (accepted 2026-10-19, qualifying: P3 2/3)

JamFree `b7e999a` (spec) → `c091494` (accepted, Candidate Bundle `c58e730d`). No CodeOS change
between the F-0007 record and acceptance; the interruptions were product defects or expected
governance, so F-0008 qualifies.

- **Executions vs attributions:** 12 Validation Evidence Sets behind 27 verification records for
  F-0008 and F-0002's two revalidations; F-0002 reused F-0008's smoke, journey, behavior and
  repeatability runs each time; Playwright and qualification ran once per state.
- **Human sessions:** 2 (F4, F8), each with explicitly stated outcomes.
- **Provider calls:** 0. F-0008 inherits F-0002's return comparison; F-0002's production
  evidence stayed current through both revalidations.
- **Interruptions, classified:** a product defect in the shared arrive-by logic (a target the
  clock skips admitted later arrivals; also F-0006's), found at Stage 8 round 1 and fixed with a
  regression test; incomplete browser assertions (fixed); F-0002 staled twice by changes to its
  return surface (expected governance); F-0002's first revalidation packet exceeded the 450 KB
  review budget by 4% and ran once with the operator override, by human decision.
- **Stage 8:** two rounds (DO NOT ADVANCE, then NO OBJECTION).
- **Status:** 4.3 s on a clone with eight features.
