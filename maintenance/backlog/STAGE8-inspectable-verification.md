# Stage 8 Inspectable Verification Evidence

Classification: PROTECTED — approved 2026-10-10 · Authority: Reviewer Tool Contract v17 (required
verification records "with their underlying content"), Review Policy v15 (summary-only required
input fails readiness), Codeos Mechanics v7 (`repeatability`: replay and idempotency evidence).

## Finding

JamFree F-0001 Stage 8 was BLOCKED twice for the same reason: verification records carried only
the command and `passed`. `check` discarded runner output, and `repeatability` ran the
`behavior` command once without comparing anything. Readiness accepted these summary-only records
as `FULL_COVERAGE`.

## Approved semantics

1. **Per-test outcomes.** `check` stores a bounded, normalized list of test cases (identity, file
   when known, status) for `behavior`, `repeatability`, and `playwright` as content-addressed
   evidence in `.codeos/06-workflow/test-evidence/`. The record binds its SHA-256 in
   `details.test_evidence_sha256`. Logs and full output are not stored.
2. **Repeatability compares.** `repeatability` runs the deterministic suite twice and records
   both runs. Any difference in case identities or outcomes fails the check as nondeterminism.
3. **Packet carries the evidence.** For each required verification record, the Stage 8 packet
   includes its bound test evidence and the test inputs the Candidate Bundle binds. Nothing else
   is added, and the packet budget still applies.
4. **Readiness fails closed.** A required record of these kinds that lacks bound evidence, or whose
   evidence is absent or does not match its identity, is `REQUIRED_INPUT_NOT_INSPECTABLE`.

## Invariants

- The reviewer remains advisory.
- Accepted receipts and old records are never rewritten. Old records stay readable and block only
  new candidate assessments.
- `status` stays read-only.
- Binding and selective-invalidation meaning is unchanged.
- No project-specific rule is added.
