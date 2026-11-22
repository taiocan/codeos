# WO-05 — Structured Action

ECP-5 · NORMAL · after WO-04 · Charter: O-4, C-7, C-8

## Goal

Each next action is machine-readable without reinterpreting free text. Read-only; nothing is
executed.

**Invariants:** an action type names exactly one next operation, never a workflow outcome or a
multi-step objective; it comes from the same requirement and outcome as actor and reason.
`actor = AUTO` means no human judgment is needed; a `command` means CodeOS knows a complete,
sufficient invocation for this exact action context. AUTO does not imply a command; only AUTO
actions with a command are candidates for ECP-6 execution.

## In scope

- `action: {type, command}` on each attention item; subject, reason, actor, and authority stay
  sibling fields rather than being duplicated. `next_action` text is unchanged.
- Closed `type` set: `author_artifact`, `repair_configuration`, `revise_specification`,
  `approve_artifact`, `record_decision`, `follow_decision_outcome`, `run_verification`,
  `collect_production_evidence`, `repair_failure`, `prepare_candidate`, `run_review`,
  `resolve_review_finding`, `restore_review_history`, `reassess_architecture`,
  `record_observation`, `route_observation`, `close_observation`, `resolve_routed_target` (the
  operations themselves are reported in the target's workflow), `commit_change` (WO-10).
  `revalidate_acceptance` is a reserved schema-v3 value that is never emitted: it named the
  multi-step revalidation outcome; WO-10 reports its atomic steps instead.
- `prepare_candidate` vs `run_review` follows whether a unique current Candidate Bundle exists.
  A checkpoint blocked by an earlier one, including `bootstrap-entry`, inherits that action.
- `command` derives from the action context (type, workflow, subject, revalidation trigger), not
  the type alone: `run_verification` (`check`, with `--revalidation --trigger <sha>` for a
  revalidation step) and a revalidation `prepare_candidate` (WO-10).

## Out of scope

Executing actions (WO-07); removing or rewording `next_action`.

## Sources

`Requirement::action` (`contract.rs`), `settle_actor` in `checker.rs`, `action_command` and
`stale_cause` in `monitor.rs`; WO-02 fields.

## Acceptance criteria

1. Every next action has a `type` from the closed set; the vocabulary is pinned in tests.
2. HUMAN and ASSISTED actions never carry a `command`.
3. Every emitted `command` is a complete `codeos-workflow` invocation that fully performs the
   action.
4. Actor classes match the v1 derivation; F8 and F9 remain HUMAN; a test pins that AUTO does not
   imply a command.

## Verification

README baseline plus `monitor.rs` unit tests and `monitor_status.rs` and `feature_workflow.rs`
assertions.
