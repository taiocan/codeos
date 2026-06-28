# Self-Development Status

> Live dashboard for in-flight Codeos toolkit changes. **Mutable** — maintained by the
> 4-step self-development loop (see repo-root `CLAUDE.md` and `prompts/codeos-self-dev.md`).
> The stable rulebook is `CLAUDE.md`; per-change detail is in `changes/[change_id].md`;
> the roadmap (wave plan) is `status/roadmap.md`; the stable backlog catalog is
> `backlog/features.md`.
>
> Step 1 activates a row; each step updates **Loop step** and **Review**. Reconcile marks
> **State: COMPLETE** once the step's compulsory (advisory) review has run **and** the human
> has accepted the result — NO OBJECTION, or CHANGES ADVISED with residual non-blocking
> findings accepted or tracked to a named follow-up (e.g. `→0004`). The review informs the
> human; it never gates by itself. A change with open, unaccepted findings stays **IN_PROGRESS**.

| Backlog item | Change ID | Class | Loop step | Review | State |
|---|---|---|---|---|---|
| — | 0001-claude-split | self-dev-governance + downstream-doctrine | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
| backlog/doc-consistency-doctrine-rename | 0002-doc-consistency-rename | documentation | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
| — (whole-backlog planning) | 0003-implementation-roadmap | backlog-only / self-dev planning | 4-Reconcile | CHANGES ADV →0004 | COMPLETE |
| — (reviewer findings) | 0004-review-fixes | documentation / self-dev-governance | 4-Reconcile | CHANGES ADV (accepted) | COMPLETE |
| backlog/UPG-0001-feature-thread-traceability.md | CHG-20260627-001 | self-dev-governance | 2-Acceptance | NO OBJ (r4) | IN_PROGRESS |

<!--
Loop step : 1-Intent / 2-Acceptance / 3-Implement / 4-Reconcile / —
Review    : latest advisory verdict — NO OBJ / CHANGES ADV / DO NOT ADV / —
            suffix `→NNNN` = findings tracked to follow-up change NNNN
            suffix `(rN)`  = review round N of an iterative review
State     : BACKLOG / IN_PROGRESS / BLOCKED / COMPLETE
-->
