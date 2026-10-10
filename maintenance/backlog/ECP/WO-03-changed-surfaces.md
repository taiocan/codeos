# WO-03 — Changed Surfaces and Evidence Identities

ECP-3 · NORMAL · after WO-02 · Charter: O-2, C-3, C-10

## Goal

Stale or blocked items show exactly which governed inputs changed and which evidence identities
apply.

**Invariant:** every reported changed or unchanged surface is the output of the same comparison
CodeOS used to decide the underlying state; the monitor never reconstructs why validity failed.

## In scope

- Structured `{binding, was, now}` alongside `CheckpointReport` strings, from `ReceiptState::Stale`
  and `RecordState::Stale`.
- For an accepted feature no longer current: accepted snapshot bindings vs current bindings.
- Attention fields: `affected_surfaces` (always), `evidence` (Candidate Bundle, receipt,
  verification record identities), `unchanged_surfaces` **only** for bindings positively compared
  equal.

## Out of scope

New identity systems; recomputing bindings not already derived.

## Sources

`hashing::binding_comparisons` (with `binding_drift` as its first change), `receipts::evaluate`,
`verification::evaluate`, `candidate::acceptance_comparisons` (behind `acceptance_current`),
`revalidation::acceptance_assessment` (behind `current_acceptance_valid`), and
`hashing::record_identity`, the existing identity Candidate Bundles persist.

## Acceptance criteria

1. An Intent changed after acceptance lists `intent` as affected with accepted and current hashes.
2. A binding not compared never appears in `unchanged_surfaces` (negative test).
3. Stale verification lists the binding named by record evaluation.
4. Existing text and JSON fields are unchanged apart from additions.

## Verification

README baseline plus `monitor_status.rs` and `feature_workflow.rs` fixtures.
