# WO-02 — Attention Class, Reason, Authority

- **Charter refs:** ECP-3, O-2, §6, C-10
- **Classification:** NORMAL
- **Depends on:** WO-01

## Goal

Each attention item states what is wrong, why it is reported, and which authority establishes it.

## In scope

- Add `class` to `AttentionItem`: BLOCKER, STALE, UNRESOLVED, or WARNING.
  - Classes are for presentation only and never replace workflow states.
  - Mapping is deterministic:
    - an accepted feature that is no longer current → STALE;
    - WAITING on a human decision → UNRESOLVED;
    - BLOCKED → BLOCKER;
    - a health domain at MEASURED or with an inventory warning → WARNING.
- Add a structured `reason`: a stable code plus the existing text. Sources are the checkpoint's
  unmet requirement kind and the revalidation reason, such as `SEMANTIC_CHANGE_REQUIRED`.
- Add `authority`: the governing policy path for the checkpoint, taken from the active DBA
  configuration (e.g. `dba/02-policies/workflow-feature/v9.md`).
- Add the attention classes to the glossary.

## Out of scope

- Affected surfaces (WO-03).
- Structured action (WO-05).
- Any change to checkpoint state or to exit codes.

## Authoritative sources to reuse

`checker::CheckpointReport`, `contract::Requirement`, `revalidation::invalidity_reason`, and
`Project::dba_configuration` plus the configuration YAML.

## Acceptance criteria

1. Every attention item has `class`, `reason.code`, and `authority`, and `authority` resolves to
   an existing file.
2. The class mapping has one test per class.
3. A feature at WAITING maps to UNRESOLVED, and its workflow status is still WAITING HUMAN.
4. Basis rule: if no authority can be resolved, the result is an explicit absence, not a guessed
   path.

## Verification

The README baseline, plus `monitor_status.rs` cases for each class.
