# WO-03 — Deterministic Preflight

NORMAL · independent · approved by the human 2026-11-17

## Goal

Cheap, provable defects surface before spec approval or F4, not at the candidate or Stage 8.

## Work

A read-only `codeos-workflow preflight --subject F-NNNN` that checks:

- the frontmatter of the Specification Package parses;
- cross-references resolve, and Reality Basis rows cite existing tests;
- prerequisites (C-1, charter gates) are settled;
- the dependency and advisory state;
- consistency only over values CodeOS already identifies mechanically.

No prose interpretation and no AI review.

## Acceptance criteria

1. One fixture per defect class seen in the window that falls under these checks: a YAML colon in
   `semantic_change`, an unsettled privacy prerequisite, an open advisory.
2. `cargo test` and `dba/04-tools/tests/run.sh` pass.
