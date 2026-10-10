# WO-05 — Structured Action

ECP-5 · NORMAL · after WO-04 · Charter: O-4, C-7, C-8

## Goal

Each next action is machine-readable without reinterpreting free text. Read-only; nothing is
executed.

## In scope

- `action: {type, subject, reason, actor, authority, command?}` alongside the `next_action` text.
- Closed `type` set, defined once (finalize during implementation): `run_verification`,
  `run_project_qualification`, `prepare_candidate`, `run_review`, `record_decision`,
  `approve_artifact`, `complete_specification`, `repair_implementation`, `revalidate_acceptance`,
  `route_observation`, `close_observation`.
- `command` only when executing it is **sufficient** to perform the action (none for
  `repair_implementation` or `approve_artifact`).
- `reason` and `authority` reused from WO-02.

## Out of scope

Executing actions (WO-07); removing or rewording `next_action`.

## Sources

`Actor::default_for` and `settle_actor` in `checker.rs`; WO-02 fields.

## Acceptance criteria

1. Every next action has a `type` from the closed set (enforced in tests).
2. HUMAN and ASSISTED actions have no `command`, except commands that only record a human-supplied
   decision; each exception is listed in a test.
3. Every emitted `command` is an existing `codeos-workflow` subcommand with complete arguments.
4. Actor classes match the v1 derivation; F8 and F9 remain HUMAN.

## Verification

README baseline plus `monitor_status.rs` and `feature_workflow.rs` assertions.
