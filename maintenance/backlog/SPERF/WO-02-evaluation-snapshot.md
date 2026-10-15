# WO-02 — Command-Scoped Evaluation Snapshot

NORMAL · after WO-01 · Charter: Goal, Invariant

## Goal

Evaluate every checkpoint against one snapshot that is built once per command.

## Work

1. Resolve the project configuration once.
2. Run cargo metadata once and derive the implementation paths from it.
3. Replace per-path Git calls with repository-wide batches: one `ls-tree -r HEAD`, one diff, one
   `ls-files --cached --others --exclude-standard`.
4. Memoize each binding by path set and binding type.
5. Pass the Bootstrap result into feature evaluation.
6. Reuse each feature's assessment and checkpoint report for the health and attention sections.

## Acceptance criteria

The workflow and reviewer suites pass, and the WO-01 golden report is unchanged.

## Results
