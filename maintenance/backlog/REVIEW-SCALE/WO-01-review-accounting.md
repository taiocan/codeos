# WO-01 — Deterministic Review Accounting

NORMAL · approved by the human 2026-11-16

## Goal

Every review states exactly what it contained, why, and what it cost, derived from existing review
records. No new store and no score.

## Work

1. **Packet manifest.** Each item gets one accounting `role` (governed, candidate,
   changed_implementation, test_definition, test_evidence, production_evidence, runtime_evidence,
   declared) and `reasons[]` listing every derivation rule that included it. A fixed precedence
   picks the role (changed_implementation over test_definition), so role totals sum exactly to
   `review_content_bytes`. The manifest also records `budget_bytes` and
   `accepted_feature_count_at_review`.
2. **After each review**, print bytes by role, the top three contributors, and model, effort,
   elapsed time and result.
3. **`codeos-reviewer review-status [--json]`**, read-only, derived from `review-log.md` and the
   packets and assessments it references: composition and budget use; attempts per feature
   (first, reassessment, execution-invalid); hotspots; first-review size against feature count;
   median elapsed time, and p95 only when n ≥ 10.

## Rules

- A missing or hash-mismatched packet or assessment makes the status `INCOMPLETE` and names it.
- Older packets get roles inferred from unambiguous path prefixes, marked `inferred` and kept
  separate from recorded roles; anything else is `unrecorded`.
- Feature count for older reviews is `unknown`, never inferred from today's count.
- Elapsed time is observational only; no outcome depends on it.

## Acceptance criteria

1. Every manifest item has one role and its full reasons; a changed file that also defines tests
   counts once; role totals equal `review_content_bytes`.
2. A fixture over a synthetic log checks attempt kinds, `unknown` feature count, `inferred` and
   `unrecorded` roles, and `INCOMPLETE` on a missing or hash-mismatched packet.
3. `cargo test` and `dba/04-tools/tests/run.sh` pass.
4. The JamFree baseline from `review-status` is recorded below.

## Baseline

JamFree, 2026-11-16, `codeos-reviewer review-status` (health OK: all 169 referenced packets and
assessments present with matching hashes).

| Measure | Value |
|---|---|
| Last F-0001 / F-0002 Stage 8 packets | 437,568 / 398,998 bytes (budget 460,800), including 18,788 bytes of verification records |
| Elapsed, last 20 | median 85 s, p95 154 s |
| Composition, last 20 | unrecorded 47 % (implementation and test-definition files before accounting), test evidence 17 %, production evidence 16 %, governed 10 %, verification records 5 %; roles other than records and diff are `inferred` |
| Hotspots, last 20 | `provider.rs` 48 KB × 20, `main.rs` 44 KB × 20, `page.e2e.ts` 46 KB × 18 |
| First reviews by feature count | 17 first reviews, all `unknown` (recorded from now on) |
