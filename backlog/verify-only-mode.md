# Upgrade: verify-only-mode — Verification-Only Mode

**Priority**: P1
**Status**: BACKLOG
**Type**: toolkit-upgrade
**Related**: reviewer-verification-packet, readiness-checklist

## Problem

During verification, Claude may "helpfully" change code or tests so that checks pass. That
blurs evidence.

## Upgrade

Add a strict read-only verification mode.

## Scope

Verification before/after sensitive stages; usable by the reviewer agent for independent
evidence.

## Proposed artifact(s)

`prompts/verify-only.md`

## Design notes

Core rule — verification-only means:

```text
Do not edit files.
Do not fix failures.
Do not rewrite tests.
Do not update snapshots.
Do not change runtime fixtures.
Do not stage, commit, or format files.
Run only the requested checks and report exact results.
```

Anti-blur enforcement. Before verification:

```bash
git status --short
git rev-parse HEAD
```

After verification:

```bash
git status --short
git diff --exit-code
git rev-parse HEAD
```

If the working tree changed, verification is invalid.

Verification report:

```markdown
# Verification-Only Report

Checks requested:
Commands run:
Pre-check commit:
Pre-check working tree:
Results:
- passed:
- failed:
- skipped:
- blocked:
Post-check commit:
Post-check working tree:
Files changed:
Verification validity:
- VALID / INVALID
If invalid, why:
```

Where used: before Stage 7 reconciliation; before Stage 8 replay; after Stage 9 refinement;
before PR readiness; by reviewer agent when independent evidence is needed.

Difference from reconciliation: verification-only is mechanical ("run checks without edits").
Reconciliation is semantic ("do all artifacts and runtime behavior align?"). Verification-only
feeds reconciliation. It does not replace it.

## Value

Medium-high. Especially useful when you need clean evidence.

## Risk

Claude still edits accidentally.

## Guardrail

Any file change invalidates the verification run.

## DBA-philosophy note

Strengthens **evidence integrity**: keeps verification from silently mutating the very state
it measures. Read-only by construction; aligns with the reviewer's read-only design.
