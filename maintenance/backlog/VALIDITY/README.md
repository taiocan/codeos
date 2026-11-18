# VALIDITY — Dependency-Accurate Validity Work Orders

**CLOSED 2026-11-18.** The initiative:

- measured the invalidation waste (WO-01);
- made validity dependency-accurate (WO-02, DBA-35);
- moved deterministic blockers earlier (WO-03, preflight);
- removed whole-configuration invalidation (WO-04, DBA-36, confirmed by JamFree's adoption);
- made the whole remaining path visible (WO-05, plan);
- investigated structured consistency and declined a mechanism the evidence did not support
  (WO-06).

Deferred items below stay deferred until new evidence justifies them.

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
| [WO-02](WO-02-dependency-accurate-validity.md) | PROTECTED | Dependency-accurate validity | Accepted 2026-11-18; DBA-35 issued (Feature Workflow v14) |
| [WO-03](WO-03-deterministic-preflight.md) | NORMAL | Deterministic preflight | Accepted 2026-11-18 |
| [WO-04](WO-04-dba-impact-adoption.md) | PROTECTED | DBA impact-aware adoption | Accepted 2026-11-18; DBA-36 issued; confirmed by the JamFree adoption; CLOSED |
| [WO-05](WO-05-whole-path-plan.md) | NORMAL | Whole-path plan | Accepted 2026-11-18 |
| [WO-06](WO-06-structured-consistency.md) | NORMAL | Structured-value consistency in preflight | CLOSED — NO APPLICABLE STRUCTURED VALUES (2026-11-18) |

This table is the only place WO status is kept.

## Deferred

- **Atomic multi-feature acceptance.** Its motivating defect was fixed in 2ae639e; no evidence of
  material cost.
- **JamFree cycle runbook script.** A JamFree WO, reconsidered after WO-05.
- **Spec-only Stage 8 review and prose-number lint.** Deferred by the human 2026-11-18. Measure what
  WO-03 still misses first. The structured-value consistency check was approved separately as
  WO-06.
- **Prose-only cross-artifact consistency (finding of WO-06).** Some important cross-artifact
  consistency constraints remain prose-only and therefore cannot be checked deterministically.
  Revisit only if repeated evidence shows the class is costly enough to justify structured semantic
  fields or an advisory spec review. Do not introduce structured ranges merely to make a check
  possible.
- **Advisory as-of provenance and freshness (U5).** Deferred out of WO-04 by the human 2026-11-18.
  Implement only when a concrete freshness rule needs it; Boundary 2 of WO-02 still holds.
