# WO-07 — Bounded AUTO Execution: Decision Brief

ECP-6 · PROTECTED · after WO-06 and its routed gaps · Charter: O-5, success criteria 11–13, C-8

## Goal

Prepare the human decision on whether, and on what terms, CodeOS may execute allowlisted `AUTO`
actions. Output is a decision brief only: no implementation, schema, or status change.

## In scope

The brief states:

- the semantic change: CodeOS initiates command execution;
- what must remain true: every HUMAN boundary of the active DBA configuration stays
  human-controlled; nothing is approved automatically (Charter, specification, architecture, F8,
  F9, Observation decisions, reviewer judgments);
- the proposed allowlist, drawn only from WO-05 types with a sufficient `command`;
- loop rules: one action at a time, provenance through existing mechanisms, re-derive `status`
  after each action, stop when the legitimate next action changes, stop at every ASSISTED or HUMAN
  boundary;
- how criteria 11–13 would be verified, and open decisions (e.g. new persisted records).

## Out of scope

Implementation WOs; they follow only after approval.

## Acceptance criteria

1. The brief meets the CLAUDE.md PROTECTED pre-implementation statement.
2. The human records approval or rejection; approved semantics enter the governing artifact before
   implementation.
