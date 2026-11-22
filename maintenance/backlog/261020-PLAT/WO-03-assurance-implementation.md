# WO-03 — Development-Assurance Projection

NORMAL · after WO-02

## Goal

Implement the approved projection without adding operational state or workflow authority.

## Work

- Parse the existing Backend row in Feature Impact Accounting alongside GUI and Persistence.
- Derive the four assurance areas from applicable current verification records.
- Add `platform_assurance` to status JSON and `status --platform` to terminal output.
- Include exact subjects and evidence basis; emit no aggregate result.

## Acceptance criteria

1. Backend-only, GUI-only, persistence-only, full-stack, failed, stale, missing, and
   not-applicable cases are covered.
2. Existing JSON fields, terminal sections, ordering, and exit codes remain compatible; standard
   status gains only the approved compact summary.
3. `status` remains read-only and performs no network access.
4. Engine tests, clippy, status snapshots, and layout checks pass.

## Result (2026-10-20)

Accepted. The status engine now derives four read-only rows from Feature Impact Accounting and
exact Development Evidence. JSON exposes the full basis; standard status summarizes every area;
`status --platform` provides the issue drill-down. Missing accounting or identities remain
`UNOBSERVED`, stale records remain `STALE`, and only failed records become `BLOCKED`.

All 206 engine tests, clippy, snapshots, and the CodeOS layout contract pass. Existing status exit
codes and workflow behavior are unchanged.
