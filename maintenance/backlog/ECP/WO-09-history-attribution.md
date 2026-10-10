# WO-09 — Process History Attribution

ECP-4 · NORMAL · from WO-06 gap F · high priority · Charter: O-3, success criterion 9, C-6

## Goal

Answer *where* engineering effort is lost, not only how much.

## In scope

- Deterministic breakdowns of existing `--history` counts by subject, checkpoint, and reason where
  the source records carry them; otherwise an explicit absence state.
- Terminal and JSON (additive to schema v3).

## Out of scope

New records; causal attribution; scores or rankings (C-6).

## Acceptance criteria

1. On JamFree, additional decision receipts and non-progressing assessments are localized to
   subjects and checkpoints.
2. Every breakdown sums to its total or states what is unattributed.
3. Plain `status` output is unchanged.
