# WO-02 — Attention Class, Reason, Authority

ECP-3 · NORMAL · after WO-01 · Charter: O-2, §6, C-10

## Goal

Each attention item states what is wrong, why, and which authority establishes it.

## In scope

- `class`: BLOCKER, STALE, UNRESOLVED, WARNING — presentation only, total over non-complete
  subjects: accepted feature no longer current → STALE; WAITING on a human → UNRESOLVED; BLOCKED,
  including an actionable prerequisite → BLOCKER. WARNING has no source yet: inventory warnings
  keep their own field, and MEASURED is not emitted (WO-01 gap). Glossary entry for the classes.
- `reason`: a typed code from the requirement that produced the next action (the same one as
  `actor`) or the revalidation reason (e.g. `SEMANTIC_CHANGE_REQUIRED`); the text stays in `issue`.
- `authority`: the governing policy path from the active DBA configuration (bootstrap, feature,
  or workflow governance for current validity); `NOT_CONFIGURED` when none is selected; an error
  when the selected file is missing from the toolkit.

## Out of scope

Affected surfaces (WO-03); structured action (WO-05); checkpoint state or exit-code changes.

## Sources

`CheckpointReport`, `contract::Requirement`, `revalidation::invalidity_reason`,
`Project::dba_configuration`.

## Acceptance criteria

1. Every attention item has `class`, `reason`, and an `authority` that resolves to an existing
   file or is `NOT_CONFIGURED`; a missing selected file fails closed.
2. One test per emitted class; the class and reason vocabularies are pinned.
3. A WAITING feature maps to UNRESOLVED while its workflow status stays WAITING HUMAN.

## Verification

README baseline plus `monitor_status.rs` cases per class.
