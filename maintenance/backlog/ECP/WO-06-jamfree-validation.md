# WO-06 — JamFree Pilot Validation

- **Charter refs:** §8 success criteria 1–10, C-9
- **Classification:** NORMAL
- **Depends on:** WO-05

## Goal

Confirm on a real project that the control panel answers the Charter's ten questions without
manual reconstruction of repository state.

## In scope

- Run `codeos-workflow status`, `status --json`, and `status --history` read-only on JamFree.
- For each success criterion 1–10, record ANSWERED, PARTIAL, or NOT ANSWERED, with the output
  excerpt that supports it.
- Produce a **diagnostic gap report** as the deliverable (location to be agreed when the WO
  starts).
- A human routes each material gap to an Observation, a backlog WO, or no action.

## Out of scope

- Creating Observations or any other workflow state.
- Changing JamFree.
- Adding JamFree-specific rules to CodeOS (C-9).
- Consuming JamFree TomTom calls (follow the quota guidance in project memory).

## Acceptance criteria

1. The report covers all ten criteria, each with evidence.
2. JamFree's `.codeos/` is byte-identical before and after the run.
3. No gap is fixed inside this WO.

## Verification

Compare checksums of JamFree's `.codeos/` before and after, and have the human review the gap
report.
