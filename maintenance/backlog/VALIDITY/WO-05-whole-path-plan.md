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

## Results (2026-11-18, awaiting acceptance)

`codeos-workflow plan --subject F-NNNN [--subject …] [--json]` is read-only and executes nothing. It
is a tool, not a checkpoint, so no DBA is involved. `checker::whole_path_plan` derives the steps
from the feature's workflow contract and current state: every applicable checkpoint not yet PASS,
including those blocked behind an earlier one.

| Step kind | Shows |
|---|---|
| verification | the mechanic and `reuse` or `rerun`, by the same currency rule `check` uses |
| production | `reuse` or `rerun`, plus the provider calls the latest run accounted |
| review | the Stage 8 review a new candidate needs |
| decision | the receipt still missing or stale, the `observation:human` claims the gate judges, and the exact `decide` command |
| specification | a requirement the checkpoint is blocked on by itself |

A human gate is offered `now` only when its checkpoint is WAITING, so its prerequisites already
pass and the human has the bound information. F8 and F9 are always `later`; release is outside
the workflow. Text output ends with "Consents the human can give now", "Later decisions" and
"Production runs needed".

**Verification:**

- `tests/whole_path_plan.rs`:
  - before any verification, the plan lists the remaining gates (F6, F9) and offers none;
  - after `check`, it offers exactly the gates `evaluate` reports WAITING for the human (F6) and
    keeps F9 `later`;
  - after acceptance, no decision remains, and the planned checkpoints equal exactly those `check`
    reports open.
- The full engine suite, clippy and `dba/04-tools/tests/run.sh` pass.
- **Read-only run on a JamFree copy (4502311),** F-0001 and F-0002:
  - every remaining checkpoint was listed with `reuse` or `rerun`;
  - current receipts (F-0002 F4, both F6s) were correctly omitted;
  - F4, F8 and F9 were listed as later decisions.

**Not covered by a fixture:** the claims column for an `observation:human` claim. The fixture
feature is non-GUI, so it has no F4/F8.
