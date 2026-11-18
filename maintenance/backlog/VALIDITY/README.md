# VALIDITY — Dependency-Accurate Validity Work Orders

Planning context only. Git remains authoritative. Approved in principle by the human 2026-11-17.

## Problem

JamFree, 2026-11-13 → 11-17, four relevant features (F-0001, F-0002, F-0009, F-0010):

| Measure | Value |
|---|---|
| Commits | 87, of which 74 workflow bookkeeping and 13 code |
| Stage 8 reviews | 36: 26 NO OBJECTION, 9 CHANGES ADVISED, 1 DO NOT ADVANCE |
| F6 reconciliations | 38 |
| F4 / F8 / F9 decisions | 17 / 21 / 18 |
| Candidate Bundles | 34 |
| DBA versions adopted mid-cycle | 4 (DBA-31, 32, 33, 34; WO-01) |
| CodeOS defects fixed mid-cycle | 3 (027894b, 062f6b2, 2ae639e) |

The Polzela change (about 30 product lines) took about 12 hours, with 13 approvals and 10 Stage 8
reviews. Stage 8 found 7 substantive gaps and stays as it is; the waste is re-establishing
unchanged facts.

The dominant cause is one defect: **dependency identity is too broad.** Receipts, evidence,
Candidate Bundles and human judgments are invalidated by inputs they did not semantically depend
on. Secondary causes: cheap deterministic checks run only at expensive gates, DBA adoption
invalidates without derived impact, and the remaining path is discovered one gate at a time.

WO-01 measured the unnecessary repetition at about 35–55 % per checkpoint (upper bounds). That is
the authoritative baseline; the earlier 60–70 % and 3–4 hour estimates are superseded hypotheses.

Prior art: STAB WO-02 (scope-aware human-decision invalidation) and FAI (feature-atomic
invalidation).

## Principle

Measure first. Fix the identity once, shift proven cheap checks left, derive DBA impact, give full
lookahead. Then measure again before adding anything.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-invalidation-replay.md) | NORMAL | Historical invalidation replay | Accepted 2026-11-17 |
| [WO-02](WO-02-dependency-accurate-validity.md) | PROTECTED | Dependency-accurate validity | Part B: DBA-35 qualified 2026-11-18 (0f2bc35b…); awaits human acceptance and issue |
| [WO-03](WO-03-deterministic-preflight.md) | NORMAL | Deterministic preflight | Approved 2026-11-17 — may proceed |
| [WO-04](WO-04-dba-impact-adoption.md) | PROTECTED | DBA impact-aware adoption | Waits for WO-02 |
| [WO-05](WO-05-whole-path-plan.md) | NORMAL | Whole-path plan | Approved 2026-11-17 — may proceed |

This table is the only place WO status is kept. No further top-level WOs are justified now.

## Deferred

- **Atomic multi-feature acceptance.** Its motivating defect was fixed in 2ae639e; no evidence of
  material cost.
- **JamFree cycle runbook script.** A JamFree WO, reconsidered after WO-05.
- **AI spec-only review and prose-number lint.** Only if WO-03 misses a measured defect class.
