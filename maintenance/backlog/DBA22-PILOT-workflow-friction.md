# DBA-22 Pilot — Workflow Friction and Hardening Candidates

Classification: NORMAL candidates — not yet analyzed or approved

Observed while running JamFree F-0003 under DBA-22 (2026-10-07).

## Direction (human decision, 2026-10-07)

- DBA-22 semantics are frozen. These items are DBA-22 conformance or tooling hardening; a new DBA
  version requires a genuinely new governance rule, not a bug fix.
- Historical state at STAB creation: 1/3 qualifying features (JamFree F-0003); F-0002 did not
  count. The forward stabilization criterion and completion decision are now owned by
  `maintenance/backlog/STAB/charter.md`.
- Priority order: items 7, 1, 2, 8 below. Everything else waits.

## Candidates

1. **[Done — human-approved]** **Test-only edits stale human receipts.** The implementation identity covers every file under
   `web/`, so a Playwright-only fix invalidated the `early_preview` receipt and required a second
   human direction call although the previewed UI was unchanged.
2. **[Done — conformance repair]** **Behavior mechanic omits web tests.** JamFree's F5 `behavior` mechanic runs only
   `cargo test --workspace`; web unit and component tests appear in no workflow verification
   record, so GUI-only features must cite them outside the workflow.
3. **Preserve-mode journey check targets `@frontier:entry`.** For a feature preserving the accepted
   `outbound_departure_decision` frontier, F3 product-journey verification ran only the entry
   frontier test.
4. **Production verification is single-route.** The production-source verifier hard-codes one
   route; when the provider truthfully classifies it unsupported, verification fails without a
   product defect.
5. **Same-second review records.** `latest_review_record` orders by second-resolution file names,
   so records written within one second have an undefined order.
6. **`plan` suggests a refused option.** Over budget, `plan` still recommends
   `review ... --base <last-review-commit>`, which Reviewer Tool Contract v17 refuses at Stage 8.
7. **[Done — conformance repair]** **Committing the review record stales F7.** F7 binds the reviewer record to the exact HEAD
   commit, so committing the record itself (or any later workflow receipt) moves HEAD and requires
   re-review unless acceptance is recorded before those records are committed.
8. **[Done — tooling hardening + per-project budget]** **Required verification inputs dominate packet size.** The F6-selected production evidence and
   its verification record alone exceed the 50 KB packet budget, so every JamFree Stage-8 review
   needs `CODEOS_PACKET_BUDGET_MODE=warn`.
9. **[Done — conformance repair]** **Revalidation trigger required HEAD.** Governance-only commits
   after a multi-commit dependency change made post-acceptance revalidation impossible (JamFree
   F-0001 after F-0003). Fixed in `62bec4e`.
10. **Journey reachability view has no data.** `codeos-workflow journey` reports accepted
    reachability MISSING because JamFree's product-journey verification records carry no
    structured per-transition observations; F-checkpoints are unaffected.
11. **Flaky implementer check.** `OUTPUT terminology authority boundary` in
    `dba/04-tools/implementer/tests/codeos-implement-tests.sh` fails intermittently.
