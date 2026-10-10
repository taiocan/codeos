# WO-06 — JamFree Pilot Validation

Real use · NORMAL · after WO-05 · Charter: success criteria 1–10, C-9

## Goal

Confirm on a real project that the panel answers the Charter's ten questions without manual
repository reconstruction.

## In scope

- Run `status`, `status --json`, `status --history` read-only on JamFree.
- Rate each success criterion ANSWERED, PARTIAL, or NOT ANSWERED with supporting output.
- Deliver a diagnostic gap report (location agreed at start); the human routes each material gap
  to an Observation, a backlog WO, or no action.

## Out of scope

Creating Observations or other workflow state; changing JamFree; JamFree rules in CodeOS (C-9);
TomTom calls.

## Acceptance criteria

1. All ten criteria are covered with evidence.
2. JamFree `.codeos/` is byte-identical before and after.
3. No gap is fixed inside this WO.

## Verification

`.codeos/` checksums before and after; human review of the gap report.
