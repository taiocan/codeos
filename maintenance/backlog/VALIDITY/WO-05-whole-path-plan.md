# WO-05 — Whole-Path Plan

NORMAL · independent · approved by the human 2026-11-17

## Goal

The agent sees the whole remaining path at once, and can ask for consents together where the human
already has the information.

## Work

Generalize `checker::revalidation_check_plan` (`dba/04-tools/workflow/engine/src/checker.rs`, today
only `check --revalidation --plan`) to any subject. For each remaining step it shows:

- the checkpoint;
- its human gate, with the claims that gate judges;
- what it needs from the provider (calls);
- the exact next command.

The plan is read-only. It groups decisions only where the human already has the bound information.
F8 and release remain later decisions.

## Acceptance criteria

1. On a fixture, the plan lists exactly the gates `check` then requires.
2. `cargo test` passes.
