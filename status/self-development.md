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
| UPG-0027 | CHG-20260629-004 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (human approved; all 12 ACs verified by functional test) | COMPLETE | — |
| UPG-0027 | CHG-20260629-005 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (human approved; all 14 ACs verified by functional test) | COMPLETE | — |
| UPG-0004 | CHG-20260630-001 | template | self-dev only | 4-Reconcile | ACCEPTED (Steps 2–4 reviewed; R2 NO OBJECTION at each gate) | COMPLETE | UPG-0031 |
| UPG-0031 | CHG-20260630-002 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (Step 4 R2 NO OBJECTION; corrected by CHG-20260630-003) | COMPLETE | — |
| UPG-0031 | CHG-20260630-003 | script-tooling + documentation | self-dev only | 4-Reconcile | ACCEPTED (all 5 ACs verified; corrects false B8b in CHG-20260630-002) | COMPLETE | — |
| UPG-0005 | CHG-20260630-004 | prompt + template | self-dev only | 4-Reconcile | ACCEPTED (Step 3 R2 NO OBJECTION; Step 4 R1 CHANGES ADVISED — F1 fixed, F2 REVIEW-BOOKKEEPING) | COMPLETE | — |
| UPG-0007 | CHG-20260630-005 | prompt | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R2 NO OBJECTION; all 7 ACs verified) | COMPLETE | — |
| UPG-0033 | CHG-20260701-001 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (all 8 ACs verified; F-A investigation-first exception accepted; F-B AJ-009 deferred) | COMPLETE | — |
| UPG-0010 | CHG-20260701-002 | prompt | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R1 NO OBJECTION; all 7 ACs verified) | COMPLETE | — |
| UPG-0011 | CHG-20260701-003 | template | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R2 NO OBJECTION; all 9 ACs verified) | COMPLETE | — |
| UPG-0012 | CHG-20260701-004 | prompt | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R1 NO OBJECTION; all 8 ACs verified) | COMPLETE | — |
| UPG-0013 | CHG-20260701-005 | template | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R1 NO OBJECTION; all 8 ACs verified) | COMPLETE | — |
| UPG-0008 | CHG-20260701-006 | prompt | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R1 NO OBJECTION; all 7 ACs verified) | COMPLETE | — |
| UPG-0009 | CHG-20260701-007 | template | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R3 NO OBJECTION; all 7 ACs verified) | COMPLETE | — |
| UPG-0006 | CHG-20260701-008 | prompt | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R1 NO OBJECTION; all 6 ACs verified) | COMPLETE | — |
| UPG-0032 | CHG-20260702-001 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (Step 3: 6 rounds, all blockers applied; Step 4 R1 NO OBJECTION; all 11 ACs verified; 33 tests; post-reconcile delta-mode fix applied) | COMPLETE | UPG-0034, UPG-0035 |
| UPG-0015 | CHG-20260702-002 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (Steps 1–4 reviewed; Step 4 R2 NO OBJECTION; all 13 ACs verified; 48 tests; coverage gate + provenance binding in Rust engine) | COMPLETE | — |
| UPG-0016 | CHG-20260702-003 | documentation | self-dev only | 4-Reconcile | ACCEPTED (Step 4 R1 NO OBJECTION; all 7 ACs verified) | COMPLETE | — |
| UPG-0014 | CHG-20260702-004 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (Step 3: 3 rounds + budget-exceeded inline fixes; Step 4 R3 NO OBJECTION; all 9 ACs verified; 55 tests) | COMPLETE | — |
| UPG-0035 | CHG-20260702-005 | script-tooling | self-dev only | 4-Reconcile | ACCEPTED (Step 3 R1 + Step 4 R1 CHANGES ADVISED — AC-6 test-exec-not-pinned REJECTED both rounds; all 6 ACs verified; 57 tests) | COMPLETE | — |

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
