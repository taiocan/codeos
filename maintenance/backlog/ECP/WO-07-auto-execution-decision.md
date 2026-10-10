# WO-07 — Bounded AUTO Execution: Decision Brief

- **Charter refs:** ECP-6, O-5, success criteria 11–13, C-8
- **Classification:** PROTECTED
- **Depends on:** WO-06 and its routed gaps

## Goal

Prepare the human decision on whether CodeOS may execute allowlisted `AUTO` actions, and on what
terms.

## Output

**A decision brief only.** This WO makes no implementation change, no schema change, and no
status change.

## In scope

The brief states:

- **The consequential semantic change.** CodeOS initiates execution of commands.
- **What must remain true.**
  - Every HUMAN boundary of the active DBA configuration remains human-controlled.
  - Nothing is approved automatically: Charter, specification, architecture, F8, F9,
    Observation decisions, and reviewer judgments are all excluded.
- **Proposed allowlist.** Drawn only from WO-05 action types with a sufficient `command`.
- **Loop rules.**
  - Run one action at a time.
  - Record provenance through existing mechanisms.
  - Re-derive `status` after every action.
  - Stop when the legitimate next action changes.
  - Stop at every ASSISTED or HUMAN boundary.
- **Verification proposal.** How criteria 11–13 would be proven.
- **Open decisions.** For example, whether any new persisted record is needed.

## Out of scope

- Writing implementation WOs. They follow only after human approval.

## Acceptance criteria

1. The brief satisfies the CLAUDE.md PROTECTED pre-implementation statement.
2. The human records approval or rejection. Approved semantics go into the authoritative
   governing artifact before any implementation.
