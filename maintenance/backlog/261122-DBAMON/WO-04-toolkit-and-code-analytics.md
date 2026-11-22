# WO-DM-04 — Toolkit Governance Growth and Code Analytics

- **Class:** NORMAL. It is read-only analysis.
- **Goal:** a separate acquisition domain, with its own snapshot and manifest, for long-term
  CodeOS health and code health. It is kept apart from WO-03 because its sources and failure modes
  differ.

## Scope

1. **Governance growth.** Measured at each issued DBA-configuration commit of the Codeos repo:
   - policy, contract and DBA-configuration versions;
   - the share of DBA-N issued as corrections;
   - CLI flags;
   - glossary terms;
   - Rust lines of code and test count;
   - downstream defects classified as CodeOS correctness defects;
   - adoption lag from issue to downstream adoption.

   Show absolute growth over time and configuration as the primary view. Growth per delivered
   feature is context only.
2. **Git churn and hotspots.** Churn per file per commit and hotspots (churn × size), with
   bookkeeping paths (`.codeos/`, `events/`, records) excluded.
3. **Static and security summaries:** moved to [WO-07](WO-07-static-and-security-summaries.md)
   by the human on 2026-11-22.

CodeOS-attributable defects and whether a DBA-N was issued as a correction are not recorded in
structured form. Their columns are null, as decided by the human on 2026-11-22.

Implemented as `dba/04-tools/monitor/codeos-toolkit-analytics.sh <external-root> [project ...]`. It
reads git history only, and its bookkeeping exclusions include project `evidence/` record files.

## Verification

- One hotspot and one governance-growth figure reconcile by hand against git.
- The manifest verifies, and the source repositories are unchanged afterwards.

## Acceptance

The verification passes and the human accepts.
