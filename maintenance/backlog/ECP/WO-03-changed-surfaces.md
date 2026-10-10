# WO-03 — Changed Surfaces and Evidence Identities

- **Charter refs:** ECP-3, O-2, C-3, C-10
- **Classification:** NORMAL
- **Depends on:** WO-02

## Goal

For stale or blocked items, show exactly which governed inputs changed and which evidence
identities are involved.

## In scope

- Record structured stale-binding data `{binding, was, now}` alongside `CheckpointReport`'s
  existing strings. Sources are `ReceiptState::Stale` and `RecordState::Stale`.
- For an accepted feature that is no longer current, diff the accepted snapshot bindings against
  current governed bindings (`revalidation.rs`, `AcceptedSnapshot.bindings`).
- Add attention fields:
  - `affected_surfaces`: always present, possibly empty.
  - `evidence`: the relevant identities, such as the Candidate Bundle, receipt hash, or
    verification record.
  - `unchanged_surfaces`: **only** for bindings CodeOS positively compared as equal. It is never
    inferred from no change having been detected.

## Out of scope

- New identity systems.
- Recomputing bindings that are not already derived.

## Authoritative sources to reuse

`receipts::evaluate`, `verification::evaluate`, `checker::current_bindings` and
`verification_bindings`, `monitor::acceptance_snapshot`, `revalidation::current_acceptance_valid`.

## Acceptance criteria

1. A changed Intent after acceptance lists `intent` as affected and shows the accepted and
   current hashes.
2. A binding that was not compared never appears in `unchanged_surfaces` (a negative test).
3. Stale verification lists the binding named by the record evaluation.
4. Text output and existing JSON fields stay unchanged apart from the additions.

## Verification

The README baseline, plus fixtures in `monitor_status.rs` and `feature_workflow.rs`.
