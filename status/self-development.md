# Self-Development Status

> Live dashboard for in-flight Codeos toolkit changes. **Mutable** — maintained by the
> 4-step self-development loop (see repo-root `CLAUDE.md` and `prompts/codeos-self-dev.md`).
> The stable rulebook is `CLAUDE.md`; per-change detail is in
> `changes/UPG-####__CHG-YYYYMMDD-NNN__slug.md`; the wave plan is `status/roadmap.md`; the
> authoritative feature-id map is `backlog/features.md`.
>
> **Identity is split** (see `backlog/UPG-0001-feature-thread-traceability.md`): a **Feature ID**
> (`UPG-####`, stable) names the backlog feature; a **Change ID** (`CHG-YYYYMMDD-NNN`) names one
> execution against it. A historical row may carry a legacy change-stem in **Change ID** and `—`
> in **Feature ID** when no backlog feature exists.
>
> Step 1 activates a row; each step updates **Loop step** and **Latest review**. Reconcile marks
> **State: COMPLETE** once the step's compulsory (advisory) review has run **and** the human has
> accepted the result — NO OBJECTION, or CHANGES ADVISED with residual non-blocking findings
> accepted or tracked to a named **Follow-up** `UPG-####`. The review informs the human; it never
> gates by itself. A change with open, unaccepted in-scope findings stays **IN_PROGRESS**.

| Feature ID | Change ID | Class | Scope | Loop step | Review (state / accepted outcome) | State | Follow-up |
|---|---|---|---|---|---|---|---|
| — | 0001-claude-split | self-dev-governance + downstream-doctrine | both | 4-Reconcile | CHANGES ADV →0004 | COMPLETE | — |
| UPG-0002 | 0002-doc-consistency-rename | documentation | downstream doctrine only | 4-Reconcile | CHANGES ADV →0004 | COMPLETE | — |
| — (planning) | 0003-implementation-roadmap | backlog-only / planning | self-dev only | 4-Reconcile | CHANGES ADV →0004 | COMPLETE | — |
| — (reviewer findings) | 0004-review-fixes | documentation / self-dev-governance | self-dev only | 4-Reconcile | CHANGES ADV (accepted) | COMPLETE | — |
| UPG-0001 | CHG-20260627-001 | self-dev-governance | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; rounds in review-log) | COMPLETE | UPG-0029 |
| UPG-0029 | CHG-20260629-001 | documentation | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; rounds in review-log) | COMPLETE | UPG-0030 |
| UPG-0030 | CHG-20260629-001 | self-dev-governance | self-dev only | 4-Reconcile | ACCEPTED (series RVS__…__S4; rounds in review-log) | COMPLETE | — |
| UPG-0027 | CHG-20260629-002 | prompt / script-tooling | self-dev only | 4-Reconcile | ACCEPTED (NO OBJECTION) | COMPLETE | — |
| UPG-0027 | CHG-20260629-003 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (NO OBJECTION) | COMPLETE | — |

<!--
Identity   : Feature ID = UPG-#### (stable). Change ID = CHG-YYYYMMDD-NNN (one execution), or a
             legacy change-stem (0001..0004) for historical/piloted work recorded truthfully.
Loop step  : 1-Intent / 2-Acceptance / 3-Implement / 4-Reconcile / —
Review     : review STATE/outcome, not a live round — in-flight: `review_state` (IN_REVIEW…) + the
             review series `RVS__…__S<N>`; closed: the accepted verdict summary. Exact `REV__…__R<N>`
             rounds + human decisions live ONLY in reviews/review-log.md (Self-Reference Boundary).
             `→UPG-####` / `→NNNN` = findings tracked to a follow-up.
State      : PROPOSED / IN_PROGRESS / BLOCKED / COMPLETE (+ feature states PILOTED/SUPERSEDED/ABANDONED)
Follow-up  : UPG-#### tracking out-of-scope findings from this change, or —
-->
