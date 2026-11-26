# OBLIGATIONS — Charter Obligations to Verified Acceptance

Planning context only. Git remains authoritative. Opened after the fact on 2026-11-26 to hold the
change report of the DBA-38 candidate.

## Origin

The EvidenceAtlas Charter-to-Intent review found 100% ID-level coverage (12/12 O, 22/22 R, 25/27 C)
yet real gaps: compound requirements only partly specified (R-8), cross-feature obligations without
end-to-end proof (C-20, C-27, C-3), and milestones without an acceptance owner. Root causes in CodeOS:
R-items were not a CodeOS concept, trace was ID-level and Intent-targeted, non-feature obligations
had no owner type, and coverage was never separated from verification.

## Human decisions (2026-11-26)

- Direction approved with modifications. No separately maintained compliance matrix; work orders and
  wall-clock sizing stay out of the governed path.
- R-items are a first-class Charter category.
- Exactly one accountable verification owner per obligation, with one or more mandatory checks.
  VERIFIED needs current passing evidence from every mandatory check. Contributions never establish
  compliance.
- Milestones aggregate existing product-journey, integration, qualification and evaluation evidence.
  No parallel milestone workflow.
- The whole-Charter hash and its integrity guarantees stay. Per-clause meaning digests are added for
  targeted invalidation only.
- Semantic reviewers stay advisory. Legacy or unassessed obligations are reported apart from
  verified ones.
- Scenario checks use scenario-tagged tests and reuse `test_evidence.rs`.
- No additional governance artifacts, review stages or workflow layers.

## Change (DBA-38 candidate)

| Component | Change |
|---|---|
| Doctrine v11 | Outcomes, Requirements, System Constraints and Milestones are Charter obligations; atomicity; one owner; ALLOCATED, SPECIFIED and VERIFIED reported apart; UNASSESSED is never verified |
| Feature Workflow v16 | Clause IDs (`R-8.3`) and meaning digests; `[S-k]` scenario IDs with `covers:` and `contributes:`; test tag `fnnnnsk`; F9 blocked while an owned obligation is not VERIFIED |
| Reviewer Tool Contract v24 | Stages 2 and 3 show the exact clause beside its covering scenarios; Stage 5 shows each scenario's test tag; advisory, no new required input |
| Engine | `obligations.rs` and `codeos-workflow obligations`; v16 records keep `details.obligation_meaning` as a recorded value, not a dependency |
| Guidance | Requirements and Milestones tables in the Charter template, scenario IDs in the Contract template, prompt guidance gated on v16, five glossary terms |
| Candidate | `maintenance/candidates/dba-38.yaml` |

Design choices to keep in view:

- A clause row `R-8.1` makes `R-8` a grouping row whose state is its weakest clause.
- Constraint clauses `C-n.k` sit in a separate table, because project qualification reports only
  `C-n`; its existing check stays intact.
- SPECIFIED means the human-approved Contract; the reviewer's judgment never sets a state.
- Under v16, a malformed `covers:` line blocks recording verification results until it is fixed.

## Verification

- 20 new tests (19 engine, 1 reviewer): an untagged passing test, a stale tagged test, and a failed,
  flaky or only skipped tagged test cannot verify; passing feature scenarios with a failed
  integrated journey cannot verify; a reworded clause under the same ID needs reassessment; legacy
  evidence never counts; an unrelated Charter edit stales the whole-Charter binding but keeps
  unaffected clause results current, while every other dependency still applies; contributions
  alone never verify; two owners, unknown IDs and invalid milestone aggregation are findings.
- `dba/04-tools/tests/run.sh` PASS; both engines clippy-clean.
- EvidenceAtlas replay on a scratch copy (the live project untouched): with R-8, C-20 and C-27
  atomized, R-8.4, R-8.6, R-8.7 and C-27.2 have no owner, M-1 cannot pass ALLOCATED, and nothing
  reaches VERIFIED. The replay's `covers:` lines were written from the review, not by EvidenceAtlas.

## Open

1. Qualify the DBA-38 candidate, run `build-qualified-release.sh`, and run JamFree regression on it.
2. Human acceptance of the protected invariant: no obligation is reported VERIFIED on false, stale or
   legacy evidence.
3. Human approval to issue DBA-38 and decide whether it becomes the new-project default.
4. EvidenceAtlas adoption: atomize its Charter and declare `covers:` in its Contracts.
