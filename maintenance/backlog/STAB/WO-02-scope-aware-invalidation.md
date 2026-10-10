# WO-02 — Scope-Aware Human-Decision Invalidation

P2 · PROTECTED · after WO-01 · Charter: Phases P2, Constraints

## Goal

A human receipt goes stale only when the surface it judged changes.

## Finding

`early_preview` binds `product_state`, which covers every non-test implementation file
(`hashing::product_state`). `reconciliation`, `final_ux_validation`, and `acceptance` bind the full
`implementation_state` (`checker::bindings_for_decision`). So an internal change that no user can
see, such as JamFree `0fd5954` in `provider.rs`, invalidates F4, F6, F8, and F9.

## Part A — Decision brief (before any code)

This part follows the CLAUDE.md PROTECTED statement and covers:

- **What each receipt judges and should bind.** For each of the four receipt checkpoints, the
  surface it judges and the binding that represents it.
- **How the surface is derived.** Mechanically, from canonical state, verification records, and
  receipts. No declared exemptions and no vouching.
- **What must remain true.** No receipt survives a change to what it judged. History is never
  rewritten. Semantics are versioned through the owning policy, not a new DBA configuration.
- **Verification and open decisions.**

The human approves or rejects Part A.

## Part B — Implementation (after approval)

The approved semantics enter the owning policy first. Tests show that:

- an internal change keeps F4 current;
- a user-visible change makes F4 stale;
- a Candidate Bundle change still invalidates F7 to F9 as it does today.

## Acceptance criteria

1. Part A is approved and recorded in the owning policy.
2. The Part B tests pass.
3. Replaying the WO-01 table under the new rules produces only stale receipts whose judged surface
   changed.

## Results
