# WO-08 — Terminal Acceptance and Health Clarity

Presentation · NORMAL · from WO-06 gaps A, B, D · Charter: O-1, C-3, C-10

## Goal

The terminal view distinguishes accepted-but-stale features from incomplete work and shows why
each health domain has its status.

## In scope

- Feature lines show historical acceptance and current validity separately; the project line no
  longer counts accepted-but-stale features as "incomplete".
- Each health domain exposes its existing `basis` compactly.

## Out of scope

JSON changes (it already carries `historically_accepted`, `current_validity`, and `basis`); new
semantics or states.

## Acceptance criteria

1. On JamFree, F-0001–F-0004 read as accepted and currently stale, not incomplete.
2. Every terminal health status is accompanied by its basis.
3. `status --json` output is unchanged.
