# WO-DM-07 — Static and Security Summaries

- **Class:** NORMAL. It is read-only analysis.
- **Origin:** split out of WO-04 by the human on 2026-11-22. It differs from WO-04's git-only
  analytics: it compiles code in a temporary worktree and downloads advisory databases, so it
  costs more and fails in different ways.
- **Goal:** clippy warning counts and cargo-audit / cargo-deny findings for the toolkit and the
  given projects, as their own snapshot through `monitor/snapshot-lib.sh`.

## Scope

- Run clippy in a temporary worktree with a scratch `CARGO_TARGET_DIR`, so no analysed repository
  gains build output. Record warning counts by lint.
- Record cargo-audit and cargo-deny findings by advisory or check kind. These tools fetch public
  advisory databases and send nothing about the project.
- Record a tool that is not installed or a fetch that fails as UNOBSERVED, never as zero findings.

## Verification

- The analysed repositories are unchanged afterwards.
- A known advisory or lint in a fixture is counted.
- A missing tool yields null.

## Acceptance

The verification passes and the human accepts.
