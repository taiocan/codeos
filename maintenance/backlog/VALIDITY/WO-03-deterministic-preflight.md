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

## Results (2026-11-18, awaiting acceptance)

`codeos-workflow preflight --subject F-NNNN` prints `PASS` or `FAIL` per check and exits 0 or the
blocked code. It records nothing. It is a read-only tool, not a checkpoint, so no DBA is involved.

| Check | What it proves | Reuses |
|---|---|---|
| specification | Intent, Contract and Event Schema exist, their frontmatter parses, and they share one `semantic_revision` and `semantic_change` | `serde_yaml`, the Feature Workflow v3 rule |
| validation-claims | the Validation Claims table is attributable (IDs, checkpoints, `run:` evidence) | `checker::subject_claims` |
| journey-contribution | the Product Journey Contribution parses and its previous frontier matches the accepted journey | `checker::product_journey_target` |
| qualification | the project's own `scripts/codeos-qualify` in acceptance mode: Charter prerequisites such as C-1, and the dependency-vulnerability, licence, secret and static-analysis baseline | `qualification::run` |

"Reality Basis rows cite existing tests" is not checked: the Reality Basis table names no test
CodeOS can resolve mechanically. Tests are cited through the `run:` evidence of the Validation
Claims, which the claims check covers.

**Verification:** `tests/preflight.rs`, one fixture per defect class, all passing:

- a YAML colon in `semantic_change` → `specification` fails;
- an unresolved C-1 → `qualification` fails naming C-1;
- an open advisory → `qualification` fails naming `dependency_vulnerabilities`;
- a clean feature passes, and the workflow store is unchanged.

The full engine suite, clippy and `dba/04-tools/tests/run.sh` pass.
