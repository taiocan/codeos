# WO-04 — DBA Impact-Aware Adoption

PROTECTED · after WO-02, reusing its dependency model

## Goal

Adopting a DBA re-runs only what its semantic change affects.

## Direction

- Compare the old and new configurations' semantic closures, and mechanically derive which bindings
  and checkpoints change. No manual "invalidates" declarations.
- Accepted features that are unaffected stay valid.
- Operating rule, not a mechanism: projects adopt DBA versions between feature cycles.

## Verification

Replay JamFree's DBA-32 → 33 → 34 adoptions, and show which re-runs were necessary.

## Observed case: JamFree DBA-34 → DBA-35 (2026-11-18)

JamFree adopted DBA-35 in c3b2e56. Read-only `status` afterwards showed:

- **v14 held.** F-0001, F-0002, F-0009 and F-0010 are VALID, F1–F9 complete, and no F6, F8 or F9
  decision is asked.
- **Still invalidated by the `codeos.yaml` bytes alone:**
  - B3, with B4 behind it (another human re-confirmation);
  - the four features' `project_qualification` records and the deployment qualification;
  - the Health and Platform Assurance states derived from them.

DBA-35 changed neither the platform baseline nor the qualification semantics. These are the
bindings WO-04 must derive from semantic impact: `checker.rs` binds `codeos_yaml` for B3, B4
(`initial_product_preview`), F6 and `project_qualification`.
