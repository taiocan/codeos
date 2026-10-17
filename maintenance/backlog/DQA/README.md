# DQA — Delivery Qualification and Assurance Work Orders

[Charter](charter.md). Planning context only. Governing policies and Git remain authoritative.

| WO | Class | Title | Status |
|---|---|---|---|
| [WO-01](WO-01-qualify-before-issuance.md) | PROTECTED | Qualify configurations before issuance | Proposed |
| [WO-02](WO-02-single-lifecycle-authority.md) | NORMAL | One executable lifecycle authority | Proposed |
| [WO-03](WO-03-isolate-verification.md) | PROTECTED | Isolate verification and account for provider calls | Proposed |
| [WO-04](WO-04-stability-proof.md) | NORMAL | Complete the delivery-stability proof | Proposed |

STAB P4 questions ([STAB WO-04](../STAB/WO-04-decide-next.md)): A (verification ordering) goes to
WO-03 and D (lifecycle rules encoded twice) to WO-02. B (dependency scope) and C (inherited evidence)
were answered by FAI and confirmed on F-0006: a test-only change left F-0001's evidence and
acceptance current, and F-0006's F5 inherited F-0001's evidence without a production run.

Each WO follows the previous one. This table is the only place WO status is kept.
