# DQA — Delivery Qualification and Assurance Work Orders

[Charter](charter.md). Planning context only. Governing policies and Git remain authoritative.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-qualify-before-issuance.md) | PROTECTED | Qualify configurations before issuance | Accepted 2026-10-18 (invariant, JamFree replay golden, `dba/` source digest) |
| [WO-02](WO-02-single-lifecycle-authority.md) | NORMAL | One executable lifecycle authority | Accepted 2026-10-18 (revalidation evidence set, replay golden order) |
| [WO-03](WO-03-isolate-verification.md) | PROTECTED | Isolate verification and account for provider calls | Accepted 2026-10-18 (Codeos-owned ledger location) |
| [WO-04](WO-04-shared-validation-baseline.md) | NORMAL | Shared-validation root-cause baseline | Accepted 2026-10-18 |
| [WO-05](WO-05-validation-semantics.md) | PROTECTED | Validation evidence and attribution decision | Approved 2026-10-18 ((a)–(c), five terms) |
| [WO-06](WO-06-shared-validation-implementation.md) | PROTECTED | Shared-validation implementation | Accepted 2026-10-18 |
| [WO-07](WO-07-qualification-replay.md) | NORMAL | Qualification and JamFree replay | Accepted 2026-10-18; DBA-29 issued and merged to main (2026-10-19); JamFree adopted it (`e119fd3`) |
| [WO-08](WO-08-stability-proof.md) | NORMAL | Complete the delivery-stability proof | Accepted 2026-10-19 (P3 3/3: F-0003, F-0008, F-0009); DQA closed |

STAB P4 questions ([STAB WO-04](../STAB/WO-04-decide-next.md)): A (verification ordering) goes to
WO-03 and D (lifecycle rules encoded twice) to WO-02. FAI answered part of B (dependency scope) and
C (inherited evidence), but JamFree then demonstrated that execution, observation, attribution,
and accepted-journey topology remain feature-scoped. WO-04–WO-07 address that residual without
reopening FAI's accepted production-evidence rules.

Each WO follows the previous one. WO-05 and WO-06 require the protected human decisions stated in
their files. This table is the only place WO status is kept.
