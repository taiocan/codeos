# WO-01 — Historical Invalidation Replay

NORMAL · analysis only · approved by the human 2026-11-17

## Goal

Measure which invalidations in the window were semantically unnecessary, so WO-02 designs only what
the data justifies. No TomTom calls, no JamFree record changes.

## Work

Read-only replay of JamFree 2026-11-13 → 11-17 (`.codeos/06-workflow/decisions.jsonl`,
`verifications.jsonl`, `candidates/`, the review log). For every re-run or re-decision of F4, F6,
F7, F8, F9 and Stage 8:

1. Classify it by the **actual inputs and bindings the old rule used**, and which of them changed
   (for example `delivered_state`, a record identity in a Candidate Bundle, `observed_surface`,
   `contract`, the DBA configuration).
2. Separately, state whether each changed input was semantically relevant to that decision.

The replay does not judge history with the new design. WO-02 turns its finding into the identity
model.

## Acceptance criteria

1. A per-checkpoint table names the responsible binding and the relevance verdict for every case.
2. Counts reproduce from one committed read-only command.
3. Every invalidation is classified, or carries a stated reason why it cannot be.
