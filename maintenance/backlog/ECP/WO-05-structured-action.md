# WO-05 — Structured Action

- **Charter refs:** ECP-5, O-4, C-7, C-8
- **Classification:** NORMAL
- **Depends on:** WO-04

## Goal

Each next action can be read by machines without reinterpreting free text. It remains read-only
projection; nothing is executed.

## In scope

- Add `action: {type, subject, reason, actor, authority, command?}` alongside the existing
  `next_action` text.
- `type` is a closed vocabulary, defined in one place. Initial set (finalize during
  implementation):
  - `run_verification`
  - `run_project_qualification`
  - `prepare_candidate`
  - `run_review`
  - `record_decision`
  - `approve_artifact`
  - `complete_specification`
  - `repair_implementation`
  - `revalidate_acceptance`
  - `route_observation`
  - `close_observation`
- `command` is emitted **only when executing it is sufficient to perform the action**, not merely
  related to it. For example, `repair_implementation` and `approve_artifact` have no command.
- `reason` and `authority` are reused from WO-02.

## Out of scope

- Executing any action (WO-07).
- Removing or rewording `next_action`.

## Authoritative sources to reuse

The actor derivation in `checker.rs` (`Actor::default_for`, `settle_actor`), and the WO-02
reason and authority fields.

## Acceptance criteria

1. Every next action has a `type` from the closed set, and a test enforces the set.
2. Every action whose actor is HUMAN or ASSISTED lacks `command`, unless the command itself only
   records a human-supplied decision. Each exception is listed in a test.
3. Every emitted `command` exists as a `codeos-workflow` subcommand, and its arguments are
   complete for the subject.
4. Actor classes match the v1 derivation. F8 and F9 remain HUMAN.

## Verification

The README baseline, plus `monitor_status.rs` and `feature_workflow.rs` assertions.
