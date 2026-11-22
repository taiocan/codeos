# WO-DM-02 — Structured Test-Evidence Durations

- **Class:** NORMAL. It becomes PROTECTED if the change would alter any execution identity, hash
  input or validity decision.
- **Goal:** each test-evidence run records `duration_ms` as structured data. Today suite timing
  exists only inside free-text summaries.

## Scope

- First check where `test-evidence/<sha>.json` content feeds identities, bindings or validity
  (`dba/04-tools/workflow/engine/src/test_evidence.rs`, `evidence_set.rs`). If adding a field
  changes any of them, stop and treat the change as PROTECTED.
- Record `duration_ms` per run, measured by the workflow engine around the command it executes.
- Existing records stay unchanged. Consumers show their duration as UNOBSERVED.

Additive-field test: CodeOS benefits through slow-suite and flaky-run diagnosis in its own
tooling.

## Verification

- `cargo test` passes.
- Existing evidence and execution identities are byte-identical before and after.
- A new run records a plausible `duration_ms`.

## Acceptance

The verification passes and the human accepts.
