---
component_question: How should a read-only verification pass execute checks and report observations?
out_of_scope: Fixing findings, advisory review judgments, approval decisions, and workflow-stage execution.
---

# Verification-Only Mode

## Purpose

Run requested checks without edits and return uncontaminated mechanical evidence. This optional mode
is not a DBA stage or gate.

## Read-Only Contract

Do not edit, format, stage, commit, update snapshots, change fixtures, or repair failures. Run only
requested checks and report exact results.

Before checks, record:

```bash
git rev-parse HEAD
git status --porcelain=v1
git diff --binary | sha256sum
git ls-files --others --exclude-standard -z | while IFS= read -r -d '' f; do sha256sum -- "$f"; done | sha256sum
```

After checks, run and record the same commands. The pass is `VALID` only when commit, porcelain
status, tracked-diff fingerprint, and untracked-content fingerprint are unchanged. This permits a
pre-existing dirty tree while detecting mutations during verification.

## Output

```markdown
# Verification-Only Report

Checks requested:
Commands run:
Pre-check state and fingerprints:

Results:
- passed:
- failed:
- skipped:
- blocked:

Post-check state and fingerprints:
Verification validity: VALID | INVALID
Reason if invalid:
```

A failed check is evidence, not permission to fix it. Attach the report to the work or review round
that requested it.
