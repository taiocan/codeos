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
3. **Static and security summaries.** clippy warning counts, plus cargo-audit and cargo-deny
   findings, run in a temporary worktree.

## Verification

- One hotspot and one governance-growth figure reconcile by hand against git.
- The manifest verifies, and the source repositories are unchanged afterwards.

## Acceptance

The verification passes and the human accepts.
