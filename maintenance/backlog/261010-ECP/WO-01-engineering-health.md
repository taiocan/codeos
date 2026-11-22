# WO-01 — Engineering Health Projection

ECP-2 · NORMAL · after Charter approval · Charter: O-1, C-3, C-4, C-5, C-6, C-9

## Goal

`status` reports one health result per engineering domain, from existing authority or with an
absence state.

## In scope

- JSON `health: [{domain, status, basis}]`; terminal ENGINEERING HEALTH section.
- WO-01 defines the allowed statuses per domain; `health.rs` (`Domain::allowed`) is the single
  implementation location enforcing them. The glossary defines Engineering Health.

  | Domain | Allowed statuses |
  |---|---|
  | Product | CURRENT, UNRESOLVED |
  | Specification, Evidence | CURRENT, STALE, UNRESOLVED, UNOBSERVED |
  | Architecture | CURRENT, STALE, UNRESOLVED, NOT_APPLICABLE |
  | Verification | PASS, BLOCKED, STALE, UNOBSERVED |
  | Independent review | PASSED, BLOCKED, STALE, UNOBSERVED |
  | Project quality, Security, Supply chain, Deployment | PASS, BLOCKED, STALE, NOT_APPLICABLE, UNOBSERVED, NOT_CONFIGURED |
  | Release | QUALIFIED, UNQUALIFIED |
  | Compatibility | SUPPORTED, UNSUPPORTED, NOT_CONFIGURED |
  | Operational learning | OPEN, NONE, NOT_CONFIGURED |
- Workflow domains (Product, Specification, Architecture, Verification, Evidence, Independent
  review, Operational learning) from facts `monitor.rs`, `checker.rs`, `revalidation.rs` compute.
- Qualification domains (Project quality, Security, Supply chain, Deployment) from the F9
  `project_qualification` record: only a binding-current record per `verification::evaluate`,
  never merely the newest; newest record stale → STALE; none → UNOBSERVED. Deployment uses that
  binding-current record's deployment-phase constraints.
- Release from the binary's embedded identity; Compatibility from
  `dba/04-tools/compatibility/support.json`. Unresolvable values report NOT_CONFIGURED.

## Out of scope

Persisting deployment-mode qualification; aggregate scores; project-specific thresholds or domains;
drill-down flags.

## Sources

`monitor::ProjectStatus::derive`, `verification::evaluate`, `VerificationStore`,
`qualification.rs` result details, `revalidation.rs`, `ObservationStore`, `release_identity.rs`.

## Acceptance criteria

1. Every domain appears in every output with a status from its allowed set (enforced in tests).
2. A stale qualification record never yields PASS.
3. No `scripts/codeos-qualify` → NOT_CONFIGURED; script without record → UNOBSERVED.
4. MEASURED only with a basis. Gap: every constraint comes from the approved Charter, so no record
   carries an unapproved measurement; MEASURED is not emitted.
5. An accepted feature that is no longer current makes Verification, Evidence, and Independent
   review STALE, and Specification STALE only for `SEMANTIC_CHANGE_REQUIRED` (any other reason
   proves the package current). Historical acceptance stays visible (C-3).
6. Every non-absence status has a non-empty `basis`.
7. Read-only; exit codes unchanged.

## Verification

README baseline plus `monitor_status.rs` and `dba25_protocols.rs` fixtures for criteria 1–7.
