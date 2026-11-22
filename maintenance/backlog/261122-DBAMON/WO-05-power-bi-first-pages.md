# WO-DM-05 — First Three Power BI Pages

- **Class:** implemented in the PowerBI repository (`projects/dba-monitor`). No CodeOS semantics
  change.
- **Goal:** a read-only PBIP/TMDL report over the WO-03 snapshot, authored on the Windows VM
  following the PowerBI repository's learning plan.

## Scope

1. **Flow and rework:**
   - lead time per feature;
   - time in each checkpoint F1–F9;
   - candidates per acceptance and BLOCKED reviews per feature;
   - revalidation burden, with the surfaces that triggered it;
   - a labelled change-failure proxy.
2. **Test health:**
   - failed verifications by kind and checkpoint;
   - flaky rate;
   - case count and growth;
   - suite mix;
   - a labelled defect-escape proxy;
   - durations, once WO-02 exists.
3. **Review effectiveness and cost:**
   - outcomes by stage and over time;
   - findings by severity, classification and blocker kind;
   - packet bytes against budget and elapsed time;
   - execution-invalid rate by CLI version and build;
   - hotspots;
   - advisory assessments kept apart from gate outcomes.

Also included:

- a prerequisite script adapted from `Test-AIISnapshotPrerequisites.ps1`;
- a KPI glossary giving each KPI's definition, source and any proxy label;
- the not-publishable flag.

A project that runs its tests through its own `runtime_verification.verification_command` (for
example AII) records no CodeOS test evidence, so its test-run visuals stay empty. That is
UNOBSERVED, not zero.

Source written 2026-11-22 to `PowerBI/projects/dba-monitor` (not yet committed there), with a
JamFree snapshot in its ignored `data/external`. Expected values at the first refresh:

| Measure | Expected value |
|---|---|
| Verification records | 1,701 |
| Failed verifications | 19 |
| Decision receipts | 317 |
| Candidate bundles | 175 |
| Revalidation candidates | 103 |
| Assessments | 195 |
| PASSED | 112 |
| BLOCKED | 42 |
| Findings | 54 |
| Execution-invalid attempts | 10 |
| Test runs | 140 |
| Test cases | 5,904 |
| Change-failure proxy | 4/10 |
| Defect-escape proxy | 4/23 |

## Verification

- The prerequisite script passes.
- For one feature, lead time, candidate count, BLOCKED count and failed verifications reconcile
  against the raw records.

## Acceptance

The verification passes and the human accepts.
