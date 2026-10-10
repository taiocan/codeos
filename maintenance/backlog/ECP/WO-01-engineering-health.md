# WO-01 — Engineering Health Projection

- **Charter refs:** ECP-2, O-1, C-3, C-4, C-5, C-6, C-9
- **Classification:** NORMAL
- **Depends on:** Charter approval (satisfied 2026-10-10)

## Goal

`status` reports one health result per engineering domain. Each result comes from existing
CodeOS authority or carries an explicit absence state.

## In scope

- JSON: `health: [{domain, status, basis}]`, where `basis` names the source record, artifact, or
  identity. Terminal: an ENGINEERING HEALTH section.
- A **closed status set per domain**, defined in one table in the code and added to the glossary.
  No domain may emit a status outside its set. Initial table (finalize during implementation):

  | Domain | Allowed statuses |
  |---|---|
  | Product | CURRENT, UNRESOLVED |
  | Specification | CURRENT, STALE, UNRESOLVED |
  | Architecture | CURRENT, STALE, UNRESOLVED, NOT_APPLICABLE |
  | Verification | PASS, BLOCKED, UNOBSERVED |
  | Evidence | CURRENT, STALE, UNOBSERVED |
  | Independent review | PASSED, BLOCKED, UNOBSERVED |
  | Project quality / Security / Supply chain | PASS, BLOCKED, STALE, MEASURED, UNOBSERVED, NOT_CONFIGURED |
  | Deployment | PASS, BLOCKED, UNOBSERVED |
  | Release | QUALIFIED, UNQUALIFIED, NOT_CONFIGURED |
  | Compatibility | SUPPORTED, UNSUPPORTED, NOT_CONFIGURED |
  | Operational learning | `n OPEN`, NONE |

- Build in ordered stages:
  - **(a) Workflow-derived domains.** Product, Specification, Architecture, Verification,
    Evidence, Independent review, and Operational learning, taken from facts that
    `monitor.rs`, `checker.rs`, and `revalidation.rs` already compute.
  - **(b) Qualification-derived domains.** Project quality, Security, Supply chain, and
    Deployment come from the `project_qualification` verification record (F9, Feature
    Workflow v9).
    - Select **only an identity-current record**: one whose bindings still match the current
      implementation, Charter, and configuration, as decided by `verification::evaluate`.
    - When the newest record is stale, report STALE. When there is no record, report UNOBSERVED.
    - A constraint with a measurement but no approved limit reports MEASURED, never PASS.
    - Deployment-mode results from `qualify-project` are not persisted, so Deployment reports
      UNOBSERVED unless a persisted deployment basis exists.
  - **(c) Release and Compatibility.**
    - Release uses the binary's embedded release identity (`release_identity.rs`).
    - Compatibility uses `dba/04-tools/compatibility/support.json` for the active configuration.
    - **Stop rule:** a value that cannot be resolved from a downstream project reports
      NOT_CONFIGURED, and the gap is listed in the commit message. This WO does not grow to
      close it.

## Out of scope

- Persisting deployment qualification.
- Any aggregate or overall score.
- Project-specific thresholds or domains (C-5, C-9).
- Drill-down flags.

## Authoritative sources to reuse

`monitor::ProjectStatus::derive`, `checker::evaluate`, `verification::evaluate` and
`VerificationStore`, `qualification::QualificationResult` details, `revalidation::*`,
`ObservationStore`, `release_identity.rs`, `compatibility/support.json`.

## Acceptance criteria

1. Every domain is present in every output, and its status is in that domain's allowed set.
   A unit test enforces the set.
2. A qualification record whose bindings are stale never yields PASS. A test with a newer stale
   record and an older current one yields STALE, not PASS.
3. A project without `scripts/codeos-qualify` reports NOT_CONFIGURED for the qualification
   domains. A project with the script but no record reports UNOBSERVED.
4. A measured constraint with no approved limit reports MEASURED.
5. A feature that is accepted but no longer current still shows STALE in the affected domain.
   Historical acceptance stays visible elsewhere (C-3).
6. Every non-absence status has a non-empty `basis`.
7. `status` stays read-only, and the existing exit codes are unchanged.

## Verification

The README baseline, plus new fixtures in `engine/tests/monitor_status.rs` and
`engine/tests/dba25_protocols.rs` for criteria 1–7.
