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

**Human direction (2026-11-18).** JamFree pays the DBA-35 refresh once now (B3, B4,
qualifications), with no TomTom calls; this case is WO-04's canonical example. WO-04's target:

> A DBA or configuration change invalidates Bootstrap or qualification only when the changed
> configuration semantics intersect that checkpoint's actual dependency set. A raw change to the
> `codeos.yaml` bytes is not sufficient by itself.

This applies to B3, B4, project and deployment qualification, and the Health and Platform Assurance
states derived from them. **Boundary:** B4 stays valid only because its own dependency identity is
unchanged; CodeOS never special-cases DBA adoption, or a product that "looks the same".

**Refresh result (JamFree, 2026-11-18).** B3 and all five qualifications passed in about 53 s, with
no provider calls. Still stale afterwards, from the same cause:

- **Health:** the F9-checkpoint `project_qualification` records bind `codeos_yaml`. Only a feature
  `check` writes them; `qualify-project` writes the candidate-checkpoint records.
- **Platform Assurance:** the deterministic F3/F5/F5g records show "execution_identity changed",
  because `evidence_set::common_inputs` includes the `codeos.yaml` bytes. Features stay VALID:
  Current Validity compares the bundle's inputs, not record currency.

A deterministic feature `check` clears both, at about 15–20 min for four features. That is the
measured cost WO-04 removes.
