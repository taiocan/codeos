# WO-02 — Attention Class, Reason, Authority

ECP-3 · NORMAL · after WO-01 · Charter: O-2, §6, C-10

## Goal

Each attention item states what is wrong, why, and which authority establishes it.

## In scope

- `class`: BLOCKER, STALE, UNRESOLVED, WARNING — presentation only, deterministic: accepted
  feature no longer current → STALE; WAITING on a human → UNRESOLVED; BLOCKED → BLOCKER; health
  MEASURED or inventory warning → WARNING. Glossary entry for the classes.
- `reason`: stable code plus existing text, from the unmet requirement kind or revalidation reason
  (e.g. `SEMANTIC_CHANGE_REQUIRED`).
- `authority`: the governing policy path from the active DBA configuration.

## Out of scope

Affected surfaces (WO-03); structured action (WO-05); checkpoint state or exit-code changes.

## Sources

`CheckpointReport`, `contract::Requirement`, `revalidation::invalidity_reason`,
`Project::dba_configuration`.

## Acceptance criteria

1. Every attention item has `class`, `reason.code`, and an `authority` that resolves to an
   existing file, or an explicit absence (Rule 1).
2. One test per class.
3. A WAITING feature maps to UNRESOLVED while its workflow status stays WAITING HUMAN.

## Verification

README baseline plus `monitor_status.rs` cases per class.
