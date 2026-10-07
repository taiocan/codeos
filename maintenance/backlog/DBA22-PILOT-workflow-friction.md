# DBA-22 Pilot — Workflow Friction and Hardening Candidates

Classification: NORMAL candidates — not yet analyzed or approved

Observed while running JamFree F-0003 under DBA-22 (2026-10-07).

1. **Test-only edits stale human receipts.** The implementation identity covers every file under
   `web/`, so a Playwright-only fix invalidated the `early_preview` receipt and required a second
   human direction call although the previewed UI was unchanged.
2. **Behavior mechanic omits web tests.** JamFree's F5 `behavior` mechanic runs only
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
