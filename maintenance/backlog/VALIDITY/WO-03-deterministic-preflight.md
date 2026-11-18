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

## Results (2026-11-18)

`codeos-workflow preflight --subject F-NNNN` prints one result per check: `PASS`, `BLOCKED(reason)`
or `NOT_APPLICABLE(reason)`. It exits with the blocked code only if a check is `BLOCKED`. It is
read-only: it runs no verification, qualification, audit or network work, and records nothing. It
is a tool, not a checkpoint, so no DBA is involved.

| Check | What it proves | Reuses |
|---|---|---|
| specification | Intent, Contract and Event Schema exist, their frontmatter parses, and they share one `semantic_revision` and `semantic_change` | `serde_yaml`, the Feature Workflow v3 rule |
| validation-claims | the Validation Claims table is attributable (IDs, checkpoints, `run:` evidence) | `checker::subject_claims` |
| journey-contribution | the Product Journey Contribution parses and its previous frontier matches the accepted journey | `checker::product_journey_target` |
| qualification | the authoritative state of Charter prerequisites such as C-1 and of the dependency baseline: the feature's latest acceptance qualification record, inspected for currency, never run | `checker::qualification_bindings`, `verification::evaluate` |

Qualification outcomes:

- `PASS`: the record is current and passed.
- `BLOCKED(QUALIFICATION_BLOCKERS)`: the record is current but failed; the detail names the failing
  constraints and baseline items.
- `BLOCKED(QUALIFICATION_REQUIRED)`: the record is missing or stale; the detail names the changed
  binding. The next step is `qualify-project`.
- `NOT_APPLICABLE(NO_PROJECT_QUALIFICATION)`: the project declares no qualification.

"Reality Basis rows cite existing tests" is not checked (accepted by the human): the Reality Basis
table names no test CodeOS can resolve mechanically. Tests are cited through the `run:` evidence of
the Validation Claims, which the claims check covers.

**Verification:** `tests/preflight.rs` has 6 tests, each asserting that preflight never ran the
qualification stub:

- a YAML colon in `semantic_change` → `specification` is `BLOCKED(DEFECT)`;
- an unresolved C-1 → `BLOCKED(QUALIFICATION_BLOCKERS)` naming C-1;
- an open advisory → `BLOCKED(QUALIFICATION_BLOCKERS)` naming `dependency_vulnerabilities`;
- a missing record, then a stale one → `BLOCKED(QUALIFICATION_REQUIRED)`;
- no declared qualification → `NOT_APPLICABLE`;
- a clean qualified feature passes, with the store unchanged.

The first version ran the project's qualification from preflight. The human rejected that as
expensive and as recomputing the authority, and it was corrected before acceptance.
